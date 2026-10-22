use std::time::{Duration, Instant};
use ui_kit::atoms::PlaybackTimeline;

#[test]
fn test_playback_timeline_linear_progression() {
    let now = Instant::now();
    let timeline = PlaybackTimeline {
        anchor_position: 10.0,
        anchor_instant: now,
        playback_rate: 1.0,
        duration: 100.0,
        playing: true,
        revision: 1,
    };

    // At exact anchor time
    assert_eq!(timeline.position_at(now), 10.0);
    assert!((timeline.fraction_at(now) - 0.1).abs() < 1e-5);

    // 2 seconds later
    let later = now + Duration::from_secs(2);
    assert!((timeline.position_at(later) - 12.0).abs() < 1e-4);
    assert!((timeline.fraction_at(later) - 0.12).abs() < 1e-4);

    // Clamped at duration
    let way_later = now + Duration::from_secs(200);
    assert_eq!(timeline.position_at(way_later), 100.0);
    assert_eq!(timeline.fraction_at(way_later), 1.0);
}

#[test]
fn test_playback_timeline_paused() {
    let now = Instant::now();
    let timeline = PlaybackTimeline {
        anchor_position: 15.5,
        anchor_instant: now,
        playback_rate: 1.0,
        duration: 100.0,
        playing: false,
        revision: 1,
    };

    let later = now + Duration::from_secs(10);
    assert_eq!(timeline.position_at(later), 15.5);
    assert!((timeline.fraction_at(later) - 0.155).abs() < 1e-5);
}


