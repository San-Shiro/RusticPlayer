//! The high-level Player object owned by the engine.
//!
//! Owns one output stream, manages playback state transitions, and drives the
//! decoder from a background thread. Communication between the engine-facing
//! API (Play / Pause / Seek / Next) and the decode thread is via a small
//! lock-free control channel; the audio callback is never touched by anything
//! except cpal.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use atomic_float::AtomicF32;

use crate::crossfade::{CrossfadeState, mix_blocks};
use crate::decoder::Decoder;
use crate::error::{AudioError, Result};
use crate::output::{AudioDiagnostics, OutputDevice, OutputStream, ring_buffer};
use crate::source::{FileSource, HttpSource, MediaSource};

/// Playback state machine. Public so the CLI/engine can subscribe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    #[default]
    Stopped,
    Playing,
    Paused,
    Buffering,
}

/// Commands the engine can send the player.
#[derive(Debug)]
pub enum PlayerCommand {
    Load {
        path: Option<PathBuf>,
        url: Option<String>,
        gain_db: f32,
    },
    Play,
    Pause,
    Seek(Duration),
    SetVolume(f32),
    Next,
    ReconnectAudio,
    Shutdown,
}

/// Periodic status emitted by the player over a broadcast channel.
#[derive(Debug, Clone)]
pub struct PlayerEvent {
    pub state: PlaybackState,
    pub position: Duration,
    pub duration: Option<Duration>,
    pub underruns: u64,
    pub volume: f32,
    pub is_eos: bool,
}

static CLOCK_EPOCH: std::sync::LazyLock<std::time::Instant> = std::sync::LazyLock::new(std::time::Instant::now);

#[inline]
pub fn now_nanos() -> u64 {
    CLOCK_EPOCH.elapsed().as_nanos() as u64
}

/// Lock-free master clock synchronized with hardware DAC output.
#[derive(Debug)]
pub struct MasterClock {
    sample_rate: AtomicU32,
    dac_frames: Arc<AtomicU64>,
    dac_time_ns: Arc<AtomicU64>,
    dac_anchor: AtomicU64,
    media_offset_ms: AtomicU64,
    max_pos_ns: AtomicU64,
    is_playing: AtomicBool,
}

impl MasterClock {
    pub fn new(dac_frames: Arc<AtomicU64>, dac_time_ns: Arc<AtomicU64>, sample_rate: u32) -> Self {
        Self {
            sample_rate: AtomicU32::new(sample_rate.max(1)),
            dac_frames,
            dac_time_ns,
            dac_anchor: AtomicU64::new(0),
            media_offset_ms: AtomicU64::new(0),
            max_pos_ns: AtomicU64::new(0),
            is_playing: AtomicBool::new(false),
        }
    }

    pub fn set_sample_rate(&self, sr: u32) {
        self.sample_rate.store(sr.max(1), Ordering::Release);
    }

    pub fn on_load(&self) {
        let dac_now = self.dac_frames.load(Ordering::Acquire);
        self.dac_anchor.store(dac_now, Ordering::Release);
        self.media_offset_ms.store(0, Ordering::Release);
        self.max_pos_ns.store(0, Ordering::Release);
        self.dac_time_ns.store(now_nanos(), Ordering::Release);
        self.is_playing.store(true, Ordering::Release);
    }

    pub fn on_play(&self) {
        let dac_now = self.dac_frames.load(Ordering::Acquire);
        self.dac_anchor.store(dac_now, Ordering::Release);
        self.dac_time_ns.store(now_nanos(), Ordering::Release);
        self.is_playing.store(true, Ordering::Release);
    }

    pub fn on_pause(&self) {
        let cur = self.current_position();
        self.media_offset_ms.store(cur.as_millis() as u64, Ordering::Release);
        self.max_pos_ns.store(cur.as_nanos() as u64, Ordering::Release);
        let dac_now = self.dac_frames.load(Ordering::Acquire);
        self.dac_anchor.store(dac_now, Ordering::Release);
        self.dac_time_ns.store(now_nanos(), Ordering::Release);
        self.is_playing.store(false, Ordering::Release);
        tracing::debug!(target: "audio::clock", "on_pause: frozen at {}ms", cur.as_millis());
    }

    pub fn on_seek(&self, target: Duration) {
        let dac_now = self.dac_frames.load(Ordering::Acquire);
        self.dac_anchor.store(dac_now, Ordering::Release);
        self.media_offset_ms.store(target.as_millis() as u64, Ordering::Release);
        self.max_pos_ns.store(target.as_nanos() as u64, Ordering::Release);
        self.dac_time_ns.store(now_nanos(), Ordering::Release);
    }

    pub fn on_stop(&self) {
        self.is_playing.store(false, Ordering::Release);
        self.media_offset_ms.store(0, Ordering::Release);
        self.max_pos_ns.store(0, Ordering::Release);
        let dac_now = self.dac_frames.load(Ordering::Acquire);
        self.dac_anchor.store(dac_now, Ordering::Release);
        self.dac_time_ns.store(now_nanos(), Ordering::Release);
    }

    pub fn on_reconnect(&self) {
        let cur = self.current_position();
        self.media_offset_ms.store(cur.as_millis() as u64, Ordering::Release);
        self.max_pos_ns.store(cur.as_nanos() as u64, Ordering::Release);
        let dac_now = self.dac_frames.load(Ordering::Acquire);
        self.dac_anchor.store(dac_now, Ordering::Release);
        self.dac_time_ns.store(now_nanos(), Ordering::Release);
        tracing::debug!(target: "audio::clock", "on_reconnect: anchored at {}ms", cur.as_millis());
    }

    pub fn current_position(&self) -> Duration {
        let base_secs = self.media_offset_ms.load(Ordering::Acquire) as f64 / 1000.0;
        if !self.is_playing.load(Ordering::Acquire) {
            Duration::from_secs_f64(base_secs)
        } else {
            let dac_now = self.dac_frames.load(Ordering::Acquire);
            let anchor = self.dac_anchor.load(Ordering::Acquire);
            let dac_time = self.dac_time_ns.load(Ordering::Acquire);
            let sr = self.sample_rate.load(Ordering::Acquire).max(1) as f64;

            let elapsed_frames = dac_now.saturating_sub(anchor) as f64;
            let dac_secs = elapsed_frames / sr;

            let now_ns = now_nanos();
            let elapsed_since_dac = if dac_time > 0 && now_ns >= dac_time {
                (now_ns - dac_time) as f64 / 1_000_000_000.0
            } else {
                0.0
            };
            let wall_delta = elapsed_since_dac.min(0.100);

            let total_secs = (base_secs + dac_secs + wall_delta).max(0.0);
            let calc_ns = (total_secs * 1_000_000_000.0) as u64;

            // Strictly monotonic: never return a position earlier than previous
            let mut prev = self.max_pos_ns.load(Ordering::Acquire);
            loop {
                if calc_ns <= prev {
                    return Duration::from_nanos(prev);
                }
                match self.max_pos_ns.compare_exchange_weak(prev, calc_ns, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => return Duration::from_nanos(calc_ns),
                    Err(actual) => prev = actual,
                }
            }
        }
    }
}

/// One active decoder + its ring + metadata. Held by the decode thread.
struct Pipeline {
    decoder: Decoder,
    gain_linear: f32,
    /// Buffer for samples read during crossfade that haven't been played yet.
    sample_buf: Vec<f32>,
}

impl Pipeline {
    fn open(source: Box<dyn MediaSource>, device_rate: u32, device_channels: usize, gain_db: f32) -> Result<Self> {
        let decoder = Decoder::open(source, device_rate, device_channels)?;
        Ok(Self {
            decoder,
            gain_linear: crate::replaygain::db_to_linear(gain_db),
            sample_buf: Vec::new(),
        })
    }
}

pub struct Player {
    cmd_tx: Sender<PlayerCommand>,
    broadcast_tx: tokio::sync::broadcast::Sender<PlayerEvent>,
    handle: Option<JoinHandle<Result<()>>>,
    volume: Arc<AtomicF32>,
    state: Arc<std::sync::RwLock<PlaybackState>>,
    clock: Arc<MasterClock>,
    diagnostics: Arc<parking_lot::RwLock<AudioDiagnostics>>,
}

pub type DiagnosticsHandle = Arc<parking_lot::RwLock<AudioDiagnostics>>;

impl Player {
    /// Spawn the player thread and return the handle. Broadcast subscribers
    /// receive `PlayerEvent` at ~20Hz and on state transitions.
    pub fn new() -> Result<(Self, tokio::sync::broadcast::Receiver<PlayerEvent>)> {
        let (cmd_tx, cmd_rx) = channel::<PlayerCommand>();
        let (broadcast_tx, broadcast_rx) = tokio::sync::broadcast::channel(256);
        let broadcast_tx_inner = broadcast_tx.clone();
        let volume = Arc::new(AtomicF32::new(1.0));
        let state = Arc::new(std::sync::RwLock::new(PlaybackState::Stopped));
        let dac_frames = Arc::new(AtomicU64::new(0));
        let dac_time_ns = Arc::new(AtomicU64::new(now_nanos()));
        let clock = Arc::new(MasterClock::new(Arc::clone(&dac_frames), Arc::clone(&dac_time_ns), 44100));

        let default_diag = AudioDiagnostics {
            device_name: "Initializing...".into(),
            sample_rate: 44100,
            channels: 2,
            sample_format: "F32".into(),
            is_running: false,
            has_error: false,
            last_error: None,
            underruns: 0,
            dac_frames: 0,
            dac_time_ns: 0,
        };
        let diagnostics = Arc::new(parking_lot::RwLock::new(default_diag));

        let vol = Arc::clone(&volume);
        let st = Arc::clone(&state);
        let clk = Arc::clone(&clock);
        let dac = Arc::clone(&dac_frames);
        let dac_t = Arc::clone(&dac_time_ns);
        let diag_clone = Arc::clone(&diagnostics);

        let handle = std::thread::Builder::new()
            .name("audio-player".into())
            .spawn(move || run(cmd_rx, broadcast_tx_inner, vol, st, clk, dac, dac_t, diag_clone))?;

        Ok((
            Self {
                cmd_tx,
                broadcast_tx,
                handle: Some(handle),
                volume,
                state,
                clock,
                diagnostics,
            },
            broadcast_rx,
        ))
    }

    pub fn current_position(&self) -> Duration {
        self.clock.current_position()
    }

    pub fn clock(&self) -> Arc<MasterClock> {
        Arc::clone(&self.clock)
    }

    pub fn diagnostics(&self) -> AudioDiagnostics {
        self.diagnostics.read().clone()
    }

    pub fn diagnostics_handle(&self) -> DiagnosticsHandle {
        Arc::clone(&self.diagnostics)
    }

    pub fn reconnect_audio(&self) -> Result<()> {
        self.cmd_tx
            .send(PlayerCommand::ReconnectAudio)
            .map_err(|_| AudioError::Cancelled)
    }

    pub fn load_path(&self, path: PathBuf, gain_db: f32) -> Result<()> {
        self.cmd_tx
            .send(PlayerCommand::Load {
                path: Some(path),
                url: None,
                gain_db,
            })
            .map_err(|_| AudioError::Cancelled)
    }

    pub fn load_url(&self, url: String, gain_db: f32) -> Result<()> {
        self.cmd_tx
            .send(PlayerCommand::Load {
                path: None,
                url: Some(url),
                gain_db,
            })
            .map_err(|_| AudioError::Cancelled)
    }

    pub fn play(&self) -> Result<()> {
        self.cmd_tx
            .send(PlayerCommand::Play)
            .map_err(|_| AudioError::Cancelled)
    }

    pub fn pause(&self) -> Result<()> {
        self.cmd_tx
            .send(PlayerCommand::Pause)
            .map_err(|_| AudioError::Cancelled)
    }

    pub fn seek(&self, d: Duration) -> Result<()> {
        self.cmd_tx
            .send(PlayerCommand::Seek(d))
            .map_err(|_| AudioError::Cancelled)
    }

    pub fn set_volume(&self, v: f32) -> Result<()> {
        self.volume
            .store(v.clamp(0.0, 1.5), Ordering::Release);
        Ok(())
    }

    pub fn state(&self) -> PlaybackState {
        *self.state.read().unwrap()
    }

    pub fn underruns(&self) -> u64 {
        self.broadcast_tx.subscribe().resubscribe().len() as u64 // placeholder
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        tracing::debug!(target: "audio::player", "Player::drop invoked, sending Shutdown");
        let _ = self.cmd_tx.send(PlayerCommand::Shutdown);
        if let Some(h) = self.handle.take() {
            tracing::debug!(target: "audio::player", "Joining player thread handle...");
            let _ = h.join();
            tracing::debug!(target: "audio::player", "Player thread joined successfully!");
        }
    }
}

fn run(
    rx: Receiver<PlayerCommand>,
    tx: tokio::sync::broadcast::Sender<PlayerEvent>,
    volume: Arc<AtomicF32>,
    state: Arc<std::sync::RwLock<PlaybackState>>,
    clock: Arc<MasterClock>,
    dac_frames: Arc<AtomicU64>,
    dac_time_ns: Arc<AtomicU64>,
    diagnostics: Arc<parking_lot::RwLock<AudioDiagnostics>>,
) -> Result<()> {
    let device = OutputDevice::default();
    let (mut sample_rate, mut channels, mut stream, mut producer) = match device {
        Ok(dev) => {
            let sr = dev.sample_rate();
            let ch = dev.channels();
            clock.set_sample_rate(sr);
            let (prod, consumer) = ring_buffer(&dev);
            let st = OutputStream::start(dev, consumer, Arc::clone(&volume), Arc::clone(&dac_frames), Arc::clone(&dac_time_ns))
                .map_err(|e| tracing::error!(target: "audio::player", "stream start failed: {e}"))
                .ok();
            if let Some(s) = &st {
                *diagnostics.write() = s.diagnostics();
            }
            (sr, ch, st, Some(prod))
        }
        Err(e) => {
            tracing::error!(target: "audio::player", "output device init failed: {e}");
            *diagnostics.write() = AudioDiagnostics {
                device_name: "No Output Device".into(),
                sample_rate: 44100,
                channels: 2,
                sample_format: "Unknown".into(),
                is_running: false,
                has_error: true,
                last_error: Some(e.to_string()),
                underruns: 0,
                dac_frames: 0,
                dac_time_ns: 0,
            };
            (44100, 2, None, None)
        }
    };

    let mut pipeline: Option<Pipeline> = None;
    let mut next_pipeline: Option<Pipeline> = None;
    let mut crossfade = CrossfadeState::new(Duration::from_secs(5), sample_rate);
    let mut paused = true;
    let mut last_broadcast = std::time::Instant::now();
    let mut last_reconnect_attempt = std::time::Instant::now();

    'outer: loop {
        // A. Drain all pending commands immediately.
        let mut should_exit = false;
        loop {
            match rx.try_recv() {
                Ok(cmd) => {
                    match cmd {
                        PlayerCommand::Load { path, url, gain_db } => {
                            let source_res: Result<Box<dyn MediaSource>> = match (path, url) {
                                (Some(p), _) => FileSource::open(p).map(|s| Box::new(s) as Box<dyn MediaSource>),
                                (_, Some(u)) => Ok(Box::new(HttpSource::new(u))),
                                _ => Err(AudioError::Cancelled),
                            };
                            let source = match source_res {
                                Ok(s) => s,
                                Err(e) => {
                                    tracing::error!(target: "audio::player", "source open failed: {e}");
                                    *state.write().unwrap() = PlaybackState::Stopped;
                                    clock.on_stop();
                                    let _ = tx.send(PlayerEvent {
                                        state: PlaybackState::Stopped,
                                        position: Duration::ZERO,
                                        duration: None,
                                        underruns: 0,
                                        volume: volume.load(Ordering::Acquire),
                                        is_eos: false,
                                    });
                                    continue;
                                }
                            };

                            match Pipeline::open(source, sample_rate, channels, gain_db) {
                                Ok(pipe) => {
                                    let dur = pipe.decoder.duration_secs().map(Duration::from_secs_f64);
                                    if paused {
                                        pipeline = Some(pipe);
                                        clock.on_load();
                                        *state.write().unwrap() = PlaybackState::Buffering;
                                        let _ = tx.send(PlayerEvent {
                                            state: PlaybackState::Buffering,
                                            position: Duration::ZERO,
                                            duration: dur,
                                            underruns: 0,
                                            volume: volume.load(Ordering::Acquire),
                                            is_eos: false,
                                        });
                                    } else {
                                        pipeline = Some(pipe);
                                        clock.on_load();
                                        clock.on_play();
                                        paused = false;
                                        if let Some(s) = &mut stream {
                                            let _ = s.play();
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(target: "audio::player", "pipeline open failed: {e:?}");
                                    *state.write().unwrap() = PlaybackState::Stopped;
                                    clock.on_stop();
                                    let _ = tx.send(PlayerEvent {
                                        state: PlaybackState::Stopped,
                                        position: Duration::ZERO,
                                        duration: None,
                                        underruns: 0,
                                        volume: volume.load(Ordering::Acquire),
                                        is_eos: false,
                                    });
                                }
                            }
                        }
                        PlayerCommand::Play => {
                            paused = false;
                            let faulted = stream.as_ref().map(|s| s.has_error.load(Ordering::Acquire)).unwrap_or(true);
                            if faulted {
                                tracing::info!(target: "audio::player", "Stream faulted on Play, triggering immediate device reconnect");
                                last_reconnect_attempt = std::time::Instant::now();
                                if let Ok(dev) = OutputDevice::default() {
                                    let dev_name = dev.name.clone();
                                    let sr = dev.sample_rate();
                                    let ch = dev.channels();
                                    clock.set_sample_rate(sr);
                                    let (prod, consumer) = ring_buffer(&dev);
                                    if let Ok(new_stream) = OutputStream::start(dev, consumer, Arc::clone(&volume), Arc::clone(&dac_frames), Arc::clone(&dac_time_ns)) {
                                        let _ = new_stream.play();
                                        sample_rate = sr;
                                        channels = ch;
                                        producer = Some(prod);
                                        clock.on_reconnect();
                                        *diagnostics.write() = new_stream.diagnostics();
                                        stream = Some(new_stream);
                                        tracing::info!(target: "audio::player", "Immediate reconnection on Play to {} successful!", dev_name);
                                    }
                                }
                            } else if let Some(s) = &mut stream {
                                if let Err(e) = s.play() {
                                    tracing::warn!(target: "audio::player", "stream.play() failed on Play: {e}");
                                }
                            }
                            clock.on_play();
                            *state.write().unwrap() = PlaybackState::Playing;
                        }
                        PlayerCommand::Pause => {
                            tracing::debug!(target: "audio::player", "Handling PlayerCommand::Pause");
                            paused = true;
                            if let Some(s) = &mut stream {
                                let _ = s.pause();
                            }
                            clock.on_pause();
                            *state.write().unwrap() = PlaybackState::Paused;
                            tracing::debug!(target: "audio::player", "PlayerCommand::Pause done, is_playing={}", clock.is_playing.load(Ordering::Acquire));
                        }
                        PlayerCommand::Seek(d) => {
                            if let Some(p) = &mut pipeline {
                                if let Err(e) = p.decoder.seek(d.as_secs_f64()) {
                                    tracing::error!(target: "audio::player", "seek error: {e}");
                                } else {
                                    p.sample_buf.clear();
                                    if let Some(s) = &stream {
                                        s.flush();
                                    }
                                    clock.on_seek(d);
                                }
                            }
                        }
                        PlayerCommand::SetVolume(v) => {
                            volume.store(v.clamp(0.0, 1.5), Ordering::Release);
                        }
                        PlayerCommand::Next => {
                            pipeline = next_pipeline.take();
                            if let Some(s) = &stream {
                                s.flush();
                            }
                            clock.on_load();
                            crossfade.active = false;
                        }
                        PlayerCommand::ReconnectAudio => {
                            tracing::info!(target: "audio::player", "Manual audio reconnection requested");
                            last_reconnect_attempt = std::time::Instant::now();
                            match OutputDevice::default() {
                                Ok(dev) => {
                                    let dev_name = dev.name.clone();
                                    let sr = dev.sample_rate();
                                    let ch = dev.channels();
                                    clock.set_sample_rate(sr);
                                    let (prod, consumer) = ring_buffer(&dev);
                                    match OutputStream::start(dev, consumer, Arc::clone(&volume), Arc::clone(&dac_frames), Arc::clone(&dac_time_ns)) {
                                        Ok(new_stream) => {
                                            if !paused {
                                                let _ = new_stream.play();
                                            }
                                            sample_rate = sr;
                                            channels = ch;
                                            producer = Some(prod);
                                            clock.on_reconnect();
                                            *diagnostics.write() = new_stream.diagnostics();
                                            stream = Some(new_stream);
                                            tracing::info!(target: "audio::player", "Reconnected to device: {}", dev_name);
                                        }
                                        Err(e) => {
                                            tracing::error!(target: "audio::player", "Reconnect stream start failed: {e}");
                                            let mut d = diagnostics.write();
                                            d.has_error = true;
                                            d.last_error = Some(e.to_string());
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(target: "audio::player", "Reconnect device default failed: {e}");
                                    let mut d = diagnostics.write();
                                    d.has_error = true;
                                    d.last_error = Some(e.to_string());
                                }
                            }
                        }
                        PlayerCommand::Shutdown => {
                            tracing::debug!(target: "audio::player", "Received Shutdown, breaking outer loop");
                            should_exit = true;
                            break;
                        }
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    tracing::debug!(target: "audio::player", "Command channel disconnected, breaking outer loop");
                    should_exit = true;
                    break;
                }
            }
        }

        if should_exit {
            break 'outer Ok(());
        }

        // B. Fill audio buffer if playing
        let mut buffer_is_full = false;
        if !paused {
            if producer.is_some() && pipeline.is_some() {
                let mut decode_iterations = 0;
                while !buffer_is_full && decode_iterations < 16 {
                    decode_iterations += 1;
                    if let Some(prod) = producer.as_ref() {
                        if prod.slots() < 1024 {
                            buffer_is_full = true;
                            break;
                        }
                    }

                    let block_res = {
                        let p = pipeline.as_mut().unwrap();
                        if !p.sample_buf.is_empty() {
                            Ok(Some(std::mem::take(&mut p.sample_buf)))
                        } else {
                            match p.decoder.next_block() {
                                Ok(Some(block)) => {
                                    let mut data = block.samples;
                                    if (p.gain_linear - 1.0).abs() > f32::EPSILON {
                                        for s in &mut data {
                                            *s = (*s * p.gain_linear).clamp(-1.0, 1.0);
                                        }
                                    }
                                    Ok(Some(data))
                                }
                                Ok(None) => Ok(None),
                                Err(e) => Err(e),
                            }
                        }
                    };

                    match block_res {
                        Ok(Some(mut data)) => {
                            let mut crossfade_switched = false;
                            if crossfade.active {
                                let mut abort_crossfade = false;
                                let mut crossfade_finished = false;
                                
                                if let Some(next_p) = &mut next_pipeline {
                                    while next_p.sample_buf.len() < data.len() {
                                        match next_p.decoder.next_block() {
                                            Ok(Some(block)) => {
                                                let mut b = block.samples;
                                                if (next_p.gain_linear - 1.0).abs() > f32::EPSILON {
                                                    for s in &mut b {
                                                        *s = (*s * next_p.gain_linear).clamp(-1.0, 1.0);
                                                    }
                                                }
                                                next_p.sample_buf.extend_from_slice(&b);
                                            }
                                            Ok(None) | Err(_) => break,
                                        }
                                    }
                                    
                                    let mix_len = data.len().min(next_p.sample_buf.len());
                                    let b_data: Vec<f32> = next_p.sample_buf.drain(..mix_len).collect();
                                    
                                    mix_blocks(&mut data, &b_data, &mut crossfade, channels);
                                    
                                    if !crossfade.active {
                                        crossfade_finished = true;
                                    }
                                    if b_data.len() < data.len() {
                                        abort_crossfade = true;
                                    }
                                } else {
                                    abort_crossfade = true;
                                }
                                
                                if abort_crossfade {
                                    crossfade.active = false;
                                    next_pipeline = None;
                                } else if crossfade_finished {
                                    pipeline = next_pipeline.take();
                                    if let Some(s) = &stream {
                                        s.flush();
                                    }
                                    clock.on_load();
                                    crossfade_switched = true;
                                }
                            }

                            if let Some(prod) = &mut producer {
                                let mut pushed = 0usize;
                                while pushed < data.len() {
                                    if prod.push(data[pushed]).is_ok() {
                                        pushed += 1;
                                    } else {
                                        // Buffer is full; save remaining samples to active pipeline and yield
                                        if !crossfade_switched {
                                            if let Some(p) = &mut pipeline {
                                                p.sample_buf = data[pushed..].to_vec();
                                            }
                                        }
                                        buffer_is_full = true;
                                        break;
                                    }
                                }
                            }

                            if *state.read().unwrap() != PlaybackState::Playing {
                                *state.write().unwrap() = PlaybackState::Playing;
                            }
                        }
                        Ok(None) => {
                            if let Some(next_p) = next_pipeline.take() {
                                pipeline = Some(next_p);
                                if let Some(s) = &stream {
                                    s.flush();
                                }
                                clock.on_load();
                                crossfade.active = false;
                            } else {
                                if let Some(prod) = &producer {
                                    if prod.slots() < prod.buffer().capacity() {
                                        std::thread::sleep(Duration::from_millis(15));
                                        continue;
                                    }
                                }
                                pipeline = None;
                                paused = true;
                                clock.on_stop();
                                *state.write().unwrap() = PlaybackState::Stopped;
                                let _ = tx.send(PlayerEvent {
                                    state: PlaybackState::Stopped,
                                    position: clock.current_position(),
                                    duration: None,
                                    underruns: 0,
                                    volume: volume.load(Ordering::Acquire),
                                    is_eos: true,
                                });
                            }
                            break;
                        }
                        Err(e) => {
                            tracing::error!(target: "audio::player", "decode error: {e:?}");
                            pipeline = None;
                            paused = true;
                            clock.on_stop();
                            *state.write().unwrap() = PlaybackState::Stopped;
                            let _ = tx.send(PlayerEvent {
                                state: PlaybackState::Stopped,
                                position: clock.current_position(),
                                duration: None,
                                underruns: 0,
                                volume: volume.load(Ordering::Acquire),
                                is_eos: false,
                            });
                            break;
                        }
                    }
                }
            } else {
                buffer_is_full = true;
            }
        }

        // Check for faulted stream and auto-reconnect if playing
        let stream_faulted = stream.as_ref().map(|s| s.has_error.load(Ordering::Acquire)).unwrap_or(true);
        if !paused && stream_faulted && last_reconnect_attempt.elapsed() >= Duration::from_secs(3) {
            last_reconnect_attempt = std::time::Instant::now();
            tracing::info!(target: "audio::player", "Auto-reconnecting faulted audio stream...");
            if let Ok(dev) = OutputDevice::default() {
                let dev_name = dev.name.clone();
                let sr = dev.sample_rate();
                let ch = dev.channels();
                clock.set_sample_rate(sr);
                let (prod, consumer) = ring_buffer(&dev);
                if let Ok(new_stream) = OutputStream::start(dev, consumer, Arc::clone(&volume), Arc::clone(&dac_frames), Arc::clone(&dac_time_ns)) {
                    let _ = new_stream.play();
                    sample_rate = sr;
                    channels = ch;
                    producer = Some(prod);
                    clock.on_reconnect();
                    *diagnostics.write() = new_stream.diagnostics();
                    stream = Some(new_stream);
                    tracing::info!(target: "audio::player", "Auto-reconnection to {} successful!", dev_name);
                }
            }
        }

        if paused {
            std::thread::sleep(Duration::from_millis(10));
        } else if buffer_is_full {
            std::thread::sleep(Duration::from_millis(5));
        }

        // C. Periodic status broadcast (~20Hz / 50ms)
        if last_broadcast.elapsed() >= Duration::from_millis(50) {
            last_broadcast = std::time::Instant::now();
            if let Some(s) = &stream {
                *diagnostics.write() = s.diagnostics();
            }
            let state_now = *state.read().unwrap();
            let pos = clock.current_position();
            let _ = tx.send(PlayerEvent {
                state: state_now,
                position: pos,
                duration: pipeline
                    .as_ref()
                    .and_then(|p| p.decoder.duration_secs().map(Duration::from_secs_f64)),
                underruns: stream.as_ref().map(|s| s.underruns.load(Ordering::Relaxed)).unwrap_or(0),
                volume: volume.load(Ordering::Acquire),
                is_eos: false,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn master_clock_tracks_hardware_and_seeks_accurately() {
        let dac_frames = Arc::new(AtomicU64::new(0));
        let dac_time_ns = Arc::new(AtomicU64::new(now_nanos()));
        let clock = MasterClock::new(Arc::clone(&dac_frames), Arc::clone(&dac_time_ns), 44100);

        // Initially stopped at 0
        assert_eq!(clock.current_position(), Duration::ZERO);

        // Load starts playing from 0
        clock.on_load();
        assert_eq!(clock.current_position().as_millis(), 0);

        // 44100 frames consumed by DAC = 1.000s
        dac_frames.store(44100, Ordering::Release);
        dac_time_ns.store(now_nanos(), Ordering::Release);
        assert_eq!(clock.current_position().as_millis(), 1000);

        // Pause freezes clock at 1.000s
        clock.on_pause();
        assert_eq!(clock.current_position().as_millis(), 1000);

        // Further DAC counter changes while paused do not alter position
        dac_frames.store(50000, Ordering::Release);
        assert_eq!(clock.current_position().as_millis(), 1000);

        // Resume from pause
        clock.on_play();
        assert_eq!(clock.current_position().as_millis(), 1000);

        // 22050 more frames after resume = 1.500s
        dac_frames.store(50000 + 22050, Ordering::Release);
        dac_time_ns.store(now_nanos(), Ordering::Release);
        assert_eq!(clock.current_position().as_millis(), 1500);

        // Seek to 30.0s
        clock.on_seek(Duration::from_secs(30));
        assert_eq!(clock.current_position().as_millis(), 30000);

        // 44100 more frames = 31.000s
        dac_frames.fetch_add(44100, Ordering::Release);
        dac_time_ns.store(now_nanos(), Ordering::Release);
        assert_eq!(clock.current_position().as_millis(), 31000);

        // Stop resets to 0
        clock.on_stop();
        assert_eq!(clock.current_position(), Duration::ZERO);
    }

    #[test]
    fn master_clock_preserves_position_across_reconnect() {
        let dac_frames = Arc::new(AtomicU64::new(0));
        let dac_time_ns = Arc::new(AtomicU64::new(now_nanos()));
        let clock = MasterClock::new(Arc::clone(&dac_frames), Arc::clone(&dac_time_ns), 44100);

        clock.on_load();
        // 44100 frames = 1000ms
        dac_frames.store(44100, Ordering::Release);
        dac_time_ns.store(now_nanos(), Ordering::Release);
        assert_eq!(clock.current_position().as_millis(), 1000);

        // Device disconnected/reconnected, sample rate changed to 48000
        clock.on_reconnect();
        clock.set_sample_rate(48000);
        assert_eq!(clock.current_position().as_millis(), 1000);

        // New device outputs 48000 frames = 1 second more -> 2000ms
        dac_frames.fetch_add(48000, Ordering::Release);
        dac_time_ns.store(now_nanos(), Ordering::Release);
        assert_eq!(clock.current_position().as_millis(), 2000);
    }
}
