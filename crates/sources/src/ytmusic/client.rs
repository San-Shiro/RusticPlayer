use async_trait::async_trait;
use rustic_core::api::{AuthFlow, AuthState, Playlist, ResolvedAudio, Track, TrackSource};
use reqwest::Client;
use serde_json::Value;
use std::sync::RwLock;

use crate::{SourceError, SourceProvider};
use super::auth::{YtmSession, YTM_ORIGIN};
use super::parser::{
    parse_artist_details, parse_browse_feed, parse_collection_browse, parse_continuation_sections,
    parse_continuation_tracks, parse_library_albums, parse_library_artists,
    parse_library_playlists, parse_library_songs, parse_lyrics, parse_queue,
    parse_queue_continuation, parse_search_results, parse_search_suggestions,
};
use super::types::{
    BrowseFeed, LibraryArtist, LibraryItem, LyricsDocument, QueueSnapshot, YtArtistDetails,
    YtCollectionDetails,
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct YtAccountInfo {
    pub name: String,
    pub handle: Option<String>,
    pub avatar_url: Option<String>,
}

pub struct YtMusicProvider {
    client: Client,
    session: RwLock<YtmSession>,
    account_info: RwLock<Option<YtAccountInfo>>,
}

impl YtMusicProvider {
    pub fn new() -> Self {
        let session = YtmSession::from_file_or_guest("cookies.json");
        let default_account = if session.is_authenticated() {
            Some(YtAccountInfo {
                name: "Ketan".into(),
                handle: Some("@ketan-saini".into()),
                avatar_url: Some("https://yt3.ggpht.com/yti/ANjgQV-ziaKBT7JfCDxK9hFLdG5yQ52fCFLOa3kW2Zb_fsbtSWY=s108-c-k-c0x00ffffff-no-rj".into()),
            })
        } else {
            None
        };
        Self {
            client: Client::builder()
                .pool_max_idle_per_host(10)
                .build()
                .unwrap_or_else(|_| Client::new()),
            session: RwLock::new(session),
            account_info: RwLock::new(default_account),
        }
    }

    pub fn with_auth(sapisid: String) -> Self {
        let session = YtmSession::Authenticated {
            cookie_header: format!("SAPISID={}", sapisid),
            sapisid,
            visitor_data: None,
        };
        Self {
            client: Client::new(),
            session: RwLock::new(session),
            account_info: RwLock::new(None),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.session.read().map(|s| s.is_authenticated()).unwrap_or(false)
    }

    /// Fetches and parses a YouTube Music browse feed (e.g. `FEmusic_home` or mood chips).
    pub async fn fetch_browse_feed(&self, browse_id: &str, params: Option<&str>) -> Result<BrowseFeed, SourceError> {
        let url = format!("{}/youtubei/v1/browse", YTM_ORIGIN);
        let mut payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "browseId": browse_id
        });

        if let Some(p) = params {
            if !p.is_empty() {
                payload["params"] = serde_json::Value::String(p.to_string());
            }
        }

        let req = self.client.post(&url).json(&payload);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        let res = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(target: "sources::ytmusic", "Network error fetching browse feed: {:?}", e);
                return Err(SourceError::Network(e.to_string()));
            }
        };

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            tracing::warn!(target: "sources::ytmusic", "YTM browse returned {}: {}", status, text);
            return Err(SourceError::Network(format!("YTM returned HTTP {}", status)));
        }

        let body: Value = res.json().await.map_err(|e| SourceError::Network(e.to_string()))?;

        // Update visitorData if returned
        if let Some(v_data) = body.pointer("/responseContext/visitorData").and_then(|v| v.as_str()) {
            if let Ok(mut s) = self.session.write() {
                if s.visitor_data().is_none() {
                    s.set_visitor_data(v_data.to_string());
                }
            }
        }

        let mut feed = parse_browse_feed(&body)?;

        // Fetch up to 3 continuations to populate all home sections (Listen again, Quick picks, Forgotten favorites, etc.)
        let mut continuation_count = 0;
        while let Some(cont_token) = feed.continuation_token.take() {
            if continuation_count >= 3 {
                feed.continuation_token = Some(cont_token);
                break;
            }
            continuation_count += 1;
            let cont_payload = serde_json::json!({
                "context": {
                    "client": {
                        "clientName": "WEB_REMIX",
                        "clientVersion": "1.20260213.01.00",
                        "hl": "en",
                        "gl": "US"
                    }
                },
                "continuation": cont_token
            });
            let cont_req = self.client.post(&url).json(&cont_payload);
            let cont_req = if let Ok(s) = self.session.read() {
                s.apply_headers(cont_req)
            } else {
                cont_req
            };
            if let Ok(cont_res) = cont_req.send().await {
                if cont_res.status().is_success() {
                    if let Ok(cont_body) = cont_res.json::<Value>().await {
                        let (extra_secs, next_tok) = parse_continuation_sections(&cont_body);
                        let has_extra = !extra_secs.is_empty();
                        feed.sections.extend(extra_secs);
                        feed.continuation_token = next_tok;
                        if !has_extra && feed.continuation_token.is_none() {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(feed)
    }

    /// Returns the current authentication state.
    pub fn auth_state(&self) -> AuthState {
        let logged_in = self.is_authenticated();
        let (account_name, account_avatar_url, account_handle) = if logged_in {
            if let Ok(guard) = self.account_info.read() {
                if let Some(ref info) = *guard {
                    (Some(info.name.clone()), info.avatar_url.clone(), info.handle.clone())
                } else {
                    (
                        Some("Ketan".into()),
                        Some("https://yt3.ggpht.com/yti/ANjgQV-ziaKBT7JfCDxK9hFLdG5yQ52fCFLOa3kW2Zb_fsbtSWY=s108-c-k-c0x00ffffff-no-rj".into()),
                        Some("@ketan-saini".into()),
                    )
                }
            } else {
                (
                    Some("Ketan".into()),
                    Some("https://yt3.ggpht.com/yti/ANjgQV-ziaKBT7JfCDxK9hFLdG5yQ52fCFLOa3kW2Zb_fsbtSWY=s108-c-k-c0x00ffffff-no-rj".into()),
                    Some("@ketan-saini".into()),
                )
            }
        } else {
            (None, None, None)
        };

        AuthState {
            provider: "ytmusic".into(),
            logged_in,
            account_name,
            account_avatar_url,
            account_handle,
        }
    }

    /// Fetches the user's account details (name, handle, avatar) from the InnerTube account menu endpoint.
    pub async fn fetch_account_info(&self) -> Result<YtAccountInfo, SourceError> {
        let session = self.session.read().map_err(|_| SourceError::NotAuthenticated("Session lock poisoned".into()))?.clone();
        if !session.is_authenticated() {
            return Err(SourceError::NotAuthenticated("Not authenticated with cookies".into()));
        }

        let url = format!("{}/youtubei/v1/account/account_menu", YTM_ORIGIN);
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            }
        });

        let req = self.client.post(&url).json(&payload);
        let req = session.apply_headers(req);

        let res = req.send().await.map_err(|e| SourceError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(SourceError::Network(format!("Failed to fetch account menu: status {}", res.status())));
        }

        let body: serde_json::Value = res.json().await.map_err(|e| SourceError::ParseError(e.to_string()))?;

        // Extract activeAccountHeaderRenderer
        let header = body.pointer("/actions/0/openPopupAction/popup/multiPageMenuRenderer/header/activeAccountHeaderRenderer")
            .ok_or_else(|| SourceError::ParseError("activeAccountHeaderRenderer not found in account menu response".into()))?;

        let name = header.pointer("/accountName/runs/0/text")
            .and_then(|v| v.as_str())
            .unwrap_or("Ketan")
            .to_string();

        let handle = header.pointer("/channelHandle/runs/0/text")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let avatar_url = header.pointer("/accountPhoto/thumbnails")
            .and_then(|t| t.as_array())
            .and_then(|arr| arr.last())
            .and_then(|thumb| thumb.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string());

        let info = YtAccountInfo {
            name,
            handle,
            avatar_url,
        };

        if let Ok(mut lock) = self.account_info.write() {
            *lock = Some(info.clone());
        }

        Ok(info)
    }

    /// Reloads cookies from a file (default `cookies.json`), searching candidate paths.
    pub fn reload_cookies_file(&self, path: &str) -> Result<AuthState, SourceError> {
        let new_session = YtmSession::from_file_or_guest(path);
        if !new_session.is_authenticated() {
            return Err(SourceError::NotAuthenticated("No valid cookies found in file".into()));
        }
        if let Ok(mut s) = self.session.write() {
            *s = new_session;
        }
        if let Ok(mut lock) = self.account_info.write() {
            *lock = Some(YtAccountInfo {
                name: "Ketan".into(),
                handle: Some("@ketan-saini".into()),
                avatar_url: Some("https://yt3.ggpht.com/yti/ANjgQV-ziaKBT7JfCDxK9hFLdG5yQ52fCFLOa3kW2Zb_fsbtSWY=s108-c-k-c0x00ffffff-no-rj".into()),
            });
        }
        tracing::info!(target: "sources::ytmusic", "Successfully reloaded cookies from {}", path);
        Ok(self.auth_state())
    }

    /// Validates and logs in with raw cookie JSON string, atomically saving to `cookies.json`.
    pub async fn login_with_cookies(&self, cookie_json: &str) -> Result<AuthState, SourceError> {
        let new_session = YtmSession::from_cookie_json(cookie_json)
            .ok_or_else(|| SourceError::NotAuthenticated("Invalid cookie JSON or missing SAPISID".into()))?;

        // Test the session against YTM
        let url = format!("{}/youtubei/v1/browse", YTM_ORIGIN);
        let payload = serde_json::json!({
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
        let req = self.client.post(&url).json(&payload);
        let req = new_session.apply_headers(req);

        let res = req.send().await.map_err(|e| SourceError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(SourceError::NotAuthenticated(format!("YouTube Music rejected cookies with status {}", res.status())));
        }

        // Atomically save to cookies.json
        let _ = super::auth::save_cookies_file(cookie_json, "cookies.json");

        // Swap active session
        if let Ok(mut s) = self.session.write() {
            *s = new_session;
        }

        let _ = self.fetch_account_info().await;

        tracing::info!(target: "sources::ytmusic", "Successfully logged in with imported cookies");
        Ok(self.auth_state())
    }

    /// Fetches dynamic watch next queue and returns the queue snapshot plus any lyrics browseId.
    pub async fn fetch_queue_with_lyrics_id(&self, video_id: &str) -> Result<(QueueSnapshot, Option<String>), SourceError> {
        let url = format!("{}/youtubei/v1/next", YTM_ORIGIN);
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "videoId": video_id,
            "playlistId": format!("RDAMVM{}", video_id)
        });

        let req = self.client.post(&url).json(&payload);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        let res = req.send().await.map_err(|e| SourceError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(SourceError::Network(format!("YTM next returned HTTP {}", res.status())));
        }

        let body: Value = res.json().await.map_err(|e| SourceError::Network(e.to_string()))?;
        parse_queue(&body)
    }

    /// Convenience wrapper returning `QueueSnapshot`.
    pub async fn fetch_queue(&self, video_id: &str) -> Result<QueueSnapshot, SourceError> {
        self.fetch_queue_with_lyrics_id(video_id).await.map(|(q, _)| q)
    }

    /// Fetches radio suggestions for a video_id and returns them as mapped `Track` instances.
    pub async fn fetch_suggestions(&self, video_id: &str) -> Result<Vec<Track>, SourceError> {
        let snap = self.fetch_queue(video_id).await?;
        let tracks: Vec<Track> = snap
            .upcoming
            .into_iter()
            .map(|item| {
                let duration_secs = item.parse_duration_secs();
                Track {
                    id: item.video_id.clone(),
                    title: item.title,
                    artists: vec![item.artist],
                    album: "YouTube Music".into(),
                    duration_secs,
                    source: TrackSource::YouTubeMusic {
                        video_id: item.video_id,
                    },
                    isrc: None,
                    artwork_url: item.thumbnail_url,
                    content_hash: None,
                }
            })
            .collect();
        Ok(tracks)
    }

    /// Reports playback telemetry to YouTube with optional playlist/album context.
    pub async fn register_playback_beacon(&self, video_id: &str, playlist_id: Option<&str>) -> Result<(), SourceError> {
        let stats_url = "https://www.youtube.com/api/stats/playback";
        let cpn: String = {
            use rand::Rng;
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
            let mut rng = rand::thread_rng();
            (0..16)
                .map(|_| {
                    let idx = rng.gen_range(0..chars.len());
                    chars[idx] as char
                })
                .collect()
        };

        let mut params = vec![
            ("ns", "yt".to_string()),
            ("el", "detailpage".to_string()),
            ("cpn", cpn),
            ("docid", video_id.to_string()),
            ("ver", "2".to_string()),
            ("c", "WEB_REMIX".to_string()),
            ("cver", "1.20260213.01.00".to_string()),
        ];
        if let Some(list) = playlist_id {
            if !list.is_empty() {
                params.push(("list", list.to_string()));
            }
        }

        let req = self.client.get(stats_url).query(&params);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        // Fire-and-forget: failure to log telemetry should not disrupt playback
        let _ = req.send().await;
        Ok(())
    }

    /// Reports playback telemetry to YouTube so the user's Watch History updates and recommendations improve.
    pub async fn report_playback_history(&self, video_id: &str) -> Result<(), SourceError> {
        self.register_playback_beacon(video_id, None).await
    }

    /// Fetches continuation tracks for radio/automix queue.
    pub async fn fetch_radio_continuation(&self, continuation: &str) -> Result<(Vec<Track>, Option<String>), SourceError> {
        let url = format!("{}/youtubei/v1/next", YTM_ORIGIN);
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "continuation": continuation
        });

        let req = self.client.post(&url).json(&payload);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        let res = req.send().await.map_err(|e| SourceError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(SourceError::Network(format!("YTM radio continuation returned HTTP {}", res.status())));
        }

        let body: Value = res.json().await.map_err(|e| SourceError::Network(e.to_string()))?;
        Ok(parse_queue_continuation(&body))
    }

    /// Fetches automix radio for a given video ID (`RDAMVM<video_id>`).
    pub async fn fetch_automix_radio(&self, video_id: &str) -> Result<QueueSnapshot, SourceError> {
        self.fetch_queue(video_id).await
    }

    /// Fetches queue for a given playlist or album browseId/playlistId.
    pub async fn fetch_queue_for_playlist(&self, playlist_id: &str) -> Result<QueueSnapshot, SourceError> {
        let url = format!("{}/youtubei/v1/next", YTM_ORIGIN);
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "playlistId": playlist_id
        });

        let req = self.client.post(&url).json(&payload);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        let res = req.send().await.map_err(|e| SourceError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(SourceError::Network(format!("YTM next returned HTTP {}", res.status())));
        }

        let body: Value = res.json().await.map_err(|e| SourceError::Network(e.to_string()))?;
        let (snap, _) = parse_queue(&body)?;
        Ok(snap)
    }

    /// Searches YouTube Music and returns matching tracks.
    pub async fn search_tracks(&self, query: &str) -> Result<Vec<Track>, SourceError> {
        self.search(query, 20).await
    }

    /// Fetches live search suggestions from YouTube Music for autofill / autocomplete.
    pub async fn get_search_suggestions(&self, query: &str) -> Result<Vec<String>, SourceError> {
        let url = format!("{}/youtubei/v1/music/get_search_suggestions", YTM_ORIGIN);
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "input": query
        });

        let req = self.client.post(&url).json(&payload);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        let res = req.send().await.map_err(|e| SourceError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(SourceError::Network(format!("YTM suggestions returned HTTP {}", res.status())));
        }

        let body: Value = res.json().await.map_err(|e| SourceError::Network(e.to_string()))?;
        parse_search_suggestions(&body)
    }

    /// Fetches lyrics using the two-step flow with the lyrics browseId (e.g. `MPLYt_...`).
    pub async fn fetch_lyrics(&self, browse_id: &str) -> Result<LyricsDocument, SourceError> {
        let url = format!("{}/youtubei/v1/browse", YTM_ORIGIN);
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "browseId": browse_id
        });

        let req = self.client.post(&url).json(&payload);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        let res = req.send().await.map_err(|e| SourceError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(SourceError::Network(format!("YTM lyrics returned HTTP {}", res.status())));
        }

        let body: Value = res.json().await.map_err(|e| SourceError::Network(e.to_string()))?;
        parse_lyrics(&body)
    }

    /// Fetches lyrics directly for a video ID by resolving the lyrics browse ID from `/youtubei/v1/next`.
    pub async fn fetch_lyrics_for_video(&self, video_id: &str) -> Result<LyricsDocument, SourceError> {
        let (_, lyrics_id) = self.fetch_queue_with_lyrics_id(video_id).await?;
        if let Some(browse_id) = lyrics_id {
            self.fetch_lyrics(&browse_id).await
        } else {
            Ok(LyricsDocument {
                plain_text: None,
                lines: vec![super::types::LyricsLine {
                    text: "Lyrics not available for this track".into(),
                    start_time_ms: None,
                }],
                source: None,
            })
        }
    }

    /// Helper to fetch raw JSON response from `/youtubei/v1/browse`
    async fn fetch_browse_raw(&self, browse_id: &str) -> Result<Value, SourceError> {
        let url = format!("{}/youtubei/v1/browse", YTM_ORIGIN);
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "browseId": browse_id
        });

        let req = self.client.post(&url).json(&payload);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        let res = req.send().await.map_err(|e| SourceError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(SourceError::Network(format!("YTM browse returned HTTP {}", res.status())));
        }

        let body: Value = res.json().await.map_err(|e| SourceError::Network(e.to_string()))?;
        Ok(body)
    }

    /// Fetches user's saved playlists (`FEmusic_liked_playlists`).
    pub async fn fetch_library_playlists(&self) -> Result<Vec<LibraryItem>, SourceError> {
        let body = self.fetch_browse_raw("FEmusic_liked_playlists").await?;
        parse_library_playlists(&body)
    }

    /// Fetches user's saved albums (`FEmusic_liked_albums`).
    pub async fn fetch_library_albums(&self) -> Result<Vec<LibraryItem>, SourceError> {
        let body = self.fetch_browse_raw("FEmusic_liked_albums").await?;
        parse_library_albums(&body)
    }

    /// Fetches user's liked songs (`FEmusic_liked_videos`), returning tracks and optional continuation token.
    pub async fn fetch_library_songs(&self) -> Result<(Vec<Track>, Option<String>), SourceError> {
        let body = self.fetch_browse_raw("FEmusic_liked_videos").await?;
        parse_library_songs(&body)
    }

    /// Fetches continuation page of library liked songs.
    pub async fn fetch_library_songs_continuation(&self, token: &str) -> Result<(Vec<Track>, Option<String>), SourceError> {
        let url = format!("{}/youtubei/v1/browse", YTM_ORIGIN);
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "continuation": token
        });

        let req = self.client.post(&url).json(&payload);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        let res = req.send().await.map_err(|e| SourceError::Network(e.to_string()))?;
        if !res.status().is_success() {
            return Err(SourceError::Network(format!("YTM browse continuation returned HTTP {}", res.status())));
        }

        let body: Value = res.json().await.map_err(|e| SourceError::Network(e.to_string()))?;
        Ok(parse_continuation_tracks(&body))
    }

    /// Fetches details and tracklist for a playlist or album.
    pub async fn fetch_collection_details(&self, id: &str) -> Result<YtCollectionDetails, SourceError> {
        let browse_id = if id.starts_with("VL") || id.starts_with("MPRE") {
            id.to_string()
        } else if id.starts_with("OLAK5uy_") || id.starts_with("RDAMPL") {
            id.to_string()
        } else {
            format!("VL{}", id)
        };

        let browse_res = self.fetch_browse_raw(&browse_id).await;
        if let Ok(ref body) = browse_res {
            if let Ok(details) = parse_collection_browse(body, id) {
                if !details.tracks.is_empty() {
                    return Ok(details);
                }
            }
        }

        // Fallback to fetch_queue_for_playlist with clean ID
        let clean_id = id.trim_start_matches("VL");
        let q_snap = self.fetch_queue_for_playlist(clean_id).await?;

        let mut tracks = Vec::new();
        if let Some(ref cur) = q_snap.current_track {
            tracks.push(Track {
                id: cur.video_id.clone(),
                title: cur.title.clone(),
                artists: vec![cur.artist.clone()],
                album: q_snap.source_playlist_title.clone().unwrap_or_else(|| "YouTube Music".into()),
                duration_secs: parse_duration_str(&cur.duration_str),
                source: TrackSource::YouTubeMusic { video_id: cur.video_id.clone() },
                isrc: None,
                artwork_url: cur.thumbnail_url.clone(),
                content_hash: None,
            });
        }
        for u in &q_snap.upcoming {
            tracks.push(Track {
                id: u.video_id.clone(),
                title: u.title.clone(),
                artists: vec![u.artist.clone()],
                album: q_snap.source_playlist_title.clone().unwrap_or_else(|| "YouTube Music".into()),
                duration_secs: parse_duration_str(&u.duration_str),
                source: TrackSource::YouTubeMusic { video_id: u.video_id.clone() },
                isrc: None,
                artwork_url: u.thumbnail_url.clone(),
                content_hash: None,
            });
        }

        let title = q_snap.source_playlist_title.unwrap_or_else(|| "Playlist".into());
        let count = tracks.len();

        Ok(YtCollectionDetails {
            id: id.to_string(),
            title,
            subtitle: format!("{} tracks", count),
            description: None,
            thumbnail_url: tracks.first().and_then(|t| t.artwork_url.clone()),
            track_count: Some(count),
            tracks,
        })
    }

    /// Fetches user's followed artists (`FEmusic_library_corpus_artists`).
    pub async fn fetch_library_artists(&self) -> Result<Vec<LibraryArtist>, SourceError> {
        let body = self.fetch_browse_raw("FEmusic_library_corpus_artists").await?;
        parse_library_artists(&body)
    }

    /// Fetches dedicated artist details, top tracks, releases, and related artists.
    pub async fn fetch_artist_details(&self, channel_id: &str) -> Result<YtArtistDetails, SourceError> {
        let body = self.fetch_browse_raw(channel_id).await?;
        parse_artist_details(&body, channel_id)
    }
}

fn parse_duration_str(s: &str) -> u32 {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(m), Ok(sec)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
            return m * 60 + sec;
        }
    } else if parts.len() == 3 {
        if let (Ok(h), Ok(m), Ok(sec)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
            return h * 3600 + m * 60 + sec;
        }
    }
    180
}

#[async_trait]
impl SourceProvider for YtMusicProvider {
    fn slug(&self) -> &'static str {
        "ytmusic"
    }

    async fn login(&self, flow: AuthFlow) -> Result<AuthState, SourceError> {
        match flow {
            AuthFlow::BrowserCookie => self.reload_cookies_file("cookies.json"),
            AuthFlow::AccessToken(json) => self.login_with_cookies(&json).await,
            _ => Err(SourceError::NotAuthenticated("Unsupported auth flow for YTM".into())),
        }
    }

    async fn search(&self, query: &str, _limit: usize) -> Result<Vec<Track>, SourceError> {
        let url = format!("{}/youtubei/v1/search", YTM_ORIGIN);
        let payload = serde_json::json!({
            "context": {
                "client": {
                    "clientName": "WEB_REMIX",
                    "clientVersion": "1.20260213.01.00",
                    "hl": "en",
                    "gl": "US"
                }
            },
            "query": query
        });

        let req = self.client.post(&url).json(&payload);
        let req = if let Ok(s) = self.session.read() {
            s.apply_headers(req)
        } else {
            req
        };

        let res = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(target: "sources", "YouTube Music search request failed: {:?}", e);
                return Err(SourceError::Network(e.to_string()));
            }
        };

        if !res.status().is_success() {
            return Err(SourceError::Network(format!("YouTube Music search returned HTTP {}", res.status())));
        }

        let body: Value = res.json().await.map_err(|e| SourceError::Network(e.to_string()))?;
        parse_search_results(&body)
    }

    async fn library(&self) -> Result<Vec<Playlist>, SourceError> {
        let items = self.fetch_library_playlists().await.unwrap_or_default();
        let playlists = items
            .into_iter()
            .map(|item| Playlist {
                id: item.id,
                provider: "ytmusic".into(),
                name: item.title,
                tracks: vec![],
            })
            .collect();
        Ok(playlists)
    }

    async fn resolve(&self, track: &Track) -> Result<ResolvedAudio, SourceError> {
        match &track.source {
            TrackSource::YouTubeMusic { video_id } => {
                let session = self
                    .session
                    .read()
                    .map_err(|_| SourceError::Unavailable("session lock poisoned".into()))?
                    .clone();
                super::stream::resolve_video_stream(&self.client, &session, video_id).await
            }
            _ => Err(SourceError::Unavailable("Not a YTM track".into())),
        }
    }
}
