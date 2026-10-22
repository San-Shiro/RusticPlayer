use sha1::{Digest, Sha1};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use serde_json::Value;

pub const YTM_ORIGIN: &str = "https://music.youtube.com";
pub const YTM_REFERER: &str = "https://music.youtube.com/";
pub const YTM_CLIENT_NAME: &str = "67"; // WEB_REMIX client id
pub const YTM_CLIENT_VERSION: &str = "1.20260213.01.00";
pub const YTM_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36";

#[derive(Debug, Clone)]
pub enum YtmSession {
    Authenticated {
        cookie_header: String,
        sapisid: String,
        visitor_data: Option<String>,
    },
    Guest {
        visitor_data: Option<String>,
    },
}

impl YtmSession {
    /// Attempts to load cookies from a JSON file (e.g. `cookies.json`), checking common candidate locations.
    /// If cookies with a valid SAPISID are found, returns `YtmSession::Authenticated`.
    /// Otherwise, falls back gracefully to `YtmSession::Guest`.
    pub fn from_file_or_guest(path: impl AsRef<Path>) -> Self {
        let requested_path = path.as_ref();
        let candidate_paths = [
            requested_path.to_path_buf(),
            std::path::PathBuf::from("cookies.json"),
            std::path::PathBuf::from("../cookies.json"),
            std::path::PathBuf::from("../../cookies.json"),
        ];

        for p in &candidate_paths {
            if let Ok(content) = std::fs::read_to_string(p) {
                if let Some(session) = Self::from_cookie_json(&content) {
                    tracing::info!(target: "sources::ytmusic", "Loaded authenticated session from {:?}", p);
                    return session;
                }
            }
        }
        tracing::info!(target: "sources::ytmusic", "No valid cookies found; operating in Guest session mode");
        YtmSession::Guest { visitor_data: None }
    }

    /// Constructs an authenticated session directly from cookie JSON content.
    pub fn from_cookie_json(content: &str) -> Option<Self> {
        let (cookie_header, sapisid) = parse_cookies_json(content)?;
        Some(YtmSession::Authenticated {
            cookie_header,
            sapisid,
            visitor_data: None,
        })
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self, YtmSession::Authenticated { .. })
    }

    pub fn visitor_data(&self) -> Option<&str> {
        match self {
            YtmSession::Authenticated { visitor_data, .. } => visitor_data.as_deref(),
            YtmSession::Guest { visitor_data } => visitor_data.as_deref(),
        }
    }

    pub fn set_visitor_data(&mut self, data: String) {
        match self {
            YtmSession::Authenticated { visitor_data, .. } => *visitor_data = Some(data),
            YtmSession::Guest { visitor_data } => *visitor_data = Some(data),
        }
    }

    /// Computes the active SAPISIDHASH Authorization header if authenticated.
    pub fn auth_header(&self) -> Option<String> {
        match self {
            YtmSession::Authenticated { sapisid, .. } => {
                Some(format!("SAPISIDHASH {}", generate_sapisid_hash(sapisid, YTM_ORIGIN)))
            }
            YtmSession::Guest { .. } => None,
        }
    }

    /// Injects YouTube Music standard and auth headers into a reqwest request builder.
    pub fn apply_headers(&self, mut builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        builder = builder
            .header("User-Agent", YTM_USER_AGENT)
            .header("Origin", YTM_ORIGIN)
            .header("X-Origin", YTM_ORIGIN)
            .header("Referer", YTM_REFERER)
            .header("X-Goog-Api-Format-Version", "1")
            .header("X-YouTube-Client-Name", YTM_CLIENT_NAME)
            .header("X-YouTube-Client-Version", YTM_CLIENT_VERSION);

        if let Some(auth) = self.auth_header() {
            builder = builder.header("Authorization", auth);
        }

        if let YtmSession::Authenticated { cookie_header, .. } = self {
            builder = builder.header("Cookie", cookie_header.clone());
        }

        if let Some(v_data) = self.visitor_data() {
            builder = builder.header("X-Goog-Visitor-Id", v_data);
        }

        builder
    }
}

/// Generates standard YouTube `SAPISIDHASH` in format `{timestamp}_{sha1_hex}`.
pub fn generate_sapisid_hash(sapisid: &str, origin: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let input = format!("{} {} {}", now, sapisid, origin);
    let mut hasher = Sha1::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let hex_digest = format!("{:x}", result);
    
    format!("{}_{}", now, hex_digest)
}

/// Parses raw cookie JSON (both EditThisCookie array format and `{name: value}` map format)
/// or raw HTTP Cookie header string format (`name=value; name2=value2`).
/// Returns `Some((cookie_header_string, sapisid_value))`.
pub fn parse_cookies_json(content: &str) -> Option<(String, String)> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Try parsing as JSON first
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        let mut cookie_str = String::new();
        let mut sapisid = String::new();

        if let Some(arr) = value.as_array() {
            for c in arr {
                if let (Some(name), Some(val)) = (c.get("name").and_then(|v| v.as_str()), c.get("value").and_then(|v| v.as_str())) {
                    cookie_str.push_str(&format!("{}={}; ", name, val));
                    if (name == "SAPISID" || name == "__Secure-3PAPISID") && sapisid.is_empty() {
                        sapisid = val.to_string();
                    }
                }
            }
        } else if let Some(map) = value.as_object() {
            for (k, v) in map {
                if let Some(val_str) = v.as_str() {
                    cookie_str.push_str(&format!("{}={}; ", k, val_str));
                    if (k == "SAPISID" || k == "__Secure-3PAPISID") && sapisid.is_empty() {
                        sapisid = val_str.to_string();
                    }
                }
            }
        }

        if !sapisid.is_empty() {
            return Some((cookie_str.trim_end_matches("; ").to_string(), sapisid));
        }
    }

    // Fallback: Try parsing as raw HTTP cookie header string (e.g. "SAPISID=xyz; HSID=abc")
    let mut cookie_str = String::new();
    let mut sapisid = String::new();

    for part in trimmed.split(';') {
        let part = part.trim();
        if let Some((name, val)) = part.split_once('=') {
            let name = name.trim();
            let val = val.trim();
            if !name.is_empty() && !val.is_empty() {
                cookie_str.push_str(&format!("{}={}; ", name, val));
                if (name == "SAPISID" || name == "__Secure-3PAPISID") && sapisid.is_empty() {
                    sapisid = val.to_string();
                }
            }
        }
    }

    if sapisid.is_empty() {
        None
    } else {
        Some((cookie_str.trim_end_matches("; ").to_string(), sapisid))
    }
}

/// Atomically saves cookie JSON to a target file using temporary file and rename.
pub fn save_cookies_file(content: &str, target_path: impl AsRef<Path>) -> std::io::Result<()> {
    let target = target_path.as_ref();
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let tmp_path = target.with_extension("tmp");
    std::fs::write(&tmp_path, content)?;
    std::fs::rename(&tmp_path, target)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sapisid_hash_format() {
        let hash = generate_sapisid_hash("test_sapisid_123", YTM_ORIGIN);
        assert!(hash.contains('_'));
        let parts: Vec<&str> = hash.split('_').collect();
        assert_eq!(parts.len(), 2);
        assert!(parts[0].parse::<u64>().is_ok());
        assert_eq!(parts[1].len(), 40); // 40-char SHA1 hex
    }

    #[test]
    fn test_parse_cookies_json_array() {
        let sample = r#"[
            {"name": "SAPISID", "value": "secret_sapisid_value"},
            {"name": "LOGIN_INFO", "value": "xyz123"}
        ]"#;
        let (header, sapisid) = parse_cookies_json(sample).expect("Should parse array cookies");
        assert_eq!(sapisid, "secret_sapisid_value");
        assert!(header.contains("SAPISID=secret_sapisid_value"));
        assert!(header.contains("LOGIN_INFO=xyz123"));
    }

    #[test]
    fn test_parse_cookies_raw_header() {
        let sample = "VISITOR_INFO1_LIVE=abc; SAPISID=my_sapisid_token; PREF=tz=UTC";
        let (header, sapisid) = parse_cookies_json(sample).expect("Should parse raw cookie header");
        assert_eq!(sapisid, "my_sapisid_token");
        assert!(header.contains("SAPISID=my_sapisid_token"));
        assert!(header.contains("VISITOR_INFO1_LIVE=abc"));
    }
}
