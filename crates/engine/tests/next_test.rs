use engine::Engine;
use rustic_core::api::{Command, CommandAction, Event, Track, TrackSource};
use std::time::Duration;

#[tokio::test]
async fn test_next_in_collection() {
    let engine = Engine::new().expect("Failed to create engine");
    let mut rx = engine.subscribe();

    let path1 = std::fs::canonicalize("dummy_library/Red Distortion/Analog Overdrive/01 - Garage Anthem.mp4")
        .unwrap_or_else(|_| std::path::PathBuf::from("dummy_library/Red Distortion/Analog Overdrive/01 - Garage Anthem.mp4"));
    let path2 = std::fs::canonicalize("dummy_library/Red Distortion/Analog Overdrive/02 - Fuzzbox Summer.mp4")
        .unwrap_or_else(|_| std::path::PathBuf::from("dummy_library/Red Distortion/Analog Overdrive/02 - Fuzzbox Summer.mp4"));

    let track1 = Track {
        id: "test1".into(),
        title: "Test Track 1".into(),
        artists: vec!["Artist 1".into()],
        album: "Album 1".into(),
        duration_secs: 180,
        source: TrackSource::Local(path1),
        isrc: None,
        artwork_url: None,
        content_hash: None,
    };
    let track2 = Track {
        id: "test2".into(),
        title: "Test Track 2".into(),
        artists: vec!["Artist 2".into()],
        album: "Album 1".into(),
        duration_secs: 180,
        source: TrackSource::Local(path2),
        isrc: None,
        artwork_url: None,
        content_hash: None,
    };

    engine.send_command(Command {
        origin: None,
        action: CommandAction::PlayCollection {
            tracks: vec![track1.clone(), track2.clone()],
            start_index: 0,
            title: "Album 1".into(),
            id: "alb1".into(),
            shuffle: false,
        },
    });

    // Wait for track 1 to start
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Send Next command
    engine.send_command(Command {
        origin: None,
        action: CommandAction::Next,
    });

    let mut track_changed_to_2 = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(ev) = rx.try_recv() {
            println!("Received event: {:?}", ev);
            if let Event::TrackChanged(Some(tr)) = ev {
                if tr.id == "test2" {
                    track_changed_to_2 = true;
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(track_changed_to_2, "Engine failed to transition to track 2 on Next");
}

#[tokio::test]
async fn test_next_advances_to_radio_suggestions() {
    let engine = Engine::new().expect("Failed to create engine");
    let mut rx = engine.subscribe();

    let path1 = std::fs::canonicalize("dummy_library/Red Distortion/Analog Overdrive/01 - Garage Anthem.mp4")
        .unwrap_or_else(|_| std::path::PathBuf::from("dummy_library/Red Distortion/Analog Overdrive/01 - Garage Anthem.mp4"));
    let path_radio = std::fs::canonicalize("dummy_library/Red Distortion/Analog Overdrive/03 - Broken Strings.mp4")
        .unwrap_or_else(|_| std::path::PathBuf::from("dummy_library/Red Distortion/Analog Overdrive/03 - Broken Strings.mp4"));

    let track1 = Track {
        id: "t1".into(),
        title: "Track 1".into(),
        artists: vec!["Artist 1".into()],
        album: "Single".into(),
        duration_secs: 180,
        source: TrackSource::Local(path1),
        isrc: None,
        artwork_url: None,
        content_hash: None,
    };
    let radio_track = Track {
        id: "r1".into(),
        title: "Radio Recommendation 1".into(),
        artists: vec!["Artist Radio".into()],
        album: "Radio".into(),
        duration_secs: 180,
        source: TrackSource::Local(path_radio),
        isrc: None,
        artwork_url: None,
        content_hash: None,
    };

    engine.send_command(Command {
        origin: None,
        action: CommandAction::PlaySingleTrack(track1),
    });

    // Ingest radio suggestions into Lane 3
    engine.send_command(Command {
        origin: None,
        action: CommandAction::IngestRadioTracks(vec![radio_track.clone()]),
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // Send Next command
    engine.send_command(Command {
        origin: None,
        action: CommandAction::Next,
    });

    let mut track_changed_to_radio = false;
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if let Ok(ev) = rx.try_recv() {
            if let Event::TrackChanged(Some(tr)) = ev {
                if tr.id == "r1" {
                    track_changed_to_radio = true;
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    assert!(track_changed_to_radio, "Engine failed to advance to radio suggestion on Next");
}

