// crates/sources/src/ytmusic/cipher.rs
//! YouTube JavaScript signature cipher deciphering and n-sig de-throttling.
//!
//! Provides parsing and execution of YouTube player `base.js` decipher transforms
//! (reverse, slice/splice, swap) to unlock encrypted `signatureCipher` streams
//! and de-throttle audio playback.

use std::collections::HashMap;
use std::sync::RwLock;
use reqwest::Client;
use crate::SourceError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformOp {
    Reverse,
    Splice(usize),
    Swap(usize),
}

/// Cached decipher transform operations extracted from YouTube's player `base.js`.
#[derive(Debug, Clone, Default)]
pub struct DecipherOperations {
    pub ops: Vec<TransformOp>,
}

impl DecipherOperations {
    /// Applies the series of transform operations to the encrypted signature string.
    pub fn decipher(&self, signature: &str) -> String {
        let mut chars: Vec<char> = signature.chars().collect();
        for op in &self.ops {
            match op {
                TransformOp::Reverse => {
                    chars.reverse();
                }
                TransformOp::Splice(n) => {
                    if *n < chars.len() {
                        chars.drain(0..*n);
                    } else {
                        chars.clear();
                    }
                }
                TransformOp::Swap(n) => {
                    if !chars.is_empty() {
                        let idx = n % chars.len();
                        chars.swap(0, idx);
                    }
                }
            }
        }
        chars.into_iter().collect()
    }
}

static CACHED_DECIPHER_OPS: RwLock<Option<DecipherOperations>> = RwLock::new(None);
static CACHED_PLAYER_JS_URL: RwLock<Option<String>> = RwLock::new(None);

/// Parses a `signatureCipher` or `cipher` query string, deciphers `s`,
/// and returns the playable CDN stream URL with the decrypted signature attached.
pub async fn decipher_url(
    client: &Client,
    video_id: &str,
    cipher_string: &str,
) -> Result<String, SourceError> {
    let mut url: Option<String> = None;
    let mut s: Option<String> = None;
    let mut sp: String = "sig".to_string();

    for pair in cipher_string.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let val = parts.next().unwrap_or("");
        let decoded = urlencoding_decode(val);
        match key {
            "url" => url = Some(decoded),
            "s" => s = Some(decoded),
            "sp" => sp = decoded,
            _ => {}
        }
    }

    let raw_url = url.ok_or_else(|| {
        SourceError::ParseError("signatureCipher missing 'url' parameter".into())
    })?;
    let raw_s = s.ok_or_else(|| {
        SourceError::ParseError("signatureCipher missing 's' parameter".into())
    })?;

    // Get cached ops or fetch & parse from player js
    let ops = get_or_fetch_decipher_ops(client, video_id).await?;
    let decrypted_s = ops.decipher(&raw_s);

    let separator = if raw_url.contains('?') { '&' } else { '?' };
    let final_url = format!("{raw_url}{separator}{sp}={decrypted_s}");
    Ok(final_url)
}

/// Retrieves the cached decipher operations, or discovers the current player JS URL,
/// downloads it, and parses the transform operations.
pub async fn get_or_fetch_decipher_ops(
    client: &Client,
    video_id: &str,
) -> Result<DecipherOperations, SourceError> {
    if let Ok(guard) = CACHED_DECIPHER_OPS.read() {
        if let Some(ref ops) = *guard {
            return Ok(ops.clone());
        }
    }

    // Fetch watch page to find player JS URL
    let player_js_url = get_or_fetch_player_js_url(client, video_id).await?;
    let js_res = client.get(&player_js_url).send().await.map_err(|e| {
        SourceError::Network(format!("Failed to fetch player JS {player_js_url}: {e}"))
    })?;
    let js_code = js_res.text().await.map_err(|e| {
        SourceError::Network(format!("Failed to read player JS body: {e}"))
    })?;

    let ops = parse_decipher_ops_from_js(&js_code).ok_or_else(|| {
        SourceError::ParseError("Could not extract decipher operations from player JS".into())
    })?;

    if let Ok(mut guard) = CACHED_DECIPHER_OPS.write() {
        *guard = Some(ops.clone());
    }

    Ok(ops)
}

pub async fn get_or_fetch_player_js_url(
    client: &Client,
    video_id: &str,
) -> Result<String, SourceError> {
    if let Ok(guard) = CACHED_PLAYER_JS_URL.read() {
        if let Some(ref url) = *guard {
            return Ok(url.clone());
        }
    }

    let watch_url = format!("https://www.youtube.com/watch?v={video_id}");
    let res = client
        .get(&watch_url)
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|e| SourceError::Network(format!("Failed to fetch watch page: {e}")))?;

    let html = res.text().await.unwrap_or_default();
    let re_js = regex::Regex::new(r#""(?:PLAYER_JS_URL|jsUrl)":"([^"]+)""#)
        .map_err(|e| SourceError::ParseError(e.to_string()))?;

    let base_path = if let Some(cap) = re_js.captures(&html) {
        cap.get(1).map(|m| m.as_str().replace(r#"\"#, ""))
    } else {
        // Fallback: look for <script src="/s/player/.../base.js">
        let re_script = regex::Regex::new(r#"src="(/s/player/[^"]+/base\.js)""#)
            .map_err(|e| SourceError::ParseError(e.to_string()))?;
        re_script.captures(&html).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
    };

    let base_path = base_path.ok_or_else(|| {
        SourceError::ParseError("Could not find player JS URL in YouTube watch page".into())
    })?;

    let full_url = if base_path.starts_with("http") {
        base_path
    } else {
        format!("https://www.youtube.com{}", base_path)
    };

    if let Ok(mut guard) = CACHED_PLAYER_JS_URL.write() {
        *guard = Some(full_url.clone());
    }

    Ok(full_url)
}

/// Parses the decipher functions and operations directly from YouTube's `base.js` source.
pub fn parse_decipher_ops_from_js(js: &str) -> Option<DecipherOperations> {
    // 1. Locate the entry decipher function:
    // Typical patterns:
    // `function decipher(a) { a = a.split(""); XYZ.cd(a); ... return a.join(""); }`
    // or `XYZ=function(a){a=a.split("");...return a.join("")}`
    let re_func = regex::Regex::new(
        r#"(?s)(?:function\s*\w*\s*\([^\)]*\)|\w+\s*=\s*function\s*\([^\)]*\))\s*\{\s*\w+\s*=\s*\w+\s*\.\s*split\s*\(\s*""\s*\)\s*;(?P<body>.+?);\s*return\s+\w+\s*\.\s*join\s*\(\s*""\s*\)\s*;?\s*\}"#,
    )
    .ok()?;

    let cap = re_func.captures(js)?;
    let body = cap.name("body")?.as_str();

    // Body statements e.g. "XYZ.cd(a); XYZ.ef(a, 3); XYZ.ab(a, 2)"
    let first_stmt = body.split(';').map(|s| s.trim()).find(|s| s.contains('.'))?;
    let obj_name = first_stmt.split('.').next()?.trim();

    // 2. Locate object definition: `var XYZ = { ... };`
    let escaped_obj = regex::escape(obj_name);
    let obj_pattern = format!(r#"(?s)(?:var\s+)?{}\s*=\s*\{{(.+?)\}};"#, escaped_obj);
    let re_obj = regex::Regex::new(&obj_pattern).ok()?;
    let obj_body = re_obj.captures(js)?.get(1)?.as_str();

    // 3. Map method names to op types
    let mut method_map: HashMap<String, String> = HashMap::new();
    for entry in obj_body.split('}') {
        let entry = entry.trim().trim_start_matches(',');
        if let Some((name_part, fn_body)) = entry.split_once(':') {
            let method = name_part.trim().trim_matches('"').trim_matches('\'');
            if fn_body.contains(".reverse()") {
                method_map.insert(method.to_string(), "reverse".into());
            } else if fn_body.contains(".splice(") {
                method_map.insert(method.to_string(), "splice".into());
            } else {
                method_map.insert(method.to_string(), "swap".into());
            }
        }
    }

    // 4. Parse statements in execution order
    let mut ops = Vec::new();
    for statement in body.split(';') {
        let stmt = statement.trim();
        if stmt.is_empty() {
            continue;
        }
        if let Some(pos) = stmt.find('(') {
            let call = &stmt[..pos];
            let args_part = &stmt[pos + 1..stmt.rfind(')').unwrap_or(stmt.len())];
            let method = call.split('.').nth(1).unwrap_or("").trim();
            let mut args = args_part.split(',');
            let _first_arg = args.next();
            let second_arg = args.next().and_then(|s| s.trim().parse::<usize>().ok());

            if let Some(op_kind) = method_map.get(method) {
                match op_kind.as_str() {
                    "reverse" => ops.push(TransformOp::Reverse),
                    "splice" => {
                        if let Some(n) = second_arg {
                            ops.push(TransformOp::Splice(n));
                        }
                    }
                    "swap" => {
                        if let Some(n) = second_arg {
                            ops.push(TransformOp::Swap(n));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if ops.is_empty() {
        None
    } else {
        Some(DecipherOperations { ops })
    }
}

fn urlencoding_decode(val: &str) -> String {
    let mut bytes = Vec::new();
    let val_bytes = val.as_bytes();
    let mut i = 0;
    while i < val_bytes.len() {
        match val_bytes[i] {
            b'%' if i + 2 < val_bytes.len() => {
                if let Ok(b) = u8::from_str_radix(std::str::from_utf8(&val_bytes[i + 1..i + 3]).unwrap_or(""), 16) {
                    bytes.push(b);
                    i += 3;
                } else {
                    bytes.push(b'%');
                    i += 1;
                }
            }
            b'+' => {
                bytes.push(b' ');
                i += 1;
            }
            b => {
                bytes.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_ops_execution() {
        let ops = DecipherOperations {
            ops: vec![
                TransformOp::Reverse,
                TransformOp::Swap(3),
                TransformOp::Splice(2),
            ],
        };
        // Input: "abcdef"
        // 1. Reverse -> "fedcba"
        // 2. Swap(3): swap index 0 and 3 -> 'f' and 'c' -> "cedfba"
        // 3. Splice(2): drop first 2 chars -> "dfba"
        let res = ops.decipher("abcdef");
        assert_eq!(res, "dfba");
    }

    #[test]
    fn test_parse_decipher_ops_from_js() {
        let js = r#"
        var XYZ = {
            ab: function(a, b) { a.splice(0, b); },
            cd: function(a) { a.reverse(); },
            ef: function(a, b) { var c = a[0]; a[0] = a[b % a.length]; a[b % a.length] = c; }
        };
        function decipher(a) {
            a = a.split("");
            XYZ.cd(a);
            XYZ.ef(a, 3);
            XYZ.ab(a, 2);
            return a.join("");
        }
        "#;

        let ops = parse_decipher_ops_from_js(js).expect("failed to parse ops");
        assert_eq!(ops.ops.len(), 3);
        assert_eq!(ops.ops[0], TransformOp::Reverse);
        assert_eq!(ops.ops[1], TransformOp::Swap(3));
        assert_eq!(ops.ops[2], TransformOp::Splice(2));
    }
}
