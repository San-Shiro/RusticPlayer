use audio::decoder::DecoderImpl;
use audio::source::HttpSource;
use sources::ytmusic::client::YtMusicProvider;
use sources::SourceProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("=== Testing YouTube Music Resolution & Symphonia HTTP Playback Pipeline ===");
    let provider = YtMusicProvider::new();
    println!("Provider authenticated: {}", provider.is_authenticated());

    println!("\n0. Searching for 'Just the Two of Us Grover Washington'...");
    let results = provider.search("Just the Two of Us Grover Washington", 3).await?;
    for (i, t) in results.iter().enumerate() {
        println!("  [{}] {} - {} (id: {})", i, t.title, t.artists.join(", "), t.id);
    }
    let track = results.into_iter().next().expect("At least one search result");
    let video_id = track.id.clone();

    println!("\n1. Resolving stream for video_id '{}'...", video_id);
    let resolved = provider.resolve(&track).await?;
    println!("SUCCESS! Resolved stream:");
    println!("  Codec: {}", resolved.codec);
    println!("  Bitrate: {} kbps", resolved.bitrate_kbps);
    println!("  Stream URL: {}...", &resolved.stream_url[..resolved.stream_url.len().min(90)]);

    println!("\n2. Spawning dedicated audio thread (simulating audio player thread)...");
    let stream_url = resolved.stream_url;
    let handle = std::thread::spawn(move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Initializing Range-Buffered HttpSource on audio thread...");
        let http_source = HttpSource::new(stream_url);

        println!("Probing and Opening Symphonia DecoderImpl (48kHz stereo)...");
        let mut decoder = match DecoderImpl::open(Box::new(http_source), 48000, 2) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Decoder open failed: {:?}", e);
                return Err(Box::new(e));
            }
        };

        println!("SUCCESS! Decoder opened.");
        println!("  Duration: {:?}", decoder.duration_secs().map(|s| format!("{:.2}s", s)));

        println!("\nDecoding first 10 blocks of PCM audio frames from remote HTTP range source...");
        let mut total_samples = 0;
        for i in 0..10 {
            match decoder.next_block() {
                Ok(Some(block)) => {
                    total_samples += block.samples.len();
                    println!("  Block [{}]: decoded {} samples (frame_offset: {})", i, block.samples.len(), block.frame_offset);
                }
                Ok(None) => {
                    println!("  EOF reached early at block {}", i);
                    break;
                }
                Err(e) => {
                    eprintln!("  Decode error at block {}: {:?}", i, e);
                    return Err(Box::new(e));
                }
            }
        }

        println!("\nSUCCESS! Successfully decoded {} total interleaved audio samples from live YouTube Music stream!", total_samples);
        assert!(total_samples > 0, "Expected decoded audio samples");
        Ok(())
    });

    handle.join().unwrap().map_err(|e| format!("{:?}", e))?;
    println!("Task 2.2 & 2.3 verification COMPLETE: Full Innertube -> HttpSource -> Symphonia pipeline functional!\n");

    Ok(())
}
