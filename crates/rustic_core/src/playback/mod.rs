use std::time::Instant;
use crate::api::PlaybackState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaybackClockSnapshot {
    pub base_position: f64,
    pub base_instant: Instant,
    pub playback_rate: f64,
    pub duration: f64,
    pub state: PlaybackState,
    pub revision: u64,
}

impl PlaybackClockSnapshot {
    pub fn new(base_position: f64, duration: f64, state: PlaybackState) -> Self {
        Self {
            base_position,
            base_instant: Instant::now(),
            playback_rate: 1.0,
            duration,
            state,
            revision: 0,
        }
    }

    #[inline]
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    #[inline]
    pub fn is_buffering(&self) -> bool {
        self.state == PlaybackState::Buffering
    }

    #[inline]
    pub fn position_at(&self, now: Instant) -> f64 {
        if self.state == PlaybackState::Playing {
            let elapsed = now.saturating_duration_since(self.base_instant).as_secs_f64() * self.playback_rate;
            (self.base_position + elapsed).clamp(0.0, self.duration.max(0.0))
        } else {
            self.base_position.clamp(0.0, self.duration.max(0.0))
        }
    }

    #[inline]
    pub fn fraction_at(&self, now: Instant) -> f32 {
        if self.duration <= 0.0 {
            0.0
        } else {
            (self.position_at(now) / self.duration).clamp(0.0, 1.0) as f32
        }
    }
}

impl Default for PlaybackClockSnapshot {
    fn default() -> Self {
        Self::new(0.0, 0.0, PlaybackState::Stopped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_clock_interpolation() {
        let clock = PlaybackClockSnapshot::new(10.0, 100.0, PlaybackState::Playing);
        let future = clock.base_instant + Duration::from_millis(500);
        let pos = clock.position_at(future);
        assert!((pos - 10.5).abs() < 0.001);
        let frac = clock.fraction_at(future);
        assert!((frac - 0.105).abs() < 0.001);

        let paused = PlaybackClockSnapshot::new(10.0, 100.0, PlaybackState::Paused);
        assert_eq!(paused.position_at(future), 10.0);
    }
}
