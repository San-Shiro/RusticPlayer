pub mod api;
pub mod db;
pub mod state;

use audio::player::{Player, PlayerEvent, PlaybackState as AudioPlaybackState};
use db::{init_db, open_db};
use rustic_core::api::{
    Command, CommandAction, EngineError, ErrorKind, Event, PlaybackState, RepeatMode, Track,
    TrackSource,
};
use rustic_core::queue::QueueManager;
use sources::resolver::SourceResolver;
use sources::spotify::client::SpotifyProvider;
use sources::ytmusic::client::YtMusicProvider;
use state::EngineState;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};

pub struct Engine {
    cmd_tx: mpsc::Sender<Command>,
    event_tx: broadcast::Sender<Event>,
    clock: Arc<audio::player::MasterClock>,
    diagnostics: audio::player::DiagnosticsHandle,
    ytm: Arc<YtMusicProvider>,
}

impl Engine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(64);
        let (event_tx, _) = broadcast::channel::<Event>(128);
        
        let (player, audio_rx) = Player::new()?;
        let clock = player.clock();
        let diagnostics = player.diagnostics_handle();
        
        let ytm = Arc::new(YtMusicProvider::new());
        let spotify = Arc::new(SpotifyProvider::new());
        let resolver = Arc::new(SourceResolver::new(ytm.clone(), spotify));
        
        let mut conn = open_db()?;
        init_db(&mut conn)?;
        let state = EngineState::load(&conn);
        let queue = state.to_queue_manager();
        
        let event_tx_clone = event_tx.clone();
        let ytm_for_loop = ytm.clone();
        let cmd_tx_loop = cmd_tx.clone();
        let clock_for_loop = Arc::clone(&clock);

        tokio::spawn(async move {
            let mut loop_state = EngineLoop {
                cmd_tx: cmd_tx_loop,
                cmd_rx,
                audio_rx,
                event_tx: event_tx_clone,
                player,
                resolver,
                ytm: ytm_for_loop,
                clock: clock_for_loop,
                state,
                queue,
                current_playback_state: PlaybackState::Stopped,
                radio_continuation: None,
                is_fetching_radio: false,
            };
            loop_state.run().await;
        });

        Ok(Self {
            cmd_tx,
            event_tx,
            clock,
            diagnostics,
            ytm,
        })
    }
    
    pub fn send_command(&self, cmd: Command) {
        if let Err(e) = self.cmd_tx.try_send(cmd) {
            tracing::warn!(target: "engine", "send_command failed: {:?}", e);
        }
    }
    
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_tx.subscribe()
    }

    pub fn current_position(&self) -> Duration {
        self.clock.current_position()
    }

    pub fn clock(&self) -> Arc<audio::player::MasterClock> {
        Arc::clone(&self.clock)
    }

    pub fn audio_diagnostics(&self) -> audio::AudioDiagnostics {
        self.diagnostics.read().clone()
    }

    pub fn ytmusic(&self) -> Arc<YtMusicProvider> {
        Arc::clone(&self.ytm)
    }
}

struct EngineLoop {
    cmd_tx: mpsc::Sender<Command>,
    cmd_rx: mpsc::Receiver<Command>,
    audio_rx: tokio::sync::broadcast::Receiver<PlayerEvent>,
    event_tx: broadcast::Sender<Event>,
    player: Player,
    resolver: Arc<SourceResolver>,
    ytm: Arc<YtMusicProvider>,
    clock: Arc<audio::player::MasterClock>,
    state: EngineState,
    queue: QueueManager,
    current_playback_state: PlaybackState,
    radio_continuation: Option<String>,
    is_fetching_radio: bool,
}

impl EngineLoop {
    async fn run(&mut self) {
        tracing::debug!(target: "engine", "EngineLoop background task running");
        let _ = self.player.set_volume(self.state.volume);
        self.sync_and_broadcast_queue();
        if let Some(t) = self.queue.current_cloned() {
            let _ = self.event_tx.send(Event::TrackChanged(Some(t)));
        }

        loop {
            tokio::select! {
                biased;
                cmd = self.cmd_rx.recv() => {
                    match cmd {
                        Some(c) => {
                            tracing::debug!(target: "engine", "Received command: {:?}", c.action);
                            self.handle_command(c).await;
                        }
                        None => {
                            tracing::debug!(target: "engine", "cmd_rx closed");
                            break;
                        }
                    }
                }
                player_event = self.audio_rx.recv() => {
                    if let Ok(pe) = player_event {
                        self.handle_audio_event(pe).await;
                    }
                }
            }
        }
    }

    fn sync_and_broadcast_queue(&mut self) {
        self.state.queue = self.queue.actual_queue.clone();
        self.state.current_index = if self.queue.actual_queue.is_empty() {
            None
        } else {
            Some(self.queue.cursor)
        };
        self.state.queue_manager = Some(self.queue.clone());
        self.save_state();

        let snap = self.queue.snapshot();
        let _ = self.event_tx.send(Event::QueueUpdated(Box::new(snap)));
        let _ = self.event_tx.send(Event::QueueChanged(self.queue.actual_queue.clone()));
    }

    fn check_and_fetch_radio(&mut self) {
        if !self.queue.autoplay_enabled || self.is_fetching_radio {
            return;
        }
        if !self.queue.needs_prefetch() {
            return;
        }

        let ytm = Arc::clone(&self.ytm);
        let cmd_tx = self.cmd_tx.clone();
        let continuation = self.radio_continuation.clone();
        let current_video_id = self.queue.current().and_then(|t| match &t.source {
            TrackSource::YouTubeMusic { video_id } => Some(video_id.clone()),
            _ => None,
        });

        if continuation.is_none() && current_video_id.is_none() {
            return;
        }

        self.is_fetching_radio = true;
        tokio::spawn(async move {
            if let Some(tok) = continuation {
                match ytm.fetch_radio_continuation(&tok).await {
                    Ok((tracks, _next_tok)) => {
                        let _ = cmd_tx.send(Command {
                            origin: None,
                            action: CommandAction::IngestRadioTracks(tracks),
                        }).await;
                    }
                    Err(e) => {
                        tracing::warn!(target: "engine", "fetch_radio_continuation failed: {:?}", e);
                        let _ = cmd_tx.send(Command {
                            origin: None,
                            action: CommandAction::IngestRadioTracks(vec![]),
                        }).await;
                    }
                }
            } else if let Some(vid) = current_video_id {
                match ytm.fetch_automix_radio(&vid).await {
                    Ok(snap) => {
                        let tracks: Vec<Track> = snap.upcoming.into_iter().map(|item| {
                            let duration_secs = item.parse_duration_secs();
                            Track {
                                id: item.video_id.clone(),
                                title: item.title,
                                artists: vec![item.artist],
                                album: "YouTube Music".into(),
                                duration_secs,
                                source: TrackSource::YouTubeMusic { video_id: item.video_id },
                                isrc: None,
                                artwork_url: item.thumbnail_url,
                                content_hash: None,
                            }
                        }).collect();
                        let _ = cmd_tx.send(Command {
                            origin: None,
                            action: CommandAction::IngestRadioTracks(tracks),
                        }).await;
                    }
                    Err(e) => {
                        tracing::warn!(target: "engine", "fetch_automix_radio failed: {:?}", e);
                        let _ = cmd_tx.send(Command {
                            origin: None,
                            action: CommandAction::IngestRadioTracks(vec![]),
                        }).await;
                    }
                }
            }
        });
    }

    async fn handle_command(&mut self, cmd: Command) {
        match cmd.action {
            CommandAction::Play => {
                tracing::debug!(target: "engine", "Handling Play, current_playback_state={:?}", self.current_playback_state);
                if self.current_playback_state == PlaybackState::Paused {
                    let _ = self.player.play();
                } else if self.current_playback_state == PlaybackState::Stopped {
                    if self.queue.current().is_some() {
                        self.play_current().await;
                    } else {
                        tracing::debug!(target: "engine", "Play: queue is empty");
                    }
                }
            }
            CommandAction::Pause => {
                tracing::debug!(target: "engine", "Executing pause command on player");
                let res = self.player.pause();
                tracing::debug!(target: "engine", "self.player.pause() res={:?}", res);
            }
            CommandAction::TogglePlay => {
                if self.current_playback_state == PlaybackState::Playing {
                    let _ = self.player.pause();
                } else if self.current_playback_state == PlaybackState::Paused {
                    let _ = self.player.play();
                } else if self.queue.current().is_some() {
                    self.play_current().await;
                }
            }
            CommandAction::Stop => {
                let _ = self.player.pause(); 
            }
            CommandAction::SetVolume(v) => {
                self.state.volume = v.clamp(0.0, 1.5);
                let _ = self.player.set_volume(self.state.volume);
                self.save_state();
                let _ = self.event_tx.send(Event::VolumeChanged(self.state.volume));
            }
            CommandAction::Seek(d) => {
                let _ = self.player.seek(d);
            }
            CommandAction::PlayTrack(track_id) => {
                if let Some(track) = self.fetch_track(&track_id) {
                    self.queue.play_single(track);
                    self.radio_continuation = None;
                    self.sync_and_broadcast_queue();
                    self.play_current().await;
                    self.check_and_fetch_radio();
                }
            }
            CommandAction::PlaySingleTrack(track) => {
                self.queue.play_single(track);
                self.radio_continuation = None;
                self.sync_and_broadcast_queue();
                self.play_current().await;
                self.check_and_fetch_radio();
            }
            CommandAction::PlayCollection { tracks, start_index, title, id, shuffle } => {
                self.queue.set_context(id, title, tracks, start_index, shuffle);
                self.radio_continuation = None;
                self.sync_and_broadcast_queue();
                self.play_current().await;
                self.check_and_fetch_radio();
            }
            CommandAction::EnqueueTrackNext(track) => {
                self.queue.enqueue_next(track);
                self.sync_and_broadcast_queue();
            }
            CommandAction::EnqueueNext(track_ids) => {
                for tid in track_ids {
                    if let Some(track) = self.fetch_track(&tid) {
                        self.queue.enqueue_next(track);
                    }
                }
                self.sync_and_broadcast_queue();
            }
            CommandAction::EnqueueTrackEnd(track) => {
                self.queue.enqueue_end(track);
                self.sync_and_broadcast_queue();
            }
            CommandAction::EnqueueEnd(track_ids) => {
                for tid in track_ids {
                    if let Some(track) = self.fetch_track(&tid) {
                        self.queue.enqueue_end(track);
                    }
                }
                self.sync_and_broadcast_queue();
            }
            CommandAction::RemoveUserQueueItem(idx) => {
                self.queue.remove_user_queue_item(idx);
                self.sync_and_broadcast_queue();
            }
            CommandAction::RemoveFromQueue(idx) => {
                self.queue.remove_user_queue_item(idx);
                self.sync_and_broadcast_queue();
            }
            CommandAction::ClearUserQueue => {
                self.queue.clear_user_queue();
                self.sync_and_broadcast_queue();
            }
            CommandAction::MoveUserQueueItem { from, to } => {
                self.queue.move_user_queue_item(from, to);
                self.sync_and_broadcast_queue();
            }
            CommandAction::MoveInQueue { from, to } => {
                self.queue.move_user_queue_item(from, to);
                self.sync_and_broadcast_queue();
            }
            CommandAction::PromoteRadioItem(idx) => {
                if let Some(_track) = self.queue.promote_radio_item(idx) {
                    self.sync_and_broadcast_queue();
                    self.check_and_fetch_radio();
                }
            }
            CommandAction::PlayRadioItem(idx) => {
                if let Some(_track) = self.queue.promote_suggestion(idx) {
                    self.sync_and_broadcast_queue();
                    self.play_current().await;
                    self.check_and_fetch_radio();
                }
            }
            CommandAction::JumpToQueueIndex(idx) => {
                if let Some(_track) = self.queue.jump_to(idx) {
                    self.sync_and_broadcast_queue();
                    self.play_current().await;
                    self.check_and_fetch_radio();
                }
            }
            CommandAction::JumpToTrackId(track_id) => {
                if let Some(_track) = self.queue.jump_to_track_id(&track_id) {
                    self.sync_and_broadcast_queue();
                    self.play_current().await;
                    self.check_and_fetch_radio();
                }
            }
            CommandAction::ToggleAutoplay(enabled) => {
                self.queue.autoplay_enabled = enabled;
                self.sync_and_broadcast_queue();
                if enabled {
                    self.check_and_fetch_radio();
                }
            }
            CommandAction::IngestRadioTracks(tracks) => {
                self.is_fetching_radio = false;
                self.queue.ingest_suggestions(tracks);
                self.sync_and_broadcast_queue();
            }
            CommandAction::SetUpcomingTracks(tracks) => {
                let keep_len = (self.queue.cursor + 1).min(self.queue.actual_queue.len());
                self.queue.actual_queue.truncate(keep_len);
                self.queue.actual_queue.extend(tracks);
                self.sync_and_broadcast_queue();
            }
            CommandAction::Next => {
                self.play_next().await;
            }
            CommandAction::Previous => {
                self.play_prev().await;
            }
            CommandAction::ReconnectAudio => {
                let _ = self.player.reconnect_audio();
            }
            CommandAction::Login { provider } => {
                if provider.eq_ignore_ascii_case("ytmusic") || provider.eq_ignore_ascii_case("youtube") {
                    match self.ytm.reload_cookies_file("cookies.json") {
                        Ok(auth) => {
                            let _ = self.event_tx.send(Event::AuthStateChanged {
                                provider: "ytmusic".into(),
                                logged_in: auth.logged_in,
                            });
                        }
                        Err(e) => {
                            tracing::warn!(target: "engine", "Login reload failed for ytmusic: {:?}", e);
                        }
                    }
                }
            }
            CommandAction::SetRepeat(mode) => {
                self.state.repeat = mode;
                self.save_state();
                let _ = self.event_tx.send(Event::RepeatChanged(mode));
            }
            CommandAction::SetShuffle(s) => {
                self.state.shuffle = s;
                self.queue.is_shuffled = s;
                self.sync_and_broadcast_queue();
                let _ = self.event_tx.send(Event::ShuffleChanged(s));
            }
            _ => {
                // Other commands
            }
        }
    }

    async fn handle_audio_event(&mut self, event: PlayerEvent) {
        if event.is_eos {
            let current_id = self.queue.current().map(|t| t.id.clone()).unwrap_or_default();
            self.current_playback_state = PlaybackState::Stopped;
            let _ = self.event_tx.send(Event::PlaybackStateChanged(PlaybackState::Stopped));
            let _ = self.event_tx.send(Event::TrackEnded { track_id: current_id });
            self.play_next().await;
            return;
        }

        let new_state = match event.state {
            AudioPlaybackState::Playing => PlaybackState::Playing,
            AudioPlaybackState::Paused => PlaybackState::Paused,
            AudioPlaybackState::Stopped => PlaybackState::Stopped,
            AudioPlaybackState::Buffering => PlaybackState::Buffering,
        };
        
        if self.current_playback_state != new_state {
            self.current_playback_state = new_state;
            let _ = self.event_tx.send(Event::PlaybackStateChanged(new_state));
        }
        
        let current_track_duration = self.queue.current()
            .map(|t| t.duration_secs as f64)
            .unwrap_or(0.0);
        let duration_secs = event.duration.map(|d| d.as_secs_f64())
            .filter(|&d| d > 0.0)
            .unwrap_or(current_track_duration);

        let _ = self.event_tx.send(Event::PositionChanged {
            position_secs: event.position.as_secs_f64(),
            duration_secs,
        });
    }

    fn save_state(&self) {
        if let Ok(conn) = open_db() {
            self.state.save(&conn);
        }
    }

    fn fetch_track(&self, track_id: &str) -> Option<Track> {
        // First check if the track already exists in queue manager
        if let Some(t) = self.queue.actual_queue.iter().find(|t| t.id == track_id) {
            return Some(t.clone());
        }
        if let Some(t) = self.queue.user_queue.iter().find(|t| t.id == track_id) {
            return Some(t.clone());
        }
        if let Some(t) = self.queue.suggested_queue.iter().find(|t| t.id == track_id) {
            return Some(t.clone());
        }

        // Check if track_id is a direct local file path
        let direct_path = std::path::Path::new(track_id);
        if direct_path.exists() {
            let filename = direct_path.file_stem().and_then(|s| s.to_str()).unwrap_or("Offline Track");
            return Some(Track {
                id: track_id.to_string(),
                title: filename.to_string(),
                artists: vec!["Offline".into()],
                album: "Downloads".into(),
                duration_secs: 180,
                source: TrackSource::Local(direct_path.to_path_buf()),
                isrc: None,
                artwork_url: None,
                content_hash: None,
            });
        }

        // Fallback for dynamic YouTube Music tracks clicked from UI feeds / queue
        let clean_id = track_id
            .trim_start_matches("ytm://")
            .trim_start_matches("https://music.youtube.com/watch?v=")
            .trim_start_matches("https://www.youtube.com/watch?v=")
            .to_string();

        if !clean_id.is_empty() {
            let source = TrackSource::YouTubeMusic { video_id: clean_id.clone() };

            Some(Track {
                id: clean_id.clone(),
                title: "Streaming Audio".into(),
                artists: vec!["YouTube Music".into()],
                album: "YouTube Music".into(),
                duration_secs: 180,
                source,
                isrc: None,
                artwork_url: None,
                content_hash: None,
            })
        } else {
            None
        }
    }

    async fn play_current(&mut self) {
        tracing::debug!(
            target: "engine",
            "play_current: cursor={}, actual_len={}",
            self.queue.cursor,
            self.queue.actual_queue.len()
        );
        if let Some(track) = self.queue.current_cloned() {
            // Instantly notify track change and buffering so UI updates without waiting for network resolve
            self.sync_and_broadcast_queue();
            let _ = self.event_tx.send(Event::TrackChanged(Some(track.clone())));
            self.current_playback_state = PlaybackState::Buffering;
            let _ = self.event_tx.send(Event::PlaybackStateChanged(PlaybackState::Buffering));

            tracing::info!(target: "engine", "Resolving track: {} ({:?})", track.title, track.source);
            let url = match self.resolver.resolve(&track).await {
                Ok(r) => {
                    tracing::debug!(target: "engine", "Resolved track to: {}", r.stream_url);
                    r.stream_url
                }
                Err(e) => {
                    tracing::error!(target: "engine", "Failed to resolve track: {}", e);
                    let _ = self.event_tx.send(Event::Error(EngineError {
                        kind: ErrorKind::Network,
                        message: format!("Failed to resolve track: {}", e),
                    }));
                    if self.queue.has_next() {
                        tracing::warn!(target: "engine", "Advancing past unplayable track to next in queue");
                        let _ = self.cmd_tx.send(Command { origin: None, action: CommandAction::Next }).await;
                    } else {
                        self.current_playback_state = PlaybackState::Stopped;
                        let _ = self.event_tx.send(Event::PlaybackStateChanged(PlaybackState::Stopped));
                    }
                    return;
                }
            };

            let _ = self.player.pause();
            if url.starts_with("http") {
                let _ = self.player.load_url(url, 0.0);
            } else {
                let _ = self.player.load_path(std::path::PathBuf::from(url), 0.0);
            }
            let _ = self.player.play();
            self.current_playback_state = PlaybackState::Playing;
            let _ = self.event_tx.send(Event::PlaybackStateChanged(PlaybackState::Playing));

            // Google History Beacon telemetry
            if let TrackSource::YouTubeMusic { ref video_id } = track.source {
                let ytm = Arc::clone(&self.ytm);
                let vid = video_id.clone();
                let ctx_id = self.queue.context_id.clone();
                tokio::spawn(async move {
                    let _ = ytm.register_playback_beacon(&vid, ctx_id.as_deref()).await;
                });
            }

            self.check_and_fetch_radio();
        }
    }

    #[allow(dead_code)]
    async fn play_current_index(&mut self) {
        self.play_current().await;
    }

    async fn play_next(&mut self) {
        if self.state.repeat == RepeatMode::One {
            let _ = self.player.seek(Duration::ZERO);
            let _ = self.player.play();
            return;
        }

        if let Some(_next) = self.queue.advance() {
            self.sync_and_broadcast_queue();
            self.play_current().await;
            self.check_and_fetch_radio();
        } else if self.state.repeat == RepeatMode::All && !self.queue.actual_queue.is_empty() {
            self.queue.cursor = 0;
            self.sync_and_broadcast_queue();
            self.play_current().await;
            self.check_and_fetch_radio();
        } else if self.queue.autoplay_enabled {
            tracing::info!(target: "engine", "play_next: queue exhausted but autoplay is enabled, fetching suggestions");
            self.check_and_fetch_radio();
        } else {
            let _ = self.player.pause();
            self.current_playback_state = PlaybackState::Stopped;
            let _ = self.event_tx.send(Event::PlaybackStateChanged(PlaybackState::Stopped));
        }
    }

    async fn play_prev(&mut self) {
        let pos_secs = self.clock.current_position().as_secs_f64();
        match self.queue.previous(pos_secs) {
            rustic_core::queue::PreviousAction::RestartTrack => {
                let _ = self.player.seek(Duration::ZERO);
                let _ = self.player.play();
                self.sync_and_broadcast_queue();
            }
            rustic_core::queue::PreviousAction::PlayTrack(_) => {
                self.sync_and_broadcast_queue();
                self.play_current().await;
            }
            rustic_core::queue::PreviousAction::None => {
                let _ = self.player.seek(Duration::ZERO);
                self.sync_and_broadcast_queue();
            }
        }
    }
}
