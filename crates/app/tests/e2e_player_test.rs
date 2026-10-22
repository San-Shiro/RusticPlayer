use engine::Engine;
use rustic_core::api::{Command, CommandAction, Event, PlaybackState};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_player_e2e_playback_seek_pause_and_clock() {
    let engine = Engine::new().expect("Failed to create engine");
    let mut rx = engine.subscribe();

    // 1. Play first track
    println!("[TEST] Sending Play command to start playback...");
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

    engine.send_command(Command {
        origin: None,
        action: CommandAction::PlayTrack(track_key),
    });

    let mut track_playing = false;
    let mut track_changed = false;
    let timeout = sleep(Duration::from_secs(10));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            _ = &mut timeout => {
                panic!("Timeout waiting for playback to start (track_changed={}, track_playing={})", track_changed, track_playing);
            }
            Ok(event) = rx.recv() => {
                println!("[TEST] Received event: {:?}", event);
                match event {
                    Event::TrackChanged(Some(track)) => {
                        println!("[TEST] Track changed to: {}", track.title);
                        track_changed = true;
                    }
                    Event::PlaybackStateChanged(state) => {
                        println!("[TEST] Playback state changed to: {:?}", state);
                        if state == PlaybackState::Playing {
                            track_playing = true;
                        }
                    }
                    _ => {}
                }
                if track_changed && track_playing {
                    break;
                }
            }
        }
    }

    assert!(track_playing && track_changed, "Expected track to be loaded and playing");

    // 2. Allow audio playback to proceed and verify master clock advances
    println!("[TEST] Letting audio play and waiting for master clock to advance...");
    let start_wait = std::time::Instant::now();
    let mut pos1 = engine.current_position();
    while pos1 < Duration::from_millis(100) && start_wait.elapsed() < Duration::from_secs(5) {
        sleep(Duration::from_millis(100)).await;
        pos1 = engine.current_position();
    }
    println!("[TEST] Master clock reported position: {:?}", pos1);
    assert!(
        pos1 >= Duration::from_millis(100),
        "Master clock should have advanced, but was {:?}",
        pos1
    );

    // 3. Test Pause
    println!("[TEST] Sending Pause command...");
    engine.send_command(Command {
        origin: None,
        action: CommandAction::Pause,
    });
    sleep(Duration::from_millis(300)).await;
    let paused_pos = engine.current_position();
    println!("[TEST] Position at pause: {:?}", paused_pos);
    sleep(Duration::from_millis(400)).await;
    let paused_pos_after = engine.current_position();
    assert_eq!(
        paused_pos.as_millis(),
        paused_pos_after.as_millis(),
        "Position should freeze while paused"
    );

    // 4. Test Seek to 5 seconds
    println!("[TEST] Seeking to 5.0s...");
    engine.send_command(Command {
        origin: None,
        action: CommandAction::Seek(Duration::from_secs(5)),
    });
    sleep(Duration::from_millis(200)).await;
    let seek_pos = engine.current_position();
    println!("[TEST] Position after seek: {:?}", seek_pos);
    assert!(
        seek_pos >= Duration::from_millis(4800) && seek_pos <= Duration::from_millis(5500),
        "Position should be near 5.0s, got {:?}",
        seek_pos
    );

    // 5. Test Resume (Play)
    println!("[TEST] Resuming playback after seek...");
    engine.send_command(Command {
        origin: None,
        action: CommandAction::Play,
    });
    sleep(Duration::from_millis(600)).await;
    let pos_resumed = engine.current_position();
    println!("[TEST] Position after resume: {:?}", pos_resumed);
    assert!(
        pos_resumed > seek_pos,
        "Position should advance past seek point: {:?} > {:?}",
        pos_resumed,
        seek_pos
    );

    // 6. Test Volume change
    println!("[TEST] Setting volume to 0.75...");
    engine.send_command(Command {
        origin: None,
        action: CommandAction::SetVolume(0.75),
    });
    
    // Wait for volume event
    let mut vol_changed = false;
    let vol_timeout = sleep(Duration::from_secs(2));
    tokio::pin!(vol_timeout);
    loop {
        tokio::select! {
            _ = &mut vol_timeout => break,
            Ok(event) = rx.recv() => {
                if let Event::VolumeChanged(v) = event {
                    if (v - 0.75).abs() < 0.01 {
                        vol_changed = true;
                        println!("[TEST] Volume change confirmed: {}", v);
                        break;
                    }
                }
            }
        }
    }
    assert!(vol_changed, "Expected VolumeChanged event to confirm volume update");

    println!("[TEST] ALL PLAYBACK & CLOCK TESTS PASSED SUCCESSFULLY!");
    drop(rx);
    drop(engine);
    tokio::time::sleep(Duration::from_millis(100)).await;
}
