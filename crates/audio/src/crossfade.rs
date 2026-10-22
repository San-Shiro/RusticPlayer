//! Crossfade mixer. Pre-decodes the next track and applies equal-power cos/sin
//! volume ramping during transitions.

use std::time::Duration;

pub struct CrossfadeState {
    pub active: bool,
    pub progress_frames: u64,
    pub duration_frames: u64,
}

impl CrossfadeState {
    pub fn new(duration: Duration, sample_rate: u32) -> Self {
        Self {
            active: false,
            progress_frames: 0,
            duration_frames: (duration.as_secs_f64() * sample_rate as f64) as u64,
        }
    }

    pub fn start(&mut self) {
        self.active = true;
        self.progress_frames = 0;
    }

    pub fn step(&mut self, frames: u64) {
        if self.active {
            self.progress_frames += frames;
            if self.progress_frames >= self.duration_frames {
                self.active = false;
            }
        }
    }

    /// Returns `(gain_a, gain_b)` for the current frame index `offset` within a block.
    pub fn gains_at(&self, offset: u64) -> (f32, f32) {
        if !self.active {
            return (1.0, 0.0);
        }
        let pos = (self.progress_frames + offset).min(self.duration_frames) as f32;
        let t = pos / self.duration_frames as f32;
        // Equal power crossfade: A = cos(t * pi/2), B = sin(t * pi/2)
        let phase = t * std::f32::consts::FRAC_PI_2;
        (phase.cos(), phase.sin())
    }
}

pub fn mix_blocks(
    a_samples: &mut [f32],
    b_samples: &[f32],
    state: &mut CrossfadeState,
    channels: usize,
) {
    if !state.active {
        return;
    }
    
    let frames = a_samples.len() / channels;
    let b_frames = b_samples.len() / channels;
    let mix_frames = frames.min(b_frames);

    for f in 0..mix_frames {
        let (gain_a, gain_b) = state.gains_at(f as u64);
        for c in 0..channels {
            let idx = f * channels + c;
            a_samples[idx] = a_samples[idx] * gain_a + b_samples[idx] * gain_b;
        }
    }
    
    state.step(mix_frames as u64);
}
