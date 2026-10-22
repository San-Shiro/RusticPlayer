// crates/sources/src/ytmusic/stream.rs
//! Resolves playable audio CDN URLs for a YouTube Music video id via the
//! Innertube `/youtubei/v1/player` endpoint.
//!
//! Uses the `ANDROID_MUSIC` client, which returns **direct, unsigned** audio
//! URLs in `streamingData.adaptiveFormats` (no JavaScript `signatureCipher`
//! deciphering required). Supports authenticated (cookie + `SAPISIDHASH`) and
//! guest sessions via [`YtmSession`].

use reqwest::Client;
use rustic_core::api::ResolvedAudio;
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::SourceError;
use super::auth::YtmSession;

// ---- VISIONOS client constants ---------------------------------------------

const VISIONOS_CLIENT_NAME: &str = "VISIONOS";
const VISIONOS_CLIENT_VERSION: &str = "1.02";
const VISIONOS_CLIENT_ID: &str = "101";
const VISIONOS_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 15_7_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.0 Safari/605.1.15";
const YOUTUBE_ORIGIN: &str = "https://www.youtube.com";

static SIGNATURE_TIMESTAMP: AtomicU64 = AtomicU64::new(20697);
static CACHED_VISITOR_DATA: RwLock<Option<String>> = RwLock::new(None);

// ---- wire types (subset of the player response) -----------------------------

#[derive(Debug, Deserialize)]
struct PlayerResponse {
    #[serde(rename = "playabilityStatus")]
    playability_status: Option<PlayabilityStatus>,
    #[serde(rename = "streamingData")]
    streaming_data: Option<StreamingData>,
}

#[derive(Debug, Deserialize)]
struct PlayabilityStatus {
    status: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct StreamingData {
    #[serde(rename = "adaptiveFormats", default)]
    adaptive_formats: Vec<Format>,
    #[serde(default)]
    formats: Vec<Format>,
    #[serde(rename = "hlsManifestUrl")]
    hls_manifest_url: Option<String>,
    /// Lifetime of the returned URLs, in seconds (JSON string). Used only as a
    /// fallback when the URL itself carries no `expire` parameter.
    #[serde(rename = "expiresInSeconds")]
    expires_in_seconds: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Format {
    #[allow(dead_code)]
    itag: Option<u32>,
    url: Option<String>,
    #[serde(rename = "mimeType")]
    mime_type: Option<String>,
    /// Peak bitrate in bits/sec.
    bitrate: Option<u64>,
    #[serde(rename = "averageBitrate")]
    average_bitrate: Option<u64>,
    /// Present only when the URL is signature-protected.
    #[serde(rename = "signatureCipher")]
    signature_cipher: Option<String>,
    #[serde(default)]
    cipher: Option<String>,
}

#[derive(Debug, Clone, Copy)]
enum InnertubeClient {
    AndroidMusic,
    VisionOs,
    WebRemix,
    TvHtml5,
}

impl InnertubeClient {
    fn payload(&self, video_id: &str, sts: u64) -> serde_json::Value {
        match self {
            Self::AndroidMusic => serde_json::json!({
                "videoId": video_id,
                "context": {
                    "client": {
                        "clientName": "ANDROID_MUSIC",
                        "clientVersion": "6.41.52",
                        "androidSdkVersion": 30,
                        "hl": "en",
                        "gl": "US"
                    }
                },
                "contentCheckOk": true,
                "racyCheckOk": true
            }),
            Self::VisionOs => serde_json::json!({
                "videoId": video_id,
                "context": {
                    "client": {
                        "clientName": VISIONOS_CLIENT_NAME,
                        "clientVersion": VISIONOS_CLIENT_VERSION,
                        "deviceMake": "Apple",
                        "deviceModel": "RealityDevice17,1",
                        "userAgent": VISIONOS_USER_AGENT,
                        "osName": "visionOS",
                        "osVersion": "26.5.23O471",
                        "hl": "en",
                        "timeZone": "UTC",
                        "utcOffsetMinutes": 0
                    }
                },
                "playbackContext": {
                    "contentPlaybackContext": {
                        "html5Preference": "HTML5_PREF_WANTS",
                        "signatureTimestamp": sts
                    }
                },
                "contentCheckOk": true,
                "racyCheckOk": true
            }),
            Self::WebRemix => serde_json::json!({
                "videoId": video_id,
                "context": {
                    "client": {
                        "clientName": "WEB_REMIX",
                        "clientVersion": "1.20240318.01.00",
                        "hl": "en",
                        "gl": "US"
                    }
                },
                "playbackContext": {
                    "contentPlaybackContext": {
                        "html5Preference": "HTML5_PREF_WANTS",
                        "signatureTimestamp": sts
                    }
                },
                "contentCheckOk": true,
                "racyCheckOk": true
            }),
            Self::TvHtml5 => serde_json::json!({
                "videoId": video_id,
                "context": {
                    "client": {
                        "clientName": "TVHTML5",
                        "clientVersion": "7.20240318.18.00",
                        "hl": "en",
                        "gl": "US"
                    }
                },
                "contentCheckOk": true,
                "racyCheckOk": true
            }),
        }
    }

    fn build_request(
        &self,
        client: &Client,
        session: &YtmSession,
        visitor_id: Option<&str>,
        payload: &serde_json::Value,
    ) -> reqwest::RequestBuilder {
        let url = "https://www.youtube.com/youtubei/v1/player?prettyPrint=false";
        let mut builder = client.post(url);

        match self {
            Self::AndroidMusic => {
                builder = builder
                    .header("User-Agent", "com.google.android.apps.youtube.music/6.41.52 (Linux; U; Android 11)")
                    .header("X-YouTube-Client-Name", "21")
                    .header("X-YouTube-Client-Version", "6.41.52");
            }
            Self::VisionOs => {
                builder = builder
                    .header("User-Agent", VISIONOS_USER_AGENT)
                    .header("Origin", YOUTUBE_ORIGIN)
                    .header("X-YouTube-Client-Name", VISIONOS_CLIENT_ID)
                    .header("X-YouTube-Client-Version", VISIONOS_CLIENT_VERSION);
            }
            Self::WebRemix => {
                builder = session.apply_headers(builder);
                builder = builder
                    .header("X-YouTube-Client-Name", "67")
                    .header("X-YouTube-Client-Version", "1.20240318.01.00")
                    .header("Origin", "https://music.youtube.com")
                    .header("Referer", "https://music.youtube.com/");
            }
            Self::TvHtml5 => {
                builder = builder
                    .header("User-Agent", "Mozilla/5.0 (ChromiumStylePlatform) Cobalt/Version")
                    .header("X-YouTube-Client-Name", "31")
                    .header("X-YouTube-Client-Version", "7.20240318.18.00");
            }
        }

        builder = builder.json(payload);
        if let Some(vid) = visitor_id.or_else(|| session.visitor_data()) {
            builder = builder.header("X-Goog-Visitor-Id", vid);
        }

        builder
    }
}

// ---- public API -------------------------------------------------------------

/// Resolves the best available audio stream for `video_id` using a resilient
/// multi-client fallback ladder (`ANDROID_MUSIC` -> `VISIONOS` -> `WEB_REMIX` -> `TVHTML5`).
/// Deciphers `signatureCipher` JavaScript transforms when encountering encrypted formats.
///
/// # Errors
/// - [`SourceError::Network`] on transport failure.
/// - [`SourceError::Unavailable`] if playability is not OK across all clients or no playable stream is found.
pub async fn resolve_video_stream(
    client: &Client,
    session: &YtmSession,
    video_id: &str,
) -> Result<ResolvedAudio, SourceError> {
    if video_id.trim().is_empty() {
        return Err(SourceError::Unavailable("empty video id".into()));
    }

    // 1. Resolve visitor data
    let visitor = match session.visitor_data() {
        Some(v) => Some(v.to_string()),
        None => {
            let cached = CACHED_VISITOR_DATA.read().ok().and_then(|g| g.clone());
            match cached {
                Some(v) => Some(v),
                None => {
                    let fetched = fetch_visitor_data(client, video_id).await;
                    if let Some(ref v) = fetched {
                        if let Ok(mut g) = CACHED_VISITOR_DATA.write() {
                            *g = Some(v.clone());
                        }
                    }
                    fetched
                }
            }
        }
    };

    let ladder = [
        InnertubeClient::AndroidMusic,
        InnertubeClient::VisionOs,
        InnertubeClient::WebRemix,
        InnertubeClient::TvHtml5,
    ];

    let mut last_err = SourceError::Unavailable("All player clients failed to resolve stream".into());

    for innertube_client in ladder {
        let sts = SIGNATURE_TIMESTAMP.load(Ordering::Relaxed);
        let payload = innertube_client.payload(video_id, sts);
        let request = innertube_client.build_request(client, session, visitor.as_deref(), &payload);

        let response = match request.send().await {
            Ok(res) => res,
            Err(e) => {
                tracing::debug!(target: "sources::ytmusic", "{:?} player request error: {e}", innertube_client);
                last_err = SourceError::Network(e.to_string());
                continue;
            }
        };

        if !response.status().is_success() {
            tracing::debug!(target: "sources::ytmusic", "{:?} returned HTTP {}", innertube_client, response.status());
            continue;
        }

        let player: PlayerResponse = match response.json().await {
            Ok(p) => p,
            Err(e) => {
                tracing::debug!(target: "sources::ytmusic", "{:?} json decode error: {e}", innertube_client);
                continue;
            }
        };

        match resolve_audio_from_player_async(client, player, video_id).await {
            Ok(audio) => {
                tracing::info!(target: "sources::ytmusic", "Successfully resolved audio for {video_id} using {:?}", innertube_client);
                return Ok(audio);
            }
            Err(e) => {
                tracing::debug!(target: "sources::ytmusic", "{:?} audio resolution failed: {e}", innertube_client);
                last_err = e;
            }
        }
    }

    // Refresh STS and retry VisionOS once if all clients failed
    if let Some(new_sts) = refresh_signature_timestamp(client, video_id).await {
        SIGNATURE_TIMESTAMP.store(new_sts, Ordering::Relaxed);
        let payload = InnertubeClient::VisionOs.payload(video_id, new_sts);
        let request = InnertubeClient::VisionOs.build_request(client, session, visitor.as_deref(), &payload);
        if let Ok(res) = request.send().await {
            if res.status().is_success() {
                if let Ok(player) = res.json::<PlayerResponse>().await {
                    if let Ok(audio) = resolve_audio_from_player_async(client, player, video_id).await {
                        return Ok(audio);
                    }
                }
            }
        }
    }

    Err(last_err)
}

async fn resolve_audio_from_player_async(
    client: &Client,
    player: PlayerResponse,
    video_id: &str,
) -> Result<ResolvedAudio, SourceError> {
    let playability = player.playability_status.as_ref();
    let pstatus = playability.and_then(|p| p.status.as_deref()).unwrap_or("UNKNOWN");
    if pstatus != "OK" {
        let reason = playability.and_then(|p| p.reason.clone()).unwrap_or_else(|| pstatus.to_string());
        return Err(SourceError::Unavailable(format!(
            "stream unavailable for {video_id} ({pstatus}): {reason}"
        )));
    }

    let streaming = player.streaming_data.ok_or_else(|| {
        SourceError::Unavailable(format!("no streamingData in player response for {video_id}"))
    })?;

    let mut resolved_formats: Vec<(String, String, u64)> = Vec::new();

    for f in streaming.adaptive_formats.into_iter().chain(streaming.formats.into_iter()) {
        let mime = match f.mime_type {
            Some(m) if m.starts_with("audio/") => m,
            _ => continue,
        };
        let bitrate = f.bitrate.or(f.average_bitrate).unwrap_or(0);

        if let Some(u) = f.url {
            if !u.is_empty() {
                resolved_formats.push((u, mime, bitrate));
                continue;
            }
        }

        if let Some(cipher_str) = f.signature_cipher.or(f.cipher) {
            match super::cipher::decipher_url(client, video_id, &cipher_str).await {
                Ok(deciphered_url) => {
                    resolved_formats.push((deciphered_url, mime, bitrate));
                }
                Err(e) => {
                    tracing::warn!(target: "sources::ytmusic", "Failed to decipher signature cipher: {e}");
                }
            }
        }
    }

    if let Some((url, mime, bitrate)) = select_best_format(resolved_formats) {
        let codec = simplify_codec(&mime);
        let bitrate_kbps = (bitrate / 1000) as u32;
        let expires_at_unix_secs = parse_expire(&url).or_else(|| {
            streaming
                .expires_in_seconds
                .as_deref()
                .and_then(|s| s.trim().parse::<u64>().ok())
                .map(|secs| now_unix().saturating_add(secs))
        });
        return Ok(ResolvedAudio {
            stream_url: url,
            codec,
            bitrate_kbps,
            expires_at_unix_secs,
        });
    }

    if let Some(hls_url) = streaming.hls_manifest_url {
        return Ok(ResolvedAudio {
            stream_url: hls_url,
            codec: "hls".into(),
            bitrate_kbps: 128,
            expires_at_unix_secs: None,
        });
    }

    Err(SourceError::Unavailable(format!(
        "no direct or decipherable audio stream found for {video_id}"
    )))
}

fn select_best_format(formats: Vec<(String, String, u64)>) -> Option<(String, String, u64)> {
    let mut best: Option<(String, String, u64)> = None;
    for f in formats {
        let is_mp4 = f.1.contains("mp4");
        let bitrate = f.2;
        match &best {
            None => best = Some(f),
            Some(curr) => {
                let curr_is_mp4 = curr.1.contains("mp4");
                if is_mp4 && !curr_is_mp4 {
                    best = Some(f);
                } else if is_mp4 == curr_is_mp4 && bitrate > curr.2 {
                    best = Some(f);
                }
            }
        }
    }
    best
}

async fn fetch_visitor_data(client: &Client, video_id: &str) -> Option<String> {
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let res = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .ok()?;
    let text = res.text().await.ok()?;
    let re = regex::Regex::new(r#""visitorData":"([^"]+)""#).ok()?;
    re.captures(&text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

async fn refresh_signature_timestamp(client: &Client, video_id: &str) -> Option<u64> {
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let res = client
        .get(&url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .ok()?;
    let text = res.text().await.ok()?;
    let re_js = regex::Regex::new(r#""(?:PLAYER_JS_URL|jsUrl)":"([^"]+)""#).ok()?;
    let base_path = re_js
        .captures(&text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().replace(r#"\"#, ""))?;
    let base_url = format!("https://www.youtube.com{}", base_path);
    let js_res = client.get(&base_url).send().await.ok()?;
    let js_text = js_res.text().await.ok()?;
    let re_sts = regex::Regex::new(r#"signatureTimestamp:(\d+)"#).ok()?;
    re_sts
        .captures(&js_text)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u64>().ok())
}

// ---- response interpretation (pure, network-free, testable) -----------------

#[allow(dead_code)]
fn resolved_from_player(
    player: PlayerResponse,
    video_id: &str,
) -> Result<ResolvedAudio, SourceError> {
    // 1. Playability gate.
    let playability = player.playability_status.as_ref();
    let pstatus = playability
        .and_then(|p| p.status.as_deref())
        .unwrap_or("UNKNOWN");
    if pstatus != "OK" {
        let reason = playability
            .and_then(|p| p.reason.clone())
            .unwrap_or_else(|| pstatus.to_string());
        return Err(SourceError::Unavailable(format!(
            "stream unavailable for {video_id} ({pstatus}): {reason}"
        )));
    }

    let streaming = player.streaming_data.ok_or_else(|| {
        SourceError::Unavailable(format!("no streamingData in player response for {video_id}"))
    })?;

    // 2. Select the highest-bitrate direct audio format.
    let mut best: Option<&Format> = None;
    let mut best_bitrate: u64 = 0;
    let mut saw_ciphered = false;

    for f in streaming
        .adaptive_formats
        .iter()
        .chain(streaming.formats.iter())
    {
        let mime = match f.mime_type.as_deref() {
            Some(m) => m,
            None => continue,
        };
        if !mime.starts_with("audio/") {
            continue;
        }
        let has_url = f.url.as_deref().map(|u| !u.is_empty()).unwrap_or(false);
        if !has_url {
            if f.signature_cipher.is_some() {
                saw_ciphered = true;
            }
            continue;
        }
        let bitrate = f.bitrate.or(f.average_bitrate).unwrap_or(0);
        let is_mp4 = mime.contains("mp4");

        match best {
            None => {
                best = Some(f);
                best_bitrate = bitrate;
            }
            Some(curr) => {
                let curr_is_mp4 = curr.mime_type.as_deref().map(|m| m.contains("mp4")).unwrap_or(false);
                if is_mp4 && !curr_is_mp4 {
                    // Prefer MP4/AAC over WebM for native Symphonia codec compatibility
                    best = Some(f);
                    best_bitrate = bitrate;
                } else if is_mp4 == curr_is_mp4 && bitrate > best_bitrate {
                    best = Some(f);
                    best_bitrate = bitrate;
                }
            }
        }
    }

    let best = best.ok_or_else(|| {
        if saw_ciphered {
            SourceError::Unavailable(format!(
                "audio formats for {video_id} require signature deciphering \
                 (unexpected for the ANDROID_MUSIC client)"
            ))
        } else {
            SourceError::Unavailable(format!(
                "no direct audio stream found in adaptiveFormats for {video_id}"
            ))
        }
    })?;

    // 3. Build ResolvedAudio.
    let stream_url = best.url.clone().unwrap_or_default();
    let mime = best.mime_type.as_deref().unwrap_or("");
    let codec = simplify_codec(mime);
    let bitrate_kbps = (best_bitrate / 1000) as u32;

    let expires_at_unix_secs = parse_expire(&stream_url).or_else(|| {
        streaming
            .expires_in_seconds
            .as_deref()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|secs| now_unix().saturating_add(secs))
    });

    Ok(ResolvedAudio {
        stream_url,
        codec,
        bitrate_kbps,
        expires_at_unix_secs,
    })
}

// ---- helpers ----------------------------------------------------------------

/// Derives a friendly codec label from a MIME type such as
/// `audio/mp4; codecs="mp4a.40.2"` or `audio/webm; codecs="opus"`.
fn simplify_codec(mime_type: &str) -> String {
    if let Some(idx) = mime_type.find("codecs=") {
        let rest = mime_type[idx + "codecs=".len()..].trim_start_matches('"');
        let raw = rest.split('"').next().unwrap_or(rest).trim();
        let lower = raw.to_ascii_lowercase();
        if lower.contains("opus") {
            return "opus".into();
        }
        if lower.starts_with("mp4a") {
            return "aac".into();
        }
        if lower.contains("vorbis") {
            return "vorbis".into();
        }
        if lower.contains("ec-3") || lower.contains("ac-3") {
            return "ac3".into();
        }
        if lower.contains("flac") {
            return "flac".into();
        }
        if !raw.is_empty() {
            return raw.to_string();
        }
    }
    // Fall back to the container type.
    if mime_type.contains("webm") {
        "opus".into()
    } else if mime_type.contains("mp4") {
        "aac".into()
    } else {
        "unknown".into()
    }
}

/// Extracts the `expire` unix timestamp from a googlevideo CDN URL.
/// Handles both the query form (`&expire=1710000000`) and the path form
/// (`/expire/1710000000/`).
fn parse_expire(url: &str) -> Option<u64> {
    if let Ok(parsed) = reqwest::Url::parse(url) {
        if let Some((_, v)) = parsed.query_pairs().find(|(k, _)| k.as_ref() == "expire") {
            if let Ok(n) = v.trim().parse::<u64>() {
                return Some(n);
            }
        }
    }
    manual_expire_scan(url)
}

fn manual_expire_scan(url: &str) -> Option<u64> {
    for marker in ["expire=", "/expire/"] {
        if let Some(pos) = url.find(marker) {
            let tail = &url[pos + marker.len()..];
            let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u64>() {
                return Some(n);
            }
        }
    }
    None
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ---- tests ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json};

    #[test]
    fn simplify_codec_mp4_is_aac() {
        assert_eq!(simplify_codec("audio/mp4; codecs=\"mp4a.40.2\""), "aac");
    }

    #[test]
    fn simplify_codec_webm_is_opus() {
        assert_eq!(simplify_codec("audio/webm; codecs=\"opus\""), "opus");
    }

    #[test]
    fn simplify_codec_falls_back_to_container() {
        assert_eq!(simplify_codec("audio/webm"), "opus");
        assert_eq!(simplify_codec("audio/mp4"), "aac");
    }

    #[test]
    fn parse_expire_from_query() {
        let url = "https://rr5.googlevideo.com/videoplayback?ei=abc&expire=1710000000&ip=1.2.3.4";
        assert_eq!(parse_expire(url), Some(1_710_000_000));
    }

    #[test]
    fn parse_expire_from_path() {
        let url = "https://rr5.googlevideo.com/videoplayback/expire/1710000000/ei/abc";
        assert_eq!(parse_expire(url), Some(1_710_000_000));
    }

    #[test]
    fn parse_expire_absent() {
        assert_eq!(parse_expire("https://example.com/videoplayback?foo=bar"), None);
    }

    #[test]
    fn picks_highest_bitrate_audio_and_parses_expire() {
        let player: PlayerResponse = from_value(json!({
            "playabilityStatus": { "status": "OK" },
            "streamingData": {
                "expiresInSeconds": "21540",
                "adaptiveFormats": [
                    {
                        "itag": 140,
                        "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"",
                        "bitrate": 130000,
                        "averageBitrate": 128000,
                        "url": "https://rr5.googlevideo.com/videoplayback?expire=1710000000&itag=140"
                    },
                    {
                        "itag": 251,
                        "mimeType": "audio/webm; codecs=\"opus\"",
                        "bitrate": 160000,
                        "averageBitrate": 155000,
                        "url": "https://rr5.googlevideo.com/videoplayback?expire=1710000123&itag=251"
                    },
                    {
                        "itag": 137,
                        "mimeType": "video/mp4; codecs=\"avc1.640028\"",
                        "bitrate": 2500000,
                        "url": "https://rr5.googlevideo.com/videoplayback?expire=1710000000&itag=137"
                    }
                ]
            }
        }))
        .unwrap();

        let resolved = resolved_from_player(player, "abc123").expect("should resolve");
        // Prefer MP4/AAC over WebM/Opus for Symphonia pure-Rust decode compatibility
        assert_eq!(resolved.codec, "aac");
        assert_eq!(resolved.bitrate_kbps, 130);
        assert_eq!(resolved.expires_at_unix_secs, Some(1_710_000_000));
        assert!(resolved.stream_url.contains("itag=140"));
    }

    #[test]
    fn picks_webm_when_no_mp4_available() {
        let player: PlayerResponse = from_value(json!({
            "playabilityStatus": { "status": "OK" },
            "streamingData": {
                "expiresInSeconds": "21540",
                "adaptiveFormats": [
                    {
                        "itag": 251,
                        "mimeType": "audio/webm; codecs=\"opus\"",
                        "bitrate": 160000,
                        "url": "https://rr5.googlevideo.com/videoplayback?expire=1710000123&itag=251"
                    }
                ]
            }
        }))
        .unwrap();

        let resolved = resolved_from_player(player, "abc123").expect("should resolve");
        assert_eq!(resolved.codec, "opus");
        assert_eq!(resolved.bitrate_kbps, 160);
        assert_eq!(resolved.expires_at_unix_secs, Some(1_710_000_123));
        assert!(resolved.stream_url.contains("itag=251"));
    }

    #[test]
    fn falls_back_to_expires_in_seconds_when_no_url_param() {
        let player: PlayerResponse = from_value(json!({
            "playabilityStatus": { "status": "OK" },
            "streamingData": {
                "expiresInSeconds": "3600",
                "adaptiveFormats": [{
                    "itag": 140,
                    "mimeType": "audio/mp4; codecs=\"mp4a.40.2\"",
                    "bitrate": 128000,
                    "url": "https://rr5.googlevideo.com/videoplayback?itag=140"
                }]
            }
        }))
        .unwrap();

        let resolved = resolved_from_player(player, "abc123").expect("should resolve");
        let exp = resolved.expires_at_unix_secs.expect("fallback expiry");
        // Should be roughly now + 3600s.
        assert!(exp >= now_unix() + 3500 && exp <= now_unix() + 3700);
    }

    #[test]
    fn non_ok_playability_is_unavailable() {
        let player: PlayerResponse = from_value(json!({
            "playabilityStatus": { "status": "LOGIN_REQUIRED", "reason": "Sign in to confirm" },
            "streamingData": { "adaptiveFormats": [] }
        }))
        .unwrap();

        match resolved_from_player(player, "abc123") {
            Err(SourceError::Unavailable(msg)) => {
                assert!(msg.contains("LOGIN_REQUIRED"));
                assert!(msg.contains("Sign in to confirm"));
            }
            other => panic!("expected Unavailable, got {other:?}"),
        }
    }

    #[test]
    fn no_audio_formats_is_unavailable() {
        let player: PlayerResponse = from_value(json!({
            "playabilityStatus": { "status": "OK" },
            "streamingData": {
                "adaptiveFormats": [{
                    "itag": 137,
                    "mimeType": "video/mp4; codecs=\"avc1.640028\"",
                    "bitrate": 2500000,
                    "url": "https://rr5.googlevideo.com/videoplayback?itag=137"
                }]
            }
        }))
        .unwrap();

        assert!(matches!(
            resolved_from_player(player, "abc123"),
            Err(SourceError::Unavailable(_))
        ));
    }
}
