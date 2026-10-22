use engine::Engine;
use rustic_core::api::{AuthFlow, Command, CommandAction, Event};
use sources::{SourceProvider, SpotifyProvider, YtMusicProvider};
use std::env;
use std::time::Duration;
use tracing::info;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let command = &args[1];
        if command == "login" && args.len() > 2 {
            let provider_name = &args[2];
            match provider_name.as_str() {
                "ytm" => {
                    let provider = YtMusicProvider::new();
                    let state = provider.login(AuthFlow::BrowserCookie).await?;
                    info!("Logged in to YouTube Music: {:?}", state.account_name);
                }
                "spotify" => {
                    let provider = SpotifyProvider::new();
                    let state = provider.login(AuthFlow::Pkce).await?;
                    info!("Logged in to Spotify: {:?}", state.account_name);
                }
                _ => println!("Unknown provider. Supported: ytm, spotify"),
            }
            return Ok(());
        } else if command == "import" {
            info!("Importing libraries from connected sources...");
            let ytm = YtMusicProvider::new();
            let playlists = ytm.library().await?;
            info!("Imported {} playlists from YTM", playlists.len());
            return Ok(());
        } else if command == "play" && args.len() > 2 {
            let track_id = &args[2];
            info!("Playing track: {}", track_id);
            let engine = Engine::new()?;
            let mut rx = engine.subscribe();
            engine.send_command(Command {
                action: CommandAction::PlayTrack(track_id.to_string()),
                origin: None,
            });
            while let Ok(event) = rx.recv().await {
                match event {
                    Event::PlaybackStateChanged(state) => println!("\nState: {:?}", state),
                    Event::PositionChanged { position_secs, .. } => {
                        print!("\rPosition: {:.1}s", position_secs);
                        let _ = std::io::stdout().flush();
                    }
                    Event::Error(err) => println!("\nError: {:?}", err),
                    _ => {}
                }
            }
            return Ok(());
        }
    }

    // Interactive REPL Mode
    println!("Starting Rustic Player CLI Interactive Mode");
    let engine = Engine::new()?;
    let mut rx = engine.subscribe();
    
    // Background task to print events (with simple UI)
    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                Event::PlaybackStateChanged(state) => {
                    println!("\r\n[State] {:?}", state);
                    print!("rustic> ");
                    let _ = std::io::stdout().flush();
                }
                Event::TrackChanged(track) => {
                    if let Some(ref t) = track {
                        println!("\r\n[Now Playing] {} - {}", t.title, t.artists.join(", "));
                        print!("rustic> ");
                        let _ = std::io::stdout().flush();
                    }
                }
                Event::QueueChanged(queue) => {
                    println!("\r\n[Queue] {} tracks", queue.len());
                    print!("rustic> ");
                    let _ = std::io::stdout().flush();
                }
                Event::Error(err) => {
                    println!("\r\n[Error] {:?}", err);
                    print!("rustic> ");
                    let _ = std::io::stdout().flush();
                }
                _ => {} // position is too spammy for a basic repl without curses
            }
        }
    });

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        print!("rustic> ");
        let _ = io::stdout().flush();
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        let mut parts = input.splitn(2, ' ');
        let cmd = parts.next().unwrap_or("");
        let arg = parts.next().unwrap_or("");

        match cmd {
            "play" => {
                if arg.is_empty() {
                    engine.send_command(Command { action: CommandAction::Play, origin: None });
                } else {
                    engine.send_command(Command {
                        action: CommandAction::PlayTrack(arg.to_string()),
                        origin: None,
                    });
                }
            }
            "pause" => engine.send_command(Command { action: CommandAction::Pause, origin: None }),
            "stop" => engine.send_command(Command { action: CommandAction::Stop, origin: None }),
            "next" => engine.send_command(Command { action: CommandAction::Next, origin: None }),
            "prev" | "previous" => engine.send_command(Command { action: CommandAction::Previous, origin: None }),
            "toggle" => engine.send_command(Command { action: CommandAction::TogglePlay, origin: None }),
            "vol" | "volume" => {
                if let Ok(v) = arg.parse::<f32>() {
                    engine.send_command(Command { action: CommandAction::SetVolume(v), origin: None });
                }
            }
            "seek" => {
                if let Ok(secs) = arg.parse::<u64>() {
                    engine.send_command(Command { action: CommandAction::Seek(Duration::from_secs(secs)), origin: None });
                }
            }
            "quit" | "exit" => break,
            "help" => {
                println!("Commands:");
                println!("  play [query]  - Resume or play a song");
                println!("  pause         - Pause playback");
                println!("  stop          - Stop playback");
                println!("  next          - Next track");
                println!("  prev          - Previous track");
                println!("  toggle        - Toggle play/pause");
                println!("  vol <0.0-1.5> - Set volume");
                println!("  seek <secs>   - Seek to seconds");
                println!("  quit/exit     - Exit");
            }
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }

    Ok(())
}
