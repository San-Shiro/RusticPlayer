use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, COOKIE};
use serde_json::Value;
use sha1::{Sha1, Digest};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        run().await
    })
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cookies_json = fs::read_to_string("cookies.json")?;
    let cookies: Vec<Value> = serde_json::from_str(&cookies_json)?;

    let mut cookie_str = String::new();
    let mut sapisid = String::new();

    for c in cookies {
        if let (Some(name), Some(val)) = (c["name"].as_str(), c["value"].as_str()) {
            cookie_str.push_str(&format!("{}={}; ", name, val));
            if name == "SAPISID" || name == "__Secure-3PAPISID" {
                sapisid = val.to_string();
            }
        }
    }

    if sapisid.is_empty() {
        return Err("SAPISID cookie not found".into());
    }

    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let origin = "https://music.youtube.com";
    let input = format!("{} {} {}", ts, sapisid, origin);
    
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    let hex_hash = hex::encode(hash);
    
    let sapisidhash = format!("SAPISIDHASH {}_{}", ts, hex_hash);
    
    println!("Generated SAPISIDHASH auth header successfully.");
    
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&sapisidhash)?);
    headers.insert(COOKIE, HeaderValue::from_str(&cookie_str)?);
    headers.insert("X-Origin", HeaderValue::from_str(origin)?);
    headers.insert("Origin", HeaderValue::from_str(origin)?);
    
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
        
    // 1. Test Innertube Player (TVHTML5) to see if we can fetch streams
    println!("\n--- Testing InnerTube Player API (TVHTML5) ---");
    let url = "https://www.youtube.com/youtubei/v1/player";
    let payload = serde_json::json!({
        "context": {
            "client": {
                "clientName": "TVHTML5",
                "clientVersion": "7.20230412.00.00",
                "hl": "en",
                "gl": "US",
            }
        },
        "videoId": "jNQXAC9IVRw" // Me at the zoo
    });

    let res = client.post(url).json(&payload).send().await?;
    println!("Status: {}", res.status());
    let body: Value = res.json().await?;
    println!("Playability: {:?}", body["playabilityStatus"]["status"]);
    if let Some(formats) = body["streamingData"]["adaptiveFormats"].as_array() {
        println!("Found {} adaptive formats.", formats.len());
    } else {
        println!("No adaptive formats found.");
    }
    
    // 2. Test Account Menu (WEB) to verify authentication is actually working
    println!("\n--- Testing Authenticated Endpoint (account_menu) ---");
    let url2 = "https://www.youtube.com/youtubei/v1/account/account_menu";
    let payload2 = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.20231010.00.00",
            }
        }
    });
    let res2 = client.post(url2).json(&payload2).send().await?;
    println!("Account Menu Status: {}", res2.status());
    let body2: Value = res2.json().await?;
    
    // If auth worked, we usually get actions/menus tied to the user
    if body2.get("actions").is_some() {
        println!("Auth SUCCESS! Received account menu actions.");
    } else {
        println!("Auth might have failed. Response: {:.200}", body2.to_string());
    }

    // 3. Test YouTube Music Home Browse with WEB_REMIX
    println!("\n--- Testing YouTube Music Browse (WEB_REMIX FEmusic_home) ---");
    let url3 = "https://music.youtube.com/youtubei/v1/browse";
    let payload3 = serde_json::json!({
        "context": {
            "client": {
                "clientName": "WEB_REMIX",
                "clientVersion": "1.20260213.01.00",
                "hl": "en",
                "gl": "US"
            }
        },
        "browseId": "FEmusic_home"
    });
    let res3 = client.post(url3).json(&payload3).send().await?;
    println!("YTM Browse Status: {}", res3.status());
    let body3: Value = res3.json().await?;
    if let Some(tabs) = body3["contents"]["singleColumnBrowseResultsRenderer"]["tabs"].as_array() {
        println!("Received {} tabs from YTM Home.", tabs.len());
        if let Some(sections) = tabs[0]["tabRenderer"]["content"]["sectionListRenderer"]["contents"].as_array() {
            println!("Found {} home sections!", sections.len());
            for (idx, sec) in sections.iter().take(5).enumerate() {
                if let Some(shelf) = sec.get("musicCarouselShelfRenderer") {
                    let title = shelf["header"]["musicCarouselShelfBasicHeaderRenderer"]["title"]["runs"][0]["text"].as_str().unwrap_or("Untitled");
                    let sub = shelf["header"]["musicCarouselShelfBasicHeaderRenderer"]["strapline"]["runs"][0]["text"].as_str().unwrap_or("");
                    println!("  Section [{}]: {} ({})", idx, title, sub);
                }
            }
        }
    }

    // 4. Test YtMusicProvider::fetch_browse_feed directly
    println!("\n--- Testing YtMusicProvider::fetch_browse_feed ---");
    let provider = sources::ytmusic::client::YtMusicProvider::new();
    println!("Provider authenticated: {}", provider.is_authenticated());
    let mut live_feed = None;
    match provider.fetch_browse_feed("FEmusic_home", None).await {
        Ok(feed) => {
            println!("SUCCESS! Parsed {} live sections from YouTube Music:", feed.sections.len());
            for (i, sec) in feed.sections.iter().take(5).enumerate() {
                println!("  [{}] '{}' (subtitle: {:?}) - {} items", i, sec.title, sec.subtitle, sec.items.len());
                for (j, item) in sec.items.iter().take(3).enumerate() {
                    println!("      - Item {}: '{}' ({}) - type: {:?}, endpoint_id: '{}'", j, item.title, item.subtitle, item.item_type, item.endpoint_id);
                }
            }
            live_feed = Some(feed);
        }
        Err(e) => {
            println!("Error fetching browse feed: {:?}", e);
        }
    }

    // 5. Test Live Audio Stream Resolution with actual track from feed
    println!("\n--- Testing Live Audio Stream Resolution with Real Track from Feed ---");
    if let Some(feed) = live_feed {
        // Find first item that is actually a track
        let maybe_track_item = feed.sections.iter()
            .flat_map(|s| s.items.iter())
            .find(|item| item.item_type == sources::ytmusic::types::ShelfItemType::Track);

        let (test_id, test_title) = if let Some(item) = maybe_track_item {
            (item.endpoint_id.clone(), item.title.clone())
        } else {
            // Fallback to a standard YouTube Music track ID (e.g. Tera Mera Rishta / Despacito)
            ("kJQP7kiw5Fk".to_string(), "Despacito (Test Track)".to_string())
        };

        println!("Testing with track: '{}' (ID: '{}')", test_title, test_id);
        let feed_track = rustic_core::api::Track {
            id: test_id.clone(),
            title: test_title,
            artists: vec!["Artist".into()],
            album: "YTM Stream".into(),
            duration_secs: 180,
            source: rustic_core::api::TrackSource::YouTubeMusic {
                video_id: test_id,
            },
            isrc: None,
            artwork_url: None,
            content_hash: None,
        };
        use sources::SourceProvider;
        match provider.resolve(&feed_track).await {
            Ok(resolved) => {
                println!("SUCCESS! Resolved live audio stream for real track:");
                println!("  Codec: {}", resolved.codec);
                println!("  Bitrate: {} kbps", resolved.bitrate_kbps);
                println!("  Expires At: {:?}", resolved.expires_at_unix_secs);
                println!("  Stream URL: {}...", &resolved.stream_url[..resolved.stream_url.len().min(100)]);
            }
            Err(e) => {
                println!("Error resolving live track: {:?}", e);
            }
        }
    }

    // 6. Test provider.fetch_queue_with_lyrics_id
    println!("\n--- Testing provider.fetch_queue_with_lyrics_id ---");
    match provider.fetch_queue_with_lyrics_id("kJQP7kiw5Fk").await {
        Ok((queue, lyrics_id)) => {
            println!("SUCCESS! Fetched queue with {} upcoming tracks. Lyrics ID: {:?}", queue.upcoming.len(), lyrics_id);
            if let Some(cur) = queue.current_track {
                println!("  Current: '{}' by {} (id: {})", cur.title, cur.artist, cur.video_id);
            }
            for (i, item) in queue.upcoming.iter().take(4).enumerate() {
                println!("  [{}] '{}' by {} ({}) - id: {}", i, item.title, item.artist, item.duration_str, item.video_id);
            }

            if let Some(lid) = lyrics_id {
                println!("\n--- Testing provider.fetch_lyrics ---");
                match provider.fetch_lyrics(&lid).await {
                    Ok(doc) => {
                        println!("SUCCESS! Fetched lyrics: {} lines, source: {:?}", doc.lines.len(), doc.source);
                        for l in doc.lines.iter().take(3) {
                            println!("    [{:?}] {}", l.start_time_ms, l.text);
                        }
                    }
                    Err(e) => println!("Error fetching lyrics: {:?}", e),
                }
            }
        }
        Err(e) => {
            println!("Error fetching queue: {:?}", e);
        }
    }

    // 7. Matrix Test of Innertube Player Clients with clean headers
    println!("\n=== Matrix Testing Innertube Player Clients (Clean Headers) ===");
    let probe_client = reqwest::Client::new();
    let test_video_id = "kwH2nYwuFdA"; // Real track: KATSEYE - Hootie Frutti from Quick Picks!

    let yt_origin = "https://www.youtube.com";
    let yt_input = format!("{} {} {}", ts, sapisid, yt_origin);
    let mut yt_hasher = Sha1::new();
    yt_hasher.update(yt_input.as_bytes());
    let yt_sapisidhash = format!("SAPISIDHASH {}_{}", ts, hex::encode(yt_hasher.finalize()));

    // Extract signatureTimestamp dynamically
    println!("\n--- Fetching signatureTimestamp from YouTube Web Player ---");
    let mut signature_timestamp: u64 = 20138;
    if let Ok(res) = probe_client.get("https://www.youtube.com/watch?v=kwH2nYwuFdA")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36")
        .send().await {
        if let Ok(text) = res.text().await {
            let re_js = regex::Regex::new(r#""(?:PLAYER_JS_URL|jsUrl)":"([^"]+)""#).unwrap();
            let base_path = re_js.captures(&text).and_then(|c| c.get(1)).map(|m| m.as_str().replace(r#"\"#, "")).unwrap_or_default();
            let base_url = if !base_path.is_empty() {
                format!("https://www.youtube.com{}", base_path)
            } else {
                String::new()
            };
            println!("  Found Player JS URL: '{}'", base_url);
            if !base_url.is_empty() {
                if let Ok(js_res) = probe_client.get(&base_url).send().await {
                    if let Ok(js_text) = js_res.text().await {
                        let re_sts = regex::Regex::new(r#"signatureTimestamp:(\d+)"#).unwrap();
                        if let Some(caps) = re_sts.captures(&js_text) {
                            if let Some(m) = caps.get(1) {
                                signature_timestamp = m.as_str().parse().unwrap_or(20138);
                                println!("  Extracted signatureTimestamp: {}", signature_timestamp);
                            }
                        }
                    }
                }
            }
        }
    }

    let test_configs = vec![
        ("WEB_REMIX_AUTH_STS", "https://music.youtube.com/youtubei/v1/player", serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "playbackContext": {
                "contentPlaybackContext": {
                    "html5Preference": "HTML5_PREF_WANTS",
                    "signatureTimestamp": signature_timestamp
                }
            },
            "videoId": test_video_id,
            "contentCheckOk": true,
            "racyCheckOk": true
        }), vec![
            ("Origin", "https://music.youtube.com"),
            ("Referer", "https://music.youtube.com/"),
            ("Authorization", sapisidhash.as_str()),
            ("Cookie", cookie_str.as_str()),
            ("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36")
        ]),

        ("TVHTML5_AUTH_STS", "https://www.youtube.com/youtubei/v1/player", serde_json::json!({
            "context": {
                "client": {
                    "clientName": "TVHTML5",
                    "clientVersion": "7.20240101.00.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "playbackContext": {
                "contentPlaybackContext": {
                    "html5Preference": "HTML5_PREF_WANTS",
                    "signatureTimestamp": signature_timestamp
                }
            },
            "videoId": test_video_id
        }), vec![
            ("Origin", "https://www.youtube.com"),
            ("Referer", "https://www.youtube.com/"),
            ("Authorization", yt_sapisidhash.as_str()),
            ("Cookie", cookie_str.as_str())
        ]),

        ("ANDROID_MUSIC_AUTH_STS", "https://music.youtube.com/youtubei/v1/player", serde_json::json!({
            "context": {
                "client": {
                    "clientName": "ANDROID_MUSIC",
                    "clientVersion": "6.41.52",
                    "androidSdkVersion": 33,
                    "hl": "en",
                    "gl": "US"
                }
            },
            "playbackContext": {
                "contentPlaybackContext": {
                    "html5Preference": "HTML5_PREF_WANTS",
                    "signatureTimestamp": signature_timestamp
                }
            },
            "videoId": test_video_id
        }), vec![
            ("User-Agent", "com.google.android.apps.youtube.music/6.41.52 (Linux; U; Android 13; en_US) gzip"),
            ("X-YouTube-Client-Name", "21"),
            ("X-YouTube-Client-Version", "6.41.52"),
            ("Authorization", sapisidhash.as_str()),
            ("Cookie", cookie_str.as_str())
        ]),

        ("IOS_AUTH", "https://www.youtube.com/youtubei/v1/player", serde_json::json!({
            "context": {
                "client": {
                    "clientName": "IOS",
                    "clientVersion": "19.29.1",
                    "deviceMake": "Apple",
                    "deviceModel": "iPhone16,2",
                    "osName": "iOS",
                    "osVersion": "17.5.1.21F90",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": test_video_id
        }), vec![
            ("User-Agent", "com.google.ios.youtube/19.29.1 (iPhone16,2; U; CPU iOS 17_5_1 like Mac OS X; en_US)"),
            ("X-YouTube-Client-Name", "5"),
            ("X-YouTube-Client-Version", "19.29.1"),
            ("Authorization", yt_sapisidhash.as_str()),
            ("Cookie", cookie_str.as_str())
        ])
    ];

    for (name, url, payload, headers) in test_configs {
        println!("\n--- Testing Client: {} ---", name);
        let mut req = probe_client.post(url).json(&payload);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        match req.send().await {
            Ok(res) => {
                let status = res.status();
                println!("  HTTP Status: {}", status);
                if let Ok(body) = res.json::<Value>().await {
                    let play_status = body["playabilityStatus"]["status"].as_str().unwrap_or("NONE");
                    let reason = body["playabilityStatus"]["reason"].as_str().unwrap_or("");
                    println!("  Playability: {} (reason: '{}')", play_status, reason);
                    if let Some(formats) = body["streamingData"]["adaptiveFormats"].as_array() {
                        let audio_formats: Vec<_> = formats.iter().filter(|f| {
                            f["mimeType"].as_str().map(|m| m.starts_with("audio/")).unwrap_or(false)
                        }).collect();
                        println!("  Found {} total formats, {} AUDIO formats!", formats.len(), audio_formats.len());
                        for f in audio_formats.iter().take(2) {
                            let itag = f["itag"].as_i64().unwrap_or(0);
                            let mime = f["mimeType"].as_str().unwrap_or("");
                            let bitrate = f["bitrate"].as_i64().unwrap_or(0) / 1000;
                            let has_direct_url = f.get("url").is_some();
                            let is_ciphered = f.get("signatureCipher").is_some();
                            let url_snippet = f["url"].as_str().unwrap_or("").chars().take(60).collect::<String>();
                            let cipher_str = f.get("signatureCipher").and_then(|v| v.as_str()).unwrap_or("");
                            println!("    -> itag: {}, mime: {}, bitrate: {}kbps, direct_url: {}, cipher: {}, url: {}...",
                                itag, mime, bitrate, has_direct_url, is_ciphered, url_snippet);
                            if !cipher_str.is_empty() {
                                println!("       cipher raw: {}", &cipher_str[..cipher_str.len().min(120)]);
                            }
                        }
                    } else {
                        println!("  No streamingData.adaptiveFormats. Full response:\n{}", serde_json::to_string_pretty(&body).unwrap_or_default());
                    }
                } else {
                    println!("  Failed to parse JSON body");
                }
            }
            Err(e) => println!("  Request failed: {:?}", e),
        }
    }

    Ok(())
}
