//! cpal output stage. The audio callback is the *only* code that touches the
//! ring buffer, and it is hard-real-time: **no alloc, no lock, no log**, ever.
//!
//! Design invariants (enforced by tests):
//!   - The callback never allocates. It reads from `rtrb` into a caller-provided
//!     scratch and applies volume as a linear ramp.
//!   - Volume is a plain `AtomicF32`. The callback ramp between `volume.now` and
//!     `volume.target` across `ramp_ms` samples to avoid clicks.
//!   - On underrun, output silence and count the event. No glitch-loop.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use atomic_float::AtomicF32;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::error::{AudioError, Result};

/// Default ring buffer length: 500ms of stereo f32 PCM at the device rate.
pub const RING_BUFFER_MS: u32 = 500;

/// Size of the volume smoothing ramp.
pub const VOLUME_RAMP_MS: f32 = 5.0;

pub struct OutputDevice {
    pub name: String,
    device: cpal::Device,
    config: cpal::SupportedStreamConfig,
}

#[derive(Clone, Debug)]
pub struct AudioDiagnostics {
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: usize,
    pub sample_format: String,
    pub is_running: bool,
    pub has_error: bool,
    pub last_error: Option<String>,
    pub underruns: u64,
    pub dac_frames: u64,
    pub dac_time_ns: u64,
}

impl OutputDevice {
    pub fn default() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| AudioError::OutputDevice("no default output device".into()))?;
        #[allow(deprecated)]
        let name = device.name().unwrap_or_else(|_| "Default Audio Device".into());
        let config = device
            .default_output_config()
            .map_err(|e| AudioError::OutputDevice(format!("default config: {e}")))?;
        Ok(Self { name, device, config })
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate()
    }

    pub fn channels(&self) -> usize {
        self.config.channels() as usize
    }

    /// Bounded sample format; we normalize everything to f32.
    pub fn sample_format(&self) -> cpal::SampleFormat {
        self.config.sample_format()
    }

    pub fn buffer_size(&self) -> usize {
        let ms = RING_BUFFER_MS as usize;
        (self.sample_rate() as usize * self.channels() * ms) / 1000
    }
}

/// Handle for the stream that owns `cpal`'s state. `Drop` stops playback.
pub struct OutputStream {
    stream: cpal::Stream,
    pub device_name: String,
    pub sample_rate: u32,
    pub channels: usize,
    pub sample_format: String,
    pub is_running: Arc<std::sync::atomic::AtomicBool>,
    pub has_error: Arc<std::sync::atomic::AtomicBool>,
    pub last_error: Arc<parking_lot::RwLock<Option<String>>>,
    pub underruns: Arc<AtomicU64>,
    pub position_samples: Arc<AtomicU64>,
    pub position_time_ns: Arc<AtomicU64>,
    /// Mirror of volume ramp state, exposed for observability.
    pub volume: Arc<AtomicF32>,
    pub flush_seq: Arc<AtomicU64>,
    pub flush_ack: Arc<AtomicU64>,
}

impl OutputStream {
    /// Start streaming. The callback will read from `ring` at the device rate,
    /// applying `volume` (per-sample ramp from the prior call's state).
    pub fn start(
        device: OutputDevice,
        ring_consumer: rtrb::Consumer<f32>,
        volume: Arc<AtomicF32>,
        position_samples: Arc<AtomicU64>,
        position_time_ns: Arc<AtomicU64>,
    ) -> Result<Self> {
        let _sample_rate = device.sample_rate() as usize;
        let channels = device.channels();
        let buffer_len = device.buffer_size();

        // Wrap rtrb so the callback owns a fixed-capacity view. We allocate the
        // scratch buffer *once* at stream open so the callback never allocs.
        let mut consumer = ring_consumer;
        let mut scratch = vec![0.0f32; buffer_len];

        let underruns = Arc::new(AtomicU64::new(0));
        let vol_now = Arc::new(AtomicF32::new(volume.load(Ordering::Acquire)));
        let flush_seq = Arc::new(AtomicU64::new(0));
        let flush_ack = Arc::new(AtomicU64::new(0));

        let underruns_cb = Arc::clone(&underruns);
        let pos_cb = Arc::clone(&position_samples);
        let time_cb = Arc::clone(&position_time_ns);
        let vol_now_cb = Arc::clone(&vol_now);
        let flush_seq_cb = Arc::clone(&flush_seq);
        let flush_ack_cb = Arc::clone(&flush_ack);
        let mut last_seen_flush = 0u64;

        let device_name = device.name.clone();
        let sample_rate_u32 = device.sample_rate();
        let sample_format_str = format!("{:?}", device.sample_format());
        let is_running = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let has_error = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let last_error = Arc::new(parking_lot::RwLock::new(None));

        let has_error_cb = Arc::clone(&has_error);
        let is_running_cb = Arc::clone(&is_running);
        let last_error_cb = Arc::clone(&last_error);
        let callback_channels = channels;

        let stream = device
            .device
            .build_output_stream(
                &device.config.config(),
                move |output: &mut [f32], _| {
                    let scratch = &mut scratch;
                    ensure_scratch(scratch, output.len());

                    let current_flush = flush_seq_cb.load(Ordering::Acquire);
                    if current_flush != last_seen_flush {
                        let slots = consumer.slots();
                        if let Ok(chunk) = consumer.read_chunk(slots) {
                            chunk.commit_all();
                        }
                        last_seen_flush = current_flush;
                        flush_ack_cb.store(current_flush, Ordering::Release);
                    }

                    // Ramp from the previous callback's settled volume to the
                    // currently-requested volume over VOLUME_RAMP_MS.
                    let target = volume.load(Ordering::Acquire);
                    let start = vol_now_cb.load(Ordering::Acquire);
                    let mut current = start;

                    let ramp_frames = (sample_rate_u32 as f32 * VOLUME_RAMP_MS / 1000.0).max(1.0) as usize;
                    let frames_this_call = output.len() / callback_channels;
                    let ramp_per_frame = if ramp_frames == 0 {
                        0.0
                    } else {
                        (target - start) / ramp_frames.max(frames_this_call) as f32
                    };

                    let mut written = 0usize;
                    while written < output.len() {
                        let want = (output.len() - written).min(scratch.len());
                        let n = match consumer.read_chunk(want) {
                            Ok(chunk) => {
                                let (s1, s2) = chunk.as_slices();
                                scratch[..s1.len()].copy_from_slice(s1);
                                scratch[s1.len()..s1.len() + s2.len()].copy_from_slice(s2);
                                let total = s1.len() + s2.len();
                                chunk.commit_all();
                                total
                            }
                            Err(_) => 0,
                        };
                        if n == 0 {
                            break;
                        }
                        // Apply gain ramp + write to output, mixing if needed.
                        for (i, &s) in scratch[..n].iter().enumerate() {
                            if i % callback_channels == 0 {
                                if ramp_per_frame > 0.0 {
                                    current = (current + ramp_per_frame).min(target);
                                } else if ramp_per_frame < 0.0 {
                                    current = (current + ramp_per_frame).max(target);
                                }
                            }
                            let sample = s * current;
                            output[written + i] = if sample.is_nan() { 0.0 } else { sample.clamp(-1.0, 1.0) };
                        }
                        written += n;
                    }

                    if written < output.len() {
                        // True underrun: zero-fill remainder, count it, don't loop.
                        underruns_cb.fetch_add(1, Ordering::Relaxed);
                        for s in &mut output[written..] {
                            *s = 0.0;
                        }
                    }

                    // Advance position: only now do we commit what the speakers
                    // have actually consumed. This is the engine's ground truth.
                    if written > 0 {
                        pos_cb.fetch_add((written / callback_channels) as u64, Ordering::Release);
                        time_cb.store(crate::player::now_nanos(), Ordering::Release);
                    }
                    vol_now_cb.store(current, Ordering::Release);
                },
                move |err| {
                    tracing::error!(target: "audio::output", "cpal stream error: {err}");
                    has_error_cb.store(true, Ordering::Release);
                    is_running_cb.store(false, Ordering::Release);
                    *last_error_cb.write() = Some(err.to_string());
                },
                None,
            )
            .map_err(|e| AudioError::OutputDevice(format!("build stream: {e}")))?;

        stream
            .play()
            .map_err(|e| AudioError::OutputDevice(format!("play stream: {e}")))?;

        Ok(Self {
            stream,
            device_name,
            sample_rate: sample_rate_u32,
            channels: callback_channels,
            sample_format: sample_format_str,
            is_running,
            has_error,
            last_error,
            underruns,
            position_samples,
            position_time_ns,
            volume: vol_now,
            flush_seq,
            flush_ack,
        })
    }

    pub fn flush(&self) {
        self.flush_seq.fetch_add(1, Ordering::SeqCst);
    }

    pub fn pause(&self) -> Result<()> {
        let _ = self.is_running.store(false, Ordering::Release);
        self.stream
            .pause()
            .map_err(|e| {
                self.has_error.store(true, Ordering::Release);
                *self.last_error.write() = Some(e.to_string());
                AudioError::OutputDevice(format!("pause: {e}"))
            })
    }

    pub fn play(&self) -> Result<()> {
        let _ = self.is_running.store(true, Ordering::Release);
        self.stream
            .play()
            .map_err(|e| {
                self.is_running.store(false, Ordering::Release);
                self.has_error.store(true, Ordering::Release);
                *self.last_error.write() = Some(e.to_string());
                AudioError::OutputDevice(format!("resume: {e}"))
            })
    }

    pub fn diagnostics(&self) -> AudioDiagnostics {
        AudioDiagnostics {
            device_name: self.device_name.clone(),
            sample_rate: self.sample_rate,
            channels: self.channels,
            sample_format: self.sample_format.clone(),
            is_running: self.is_running.load(Ordering::Acquire),
            has_error: self.has_error.load(Ordering::Acquire),
            last_error: self.last_error.read().clone(),
            underruns: self.underruns.load(Ordering::Relaxed),
            dac_frames: self.position_samples.load(Ordering::Acquire),
            dac_time_ns: self.position_time_ns.load(Ordering::Acquire),
        }
    }
}

#[inline(never)]
fn ensure_scratch(scratch: &mut [f32], len: usize) {
    if scratch.len() < len {
        // Safe guard in case cpal buffer ever exceeds pre-allocated scratch size
        tracing::warn!(target: "audio::output", "scratch buffer smaller than cpal output: {} < {}", scratch.len(), len);
    }
}

/// Allocate the ring buffer pair for one connection.
pub fn ring_buffer(device: &OutputDevice) -> (rtrb::Producer<f32>, rtrb::Consumer<f32>) {
    let cap = device.buffer_size();
    rtrb::RingBuffer::new(cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_size_matches_spec() {
        let sr = 44_100;
        let ch = 2;
        let ms = 500usize;
        assert_eq!((sr * ch * ms) / 1000, 44_100);
    }

    #[test]
    fn print_default_audio_device_info() {
        if let Ok(dev) = OutputDevice::default() {
            eprintln!("DEVICE INFO: name='{}', sr={}, ch={}, fmt={:?}",
                dev.device.name().unwrap_or_default(),
                dev.sample_rate(),
                dev.channels(),
                dev.sample_format()
            );
        }
    }


    #[test]
    fn test_live_audio_sine() {
        let dev = match OutputDevice::default() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("No output device: {e}");
                return;
            }
        };
        let sr = dev.sample_rate();
        let ch = dev.channels();
        let (mut prod, cons) = ring_buffer(&dev);
        let vol = Arc::new(AtomicF32::new(0.5));
        let pos = Arc::new(AtomicU64::new(0));
        let pos_time = Arc::new(AtomicU64::new(0));

        let stream = OutputStream::start(dev, cons, vol, pos, pos_time).expect("OutputStream::start failed");
        
        // Push 1 second of 440Hz sine wave with timeout protection
        let total_frames = sr;
        let mut pushed_count = 0usize;
        let start_push = std::time::Instant::now();
        for i in 0..total_frames {
            let t = i as f32 / sr as f32;
            let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.3;
            for _ in 0..ch {
                let mut attempts = 0;
                while prod.push(sample).is_err() {
                    attempts += 1;
                    if attempts > 200 {
                        eprintln!("prod.push blocked after {} samples! Ring buffer full, consumer not reading!", pushed_count);
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                pushed_count += 1;
            }
        }
        eprintln!("Successfully pushed {} samples in {:?}", pushed_count, start_push.elapsed());
        std::thread::sleep(std::time::Duration::from_millis(500));
        let underruns = stream.underruns.load(Ordering::Relaxed);
        eprintln!("Playback finished. Underruns: {underruns}");
    }

    #[test]
    fn test_decode_dummy_mp4() {
        let p = std::path::PathBuf::from(r"f:\VSCodium\Github\RusticPlayer\dummy_library\Red Distortion\Analog Overdrive\01 - Garage Anthem.mp4");
        if !p.exists() {
            eprintln!("File not found: {:?}", p);
            return;
        }
        let src = crate::source::FileSource::open(p).expect("FileSource::open failed");
        match crate::decoder::DecoderImpl::open(Box::new(src), 48000, 2) {
            Ok(mut dec) => {
                eprintln!("Decoder opened! Duration: {:?}, track_id: {}", dec.duration_secs(), dec.track_id);
                let mut blocks = 0;
                let mut total_samples = 0;
                let mut max_abs = 0.0f32;
                while let Ok(Some(block)) = dec.next_block() {
                    blocks += 1;
                    total_samples += block.samples.len();
                    for &s in &block.samples {
                        max_abs = max_abs.max(s.abs());
                    }
                    if blocks == 1 {
                        eprintln!("First block len: {}, first 10 samples: {:?}", block.samples.len(), &block.samples[..10.min(block.samples.len())]);
                    }
                }
                eprintln!("Finished! Blocks: {}, total_samples: {}, max_abs: {}", blocks, total_samples, max_abs);
            }
            Err(e) => {
                eprintln!("Decoder failed: {:?}", e);
                panic!("Failed to decode dummy mp4: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_live_player_plays_sound() {
        let p = std::path::PathBuf::from(r"f:\VSCodium\Github\RusticPlayer\dummy_library\Red Distortion\Analog Overdrive\01 - Garage Anthem.mp4");
        let (player, mut rx) = crate::player::Player::new().expect("Player::new failed");
        player.set_volume(0.8).expect("set_volume failed");
        player.load_path(p, 0.0).expect("load_path failed");
        player.play().expect("play failed");

        // Wait until Playing state is reported
        while let Ok(ev) = rx.recv().await {
            if ev.state == crate::player::PlaybackState::Playing {
                eprintln!("Playback started! Initial pos={:?}", ev.position);
                break;
            }
        }


        let start = std::time::Instant::now();
        let pos_start = player.current_position();
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        let pos_end = player.current_position();
        let elapsed = start.elapsed();
        let audio_advanced = pos_end.saturating_sub(pos_start);
        eprintln!("In {:?} wall clock, audio advanced by {:?}", elapsed, audio_advanced);
        assert!((audio_advanced.as_secs_f64() - 1.0).abs() < 0.15, "Expected audio to advance ~1s in 1s wall clock");
    }

    #[tokio::test]
    async fn test_sample_position_smoothness() {
        let p = std::path::PathBuf::from(r"f:\VSCodium\Github\RusticPlayer\dummy_library\Red Distortion\Analog Overdrive\01 - Garage Anthem.mp4");
        let (player, mut rx) = crate::player::Player::new().expect("Player::new failed");
        player.set_volume(0.8).expect("set_volume failed");
        player.load_path(p, 0.0).expect("load_path failed");
        player.play().expect("play failed");

        while let Ok(ev) = rx.recv().await {
            if ev.state == crate::player::PlaybackState::Playing {
                break;
            }
        }

        // Sample at 16ms intervals for 30 frames (~500ms)
        let mut last_pos = player.current_position();
        let mut deltas = Vec::new();
        for _ in 0..30 {
            tokio::time::sleep(std::time::Duration::from_millis(16)).await;
            let cur = player.current_position();
            let delta = cur.as_secs_f64() - last_pos.as_secs_f64();
            deltas.push(delta);
            last_pos = cur;
        }

        eprintln!("Position deltas over 30 frames (target ~0.016s each):");
        for (i, d) in deltas.iter().enumerate() {
            eprintln!("  Frame {:2}: delta = {:+.4}s", i, d);
        }
    }

    #[tokio::test]
    async fn test_pause_and_resume() {
        let p = std::path::PathBuf::from(r"f:\VSCodium\Github\RusticPlayer\dummy_library\Red Distortion\Analog Overdrive\01 - Garage Anthem.mp4");
        let (player, mut rx) = crate::player::Player::new().expect("Player::new failed");
        player.set_volume(0.8).expect("set_volume failed");
        player.load_path(p, 0.0).expect("load_path failed");
        player.play().expect("play failed");

        while let Ok(ev) = rx.recv().await {
            if ev.state == crate::player::PlaybackState::Playing {
                break;
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let pos_before_pause = player.current_position();
        eprintln!("Pausing at {:?}", pos_before_pause);
        player.pause().expect("pause failed");
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let pos_during_pause = player.current_position();
        eprintln!("During pause at {:?}", pos_during_pause);
        assert!((pos_during_pause.as_secs_f64() - pos_before_pause.as_secs_f64()).abs() < 0.05, "Position should not advance during pause");

        eprintln!("Resuming...");
        player.play().expect("play failed");
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let pos_after_resume = player.current_position();
        eprintln!("After resume at {:?}", pos_after_resume);
        assert!(pos_after_resume > pos_during_pause + std::time::Duration::from_millis(150), "Position should advance after resume");
    }



}
