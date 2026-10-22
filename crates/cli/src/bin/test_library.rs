use sources::ytmusic::auth::generate_sapisid_hash;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let content = std::fs::read_to_string("cookies.json")?;
    let cookies_arr: Vec<Value> = serde_json::from_str(&content)?;

    let mut cookie_header = String::new();
    let mut sapisid = String::new();
    for c in &cookies_arr {
        let name = c["name"].as_str().unwrap_or("");
        let val = c["value"].as_str().unwrap_or("");
        cookie_header.push_str(&format!("{}={}; ", name, val));
        if name == "SAPISID" {
            sapisid = val.to_string();
        }
    }
    let cookie_header = cookie_header.trim_end_matches("; ");

    // Test 1: www.youtube.com with Origin https://www.youtube.com
    let hash_yt = generate_sapisid_hash(&sapisid, "https://www.youtube.com");
    let res1 = client.post("https://www.youtube.com/youtubei/v1/account/account_menu")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36")
        .header("Origin", "https://www.youtube.com")
        .header("Referer", "https://www.youtube.com/")
        .header("X-Origin", "https://www.youtube.com")
        .header("Authorization", format!("SAPISIDHASH {}", hash_yt))
        .header("Cookie", cookie_header)
        .json(&serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB",
                    "clientVersion": "2.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            }
        }))
        .send().await?;

    println!("WWW Account Menu Status: {}", res1.status());
    let body1: Value = res1.json().await?;
    println!("WWW Account Menu Response: {:.300}", body1.to_string());

    // Test 2: Check if any actions or login error
    if let Some(actions) = body1.get("actions") {
        println!("WWW SUCCESS! Found actions: {:?}", actions);
    }

    Ok(())
}
