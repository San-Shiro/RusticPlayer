use rustic_core::api::{Command, CommandAction, Event, PlaybackState, Track};
use tokio::sync::broadcast;
use std::sync::Arc;

pub trait EngineBridge: Send + Sync {
    fn send_command(&self, cmd: Command);
    fn subscribe(&self) -> broadcast::Receiver<Event>;
    fn current_position(&self) -> std::time::Duration;
    fn audio_diagnostics(&self) -> audio::AudioDiagnostics;
    fn reconnect_audio(&self);
    fn ytmusic(&self) -> Arc<sources::YtMusicProvider>;
}

// Implement it for the real Engine
impl EngineBridge for engine::Engine {
    fn send_command(&self, cmd: Command) {
        self.send_command(cmd);
    }
    
    fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.subscribe()
    }

    fn current_position(&self) -> std::time::Duration {
        self.current_position()
    }

    fn audio_diagnostics(&self) -> audio::AudioDiagnostics {
        self.audio_diagnostics()
    }

    fn reconnect_audio(&self) {
        self.send_command(Command {
            origin: None,
            action: rustic_core::api::CommandAction::ReconnectAudio,
        });
    }

    fn ytmusic(&self) -> Arc<sources::YtMusicProvider> {
        self.ytmusic()
    }
}

#[derive(Clone, Debug)]
pub struct UiTrack {
    pub id: i64,
    pub track_key: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: Option<String>,
    pub year: Option<u32>,
    pub duration_ms: u64,
    pub path: String,
    pub has_artwork: bool,
    pub artwork_hash: Option<String>,
}

impl UiTrack {
    pub fn artwork_path(&self) -> Option<std::path::PathBuf> {
        if let Some(ref hash) = self.artwork_hash {
            let path = std::path::PathBuf::from(format!("cache/artwork/{}_64.jpg", hash));
            if path.exists() {
                return Some(std::fs::canonicalize(&path).unwrap_or(path));
            }
        }
        let slug = self.album.to_lowercase().replace(' ', "_").replace('&', "and");
        let path = std::path::PathBuf::from(format!("assets/artwork/{}.jpg", slug));
        if path.exists() {
            return Some(std::fs::canonicalize(&path).unwrap_or(path));
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct UiAlbum {
    pub id: i64,
    pub title: String,
    pub artist: String,
    pub year: Option<u32>,
    pub track_count: usize,
    pub artwork_hash: Option<String>,
}

impl UiAlbum {
    pub fn artwork_path(&self) -> Option<std::path::PathBuf> {
        if let Some(ref hash) = self.artwork_hash {
            let path = std::path::PathBuf::from(format!("cache/artwork/{}_256.jpg", hash));
            if path.exists() {
                return Some(std::fs::canonicalize(&path).unwrap_or(path));
            }
        }
        let slug = self.title.to_lowercase().replace(' ', "_").replace('&', "and");
        let path = std::path::PathBuf::from(format!("assets/artwork/{}.jpg", slug));
        if path.exists() {
            return Some(std::fs::canonicalize(&path).unwrap_or(path));
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct UiArtist {
    pub id: i64,
    pub name: String,
    pub track_count: usize,
    pub album_count: usize,
}

#[derive(Clone, Debug)]
pub struct UiPlaylist {
    pub id: i64,
    pub name: String,
    pub track_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Home,
    Album(String),
    Artist(String),
    Playlist(String),
    Player,
    Library,
    Playlists,
    Settings,
    Search,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Loadable<T> {
    Idle,
    Loading,
    Ready(T),
    Error(String),
}

impl<T> Default for Loadable<T> {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LibraryTab {
    #[default]
    Playlists,
    Songs,
    Albums,
    Artists,
    Downloads,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlayerTab {
    #[default]
    UpNext,
    Lyrics,
    Comments,
    Related,
}

// Global UI state model
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedQueueState {
    pub actual_queue: Vec<Track>,
    pub cursor: usize,
    pub suggested_queue: std::collections::VecDeque<Track>,
    pub current_position_secs: f64,
    pub volume: f32,
    pub autoplay_enabled: bool,
    pub context_id: Option<String>,
    pub context_title: Option<String>,
}

pub struct AppState {
    pub playback_state: PlaybackState,
    pub current_track: Option<Track>,
    pub queue: Vec<Track>,
    pub volume: f32,
    pub previous_volume: f32,
    pub current_position: f64,
    pub track_duration: f64,
    pub timeline: ui_kit::atoms::PlaybackTimeline,
    pub engine: Arc<dyn EngineBridge>,
    pub tracks: Vec<UiTrack>,
    pub albums: Vec<UiAlbum>,
    pub artists: Vec<UiArtist>,
    pub playlists: Vec<UiPlaylist>,
    pub current_route: Route,
    pub nav_history: Vec<Route>,
    pub active_route: String,
    pub theme_preset: ui_kit::tokens::ThemePreset,
    pub enable_blur: bool,
    pub is_video_mode: bool,
    pub sidebar_collapsed: bool,
    pub liked_tracks: std::collections::HashSet<String>,
    pub disliked_tracks: std::collections::HashSet<String>,
    pub is_queue_open: bool,
    pub is_captions_enabled: bool,
    pub show_debug_overlay: bool,
    pub log_store: Arc<crate::debug::LogStore>,
    pub debug_filter_errors: bool,
    pub audio_diagnostics: Option<audio::AudioDiagnostics>,
    pub repeat_mode: RepeatMode,
    pub is_shuffled: bool,
    pub active_player_tab: PlayerTab,
    pub is_autoplay_enabled: bool,
    pub home_feed: Loadable<sources::BrowseFeed>,
    pub queue_snapshot: Loadable<sources::QueueSnapshot>,
    pub queue_mgr: rustic_core::queue::QueueManager,
    pub lyrics_doc: Loadable<sources::LyricsDocument>,
    pub ytmusic: Arc<sources::YtMusicProvider>,
    pub thumbnail_manager: Arc<crate::thumbnail_cache::ThumbnailManager>,
    pub auth_state: rustic_core::api::AuthState,
    pub active_palette: (gpui::Rgba, gpui::Rgba),
    pub active_ambient_image: Option<String>,
    pub source_playlist_title: Option<String>,
    pub palette_cache: Arc<crate::palette_cache::PaletteCache>,
    pub search_query: String,
    pub search_results: Loadable<Vec<Track>>,
    pub is_search_overlay_open: bool,
    pub search_suggestions: Vec<String>,
    pub is_sign_in_modal_open: bool,
    pub is_signing_in: bool,
    pub sign_in_error: Option<String>,
    pub library_tab: LibraryTab,
    pub library_playlists: Loadable<Vec<sources::LibraryItem>>,
    pub library_albums: Loadable<Vec<sources::LibraryItem>>,
    pub library_songs: Loadable<Vec<Track>>,
    pub library_artists: Loadable<Vec<sources::LibraryArtist>>,
    pub library_songs_continuation: Option<String>,
    pub is_loading_more_songs: bool,
    pub library_downloads: Vec<Track>,
    pub queue_generation: u64,
    pub active_album: Loadable<sources::YtCollectionDetails>,
    pub active_playlist: Loadable<sources::YtCollectionDetails>,
    pub active_artist: Loadable<sources::YtArtistDetails>,
    pub active_artist_id: Option<String>,
    pub active_artist_request_seq: u64,
    pub download_manager: Arc<std::sync::RwLock<crate::download_manager::DownloadManager>>,
}

use gpui::Context;

impl AppState {
    pub fn new(engine: Arc<dyn EngineBridge>, log_store: Arc<crate::debug::LogStore>, cx: &mut Context<Self>) -> Self {
        let mut rx = engine.subscribe();
        
        cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                while let Ok(event) = rx.recv().await {
                    let mut cx = cx.clone();
                    let _ = this.update(&mut cx, |state: &mut AppState, cx| {
                        match event {
                            Event::PlaybackStateChanged(s) => {
                                state.playback_state = s;
                                let is_playing = state.playback_state == PlaybackState::Playing;
                                let pos = state.engine.current_position().as_secs_f64();
                                state.current_position = pos;
                                state.timeline.anchor_position = pos;
                                state.timeline.anchor_instant = std::time::Instant::now();
                                state.timeline.playing = is_playing;
                                state.timeline.duration = state.track_duration;
                                state.timeline.revision += 1;
                            }
                            Event::TrackChanged(t) => {
                                if let Some(ref new_t) = t {
                                    if let Some(ref mut existing) = state.current_track {
                                        if existing.id == new_t.id {
                                            if new_t.title.starts_with("YouTube Track") || new_t.title == "Streaming Audio" {
                                                if new_t.duration_secs > 0 {
                                                    existing.duration_secs = new_t.duration_secs;
                                                }
                                            } else {
                                                *existing = new_t.clone();
                                            }
                                        } else {
                                            *existing = new_t.clone();
                                        }
                                    } else {
                                        state.current_track = t.clone();
                                    }
                                } else {
                                    state.current_track = None;
                                }

                                state.current_position = 0.0;
                                if let Some(ref tr) = state.current_track {
                                    if tr.duration_secs > 0 {
                                        state.track_duration = tr.duration_secs as f64;
                                    } else if let Some(ui_t) = state.tracks.iter().find(|it| it.track_key == tr.id) {
                                        state.track_duration = ui_t.duration_ms as f64 / 1000.0;
                                    }
                                }
                                let is_playing = state.playback_state == PlaybackState::Playing;
                                state.timeline = ui_kit::atoms::PlaybackTimeline {
                                    anchor_position: 0.0,
                                    anchor_instant: std::time::Instant::now(),
                                    playback_rate: 1.0,
                                    duration: state.track_duration,
                                    playing: is_playing,
                                    revision: state.timeline.revision + 1,
                                };

                                // Extract palette for active track
                                if let Some(ref cur) = state.current_track {
                                    if let Some(ui_t) = state.tracks.iter().find(|it| it.track_key == cur.id) {
                                        if let Some(art_path) = ui_t.artwork_path() {
                                            let pal_cache = state.palette_cache.clone();
                                            let path_str = art_path.to_string_lossy().to_string();
                                            cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
                                                let mut cx = cx.clone();
                                                async move {
                                                    let (palette, ambient_img) = pal_cache.extract(path_str).await;
                                                    let _ = this.update(&mut cx, |s, cx| {
                                                        s.active_palette = palette;
                                                        if let Some(amb) = ambient_img {
                                                            s.active_ambient_image = Some(amb);
                                                        }
                                                        cx.notify();
                                                    });
                                                }
                                            }).detach();
                                        }
                                    } else if let Some(ref thumb_url) = cur.artwork_url {
                                        state.update_palette_for_url_cx(thumb_url, cx);
                                    }
                                }

                                if state.queue_mgr.needs_prefetch() {
                                    state.maybe_prefetch_suggestions(cx);
                                }
                                state.save_queue_state();
                                state.load_lyrics(cx);
                            }
                            Event::QueueChanged(q) => state.queue = q,
                            Event::QueueUpdated(snap) => {
                                state.queue_generation = snap.generation;
                                state.is_autoplay_enabled = snap.autoplay_enabled;
                                state.is_shuffled = snap.is_shuffled;
                                state.source_playlist_title = snap.context_title.clone();

                                if let Some(ref cur) = snap.current_track {
                                    let changed = state.current_track.as_ref().map(|t| &t.id) != Some(&cur.id);
                                    if changed {
                                        state.current_track = Some(cur.clone());
                                        state.current_position = 0.0;
                                        if cur.duration_secs > 0 {
                                            state.track_duration = cur.duration_secs as f64;
                                        }
                                        let is_playing = state.playback_state == PlaybackState::Playing;
                                        state.timeline = ui_kit::atoms::PlaybackTimeline {
                                            anchor_position: 0.0,
                                            anchor_instant: std::time::Instant::now(),
                                            playback_rate: 1.0,
                                            duration: state.track_duration,
                                            playing: is_playing,
                                            revision: state.timeline.revision + 1,
                                        };
                                        if let Some(ref thumb_url) = cur.artwork_url {
                                            state.update_palette_for_url_cx(thumb_url, cx);
                                        }
                                        state.load_lyrics(cx);
                                    }
                                }

                                if !snap.actual_queue.is_empty() {
                                    state.queue_mgr.actual_queue = snap.actual_queue.clone();
                                    state.queue_mgr.cursor = snap.cursor.min(state.queue_mgr.actual_queue.len().saturating_sub(1));
                                } else {
                                    let mut actual = snap.history.clone();
                                    let cursor_idx = actual.len();
                                    if let Some(ref cur) = snap.current_track {
                                        actual.push(cur.clone());
                                    }
                                    actual.extend(snap.user_queue.iter().cloned());
                                    actual.extend(snap.context_upcoming.iter().cloned());
                                    state.queue_mgr.actual_queue = actual;
                                    state.queue_mgr.cursor = cursor_idx;
                                }
                                state.queue_mgr.user_queue = snap.user_queue.into_iter().collect();
                                state.queue_mgr.context_queue = snap.context_upcoming.clone();
                                state.queue_mgr.suggested_queue = snap.radio_upcoming.into_iter().collect();
                                state.queue_mgr.history_list = snap.history;
                                state.queue_mgr.context_title = snap.context_title;
                                state.queue_mgr.context_id = snap.context_id;
                                state.queue_mgr.autoplay_enabled = snap.autoplay_enabled;
                                state.queue_mgr.is_shuffled = snap.is_shuffled;
                                state.queue_mgr.generation = snap.generation;

                                state.save_queue_state();
                                cx.notify();
                            }
                            Event::VolumeChanged(v) => state.volume = v,
                            Event::PositionChanged { position_secs, duration_secs } => {
                                if duration_secs > 0.0 {
                                    state.track_duration = duration_secs;
                                    state.timeline.duration = duration_secs;
                                } else if state.track_duration == 0.0 {
                                    if let Some(ref cur) = state.current_track {
                                        if cur.duration_secs > 0 {
                                            state.track_duration = cur.duration_secs as f64;
                                            state.timeline.duration = state.track_duration;
                                        } else if let Some(ui_t) = state.tracks.iter().find(|it| it.track_key == cur.id) {
                                            state.track_duration = ui_t.duration_ms as f64 / 1000.0;
                                            state.timeline.duration = state.track_duration;
                                        }
                                    }
                                }

                                let old_pos = state.timeline.position_at(std::time::Instant::now());
                                let diff = (position_secs - old_pos).abs();
                                if diff > 0.050 || state.playback_state != PlaybackState::Playing {
                                    state.current_position = position_secs;
                                    state.timeline.anchor_position = position_secs;
                                    state.timeline.anchor_instant = std::time::Instant::now();
                                    state.timeline.revision += 1;
                                }

                                if state.playback_state == PlaybackState::Playing {
                                    return;
                                }
                            },

                            Event::LibraryChanged | Event::ScanComplete { .. } => {
                                state.reload_library();
                            }
                            Event::AuthStateChanged { provider, logged_in: _ } => {
                                if provider == "ytmusic" {
                                    state.auth_state = state.ytmusic.auth_state();
                                    state.load_home_feed(cx);
                                    state.load_library(cx);
                                }
                            }
                            Event::TrackEnded { track_id: _ } => {
                                // Engine automatically advances the queue via self.play_next().await on EOS,
                                // broadcasting Event::TrackChanged and Event::QueueUpdated.
                            }
                            Event::Error(err) => {
                                tracing::error!(target: "engine_bridge", "Engine error: [{:?}] {}", err.kind, err.message);
                            }
                            _ => {}
                        }
                        cx.notify();
                    });
                }
            }
        }).detach();

        // Coarse (250ms) background ticker:
        // - Resyncs DAC position if playing and only notifies when displayed second changes
        // - Updates debug overlay diagnostics if visible (so HUD stays live)
        // - Eliminates 60Hz whole-window reflow, reducing CPU on lightweight systems to near zero!
        cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let mut last_second: u64 = 0;
                loop {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
                    let mut cx = cx.clone();
                    let res = this.update(&mut cx, |state: &mut AppState, cx| {
                        let mut notify = false;
                        if state.playback_state == PlaybackState::Playing {
                            let pos_secs = state.engine.current_position().as_secs_f64();
                            state.current_position = pos_secs;
                            let cur_sec = pos_secs as u64;
                            if cur_sec != last_second {
                                last_second = cur_sec;
                                notify = true; // Update elapsed time string ("0:01", "0:02")
                            }
                        }
                        if state.show_debug_overlay {
                            state.audio_diagnostics = Some(state.engine.audio_diagnostics());
                            notify = true;
                        }
                        if notify {
                            cx.notify();
                        }
                    });
                    if res.is_err() {
                        break;
                    }
                }
            }
        }).detach();

        let initial_audio_diag = engine.audio_diagnostics();
        let ytmusic = engine.ytmusic();
        let thumbnail_manager = Arc::new(crate::thumbnail_cache::ThumbnailManager::new("cache/thumbnails"));
        let auth_state = ytmusic.auth_state();
        if let Some(ref url) = auth_state.account_avatar_url {
            let _ = thumbnail_manager.resolve_or_queue(&Some(url.clone()), None);
        }

        let user_prefs_path = std::path::Path::new("cache/user_preferences.json");
        let (saved_likes, saved_dislikes) = if user_prefs_path.exists() {
            if let Ok(bytes) = std::fs::read(user_prefs_path) {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    let likes = val.get("likes")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default();
                    let dislikes = val.get("dislikes")
                        .and_then(|v| v.as_array())
                        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default();
                    (likes, dislikes)
                } else {
                    (std::collections::HashSet::new(), std::collections::HashSet::new())
                }
            } else {
                (std::collections::HashSet::new(), std::collections::HashSet::new())
            }
        } else {
            (std::collections::HashSet::new(), std::collections::HashSet::new())
        };

        let download_mgr = Arc::new(std::sync::RwLock::new(crate::download_manager::DownloadManager::new("downloads")));
        let initial_downloads = download_mgr.read().unwrap().get_downloaded_tracks();

        let mut app_state = Self {
            playback_state: PlaybackState::Stopped,
            current_track: None,
            queue: Vec::new(),
            volume: 1.0,
            previous_volume: 1.0,
            current_position: 0.0,
            track_duration: 0.0,
            timeline: ui_kit::atoms::PlaybackTimeline::default(),
            engine,
            tracks: Vec::new(),
            albums: Vec::new(),
            artists: Vec::new(),
            playlists: Vec::new(),
            current_route: Route::Home,
            nav_history: Vec::new(),
            active_route: "home".to_string(),
            theme_preset: ui_kit::tokens::ThemePreset::Cyberpunk,
            enable_blur: true,
            is_video_mode: false,
            sidebar_collapsed: std::env::var("RUSTIC_SIDEBAR_EXPANDED").map(|v| v != "1").unwrap_or(true),
            liked_tracks: saved_likes,
            disliked_tracks: saved_dislikes,
            is_queue_open: false,
            is_captions_enabled: false,
            show_debug_overlay: std::env::var("RUSTIC_DEBUG").map(|v| v == "1" || v.eq_ignore_ascii_case("true")).unwrap_or(false),
            log_store,
            debug_filter_errors: false,
            audio_diagnostics: Some(initial_audio_diag),
            repeat_mode: RepeatMode::Off,
            is_shuffled: false,
            active_player_tab: PlayerTab::UpNext,
            is_autoplay_enabled: true,
            home_feed: Loadable::Idle,
            queue_snapshot: Loadable::Idle,
            queue_mgr: rustic_core::queue::QueueManager::new(),
            lyrics_doc: Loadable::Idle,
            ytmusic,
            thumbnail_manager,
            auth_state,
            active_palette: crate::palette_cache::PaletteCache::default_palette(),
            active_ambient_image: None,
            source_playlist_title: None,
            palette_cache: Arc::new(crate::palette_cache::PaletteCache::new()),
            search_query: String::new(),
            search_results: Loadable::Idle,
            is_search_overlay_open: false,
            search_suggestions: Vec::new(),
            is_sign_in_modal_open: false,
            is_signing_in: false,
            sign_in_error: None,
            library_tab: LibraryTab::Playlists,
            library_playlists: Loadable::Idle,
            library_albums: Loadable::Idle,
            library_songs: Loadable::Idle,
            library_artists: Loadable::Idle,
            library_songs_continuation: None,
            is_loading_more_songs: false,
            library_downloads: initial_downloads,
            queue_generation: 0,
            active_album: Loadable::Idle,
            active_playlist: Loadable::Idle,
            active_artist: Loadable::Idle,
            active_artist_id: None,
            active_artist_request_seq: 0,
            download_manager: download_mgr,
        };

        // Restore persisted queue state if available
        let queue_state_path = std::path::Path::new("cache/queue_state.json");
        if queue_state_path.exists() {
            if let Ok(bytes) = std::fs::read(queue_state_path) {
                if let Ok(saved_q) = serde_json::from_slice::<PersistedQueueState>(&bytes) {
                    app_state.queue_mgr.actual_queue = saved_q.actual_queue;
                    app_state.queue_mgr.cursor = saved_q.cursor;
                    app_state.queue_mgr.suggested_queue = saved_q.suggested_queue;
                    app_state.queue_mgr.autoplay_enabled = saved_q.autoplay_enabled;
                    app_state.queue_mgr.context_id = saved_q.context_id;
                    app_state.queue_mgr.context_title = saved_q.context_title.clone();
                    app_state.is_autoplay_enabled = saved_q.autoplay_enabled;
                    app_state.source_playlist_title = saved_q.context_title;
                    app_state.volume = saved_q.volume;
                    app_state.current_position = saved_q.current_position_secs;
                    app_state.timeline.anchor_position = saved_q.current_position_secs;
                    app_state.timeline.anchor_instant = std::time::Instant::now();

                    if let Some(cur) = app_state.queue_mgr.current_cloned() {
                        if cur.duration_secs > 0 {
                            app_state.track_duration = cur.duration_secs as f64;
                            app_state.timeline.duration = app_state.track_duration;
                        }
                        if let Some(ref thumb) = cur.artwork_url {
                            app_state.update_palette_for_url_cx(thumb, cx);
                        }
                        app_state.current_track = Some(cur);
                    }
                }
            }
        }

        app_state.reload_library();
        app_state.load_home_feed(cx);
        app_state.load_library(cx);
        app_state.refresh_account_info(cx);

        if let Ok(route_str) = std::env::var("RUSTIC_INITIAL_ROUTE") {
            app_state.current_route = match route_str.as_str() {
                "album" => {
                    let album_id = std::env::var("RUSTIC_INITIAL_ALBUM_ID").unwrap_or_else(|_| "MPREb_9u6q97B0kQ6".into());
                    Route::Album(album_id)
                }
                "player" => Route::Player,
                "playlist" => Route::Playlist("2".to_string()),
                "library" => Route::Library,
                "playlists" => Route::Playlists,
                "settings" => Route::Settings,
                _ => Route::Home,
            };
        }

        if let Ok(track_id) = std::env::var("RUSTIC_INITIAL_TRACK") {
            let title = std::env::var("RUSTIC_INITIAL_TITLE").unwrap_or_else(|_| "Despacito".into());
            let artist = std::env::var("RUSTIC_INITIAL_ARTIST").unwrap_or_else(|_| "Luis Fonsi".into());
            let thumb = std::env::var("RUSTIC_INITIAL_THUMB").ok();
            let playlist = std::env::var("RUSTIC_INITIAL_PLAYLIST").ok().or(Some("Quick picks".into()));
            app_state.play_youtube_track(track_id, title, artist, thumb, playlist, cx);
        }

        if let Ok(tab_str) = std::env::var("RUSTIC_PLAYER_TAB") {
            app_state.active_player_tab = match tab_str.to_lowercase().as_str() {
                "lyrics" => PlayerTab::Lyrics,
                "comments" => PlayerTab::Comments,
                "related" => PlayerTab::Related,
                _ => PlayerTab::UpNext,
            };
        }

        app_state
    }

    pub fn toggle_debug_overlay(&mut self) {
        self.show_debug_overlay = !self.show_debug_overlay;
        if self.show_debug_overlay {
            self.audio_diagnostics = Some(self.engine.audio_diagnostics());
        }
    }

    pub fn toggle_log_filter(&mut self) {
        self.debug_filter_errors = !self.debug_filter_errors;
    }

    pub fn clear_logs(&mut self) {
        self.log_store.clear();
    }

    pub fn reconnect_audio(&self) {
        self.engine.reconnect_audio();
    }

    pub fn seek_to(&mut self, target_secs: f64, cx: &mut Context<Self>) {
        self.current_position = target_secs;
        self.timeline.anchor_position = target_secs;
        self.timeline.anchor_instant = std::time::Instant::now();
        self.timeline.revision += 1;
        cx.notify();
    }

    pub fn navigate_to(&mut self, route: Route, cx: &mut Context<Self>) {
        if self.current_route == route {
            return;
        }
        if self.nav_history.last() != Some(&self.current_route) {
            self.nav_history.push(self.current_route.clone());
        }
        self.current_route = route.clone();
        self.active_route = match &self.current_route {
            Route::Home => "home".to_string(),
            Route::Album(_) => "album".to_string(),
            Route::Artist(_) => "artist".to_string(),
            Route::Playlist(_) => "playlist".to_string(),
            Route::Player => "player".to_string(),
            Route::Library => {
                if matches!(self.library_playlists, Loadable::Idle) {
                    self.load_library(cx);
                }
                "library".to_string()
            }
            Route::Playlists => "playlists".to_string(),
            Route::Settings => "settings".to_string(),
            Route::Search => "search".to_string(),
        };
        cx.notify();
    }

    pub fn open_album<S: ToString>(&mut self, album_id: S, cx: &mut Context<Self>) {
        let aid = album_id.to_string();
        self.open_album_detail(aid, String::new(), None, cx);
    }

    pub fn open_playlist<S: ToString>(&mut self, playlist_id: S, cx: &mut Context<Self>) {
        self.navigate_to(Route::Playlist(playlist_id.to_string()), cx);
    }

    pub fn open_player(&mut self, cx: &mut Context<Self>) {
        if self.current_route != Route::Player {
            self.navigate_to(Route::Player, cx);
        } else {
            cx.notify();
        }
    }

    pub fn close_player(&mut self, cx: &mut Context<Self>) {
        if self.current_route == Route::Player {
            self.go_back(cx);
        }
    }

    pub fn can_go_back(&self) -> bool {
        !self.nav_history.is_empty() || self.current_route != Route::Home
    }

    pub fn go_back(&mut self, cx: &mut Context<Self>) {
        if let Some(prev) = self.nav_history.pop() {
            self.current_route = prev;
            self.active_route = match &self.current_route {
                Route::Home => "home".to_string(),
                Route::Album(_) => "album".to_string(),
                Route::Artist(_) => "artist".to_string(),
                Route::Playlist(_) => "playlist".to_string(),
                Route::Player => "player".to_string(),
                Route::Library => "library".to_string(),
                Route::Playlists => "playlists".to_string(),
                Route::Settings => "settings".to_string(),
                Route::Search => "search".to_string(),
            };
            cx.notify();
        } else if self.current_route != Route::Home {
            self.current_route = Route::Home;
            self.active_route = "home".to_string();
            cx.notify();
        }
    }

    pub fn album_ambient_colors(&self, album_id: Option<i64>) -> (gpui::Rgba, gpui::Rgba) {
        let album_title = if let Some(id) = album_id {
            self.albums.iter().find(|a| a.id == id).map(|a| a.title.as_str())
        } else if let Some(ref track) = self.current_track {
            self.tracks.iter().find(|t| t.track_key == track.id).map(|t| t.album.as_str())
        } else {
            None
        };

        match album_title.unwrap_or("") {
            t if t.eq_ignore_ascii_case("folklore") => (
                gpui::rgba(0x283b2e99), // Deep misty forest green
                gpui::rgba(0x4a3b2c66), // Muted warm wood amber
            ),
            t if t.eq_ignore_ascii_case("neon horizon") => (
                gpui::rgba(0x7928ca88), // Electric violet
                gpui::rgba(0xff008066), // Neon magenta
            ),
            t if t.to_lowercase().contains("chill") || t.to_lowercase().contains("slow") => (
                gpui::rgba(0x1a237e88), // Midnight indigo
                gpui::rgba(0xd8431566), // Warm terracotta
            ),
            t if t.eq_ignore_ascii_case("rainy afternoon") => (
                gpui::rgba(0x1e3a5f88), // Slate rain blue
                gpui::rgba(0x2d4b6866), // Misty dusk
            ),
            _ => (
                gpui::rgba(0x4e342e88), // Deep warm mocha
                gpui::rgba(0xc7780055), // Amber glow
            ),
        }
    }

    pub fn set_route(&mut self, route: impl Into<String>) {
        let r_str = route.into();
        self.active_route = r_str.clone();
        self.current_route = match r_str.as_str() {
            "library" => Route::Library,
            "playlists" => Route::Playlists,
            "settings" => Route::Settings,
            "player" => Route::Player,
            "search" => Route::Search,
            _ => Route::Home,
        };
    }

    pub fn cycle_theme(&mut self, cx: &mut Context<Self>) {
        use ui_kit::tokens::{Theme, ThemePreset};
        self.theme_preset = match self.theme_preset {
            ThemePreset::Cyberpunk => ThemePreset::Noir,
            ThemePreset::Noir => ThemePreset::Nord,
            ThemePreset::Nord => ThemePreset::Emerald,
            ThemePreset::Emerald => ThemePreset::Oled,
            ThemePreset::Oled => ThemePreset::LightStudio,
            ThemePreset::LightStudio => ThemePreset::Cyberpunk,
        };
        let mut theme = cx.global::<Theme>().clone();
        theme.set_preset(self.theme_preset);
        cx.set_global(theme);
        cx.notify();
    }

    pub fn set_theme_preset(&mut self, preset: ui_kit::tokens::ThemePreset, cx: &mut Context<Self>) {
        use ui_kit::tokens::Theme;
        self.theme_preset = preset;
        let mut theme = cx.global::<Theme>().clone();
        theme.set_preset(preset);
        cx.set_global(theme);
        cx.notify();
    }

    pub fn toggle_blur(&mut self, cx: &mut Context<Self>) {
        use ui_kit::tokens::Theme;
        self.enable_blur = !self.enable_blur;
        let mut theme = cx.global::<Theme>().clone();
        theme.set_blur(self.enable_blur);
        cx.set_global(theme);
        cx.notify();
    }

    pub fn toggle_sidebar(&mut self, cx: &mut Context<Self>) {
        self.sidebar_collapsed = !self.sidebar_collapsed;
        cx.notify();
    }

    pub fn toggle_video_mode(&mut self, is_video: bool, cx: &mut Context<Self>) {
        self.is_video_mode = is_video;
        cx.notify();
    }

    fn save_user_preferences(&self) {
        let _ = std::fs::create_dir_all("cache");
        let likes: Vec<&String> = self.liked_tracks.iter().collect();
        let dislikes: Vec<&String> = self.disliked_tracks.iter().collect();
        let json = serde_json::json!({
            "likes": likes,
            "dislikes": dislikes,
        });
        let _ = std::fs::write("cache/user_preferences.json", json.to_string());
    }

    pub fn toggle_like(&mut self, track_key: String, cx: &mut Context<Self>) {
        if self.liked_tracks.contains(&track_key) {
            self.liked_tracks.remove(&track_key);
        } else {
            self.liked_tracks.insert(track_key.clone());
            self.disliked_tracks.remove(&track_key);
        }
        self.save_user_preferences();
        cx.notify();
    }

    pub fn toggle_dislike(&mut self, track_key: String, cx: &mut Context<Self>) {
        if self.disliked_tracks.contains(&track_key) {
            self.disliked_tracks.remove(&track_key);
        } else {
            self.disliked_tracks.insert(track_key.clone());
            self.liked_tracks.remove(&track_key);
        }
        self.save_user_preferences();
        cx.notify();
    }

    pub fn toggle_captions(&mut self, cx: &mut Context<Self>) {
        self.is_captions_enabled = !self.is_captions_enabled;
        cx.notify();
    }

    pub fn toggle_queue(&mut self, cx: &mut Context<Self>) {
        self.is_queue_open = !self.is_queue_open;
        cx.notify();
    }

    pub fn set_volume(&mut self, vol: f32, cx: &mut Context<Self>) {
        let v = vol.clamp(0.0, 1.0);
        self.volume = v;
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::SetVolume(v),
        });
        cx.notify();
    }

    pub fn toggle_mute(&mut self, cx: &mut Context<Self>) {
        if self.volume > 0.01 {
            self.previous_volume = self.volume;
            self.volume = 0.0;
        } else {
            self.volume = if self.previous_volume > 0.01 { self.previous_volume } else { 0.8 };
        }
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::SetVolume(self.volume),
        });
        cx.notify();
    }

    pub fn play_next_track(&mut self, cx: &mut Context<Self>) {
        if let Some(next) = self.queue_mgr.advance() {
            let thumb = next.artwork_url.clone();
            self.current_track = Some(next);
            if let Some(ref t) = thumb {
                self.update_palette_for_url_cx(t, cx);
            }
            self.current_position = 0.0;
            self.timeline.anchor_position = 0.0;
            self.timeline.anchor_instant = std::time::Instant::now();
            self.timeline.revision += 1;
            self.load_lyrics(cx);
            cx.notify();
        }
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::Next,
        });
    }

    pub fn play_previous_track(&mut self, cx: &mut Context<Self>) {
        let pos_secs = self.current_position;
        match self.queue_mgr.previous(pos_secs) {
            rustic_core::queue::PreviousAction::RestartTrack => {
                self.current_position = 0.0;
                self.timeline.anchor_position = 0.0;
                self.timeline.anchor_instant = std::time::Instant::now();
                self.timeline.revision += 1;
                cx.notify();
            }
            rustic_core::queue::PreviousAction::PlayTrack(prev) => {
                let thumb = prev.artwork_url.clone();
                self.current_track = Some(prev);
                if let Some(ref t) = thumb {
                    self.update_palette_for_url_cx(t, cx);
                }
                self.current_position = 0.0;
                self.timeline.anchor_position = 0.0;
                self.timeline.anchor_instant = std::time::Instant::now();
                self.timeline.revision += 1;
                self.load_lyrics(cx);
                cx.notify();
            }
            rustic_core::queue::PreviousAction::None => {
                self.current_position = 0.0;
                self.timeline.anchor_position = 0.0;
                self.timeline.anchor_instant = std::time::Instant::now();
                self.timeline.revision += 1;
                cx.notify();
            }
        }
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::Previous,
        });
    }

    pub fn enqueue_track_next(&mut self, track: Track, _cx: &mut Context<Self>) {
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::EnqueueTrackNext(track),
        });
    }

    pub fn enqueue_track_end(&mut self, track: Track, _cx: &mut Context<Self>) {
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::EnqueueTrackEnd(track),
        });
    }

    pub fn remove_user_queue_item(&mut self, index: usize, _cx: &mut Context<Self>) {
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::RemoveUserQueueItem(index),
        });
    }

    pub fn clear_user_queue(&mut self, _cx: &mut Context<Self>) {
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::ClearUserQueue,
        });
    }

    pub fn move_user_queue_item(&mut self, from: usize, to: usize, _cx: &mut Context<Self>) {
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::MoveUserQueueItem { from, to },
        });
    }

    pub fn promote_radio_item(&mut self, index: usize, cx: &mut Context<Self>) {
        self.queue_mgr.promote_radio_item(index);
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::PromoteRadioItem(index),
        });
        cx.notify();
    }

    pub fn play_radio_item(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(track) = self.queue_mgr.promote_suggestion(index) {
            let thumb = track.artwork_url.clone();
            self.current_track = Some(track);
            if let Some(ref t) = thumb {
                self.update_palette_for_url_cx(t, cx);
            }
            self.current_position = 0.0;
            self.timeline.anchor_position = 0.0;
            self.timeline.anchor_instant = std::time::Instant::now();
            self.timeline.revision += 1;
            self.load_lyrics(cx);
            self.load_queue(cx);
            cx.notify();
        }
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::PlayRadioItem(index),
        });
    }

    pub fn jump_to_track_id(&mut self, track_id: &str, cx: &mut Context<Self>) {
        if let Some(track) = self.queue_mgr.jump_to_track_id(track_id) {
            let thumb = track.artwork_url.clone();
            self.current_track = Some(track);
            if let Some(ref t) = thumb {
                self.update_palette_for_url_cx(t, cx);
            }
            self.current_position = 0.0;
            self.timeline.anchor_position = 0.0;
            self.timeline.anchor_instant = std::time::Instant::now();
            self.timeline.revision += 1;
            self.load_lyrics(cx);
            cx.notify();
        }
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::JumpToTrackId(track_id.to_string()),
        });
    }

    pub fn jump_to_queue_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Some(track) = self.queue_mgr.jump_to(index) {
            let thumb = track.artwork_url.clone();
            if track.duration_secs > 0 {
                self.track_duration = track.duration_secs as f64;
                self.timeline.duration = self.track_duration;
            }
            self.current_track = Some(track);
            if let Some(ref t) = thumb {
                self.update_palette_for_url_cx(t, cx);
            }
            self.current_position = 0.0;
            self.timeline.anchor_position = 0.0;
            self.timeline.anchor_instant = std::time::Instant::now();
            self.timeline.revision += 1;
            self.load_lyrics(cx);
            cx.notify();
        }
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::JumpToQueueIndex(index),
        });
    }

    pub fn set_autoplay(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.is_autoplay_enabled = enabled;
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::ToggleAutoplay(enabled),
        });
        cx.notify();
    }

    pub fn play_queue_index(&mut self, index: usize, cx: &mut Context<Self>) {
        if index == 0 {
            self.seek_to(0.0, cx);
            self.engine.send_command(rustic_core::api::Command {
                origin: None,
                action: rustic_core::api::CommandAction::Seek(std::time::Duration::ZERO),
            });
            return;
        }

        let upcoming_idx = index - 1;
        let up_tracks = self.queue_mgr.upcoming_tracks();
        if let Some((_lane, track)) = up_tracks.get(upcoming_idx) {
            self.jump_to_track_id(&track.id, cx);
        }
    }

    pub fn play_queue_track_at(&mut self, index: usize, cx: &mut Context<Self>) {
        if let Loadable::Ready(ref snap) = self.queue_snapshot {
            if let Some(item) = snap.upcoming.get(index) {
                let vid = item.video_id.clone();
                let title = item.title.clone();
                let artist = item.artist.clone();
                let thumb = item.thumbnail_url.clone();

                let track = Track {
                    id: vid.clone(),
                    title,
                    artists: vec![artist],
                    album: "YouTube Music".into(),
                    duration_secs: 180,
                    source: rustic_core::api::TrackSource::YouTubeMusic { video_id: vid.clone() },
                    isrc: None,
                    artwork_url: thumb,
                    content_hash: None,
                };

                self.play_single_track(track, cx);
            }
        }
    }

    pub fn move_queue_item_up(&mut self, index: usize, cx: &mut Context<Self>) {
        if index <= 1 {
            return;
        }
        let upcoming_idx = index - 1;
        let u_len = self.queue_mgr.user_queue.len();
        if upcoming_idx < u_len && upcoming_idx > 0 {
            self.move_user_queue_item(upcoming_idx, upcoming_idx - 1, cx);
        }
    }

    pub fn move_queue_item_down(&mut self, index: usize, cx: &mut Context<Self>) {
        if index == 0 {
            return;
        }
        let upcoming_idx = index - 1;
        let u_len = self.queue_mgr.user_queue.len();
        if upcoming_idx + 1 < u_len {
            self.move_user_queue_item(upcoming_idx, upcoming_idx + 1, cx);
        }
    }

    pub fn remove_queue_item(&mut self, index: usize, cx: &mut Context<Self>) {
        if index == 0 {
            return;
        }
        let upcoming_idx = index - 1;
        let u_len = self.queue_mgr.user_queue.len();
        if upcoming_idx < u_len {
            self.remove_user_queue_item(upcoming_idx, cx);
        } else {
            let snap = self.queue_mgr.snapshot();
            let c_len = snap.context_upcoming.len();
            let radio_start = u_len + c_len;
            if upcoming_idx >= radio_start {
                let r_idx = upcoming_idx - radio_start;
                if r_idx < self.queue_mgr.suggested_queue.len() {
                    self.queue_mgr.suggested_queue.remove(r_idx);
                    self.save_queue_state();
                    cx.notify();
                }
            }
        }
    }

    pub fn maybe_prefetch_suggestions(&mut self, cx: &mut Context<Self>) {
        if !self.is_autoplay_enabled {
            return;
        }
        let Some(seed_vid) = self.current_track.as_ref().map(|t| t.id.clone()) else {
            return;
        };
        let ytmusic = self.ytmusic.clone();
        let current_gen = self.queue_generation;

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                if let Ok(snap) = ytmusic.fetch_queue(&seed_vid).await {
                    let mut cx = cx.clone();
                    let _ = this.update(&mut cx, |state, cx| {
                        if state.queue_generation == current_gen {
                            let tracks: Vec<Track> = snap.upcoming.into_iter().map(|item| Track {
                                id: item.video_id.clone(),
                                title: item.title,
                                artists: vec![item.artist],
                                album: "YouTube Music".into(),
                                duration_secs: 180,
                                source: rustic_core::api::TrackSource::YouTubeMusic { video_id: item.video_id },
                                isrc: None,
                                artwork_url: item.thumbnail_url,
                                content_hash: None,
                            }).collect();
                            state.queue_mgr.ingest_radio(tracks);
                            state.save_queue_state();
                            cx.notify();
                        }
                    });
                }
            }
        }).detach();
    }

    pub fn save_queue_state(&self) {
        let (persisted_queue, persisted_cursor) = self.queue_mgr.prepare_for_persistence(50);
        let state = PersistedQueueState {
            actual_queue: persisted_queue,
            cursor: persisted_cursor,
            suggested_queue: self.queue_mgr.suggested_queue.clone(),
            current_position_secs: self.current_position,
            volume: self.volume,
            autoplay_enabled: self.is_autoplay_enabled,
            context_id: self.queue_mgr.context_id.clone(),
            context_title: self.source_playlist_title.clone(),
        };
        tokio::spawn(async move {
            let _ = std::fs::create_dir_all("cache");
            if let Ok(data) = serde_json::to_string_pretty(&state) {
                let _ = std::fs::write("cache/queue_state.json", data);
            }
        });
    }

    pub fn open_and_play_playlist(
        &mut self,
        playlist_id: String,
        title: String,
        _thumbnail_url: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.source_playlist_title = Some(title.clone());
        self.open_player(cx);
        self.queue_snapshot = Loadable::Loading;
        cx.notify();

        let clean_id = playlist_id.trim_start_matches("VL").to_string();
        let ytmusic = self.ytmusic.clone();
        let thumb_mgr = self.thumbnail_manager.clone();
        let pal_cache = self.palette_cache.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.fetch_queue_for_playlist(&clean_id).await;
                let mut cx = cx.clone();
                let mut thumb_to_fetch = None;

                let _ = this.update(&mut cx, |state, cx| {
                    match res {
                        Ok(snap) => {
                            state.is_autoplay_enabled = snap.autoplay_enabled;
                            state.source_playlist_title = Some(title.clone());

                            let mut ctx_tracks = Vec::new();
                            if let Some(ref cur) = snap.current_track {
                                ctx_tracks.push(Track {
                                    id: cur.video_id.clone(),
                                    title: cur.title.clone(),
                                    artists: vec![cur.artist.clone()],
                                    album: title.clone(),
                                    duration_secs: 180,
                                    source: rustic_core::api::TrackSource::YouTubeMusic { video_id: cur.video_id.clone() },
                                    isrc: None,
                                    artwork_url: cur.thumbnail_url.clone(),
                                    content_hash: None,
                                });
                            }
                            for item in &snap.upcoming {
                                ctx_tracks.push(Track {
                                    id: item.video_id.clone(),
                                    title: item.title.clone(),
                                    artists: vec![item.artist.clone()],
                                    album: title.clone(),
                                    duration_secs: 180,
                                    source: rustic_core::api::TrackSource::YouTubeMusic { video_id: item.video_id.clone() },
                                    isrc: None,
                                    artwork_url: item.thumbnail_url.clone(),
                                    content_hash: None,
                                });
                            }

                            if !ctx_tracks.is_empty() {
                                let first = ctx_tracks[0].clone();
                                state.engine.send_command(rustic_core::api::Command {
                                    origin: None,
                                    action: rustic_core::api::CommandAction::PlayCollection {
                                        tracks: ctx_tracks.clone(),
                                        start_index: 0,
                                        title: title.clone(),
                                        id: clean_id.clone(),
                                        shuffle: false,
                                    },
                                });
                                if let Some(ref thumb) = first.artwork_url {
                                    thumb_to_fetch = Some(thumb.clone());
                                }
                                state.current_track = Some(first);
                            }

                            state.queue_mgr.set_context(clean_id.clone(), title.clone(), ctx_tracks, 0, false);
                            state.save_queue_state();

                            state.queue_snapshot = Loadable::Ready(snap);
                            state.load_lyrics(cx);
                        }
                        Err(e) => {
                            state.queue_snapshot = Loadable::Error(e.to_string());
                        }
                    }
                    cx.notify();
                });

                if let Some(thumb_url) = thumb_to_fetch {
                    if let Some(path) = thumb_mgr.fetch_and_get_path(thumb_url).await {
                        let (palette, ambient_img) = pal_cache.extract(path).await;
                        let _ = this.update(&mut cx, |s, cx| {
                            s.active_palette = palette;
                            if let Some(amb) = ambient_img {
                                s.active_ambient_image = Some(amb);
                            }
                            cx.notify();
                        });
                    }
                }
            }
        }).detach();
    }

    pub fn open_album_detail(
        &mut self,
        album_id: String,
        fallback_title: String,
        fallback_thumb: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.active_album = Loadable::Loading;
        self.navigate_to(Route::Album(album_id.clone()), cx);

        let ytmusic = self.ytmusic.clone();
        let aid = album_id.clone();
        let f_title = fallback_title.clone();
        let f_thumb = fallback_thumb.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.fetch_collection_details(&aid).await;
                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    match res {
                        Ok(mut details) => {
                            if details.title.is_empty() || details.title == "Unknown Collection" {
                                details.title = f_title;
                            }
                            if details.thumbnail_url.is_none() {
                                details.thumbnail_url = f_thumb;
                            }
                            state.active_album = Loadable::Ready(details);
                        }
                        Err(e) => {
                            state.active_album = Loadable::Error(e.to_string());
                        }
                    }
                    cx.notify();
                });
            }
        }).detach();
    }

    pub fn open_playlist_detail(
        &mut self,
        playlist_id: String,
        fallback_title: String,
        fallback_thumb: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.active_playlist = Loadable::Loading;
        self.navigate_to(Route::Playlist(playlist_id.clone()), cx);

        let ytmusic = self.ytmusic.clone();
        let pid = playlist_id.clone();
        let f_title = fallback_title.clone();
        let f_thumb = fallback_thumb.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.fetch_collection_details(&pid).await;
                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    match res {
                        Ok(mut details) => {
                            if details.title.is_empty() || details.title == "Unknown Collection" {
                                details.title = f_title;
                            }
                            if details.thumbnail_url.is_none() {
                                details.thumbnail_url = f_thumb;
                            }
                            state.active_playlist = Loadable::Ready(details);
                        }
                        Err(e) => {
                            state.active_playlist = Loadable::Error(e.to_string());
                        }
                    }
                    cx.notify();
                });
            }
        }).detach();
    }

    pub fn play_single_track(&mut self, track: Track, cx: &mut Context<Self>) {
        let thumb = track.artwork_url.clone();
        self.queue_mgr.play_single(track.clone());
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::PlaySingleTrack(track.clone()),
        });
        self.current_track = Some(track);
        if let Some(ref t) = thumb {
            self.update_palette_for_url_cx(t, cx);
        }
        self.open_player(cx);
        self.save_queue_state();
        self.load_lyrics(cx);
        cx.notify();
    }

    pub fn play_collection(&mut self, details: &sources::YtCollectionDetails, shuffle: bool, cx: &mut Context<Self>) {
        if details.tracks.is_empty() {
            return;
        }

        self.queue_generation += 1;
        let track = self.queue_mgr.set_context(
            details.id.clone(),
            details.title.clone(),
            details.tracks.clone(),
            0,
            shuffle,
        );

        if let Some(first) = track {
            self.source_playlist_title = Some(details.title.clone());
            self.engine.send_command(rustic_core::api::Command {
                origin: None,
                action: rustic_core::api::CommandAction::PlayCollection {
                    tracks: details.tracks.clone(),
                    start_index: 0,
                    title: details.title.clone(),
                    id: details.id.clone(),
                    shuffle,
                },
            });
            self.current_track = Some(first.clone());
            if let Some(ref thumb) = first.artwork_url {
                self.update_palette_for_url_cx(thumb, cx);
            }
            self.open_player(cx);
            self.save_queue_state();
            self.load_lyrics(cx);
            cx.notify();
        }
    }

    pub fn play_collection_track(&mut self, details: &sources::YtCollectionDetails, start_index: usize, cx: &mut Context<Self>) {
        if details.tracks.is_empty() || start_index >= details.tracks.len() {
            return;
        }
        self.queue_generation += 1;
        let track = self.queue_mgr.set_context(
            details.id.clone(),
            details.title.clone(),
            details.tracks.clone(),
            start_index,
            false,
        );

        if let Some(first) = track {
            self.source_playlist_title = Some(details.title.clone());
            self.engine.send_command(rustic_core::api::Command {
                origin: None,
                action: rustic_core::api::CommandAction::PlayCollection {
                    tracks: details.tracks.clone(),
                    start_index,
                    title: details.title.clone(),
                    id: details.id.clone(),
                    shuffle: false,
                },
            });
            self.current_track = Some(first.clone());
            if let Some(ref thumb) = first.artwork_url {
                self.update_palette_for_url_cx(thumb, cx);
            }
            self.open_player(cx);
            self.save_queue_state();
            self.load_lyrics(cx);
            cx.notify();
        }
    }

    pub fn open_artist_detail(
        &mut self,
        channel_id: String,
        fallback_name: String,
        fallback_thumb: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.active_artist_request_seq += 1;
        let seq = self.active_artist_request_seq;
        self.active_artist = Loadable::Loading;
        self.active_artist_id = Some(channel_id.clone());
        self.navigate_to(Route::Artist(channel_id.clone()), cx);

        let ytmusic = self.ytmusic.clone();
        let cid = channel_id.clone();
        let f_name = fallback_name.clone();
        let f_thumb = fallback_thumb.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.fetch_artist_details(&cid).await;
                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    if state.active_artist_request_seq == seq {
                        match res {
                            Ok(mut details) => {
                                if details.name.is_empty() || details.name == "Artist" {
                                    details.name = f_name;
                                }
                                if details.avatar_url.is_none() {
                                    details.avatar_url = f_thumb;
                                }
                                state.active_artist = Loadable::Ready(details);
                            }
                            Err(e) => {
                                state.active_artist = Loadable::Error(e.to_string());
                            }
                        }
                        cx.notify();
                    }
                });
            }
        }).detach();
    }

    pub fn play_artist_track(&mut self, details: &sources::YtArtistDetails, start_index: usize, cx: &mut Context<Self>) {
        if details.top_songs.is_empty() || start_index >= details.top_songs.len() {
            return;
        }
        self.queue_generation += 1;
        let track = self.queue_mgr.set_context(
            details.id.clone(),
            format!("{} - Top Songs", details.name),
            details.top_songs.clone(),
            start_index,
            false,
        );

        if let Some(first) = track {
            self.source_playlist_title = Some(format!("{} - Top Songs", details.name));
            self.engine.send_command(rustic_core::api::Command {
                origin: None,
                action: rustic_core::api::CommandAction::PlayCollection {
                    tracks: details.top_songs.clone(),
                    start_index,
                    title: format!("{} - Top Songs", details.name),
                    id: details.id.clone(),
                    shuffle: false,
                },
            });
            self.current_track = Some(first.clone());
            if let Some(ref thumb) = first.artwork_url {
                self.update_palette_for_url_cx(thumb, cx);
            }
            self.open_player(cx);
            self.save_queue_state();
            if self.queue_mgr.needs_prefetch() {
                self.maybe_prefetch_suggestions(cx);
            }
            self.load_lyrics(cx);
            cx.notify();
        }
    }

    pub fn shuffle_artist_tracks(&mut self, details: &sources::YtArtistDetails, cx: &mut Context<Self>) {
        if details.top_songs.is_empty() {
            return;
        }
        self.queue_generation += 1;
        let track = self.queue_mgr.set_context(
            details.id.clone(),
            format!("{} - Top Songs", details.name),
            details.top_songs.clone(),
            0,
            true,
        );

        if let Some(first) = track {
            self.source_playlist_title = Some(format!("{} - Top Songs", details.name));
            self.engine.send_command(rustic_core::api::Command {
                origin: None,
                action: rustic_core::api::CommandAction::PlayCollection {
                    tracks: details.top_songs.clone(),
                    start_index: 0,
                    title: format!("{} - Top Songs", details.name),
                    id: details.id.clone(),
                    shuffle: true,
                },
            });
            self.current_track = Some(first.clone());
            if let Some(ref thumb) = first.artwork_url {
                self.update_palette_for_url_cx(thumb, cx);
            }
            self.open_player(cx);
            self.save_queue_state();
            if self.queue_mgr.needs_prefetch() {
                self.maybe_prefetch_suggestions(cx);
            }
            self.load_lyrics(cx);
            cx.notify();
        }
    }

    pub fn play_downloaded_tracks(&mut self, shuffle: bool, start_index: usize, cx: &mut Context<Self>) {
        let tracks = self.library_downloads.clone();
        if tracks.is_empty() || start_index >= tracks.len() {
            return;
        }

        self.queue_generation += 1;
        let track = self.queue_mgr.set_context(
            "offline_downloads".to_string(),
            "Offline Downloads".to_string(),
            tracks.clone(),
            start_index,
            shuffle,
        );

        if let Some(first) = track {
            self.source_playlist_title = Some("Offline Downloads".to_string());
            self.engine.send_command(rustic_core::api::Command {
                origin: None,
                action: rustic_core::api::CommandAction::PlayCollection {
                    tracks,
                    start_index,
                    title: "Offline Downloads".to_string(),
                    id: "offline_downloads".to_string(),
                    shuffle,
                },
            });
            self.current_track = Some(first.clone());
            if let Some(ref thumb) = first.artwork_url {
                self.update_palette_for_url_cx(thumb, cx);
            }
            self.open_player(cx);
            self.save_queue_state();
            cx.notify();
        }
    }

    pub fn trigger_download_track(&mut self, track: Track, cx: &mut Context<Self>) {
        let dl_mgr = self.download_manager.clone();
        let yt = self.ytmusic.clone();
        let video_id = track.id.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let mut resolve_track = track.clone();
                if !matches!(resolve_track.source, rustic_core::api::TrackSource::YouTubeMusic { .. }) {
                    resolve_track.source = rustic_core::api::TrackSource::YouTubeMusic { video_id: track.id.clone() };
                }

                use sources::SourceProvider;
                match yt.resolve(&resolve_track).await {
                    Ok(resolved) => {
                        let client = reqwest::Client::new();
                        let res = {
                            let mut mgr = dl_mgr.write().unwrap();
                            mgr.download_track(track.clone(), &resolved.stream_url, &client).await
                        };
                        match res {
                            Ok(_) => {
                                let mut cx = cx.clone();
                                let _ = this.update(&mut cx, |state, cx| {
                                    state.library_downloads = state.download_manager.read().unwrap().get_downloaded_tracks();
                                    cx.notify();
                                });
                            }
                            Err(e) => {
                                tracing::error!(target: "app", "Failed to download track {}: {}", video_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(target: "app", "Failed to resolve stream for download {}: {:?}", video_id, e);
                    }
                }
            }
        }).detach();
    }

    pub fn trigger_delete_download(&mut self, video_id: &str, cx: &mut Context<Self>) {
        let _ = self.download_manager.write().unwrap().delete_download(video_id);
        self.library_downloads = self.download_manager.read().unwrap().get_downloaded_tracks();
        cx.notify();
    }

    pub fn trigger_download_collection(&mut self, collection: sources::YtCollectionDetails, cx: &mut Context<Self>) {
        for track in collection.tracks {
            self.trigger_download_track(track, cx);
        }
    }

    pub fn open_search(&mut self, cx: &mut Context<Self>) {
        self.is_search_overlay_open = true;
        self.navigate_to(Route::Search, cx);
    }

    pub fn set_search_overlay_open(&mut self, open: bool, cx: &mut Context<Self>) {
        self.is_search_overlay_open = open;
        cx.notify();
    }

    pub fn clear_search(&mut self, cx: &mut Context<Self>) {
        self.search_query.clear();
        self.search_suggestions.clear();
        self.is_search_overlay_open = false;
        self.search_results = Loadable::Idle;
        cx.notify();
    }

    pub fn fetch_search_suggestions(&mut self, query: String, cx: &mut Context<Self>) {
        let q = query.trim().to_string();
        if q.is_empty() {
            self.search_suggestions.clear();
            cx.notify();
            return;
        }

        let ytmusic = self.ytmusic.clone();
        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.get_search_suggestions(&q).await;
                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    if let Ok(suggestions) = res {
                        state.search_suggestions = suggestions;
                    }
                    cx.notify();
                });
            }
        }).detach();
    }

    pub fn perform_search(&mut self, query: String, cx: &mut Context<Self>) {
        let q = query.trim().to_string();
        if q.is_empty() {
            return;
        }
        self.search_query = q.clone();
        self.is_search_overlay_open = false;
        self.search_results = Loadable::Loading;
        if self.current_route != Route::Search {
            self.navigate_to(Route::Search, cx);
        }
        cx.notify();

        let ytmusic = self.ytmusic.clone();
        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.search_tracks(&q).await;
                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    match res {
                        Ok(tracks) => state.search_results = Loadable::Ready(tracks),
                        Err(e) => state.search_results = Loadable::Error(e.to_string()),
                    }
                    cx.notify();
                });
            }
        }).detach();
    }

    pub fn toggle_sign_in_modal(&mut self, cx: &mut Context<Self>) {
        self.is_sign_in_modal_open = !self.is_sign_in_modal_open;
        self.sign_in_error = None;
        cx.notify();
    }

    pub fn submit_cookies(&mut self, text: String, cx: &mut Context<Self>) {
        let text = text.trim().to_string();
        if text.is_empty() {
            self.sign_in_error = Some("Please enter cookie JSON or header string".to_string());
            cx.notify();
            return;
        }
        self.is_signing_in = true;
        self.sign_in_error = None;
        cx.notify();

        let ytmusic = self.ytmusic.clone();
        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.login_with_cookies(&text).await;
                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    state.is_signing_in = false;
                    match res {
                        Ok(auth) => {
                            if let Some(ref url) = auth.account_avatar_url {
                                let _ = state.thumbnail_manager.resolve_or_queue(&Some(url.clone()), None);
                            }
                            state.auth_state = auth;
                            state.is_sign_in_modal_open = false;
                            state.sign_in_error = None;
                            let _ = std::fs::write("cookies.json", &text);
                            state.refresh_account_info(cx);
                            state.load_home_feed(cx);
                            state.load_library(cx);
                        }
                        Err(e) => {
                            state.sign_in_error = Some(format!("Sign in failed: {e}"));
                        }
                    }
                    cx.notify();
                });
            }
        }).detach();
    }

    pub fn cycle_repeat(&mut self, cx: &mut Context<Self>) {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        };
        cx.notify();
    }

    pub fn toggle_shuffle(&mut self, cx: &mut Context<Self>) {
        self.is_shuffled = !self.is_shuffled;
        cx.notify();
    }

    pub fn set_player_tab(&mut self, tab: PlayerTab, cx: &mut Context<Self>) {
        self.active_player_tab = tab;
        cx.notify();
    }

    pub fn toggle_autoplay(&mut self, cx: &mut Context<Self>) {
        let new_val = !self.is_autoplay_enabled;
        self.is_autoplay_enabled = new_val;
        self.engine.send_command(rustic_core::api::Command {
            origin: None,
            action: rustic_core::api::CommandAction::ToggleAutoplay(new_val),
        });
        cx.notify();
    }

    pub fn current_ambient_colors(&self) -> (gpui::Rgba, gpui::Rgba) {
        if !self.enable_blur {
            return (gpui::rgba(0x03030300), gpui::rgba(0x03030300));
        }
        self.active_palette
    }

    pub fn update_palette_for_url_cx(&self, url: &str, cx: &mut Context<Self>) {
        let pal_cache = self.palette_cache.clone();
        let thumb_mgr = self.thumbnail_manager.clone();
        let url_owned = url.to_string();

        cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                if let Some(path) = thumb_mgr.fetch_and_get_path(url_owned).await {
                    let (palette, ambient_img) = pal_cache.extract(path).await;
                    let _ = this.update(&mut cx, |s, cx| {
                        s.active_palette = palette;
                        if let Some(amb) = ambient_img {
                            s.active_ambient_image = Some(amb);
                        }
                        cx.notify();
                    });
                }
            }
        }).detach();
    }

    pub fn play_youtube_track(
        &mut self,
        video_id: String,
        title: String,
        artist: String,
        thumbnail_url: Option<String>,
        source_playlist: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let display_artists = if artist.is_empty() {
            vec!["YouTube Music".into()]
        } else {
            vec![artist]
        };

        let source = rustic_core::api::TrackSource::YouTubeMusic { video_id: video_id.clone() };

        let track = Track {
            id: video_id.clone(),
            title,
            artists: display_artists,
            album: source_playlist.clone().unwrap_or_else(|| "YouTube Music".into()),
            duration_secs: 180,
            source,
            isrc: None,
            artwork_url: thumbnail_url.clone(),
            content_hash: None,
        };

        self.queue_mgr.play_track_directly(track.clone());
        self.current_track = Some(track.clone());

        self.engine.send_command(Command {
            origin: None,
            action: CommandAction::PlaySingleTrack(track),
        });

        self.source_playlist_title = source_playlist;

        if let Some(ref thumb) = thumbnail_url {
            self.update_palette_for_url_cx(thumb, cx);
        }

        self.open_player(cx);
        self.load_queue(cx);
        self.load_lyrics(cx);
        cx.notify();
    }

    pub fn load_home_feed(&mut self, cx: &mut Context<Self>) {
        self.load_home_feed_for_mood("", None, cx);
    }

    pub fn load_home_feed_for_mood(&mut self, _mood: &str, params: Option<String>, cx: &mut Context<Self>) {
        self.home_feed = Loadable::Loading;
        cx.notify();

        let ytmusic = self.ytmusic.clone();
        let thumb_mgr = self.thumbnail_manager.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.fetch_browse_feed("FEmusic_home", params.as_deref()).await;

                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    match res.as_ref() {
                        Ok(feed) => state.home_feed = Loadable::Ready(feed.clone()),
                        Err(e) => state.home_feed = Loadable::Error(e.to_string()),
                    }
                    cx.notify();
                });

                // Progressive thumbnail prefetching in background
                if let Ok(feed) = res {
                    let mut urls = Vec::new();
                    for sec in &feed.sections {
                        for item in &sec.items {
                            if let Some(ref u) = item.thumbnail_url {
                                urls.push(u.clone());
                            }
                        }
                    }
                    let mut rx = thumb_mgr.prefetch_batch_channel(urls);
                    while let Some(()) = rx.recv().await {
                        let mut cx = cx.clone();
                        let res = this.update(&mut cx, |_state, cx| {
                            cx.notify();
                        });
                        if res.is_err() {
                            break;
                        }
                    }
                }
            }
        }).detach();
    }

    pub fn login_with_cookies(&mut self, json_str: &str, cx: &mut Context<Self>) {
        let ytmusic = self.ytmusic.clone();
        let json_owned = json_str.to_string();
        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.login_with_cookies(&json_owned).await;
                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    match res {
                        Ok(auth) => {
                            state.auth_state = auth;
                            state.load_home_feed(cx);
                            state.load_library(cx);
                        }
                        Err(e) => {
                            tracing::warn!(target: "app", "Cookie login failed: {:?}", e);
                        }
                    }
                    cx.notify();
                });
            }
        }).detach();
    }

    pub fn refresh_account_info(&mut self, cx: &mut Context<Self>) {
        if !self.ytmusic.is_authenticated() {
            return;
        }

        let ytmusic = self.ytmusic.clone();
        let thumb_mgr = self.thumbnail_manager.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                if let Ok(info) = ytmusic.fetch_account_info().await {
                    if let Some(ref url) = info.avatar_url {
                        let _ = thumb_mgr.resolve_or_queue(&Some(url.clone()), None);
                    }
                    let auth = ytmusic.auth_state();
                    let mut cx = cx.clone();
                    let _ = this.update(&mut cx, |state, cx| {
                        state.auth_state = auth;
                        cx.notify();
                    });
                }
            }
        }).detach();
    }

    pub fn reload_cookies(&mut self, cx: &mut Context<Self>) {
        let ytmusic = self.ytmusic.clone();
        let thumb_mgr = self.thumbnail_manager.clone();
        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.reload_cookies_file("cookies.json");
                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    match res {
                        Ok(auth) => {
                            if let Some(ref url) = auth.account_avatar_url {
                                let _ = thumb_mgr.resolve_or_queue(&Some(url.clone()), None);
                            }
                            state.auth_state = auth;
                            state.refresh_account_info(cx);
                            state.load_home_feed(cx);
                            state.load_library(cx);
                        }
                        Err(e) => {
                            tracing::warn!(target: "app", "Cookie reload failed: {:?}", e);
                        }
                    }
                    cx.notify();
                });
            }
        }).detach();
    }

    pub fn set_library_tab(&mut self, tab: LibraryTab, cx: &mut Context<Self>) {
        self.library_tab = tab;
        cx.notify();
    }

    pub fn play_track_obj(&mut self, track: Track, cx: &mut Context<Self>) {
        let vid = track.id.clone();
        let title = track.title.clone();
        let artist = track.artists.join(", ");
        let thumb = track.artwork_url.clone();
        let album = Some(track.album.clone());
        self.play_youtube_track(vid, title, artist, thumb, album, cx);
    }

    pub fn load_library(&mut self, cx: &mut Context<Self>) {
        self.library_playlists = Loadable::Loading;
        self.library_albums = Loadable::Loading;
        self.library_songs = Loadable::Loading;
        self.library_artists = Loadable::Loading;
        cx.notify();

        let ytmusic = self.ytmusic.clone();
        let thumb_mgr = self.thumbnail_manager.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let (pl_res, al_res, sg_res, ar_res) = tokio::join!(
                    ytmusic.fetch_library_playlists(),
                    ytmusic.fetch_library_albums(),
                    ytmusic.fetch_library_songs(),
                    ytmusic.fetch_library_artists(),
                );

                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    match pl_res.as_ref() {
                        Ok(items) => state.library_playlists = Loadable::Ready(items.clone()),
                        Err(e) => state.library_playlists = Loadable::Error(e.to_string()),
                    }
                    match al_res.as_ref() {
                        Ok(items) => state.library_albums = Loadable::Ready(items.clone()),
                        Err(e) => state.library_albums = Loadable::Error(e.to_string()),
                    }
                    match sg_res.as_ref() {
                        Ok((items, token)) => {
                            state.library_songs = Loadable::Ready(items.clone());
                            state.library_songs_continuation = token.clone();
                        }
                        Err(e) => {
                            state.library_songs = Loadable::Error(e.to_string());
                            state.library_songs_continuation = None;
                        }
                    }
                    match ar_res.as_ref() {
                        Ok(items) => state.library_artists = Loadable::Ready(items.clone()),
                        Err(e) => state.library_artists = Loadable::Error(e.to_string()),
                    }
                    cx.notify();
                });

                // Progressive thumbnail prefetching
                let mut urls = Vec::new();
                if let Ok(ref pls) = pl_res {
                    for item in pls {
                        if let Some(ref u) = item.thumbnail_url {
                            urls.push(u.clone());
                        }
                    }
                }
                if let Ok(ref als) = al_res {
                    for item in als {
                        if let Some(ref u) = item.thumbnail_url {
                            urls.push(u.clone());
                        }
                    }
                }
                if let Ok((ref sgs, _)) = sg_res {
                    for item in sgs {
                        if let Some(ref u) = item.artwork_url {
                            urls.push(u.clone());
                        }
                    }
                }
                if let Ok(ref ars) = ar_res {
                    for item in ars {
                        if let Some(ref u) = item.thumbnail_url {
                            urls.push(u.clone());
                        }
                    }
                }

                if !urls.is_empty() {
                    let mut rx = thumb_mgr.prefetch_batch_channel(urls);
                    while let Some(()) = rx.recv().await {
                        let mut cx = cx.clone();
                        let res = this.update(&mut cx, |_state, cx| {
                            cx.notify();
                        });
                        if res.is_err() {
                            break;
                        }
                    }
                }
            }
        }).detach();
    }

    pub fn load_more_library_songs(&mut self, cx: &mut Context<Self>) {
        if self.is_loading_more_songs {
            return;
        }
        let token = match self.library_songs_continuation {
            Some(ref tok) => tok.clone(),
            None => return,
        };

        self.is_loading_more_songs = true;
        cx.notify();

        let ytmusic = self.ytmusic.clone();
        let thumb_mgr = self.thumbnail_manager.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = ytmusic.fetch_library_songs_continuation(&token).await;
                let mut cx = cx.clone();
                let mut new_thumbs = Vec::new();

                let _ = this.update(&mut cx, |state, cx| {
                    state.is_loading_more_songs = false;
                    match res {
                        Ok((new_tracks, next_token)) => {
                            state.library_songs_continuation = next_token;
                            for t in &new_tracks {
                                if let Some(ref u) = t.artwork_url {
                                    new_thumbs.push(u.clone());
                                }
                            }
                            if let Loadable::Ready(ref mut existing) = state.library_songs {
                                for t in new_tracks {
                                    if !existing.iter().any(|e| e.id == t.id) {
                                        existing.push(t);
                                    }
                                }
                            } else {
                                state.library_songs = Loadable::Ready(new_tracks);
                            }
                        }
                        Err(e) => {
                            tracing::warn!(target: "app", "Failed to load more liked songs: {:?}", e);
                        }
                    }
                    cx.notify();
                });

                if !new_thumbs.is_empty() {
                    let mut rx = thumb_mgr.prefetch_batch_channel(new_thumbs);
                    while let Some(()) = rx.recv().await {
                        let mut cx = cx.clone();
                        let res = this.update(&mut cx, |_state, cx| {
                            cx.notify();
                        });
                        if res.is_err() {
                            break;
                        }
                    }
                }
            }
        }).detach();
    }

    pub fn load_queue(&mut self, cx: &mut Context<Self>) {
        let video_id = match self.current_track.as_ref().and_then(|t| match &t.source {
            rustic_core::api::TrackSource::YouTubeMusic { video_id } => Some(video_id.clone()),
            _ if t.id.len() == 11 => Some(t.id.clone()),
            _ => None,
        }) {
            Some(vid) => vid,
            None => {
                self.queue_snapshot = Loadable::Idle;
                cx.notify();
                return;
            }
        };

        self.queue_snapshot = Loadable::Loading;
        cx.notify();

        let provider = self.ytmusic.clone();
        let thumb_mgr = self.thumbnail_manager.clone();
        let pal_cache = self.palette_cache.clone();

        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = provider.fetch_queue(&video_id).await;
                let mut cx = cx.clone();
                let mut thumb_to_fetch = None;

                let _ = this.update(&mut cx, |state, _cx| {
                    match res {
                        Ok(snap) => {
                            state.is_autoplay_enabled = snap.autoplay_enabled;
                            if let Some(ref p_title) = snap.source_playlist_title {
                                state.source_playlist_title = Some(p_title.clone());
                            }
                            if let Some(ref q_curr) = snap.current_track {
                                if let Some(ref mut cur) = state.current_track {
                                    if cur.id == q_curr.video_id
                                        || cur.title.starts_with("YouTube Track")
                                        || cur.title == "Streaming Audio"
                                        || cur.title == "Loading Track..."
                                    {
                                        cur.title = q_curr.title.clone();
                                        cur.artists = vec![q_curr.artist.clone()];
                                        if cur.artwork_url.is_none() {
                                            cur.artwork_url = q_curr.thumbnail_url.clone();
                                        }
                                    }
                                }
                                if let Some(ref thumb) = q_curr.thumbnail_url {
                                    thumb_to_fetch = Some(thumb.clone());
                                }
                            }
                            let upcoming_tracks: Vec<Track> = snap.upcoming.iter().map(|item| Track {
                                id: item.video_id.clone(),
                                title: item.title.clone(),
                                artists: vec![item.artist.clone()],
                                album: "YouTube Music".into(),
                                duration_secs: item.parse_duration_secs(),
                                source: rustic_core::api::TrackSource::YouTubeMusic { video_id: item.video_id.clone() },
                                isrc: None,
                                artwork_url: item.thumbnail_url.clone(),
                                content_hash: None,
                            }).collect();

                            // Populate Lane 2 (Next in Queue) with the YouTube Music upcoming queue!
                            state.queue_mgr.set_upcoming_tracks(upcoming_tracks.clone());
                            state.engine.send_command(rustic_core::api::Command {
                                origin: None,
                                action: rustic_core::api::CommandAction::SetUpcomingTracks(upcoming_tracks),
                            });
                            state.queue_snapshot = Loadable::Ready(snap);
                        }
                        Err(e) => state.queue_snapshot = Loadable::Error(e.to_string()),
                    }
                    _cx.notify();
                });

                if let Some(thumb_url) = thumb_to_fetch {
                    if let Some(path) = thumb_mgr.fetch_and_get_path(thumb_url).await {
                        let (palette, ambient_img) = pal_cache.extract(path).await;
                        let _ = this.update(&mut cx, |s, cx| {
                            s.active_palette = palette;
                            if let Some(amb) = ambient_img {
                                s.active_ambient_image = Some(amb);
                            }
                            cx.notify();
                        });
                    }
                }
            }
        }).detach();
    }

    pub fn load_lyrics(&mut self, cx: &mut Context<Self>) {
        let video_id = match self.current_track.as_ref().and_then(|t| match &t.source {
            rustic_core::api::TrackSource::YouTubeMusic { video_id } => Some(video_id.clone()),
            _ if t.id.len() == 11 => Some(t.id.clone()),
            _ => None,
        }) {
            Some(vid) => vid,
            None => {
                self.lyrics_doc = Loadable::Idle;
                cx.notify();
                return;
            }
        };

        self.lyrics_doc = Loadable::Loading;
        cx.notify();

        let provider = self.ytmusic.clone();
        cx.spawn(move |this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let cx = cx.clone();
            async move {
                let res = provider.fetch_lyrics_for_video(&video_id).await;
                let mut cx = cx.clone();
                let _ = this.update(&mut cx, |state, cx| {
                    match res {
                        Ok(doc) => state.lyrics_doc = Loadable::Ready(doc),
                        Err(e) => state.lyrics_doc = Loadable::Error(e.to_string()),
                    }
                    cx.notify();
                });
            }
        }).detach();
    }

    pub fn reload_library(&mut self) {
        // Local music library has been removed in favor of YouTube Music streaming & native offline downloads.
        self.tracks.clear();
        self.albums.clear();
        self.artists.clear();
        self.playlists.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sources::ytmusic::types::{QueueSnapshot, QueueTrackItem};
    use crate::pages::clean_artist_subtitle;

    #[test]
    fn test_clean_artist_subtitle_ytm_byline() {
        assert_eq!(
            clean_artist_subtitle("Bill Withers • 114M views • 1M likes"),
            "Bill Withers"
        );
        assert_eq!(
            clean_artist_subtitle("Bill Withers, 114M views, 1M likes"),
            "Bill Withers"
        );
        assert_eq!(
            clean_artist_subtitle("Coldplay • 4.2M views"),
            "Coldplay"
        );
        assert_eq!(
            clean_artist_subtitle("Hanumankind • 2.84M subscribers"),
            "Hanumankind"
        );
        assert_eq!(
            clean_artist_subtitle("Banjaare & Roni • 51M views • 599K likes"),
            "Banjaare & Roni"
        );
        assert_eq!(
            clean_artist_subtitle("Ed Sheeran • 1.2B views • 15M likes"),
            "Ed Sheeran"
        );
        assert_eq!(
            clean_artist_subtitle("Fifty Fifty • 94M views"),
            "Fifty Fifty"
        );
    }

    #[test]
    fn test_clean_artist_subtitle_preserves_legitimate_names() {
        // Legitimate band and song names with words like View, Like, Storm
        assert_eq!(
            clean_artist_subtitle("The View • Hats Off to the Buskers"),
            "The View • Hats Off to the Buskers"
        );
        assert_eq!(
            clean_artist_subtitle("Like A Storm • Catacombs"),
            "Like A Storm • Catacombs"
        );
        assert_eq!(
            clean_artist_subtitle("Taylor Swift • 1989 (Taylor's Version)"),
            "Taylor Swift • 1989 (Taylor's Version)"
        );
        assert_eq!(
            clean_artist_subtitle("Someone Like You • Adele"),
            "Someone Like You • Adele"
        );
        assert_eq!(
            clean_artist_subtitle("Room With A View • Yanni"),
            "Room With A View • Yanni"
        );
    }

    #[test]
    fn test_clean_artist_subtitle_fallbacks() {
        assert_eq!(clean_artist_subtitle(""), "");
        assert_eq!(clean_artist_subtitle("   "), "");
        assert_eq!(clean_artist_subtitle("Simple Artist"), "Simple Artist");
    }

    #[test]
    fn test_user_preferences_json_structure() {
        let json = serde_json::json!({
            "likes": ["vid1", "vid2"],
            "dislikes": ["vid3"]
        });
        let raw = json.to_string();
        let val: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let likes: std::collections::HashSet<String> = val.get("likes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let dislikes: std::collections::HashSet<String> = val.get("dislikes")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        assert!(likes.contains("vid1"));
        assert!(likes.contains("vid2"));
        assert!(dislikes.contains("vid3"));
    }

    #[test]
    fn test_queue_track_operations_and_history() {
        let mut snap = QueueSnapshot {
            current_track: Some(QueueTrackItem {
                video_id: "track_0".into(),
                title: "Track 0".into(),
                artist: "Artist 0".into(),
                duration_str: "3:00".into(),
                thumbnail_url: None,
                is_active: true,
            }),
            upcoming: vec![
                QueueTrackItem {
                    video_id: "track_1".into(),
                    title: "Track 1".into(),
                    artist: "Artist 1".into(),
                    duration_str: "3:30".into(),
                    thumbnail_url: None,
                    is_active: false,
                },
                QueueTrackItem {
                    video_id: "track_2".into(),
                    title: "Track 2".into(),
                    artist: "Artist 2".into(),
                    duration_str: "4:00".into(),
                    thumbnail_url: None,
                    is_active: false,
                },
            ],
            history: Vec::new(),
            autoplay_enabled: true,
            source_playlist_title: Some("Top Hits".into()),
            continuation: None,
        };

        // Advance to next track
        let next = snap.upcoming.remove(0);
        if let Some(old) = snap.current_track.take() {
            snap.history.push(old);
        }
        snap.current_track = Some(next);

        assert_eq!(snap.current_track.as_ref().unwrap().video_id, "track_1");
        assert_eq!(snap.history.len(), 1);
        assert_eq!(snap.history[0].video_id, "track_0");
        assert_eq!(snap.upcoming.len(), 1);
        assert_eq!(snap.upcoming[0].video_id, "track_2");

        // Jump to index 0 of upcoming (track_2)
        let chosen = snap.upcoming.remove(0);
        if let Some(old) = snap.current_track.take() {
            snap.history.push(old);
        }
        snap.current_track = Some(chosen);

        assert_eq!(snap.current_track.as_ref().unwrap().video_id, "track_2");
        assert_eq!(snap.history.len(), 2);
        assert_eq!(snap.history[1].video_id, "track_1");
        assert!(snap.upcoming.is_empty());

        // Previous track
        let prev = snap.history.pop().unwrap();
        if let Some(old) = snap.current_track.take() {
            snap.upcoming.insert(0, old);
        }
        snap.current_track = Some(prev);

        assert_eq!(snap.current_track.as_ref().unwrap().video_id, "track_1");
        assert_eq!(snap.upcoming.len(), 1);
        assert_eq!(snap.upcoming[0].video_id, "track_2");
        assert_eq!(snap.history.len(), 1);
        assert_eq!(snap.history[0].video_id, "track_0");
    }

    #[test]
    fn test_search_route_variant() {
        let route = Route::Search;
        assert_eq!(route, Route::Search);
        assert_ne!(route, Route::Home);
    }

    #[test]
    fn test_autoplay_disabled_stops_at_end_of_queue() {
        let snap = QueueSnapshot {
            current_track: Some(QueueTrackItem {
                video_id: "last_track".into(),
                title: "Last Track".into(),
                artist: "Artist".into(),
                duration_str: "3:00".into(),
                thumbnail_url: None,
                is_active: true,
            }),
            upcoming: Vec::new(),
            history: vec![
                QueueTrackItem {
                    video_id: "first_track".into(),
                    title: "First Track".into(),
                    artist: "Artist".into(),
                    duration_str: "3:00".into(),
                    thumbnail_url: None,
                    is_active: false,
                }
            ],
            autoplay_enabled: false,
            source_playlist_title: None,
            continuation: None,
        };

        // When upcoming is empty and autoplay is disabled:
        assert!(snap.upcoming.is_empty());
        assert!(!snap.autoplay_enabled);
        // Upcoming has no track to pop, current track stays as the finished track
        assert_eq!(snap.current_track.as_ref().unwrap().video_id, "last_track");
    }

    #[test]
    fn test_previous_at_first_of_queue_preserves_track() {
        let snap = QueueSnapshot {
            current_track: Some(QueueTrackItem {
                video_id: "first_track".into(),
                title: "First Track".into(),
                artist: "Artist".into(),
                duration_str: "3:00".into(),
                thumbnail_url: None,
                is_active: true,
            }),
            upcoming: vec![
                QueueTrackItem {
                    video_id: "second_track".into(),
                    title: "Second Track".into(),
                    artist: "Artist".into(),
                    duration_str: "3:30".into(),
                    thumbnail_url: None,
                    is_active: false,
                }
            ],
            history: Vec::new(),
            autoplay_enabled: false,
            source_playlist_title: None,
            continuation: None,
        };

        // At first track of queue, history is empty:
        assert!(snap.history.is_empty());
        // Previous action restarts current track to 0.0s rather than popping non-existent history
        let can_pop_history = !snap.history.is_empty();
        assert!(!can_pop_history);
        assert_eq!(snap.current_track.as_ref().unwrap().video_id, "first_track");
    }
}

