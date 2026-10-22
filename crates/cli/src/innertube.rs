use reqwest::Client;
use serde_json::Value;
use std::error::Error;

pub async fn resolve_stream_url(client: &Client, video_id: &str) -> Result<String, Box<dyn Error>> {
    let url = "https://www.youtube.com/youtubei/v1/player";
    
    let payload = serde_json::json!({
        "context": {
            "client": {
                "clientName": "TVHTML5",
                "clientVersion": "7.20230412.00.00",
                "hl": "en",
                "gl": "US",
                "deviceMake": "Generic",
                "deviceModel": "SmartTV",
                "osName": "Windows",
                "osVersion": "10.0"
            }
        },
        "videoId": video_id
    });

    let res = client
        .post(url)
        .json(&payload)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("InnerTube player API returned {}", res.status()).into());
    }

    let body: Value = res.json().await?;
    
    // Check for adaptive formats
    let formats = body["streamingData"]["adaptiveFormats"]
        .as_array()
        .ok_or("No adaptive formats found")?;

    // Try to find the highest quality audio/mp4 stream
    for format in formats {
        if let Some(mime) = format["mimeType"].as_str() {
            if mime.starts_with("audio/") {
                if let Some(url) = format["url"].as_str() {
                    return Ok(url.to_string());
                }
            }
        }
    }

    Err("No audio stream URL found in adaptive formats".into())
}
