use engine::Engine;
use rustic_core::api::{Command, CommandAction, Event, PlaybackState};
use std::time::Duration;

#[tokio::test]
async fn test_live_engine_plays_track() {
    let engine = Engine::new().expect("Failed to create engine");
    let mut rx = engine.subscribe();

    let candidates = [
        std::path::PathBuf::from(r"dummy_library\Red Distortion\Analog Overdrive\01 - Garage Anthem.mp4"),
        std::path::PathBuf::from(r"../../dummy_library\Red Distortion\Analog Overdrive\01 - Garage Anthem.mp4"),
        std::path::PathBuf::from(r"../dummy_library\Red Distortion\Analog Overdrive\01 - Garage Anthem.mp4"),
    ];
    let track_key = candidates
        .into_iter()
        .find(|p| p.exists())
        .map(|p| std::fs::canonicalize(&p).unwrap_or(p).to_string_lossy().to_string())
        .unwrap_or_else(|| "local_23b59ce64ae918b71cadeeceb40dc326".into());

    // Send PlayTrack command
    engine.send_command(Command {
        origin: None,
        action: CommandAction::PlayTrack(track_key),
    });

    let mut became_playing = false;
    let start = std::time::Instant::now();

    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(ev) = rx.try_recv() {
            println!("Received event: {:?}", ev);
            if let Event::PlaybackStateChanged(PlaybackState::Playing) = ev {
                became_playing = true;
                break;
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(became_playing, "Engine failed to transition to Playing state");

    // Wait 1 second and verify current_position advances
    let pos_start = engine.current_position();
    tokio::time::sleep(Duration::from_millis(1000)).await;
    let pos_end = engine.current_position();
    println!("pos_start: {:?}, pos_end: {:?}", pos_start, pos_end);

    assert!(pos_end > pos_start, "Engine current_position did not advance!");
}
