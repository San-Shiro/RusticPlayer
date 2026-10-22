use serde_json::Value;
use crate::SourceError;
use super::types::{
    BrowseFeed, BrowseSection, LibraryArtist, LibraryItem, LibraryItemType, LyricsDocument,
    LyricsLine, MoodChip, QueueSnapshot, QueueTrackItem, ShelfItem, ShelfItemType,
    YtAlbumItem, YtArtistDetails, YtCollectionDetails, YtRelatedArtist,
};

/// Safely extracts text from either a runs array or a simpleText field.
pub fn extract_text(val: &Value) -> Option<String> {
    if let Some(text) = val.get("simpleText").and_then(|v| v.as_str()) {
        return Some(text.to_string());
    }
    if let Some(runs) = val.get("runs").and_then(|v| v.as_array()) {
        let joined: String = runs
            .iter()
            .filter_map(|r| r.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join("");
        if !joined.is_empty() {
            return Some(joined);
        }
    }
    None
}

/// Normalizes image URLs to high-resolution square format (=w544-h544-l90-rj) for Google user content.
pub fn normalize_thumbnail_url(url: &str) -> String {
    if url.contains("googleusercontent.com") || url.contains("ggpht.com") {
        if let Some(idx) = url.find("=w") {
            let base = &url[..idx];
            return format!("{}=w544-h544-l90-rj", base);
        } else if let Some(idx) = url.find("=s") {
            let base = &url[..idx];
            return format!("{}=w544-h544-l90-rj", base);
        }
    }
    url.to_string()
}

/// Extracts the largest/best thumbnail URL from a thumbnail container, normalized to square high-res.
pub fn extract_thumbnail(val: &Value) -> Option<String> {
    let raw = val.get("thumbnails")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.last())
        .and_then(|t| t.get("url"))
        .and_then(|u| u.as_str())?;
    Some(normalize_thumbnail_url(raw))
}

/// Parses YouTube Music home / browse responses into structured `BrowseFeed`.
pub fn parse_browse_feed(value: &Value) -> Result<BrowseFeed, SourceError> {
    let tabs = value
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| SourceError::ParseError("Missing tabs in browse response".into()))?;

    let sec_list = tabs
        .first()
        .and_then(|t| t.pointer("/tabRenderer/content/sectionListRenderer"));

    let first_tab = sec_list
        .and_then(|sl| sl.get("contents"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| SourceError::ParseError("Missing sectionListRenderer contents".into()))?;

    let mut chips = Vec::new();
    if let Some(chip_cloud) = sec_list
        .and_then(|sl| sl.pointer("/header/chipCloudRenderer/chips"))
        .and_then(|v| v.as_array())
    {
        for chip_val in chip_cloud {
            if let Some(c) = chip_val.get("chipCloudChipRenderer") {
                let title = c
                    .get("text")
                    .and_then(extract_text)
                    .unwrap_or_default();
                let params = c
                    .pointer("/navigationEndpoint/browseEndpoint/params")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let is_selected = c.get("isSelected").and_then(|v| v.as_bool()).unwrap_or(false);
                if !title.is_empty() {
                    chips.push(MoodChip {
                        title,
                        params,
                        is_selected,
                    });
                }
            }
        }
    }

    let mut continuation_token = None;
    let mut sections = Vec::new();

    for section_val in first_tab {
        if let Some(cont) = section_val.get("continuationItemRenderer") {
            if let Some(token) = cont.pointer("/continuationEndpoint/continuationCommand/token").and_then(|v| v.as_str()) {
                continuation_token = Some(token.to_string());
            }
        }
        // 1. MusicCarouselShelfRenderer
        else if let Some(shelf) = section_val.get("musicCarouselShelfRenderer") {
            if let Some(parsed_sec) = parse_carousel_shelf(shelf) {
                sections.push(parsed_sec);
            }
        }
        // 2. MusicImmersiveCarouselShelfRenderer (Hero banner / featured)
        else if let Some(shelf) = section_val.get("musicImmersiveCarouselShelfRenderer") {
            if let Some(parsed_sec) = parse_carousel_shelf(shelf) {
                sections.push(parsed_sec);
            }
        }
        // 3. MusicShelfRenderer
        else if let Some(shelf) = section_val.get("musicShelfRenderer") {
            if let Some(parsed_sec) = parse_music_shelf(shelf) {
                sections.push(parsed_sec);
            }
        }
    }

    // Check fallback continuations array if not found in contents
    if continuation_token.is_none() {
        if let Some(conts) = sec_list.and_then(|sl| sl.get("continuations")).and_then(|v| v.as_array()) {
            for cont in conts {
                if let Some(token) = cont.pointer("/nextContinuationData/continuation").and_then(|v| v.as_str()) {
                    continuation_token = Some(token.to_string());
                    break;
                }
            }
        }
    }

    Ok(BrowseFeed {
        title: "Home".to_string(),
        chips,
        sections,
        continuation_token,
    })
}

/// Parses continuation responses into additional `BrowseSection`s and next continuation token.
pub fn parse_continuation_sections(value: &Value) -> (Vec<BrowseSection>, Option<String>) {
    let mut sections = Vec::new();
    let mut next_token = None;

    let items = value
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
        .or_else(|| value.pointer("/continuationContents/sectionListContinuation/contents"))
        .and_then(|v| v.as_array());

    if let Some(items) = items {
        for section_val in items {
            if let Some(shelf) = section_val.get("musicCarouselShelfRenderer") {
                if let Some(parsed_sec) = parse_carousel_shelf(shelf) {
                    sections.push(parsed_sec);
                }
            } else if let Some(shelf) = section_val.get("musicImmersiveCarouselShelfRenderer") {
                if let Some(parsed_sec) = parse_carousel_shelf(shelf) {
                    sections.push(parsed_sec);
                }
            } else if let Some(shelf) = section_val.get("musicShelfRenderer") {
                if let Some(parsed_sec) = parse_music_shelf(shelf) {
                    sections.push(parsed_sec);
                }
            } else if let Some(cont) = section_val.get("continuationItemRenderer") {
                if let Some(tok) = cont.pointer("/continuationEndpoint/continuationCommand/token").and_then(|v| v.as_str()) {
                    next_token = Some(tok.to_string());
                }
            }
        }
    }

    // Fallback continuation check
    if next_token.is_none() {
        if let Some(conts) = value.pointer("/continuationContents/sectionListContinuation/continuations").and_then(|v| v.as_array()) {
            for cont in conts {
                if let Some(tok) = cont.pointer("/nextContinuationData/continuation").and_then(|v| v.as_str()) {
                    next_token = Some(tok.to_string());
                    break;
                }
            }
        }
    }

    (sections, next_token)
}

fn parse_carousel_shelf(shelf: &Value) -> Option<BrowseSection> {
    let header = shelf.get("header")?;
    let basic_header = header
        .get("musicCarouselShelfBasicHeaderRenderer")
        .or_else(|| header.get("musicImmersiveCarouselShelfBasicHeaderRenderer"))?;

    let title = basic_header
        .get("title")
        .and_then(extract_text)
        .unwrap_or_else(|| "Featured".into());

    let subtitle = basic_header.get("strapline").and_then(extract_text);

    let raw_items = shelf.get("contents").and_then(|v| v.as_array())?;
    let mut items = Vec::new();

    for item_val in raw_items {
        if let Some(two_row) = item_val.get("musicTwoRowItemRenderer") {
            if let Some(item) = parse_two_row_item(two_row) {
                items.push(item);
            }
        } else if let Some(responsive) = item_val.get("musicResponsiveListItemRenderer") {
            if let Some(item) = parse_responsive_list_item(responsive) {
                items.push(item);
            }
        }
    }

    if items.is_empty() {
        None
    } else {
        Some(BrowseSection {
            title,
            subtitle,
            items,
        })
    }
}

fn parse_music_shelf(shelf: &Value) -> Option<BrowseSection> {
    let title = shelf
        .get("title")
        .and_then(extract_text)
        .unwrap_or_else(|| "Tracks".into());

    let raw_items = shelf.get("contents").and_then(|v| v.as_array())?;
    let mut items = Vec::new();

    for item_val in raw_items {
        if let Some(responsive) = item_val.get("musicResponsiveListItemRenderer") {
            if let Some(item) = parse_responsive_list_item(responsive) {
                items.push(item);
            }
        }
    }

    if items.is_empty() {
        None
    } else {
        Some(BrowseSection {
            title,
            subtitle: None,
            items,
        })
    }
}

fn parse_two_row_item(renderer: &Value) -> Option<ShelfItem> {
    let title = renderer.get("title").and_then(extract_text)?;
    let subtitle = renderer.get("subtitle").and_then(extract_text).unwrap_or_default();

    let thumbnail_url = renderer
        .pointer("/thumbnailRenderer/musicThumbnailRenderer/thumbnail")
        .and_then(extract_thumbnail);

    let nav = renderer.get("navigationEndpoint")?;
    let (id, endpoint_id, item_type) = if let Some(watch) = nav.get("watchEndpoint") {
        let vid = watch.get("videoId").and_then(|v| v.as_str())?.to_string();
        (vid.clone(), vid, ShelfItemType::Track)
    } else if let Some(browse) = nav.get("browseEndpoint") {
        let bid = browse.get("browseId").and_then(|v| v.as_str())?.to_string();
        let page_type = browse
            .pointer("/browseEndpointContextSupportedConfigs/browseEndpointContextMusicConfig/pageType")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let itype = match page_type {
            "MUSIC_PAGE_TYPE_ALBUM" => ShelfItemType::Album,
            "MUSIC_PAGE_TYPE_ARTIST" => ShelfItemType::Artist,
            "MUSIC_PAGE_TYPE_PLAYLIST" => ShelfItemType::Playlist,
            _ => {
                if bid.starts_with("MPRE") {
                    ShelfItemType::Album
                } else if bid.starts_with("VL") || bid.starts_with("RD") {
                    ShelfItemType::Playlist
                } else {
                    ShelfItemType::Album
                }
            }
        };
        (bid.clone(), bid, itype)
    } else {
        return None;
    };

    Some(ShelfItem {
        id,
        title,
        subtitle,
        thumbnail_url,
        item_type,
        endpoint_id,
        duration_secs: None,
        is_explicit: false,
    })
}

fn parse_responsive_list_item(renderer: &Value) -> Option<ShelfItem> {
    let flex_cols = renderer.get("flexColumns").and_then(|v| v.as_array())?;

    let title_col = flex_cols.first()?.pointer("/musicResponsiveListItemFlexColumnRenderer/text")?;
    let title = extract_text(title_col)?;

    let subtitle = flex_cols
        .get(1)
        .and_then(|c| c.pointer("/musicResponsiveListItemFlexColumnRenderer/text"))
        .and_then(extract_text)
        .unwrap_or_default();

    let thumbnail_url = renderer
        .pointer("/thumbnail/musicThumbnailRenderer/thumbnail")
        .and_then(extract_thumbnail);

    let video_id = renderer
        .pointer("/playlistItemData/videoId")
        .and_then(|v| v.as_str())
        .or_else(|| {
            renderer
                .pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/navigationEndpoint/watchEndpoint/videoId")
                .and_then(|v| v.as_str())
        })?
        .to_string();

    Some(ShelfItem {
        id: video_id.clone(),
        title,
        subtitle,
        thumbnail_url,
        item_type: ShelfItemType::Track,
        endpoint_id: video_id,
        duration_secs: None,
        is_explicit: false,
    })
}

/// Parses the `/youtubei/v1/next` payload into a `QueueSnapshot` and extracts any lyrics `browseId`.
pub fn parse_queue(value: &Value) -> Result<(QueueSnapshot, Option<String>), SourceError> {
    let tabs = value
        .pointer("/contents/singleColumnMusicWatchNextResultsRenderer/tabbedRenderer/watchNextTabbedResultsRenderer/tabs")
        .and_then(|v| v.as_array())
        .ok_or_else(|| SourceError::ParseError("Missing tabs in watch next results".into()))?;

    let mut lyrics_browse_id: Option<String> = None;
    let mut upcoming = Vec::new();
    let mut current_track: Option<QueueTrackItem> = None;
    let mut source_playlist_title: Option<String> = None;
    let mut queue_continuation: Option<String> = None;

    for tab in tabs {
        let tab_renderer = match tab.get("tabRenderer") {
            Some(t) => t,
            None => continue,
        };

        // Check if this tab is the Lyrics tab
        if let Some(bid) = tab_renderer.pointer("/endpoint/browseEndpoint/browseId").and_then(|v| v.as_str()) {
            if bid.starts_with("MPLY") {
                lyrics_browse_id = Some(bid.to_string());
            }
        }

        // Queue tab
        if let Some(panel) = tab_renderer.pointer("/content/musicQueueRenderer/content/playlistPanelRenderer") {
            if let Some(t) = panel.get("title").and_then(|v| v.as_str()) {
                source_playlist_title = Some(t.to_string());
            }

            if let Some(contents) = panel.get("contents").and_then(|v| v.as_array()) {
                for item_val in contents {
                    if let Some(video_renderer) = item_val.get("playlistPanelVideoRenderer") {
                        let video_id = match video_renderer.get("videoId").and_then(|v| v.as_str()) {
                            Some(id) => id.to_string(),
                            None => continue,
                        };

                        let title = video_renderer
                            .get("title")
                            .and_then(extract_text)
                            .unwrap_or_else(|| "Unknown Title".into());

                        let artist = video_renderer
                            .get("shortBylineText")
                            .or_else(|| video_renderer.get("longBylineText"))
                            .and_then(extract_text)
                            .unwrap_or_else(|| "Unknown Artist".into());

                        let duration_str = video_renderer
                            .get("lengthText")
                            .and_then(extract_text)
                            .unwrap_or_else(|| "3:00".into());

                        let thumbnail_url = video_renderer
                            .get("thumbnail")
                            .and_then(extract_thumbnail);

                        let is_active = video_renderer
                            .get("selected")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);

                        let track_item = QueueTrackItem {
                            video_id,
                            title,
                            artist,
                            duration_str,
                            thumbnail_url,
                            is_active,
                        };

                        if is_active {
                            current_track = Some(track_item.clone());
                        } else {
                            upcoming.push(track_item);
                        }
                    }
                }
            }

            if let Some(conts) = panel.get("continuations").and_then(|v| v.as_array()) {
                for c in conts {
                    if let Some(tok) = c.pointer("/nextContinuationData/continuation")
                        .or_else(|| c.pointer("/nextRadioContinuationData/continuation"))
                        .and_then(|v| v.as_str())
                    {
                        queue_continuation = Some(tok.to_string());
                        break;
                    }
                }
            }
        }
    }

    Ok((
        QueueSnapshot {
            current_track,
            upcoming,
            history: Vec::new(),
            autoplay_enabled: true,
            source_playlist_title,
            continuation: queue_continuation,
        },
        lyrics_browse_id,
    ))
}

/// Parses the `/youtubei/v1/next` continuation response into a batch of tracks and the next continuation token.
pub fn parse_queue_continuation(body: &Value) -> (Vec<rustic_core::api::Track>, Option<String>) {
    use rustic_core::api::{Track, TrackSource};
    let mut tracks = Vec::new();
    let mut continuation_token = None;

    let panel = body
        .pointer("/continuationContents/playlistPanelContinuation")
        .or_else(|| body.pointer("/continuationContents/musicQueueContinuation"));

    if let Some(panel) = panel {
        if let Some(contents) = panel.get("contents").and_then(|v| v.as_array()) {
            for item in contents {
                if let Some(video_renderer) = item.get("playlistPanelVideoRenderer") {
                    let video_id = match video_renderer.get("videoId").and_then(|v| v.as_str()) {
                        Some(id) => id.to_string(),
                        None => continue,
                    };
                    let title = video_renderer
                        .get("title")
                        .and_then(extract_text)
                        .unwrap_or_else(|| "Unknown Title".into());
                    let artist = video_renderer
                        .get("shortBylineText")
                        .or_else(|| video_renderer.get("longBylineText"))
                        .and_then(extract_text)
                        .unwrap_or_else(|| "Unknown Artist".into());
                    let duration_str = video_renderer
                        .get("lengthText")
                        .and_then(extract_text)
                        .unwrap_or_else(|| "3:00".into());

                    let mut duration_secs = 180;
                    let parts: Vec<&str> = duration_str.split(':').collect();
                    if parts.len() == 2 {
                        if let (Ok(m), Ok(s)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                            duration_secs = m * 60 + s;
                        }
                    } else if parts.len() == 3 {
                        if let (Ok(h), Ok(m), Ok(s)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>()) {
                            duration_secs = h * 3600 + m * 60 + s;
                        }
                    }

                    let artwork_url = video_renderer.get("thumbnail").and_then(extract_thumbnail);

                    tracks.push(Track {
                        id: video_id.clone(),
                        title,
                        artists: vec![artist],
                        album: "YouTube Music".into(),
                        duration_secs,
                        source: TrackSource::YouTubeMusic { video_id },
                        isrc: None,
                        artwork_url,
                        content_hash: None,
                    });
                }
            }
        }

        if let Some(conts) = panel.get("continuations").and_then(|v| v.as_array()) {
            for c in conts {
                if let Some(tok) = c.pointer("/nextContinuationData/continuation")
                    .or_else(|| c.pointer("/nextRadioContinuationData/continuation"))
                    .and_then(|v| v.as_str())
                {
                    continuation_token = Some(tok.to_string());
                    break;
                }
            }
        }
    }

    (tracks, continuation_token)
}

/// Parses the `/youtubei/v1/browse` response for a lyrics browseId (`MPLYt_...`).
pub fn parse_lyrics(value: &Value) -> Result<LyricsDocument, SourceError> {
    if let Some(msg_renderer) = value.pointer("/contents/messageRenderer") {
        let msg = msg_renderer
            .get("text")
            .and_then(extract_text)
            .unwrap_or_else(|| "Lyrics not available".into());
        return Err(SourceError::Unavailable(msg));
    }

    let shelf = value
        .pointer("/contents/sectionListRenderer/contents/0/musicDescriptionShelfRenderer")
        .ok_or_else(|| SourceError::ParseError("Missing musicDescriptionShelfRenderer for lyrics".into()))?;

    let plain_text = shelf.get("description").and_then(extract_text);
    let source = shelf.get("footer").and_then(extract_text);

    let mut lines = Vec::new();
    if let Some(ref text) = plain_text {
        for line_str in text.lines() {
            let trimmed = line_str.trim();
            if !trimmed.is_empty() {
                lines.push(LyricsLine {
                    text: trimmed.to_string(),
                    start_time_ms: None,
                });
            }
        }
    }

    Ok(LyricsDocument {
        plain_text,
        lines,
        source,
    })
}

/// Parses YouTube Music search response into structured `Vec<Track>`.
pub fn parse_search_results(body: &Value) -> Result<Vec<rustic_core::api::Track>, SourceError> {
    use rustic_core::api::{Track, TrackSource};
    let mut tracks = Vec::new();

    let sections = body
        .pointer("/contents/tabbedSearchResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
        .and_then(|v| v.as_array());

    if let Some(sections) = sections {
        for section in sections {
            // 1. Check musicCardShelfRenderer (Top Result / Hero Card)
            if let Some(card) = section.get("musicCardShelfRenderer") {
                let video_id = card.pointer("/onTap/watchEndpoint/videoId")
                    .or_else(|| card.pointer("/title/runs/0/navigationEndpoint/watchEndpoint/videoId"))
                    .or_else(|| card.pointer("/buttons/0/buttonRenderer/navigationEndpoint/watchEndpoint/videoId"))
                    .and_then(|v| v.as_str());

                if let Some(vid) = video_id {
                    let title = card.pointer("/title/runs/0/text").and_then(|v| v.as_str()).unwrap_or("Top Result");
                    let artist = card.pointer("/subtitle/runs/0/text").and_then(|v| v.as_str()).unwrap_or("YouTube Artist");
                    let thumb = card.pointer("/thumbnail/musicThumbnailRenderer/thumbnail/thumbnails/0/url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    tracks.push(Track {
                        id: vid.to_string(),
                        title: title.to_string(),
                        artists: vec![artist.to_string()],
                        album: "YouTube Music".into(),
                        duration_secs: 180,
                        source: TrackSource::YouTubeMusic { video_id: vid.to_string() },
                        isrc: None,
                        artwork_url: thumb,
                        content_hash: None,
                    });
                }
            }

            // 2. Direct musicShelfRenderer
            if let Some(items) = section.pointer("/musicShelfRenderer/contents").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(t) = parse_music_responsive_list_item(item) {
                        if !tracks.iter().any(|existing| existing.id == t.id) {
                            tracks.push(t);
                        }
                    }
                }
            }

            // 3. itemSectionRenderer / contents (all items or nested shelves)
            if let Some(items) = section.pointer("/itemSectionRenderer/contents").and_then(|v| v.as_array()) {
                for item in items {
                    if let Some(t) = parse_music_responsive_list_item(item) {
                        if !tracks.iter().any(|existing| existing.id == t.id) {
                            tracks.push(t);
                        }
                    } else if let Some(nested_shelf) = item.pointer("/musicShelfRenderer/contents").and_then(|v| v.as_array()) {
                        for nested in nested_shelf {
                            if let Some(t) = parse_music_responsive_list_item(nested) {
                                if !tracks.iter().any(|existing| existing.id == t.id) {
                                    tracks.push(t);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(tracks)
}

fn parse_music_responsive_list_item(item: &Value) -> Option<rustic_core::api::Track> {
    use rustic_core::api::{Track, TrackSource};
    let renderer = item.get("musicResponsiveListItemRenderer")?;

    let video_id = renderer
        .pointer("/playlistItemData/videoId")
        .or_else(|| renderer.pointer("/overlay/musicItemThumbnailOverlayRenderer/content/musicPlayButtonRenderer/playNavigationEndpoint/watchEndpoint/videoId"))
        .or_else(|| renderer.pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/navigationEndpoint/watchEndpoint/videoId"))
        .or_else(|| renderer.pointer("/navigationEndpoint/watchEndpoint/videoId"))
        .or_else(|| renderer.pointer("/onTap/watchEndpoint/videoId"))
        .and_then(|v| v.as_str())?;

    let title = renderer
        .pointer("/flexColumns/0/musicResponsiveListItemFlexColumnRenderer/text/runs/0/text")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Title");

    let artist = renderer
        .pointer("/flexColumns/1/musicResponsiveListItemFlexColumnRenderer/text/runs/0/text")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Artist");

    let mut duration_secs = 180;
    if let Some(cols) = renderer.get("flexColumns").and_then(|v| v.as_array()) {
        for col in cols {
            if let Some(runs) = col.pointer("/musicResponsiveListItemFlexColumnRenderer/text/runs").and_then(|v| v.as_array()) {
                for r in runs {
                    if let Some(txt) = r.get("text").and_then(|v| v.as_str()) {
                        let parts: Vec<&str> = txt.split(':').collect();
                        if parts.len() == 2 {
                            if let (Ok(m), Ok(s)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                                duration_secs = m * 60 + s;
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(cols) = renderer.get("fixedColumns").and_then(|v| v.as_array()) {
        for col in cols {
            if let Some(runs) = col.pointer("/musicResponsiveListItemFixedColumnRenderer/text/runs").and_then(|v| v.as_array()) {
                for r in runs {
                    if let Some(txt) = r.get("text").and_then(|v| v.as_str()) {
                        let parts: Vec<&str> = txt.split(':').collect();
                        if parts.len() == 2 {
                            if let (Ok(m), Ok(s)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                                duration_secs = m * 60 + s;
                            }
                        }
                    }
                }
            }
        }
    }

    let thumb = renderer
        .pointer("/thumbnail/musicThumbnailRenderer/thumbnail")
        .and_then(extract_thumbnail);

    Some(Track {
        id: video_id.to_string(),
        title: title.to_string(),
        artists: vec![artist.to_string()],
        album: "YouTube Music".into(),
        duration_secs,
        source: TrackSource::YouTubeMusic { video_id: video_id.to_string() },
        isrc: None,
        artwork_url: thumb,
        content_hash: None,
    })
}

/// Parses YouTube Music search suggestions response into structured list of query strings.
pub fn parse_search_suggestions(body: &Value) -> Result<Vec<String>, SourceError> {
    let mut suggestions = Vec::new();
    if let Some(contents) = body.pointer("/contents/0/searchSuggestionsSectionRenderer/contents").and_then(|v| v.as_array()) {
        for item in contents {
            if let Some(sug) = item.get("searchSuggestionRenderer") {
                if let Some(runs) = sug.pointer("/suggestion/runs").and_then(|v| v.as_array()) {
                    let text: String = runs.iter().filter_map(|r| r.get("text").and_then(|t| t.as_str())).collect();
                    if !text.is_empty() && !suggestions.contains(&text) {
                        suggestions.push(text);
                    }
                }
            } else if let Some(hist) = item.get("historySuggestionRenderer") {
                if let Some(runs) = hist.pointer("/suggestion/runs").and_then(|v| v.as_array()) {
                    let text: String = runs.iter().filter_map(|r| r.get("text").and_then(|t| t.as_str())).collect();
                    if !text.is_empty() && !suggestions.contains(&text) {
                        suggestions.push(text);
                    }
                }
            }
        }
    }
    Ok(suggestions)
}

/// Parses user playlists from FEmusic_liked_playlists.
pub fn parse_library_playlists(body: &Value) -> Result<Vec<LibraryItem>, SourceError> {
    let mut playlists = Vec::new();
    let items = body
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/gridRenderer/items")
        .and_then(|v| v.as_array());

    if let Some(items) = items {
        for it in items {
            if let Some(two_row) = it.get("musicTwoRowItemRenderer") {
                let pid = two_row
                    .pointer("/navigationEndpoint/browseEndpoint/browseId")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if pid.is_empty() {
                    continue; // Skip "New playlist" placeholder
                }
                let title = two_row.get("title").and_then(extract_text).unwrap_or_default();
                let subtitle = two_row.get("subtitle").and_then(extract_text).unwrap_or_default();
                let thumbnail_url = two_row
                    .pointer("/thumbnailRenderer/musicThumbnailRenderer/thumbnail")
                    .and_then(extract_thumbnail);

                let track_count = subtitle
                    .split('•')
                    .find_map(|part| {
                        let trimmed = part.trim();
                        if trimmed.contains("track") || trimmed.contains("song") {
                            trimmed.split_whitespace().next().and_then(|n| n.parse::<usize>().ok())
                        } else {
                            None
                        }
                    });

                playlists.push(LibraryItem {
                    id: pid.to_string(),
                    title,
                    subtitle,
                    item_type: LibraryItemType::Playlist,
                    thumbnail_url,
                    track_count,
                });
            }
        }
    }

    Ok(playlists)
}

/// Parses user albums from FEmusic_liked_albums.
pub fn parse_library_albums(body: &Value) -> Result<Vec<LibraryItem>, SourceError> {
    let mut albums = Vec::new();
    let items = body
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/gridRenderer/items")
        .and_then(|v| v.as_array());

    if let Some(items) = items {
        for it in items {
            if let Some(two_row) = it.get("musicTwoRowItemRenderer") {
                let aid = two_row
                    .pointer("/navigationEndpoint/browseEndpoint/browseId")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if aid.is_empty() {
                    continue;
                }
                let title = two_row.get("title").and_then(extract_text).unwrap_or_default();
                let subtitle = two_row.get("subtitle").and_then(extract_text).unwrap_or_default();
                let thumbnail_url = two_row
                    .pointer("/thumbnailRenderer/musicThumbnailRenderer/thumbnail")
                    .and_then(extract_thumbnail);

                albums.push(LibraryItem {
                    id: aid.to_string(),
                    title,
                    subtitle,
                    item_type: LibraryItemType::Album,
                    thumbnail_url,
                    track_count: None,
                });
            }
        }
    }

    Ok(albums)
}

/// Parses user liked songs from FEmusic_liked_videos, returning tracks and optional continuation token.
pub fn parse_library_songs(body: &Value) -> Result<(Vec<rustic_core::api::Track>, Option<String>), SourceError> {
    let mut tracks: Vec<rustic_core::api::Track> = Vec::new();
    let mut continuation_token: Option<String> = None;

    let shelf = body
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/musicShelfRenderer")
        .or_else(|| body.pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/musicPlaylistShelfRenderer"));

    let items = shelf
        .and_then(|s| s.get("contents"))
        .and_then(|v| v.as_array());

    if let Some(items) = items {
        for item in items {
            if let Some(cont) = item.get("continuationItemRenderer") {
                if let Some(token) = cont.pointer("/continuationEndpoint/continuationCommand/token").and_then(|v| v.as_str()) {
                    continuation_token = Some(token.to_string());
                }
            } else if let Some(t) = parse_music_responsive_list_item(item) {
                if !tracks.iter().any(|existing| existing.id == t.id) {
                    tracks.push(t);
                }
            }
        }
    }

    if continuation_token.is_none() {
        if let Some(shelf) = shelf {
            if let Some(tok) = shelf.pointer("/continuations/0/nextContinuationData/continuation")
                .or_else(|| shelf.pointer("/continuations/0/reloadContinuationData/continuation"))
                .and_then(|v| v.as_str()) {
                continuation_token = Some(tok.to_string());
            }
        }
    }

    Ok((tracks, continuation_token))
}

/// Parses continuation responses for track lists (e.g. liked songs pagination).
pub fn parse_continuation_tracks(body: &Value) -> (Vec<rustic_core::api::Track>, Option<String>) {
    let mut tracks: Vec<rustic_core::api::Track> = Vec::new();
    let mut continuation_token = None;

    let contents = body
        .pointer("/continuationContents/musicPlaylistShelfContinuation/contents")
        .or_else(|| body.pointer("/continuationContents/musicShelfContinuation/contents"))
        .or_else(|| body.pointer("/continuationContents/sectionListContinuation/contents"))
        .and_then(|v| v.as_array());

    if let Some(items) = contents {
        for item in items {
            if let Some(cont) = item.get("continuationItemRenderer") {
                if let Some(tok) = cont.pointer("/continuationEndpoint/continuationCommand/token").and_then(|v| v.as_str()) {
                    continuation_token = Some(tok.to_string());
                }
            } else if let Some(t) = parse_music_responsive_list_item(item) {
                if !tracks.iter().any(|existing| existing.id == t.id) {
                    tracks.push(t);
                }
            }
        }
    }

    if continuation_token.is_none() {
        let cont_tok = body
            .pointer("/continuationContents/musicPlaylistShelfContinuation/continuations/0/nextContinuationData/continuation")
            .or_else(|| body.pointer("/continuationContents/musicShelfContinuation/continuations/0/nextContinuationData/continuation"))
            .or_else(|| body.pointer("/continuationContents/sectionListContinuation/continuations/0/nextContinuationData/continuation"))
            .and_then(|v| v.as_str());
        if let Some(tok) = cont_tok {
            continuation_token = Some(tok.to_string());
        }
    }

    (tracks, continuation_token)
}

/// Parses an album or playlist browse page into `YtCollectionDetails`.
pub fn parse_collection_browse(body: &Value, id: &str) -> Result<YtCollectionDetails, SourceError> {
    let header = body.pointer("/header/musicDetailHeaderRenderer")
        .or_else(|| body.pointer("/header/musicResponsiveHeaderRenderer"));

    let title = header
        .and_then(|h| h.get("title"))
        .and_then(extract_text)
        .or_else(|| body.pointer("/contents/twoColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/musicResponsiveHeaderRenderer/title").and_then(extract_text))
        .unwrap_or_else(|| "Unknown Collection".into());

    let subtitle = header
        .and_then(|h| h.get("subtitle"))
        .and_then(extract_text)
        .or_else(|| header.and_then(|h| h.get("straplineTextOne")).and_then(extract_text))
        .unwrap_or_default();

    let second_sub = header
        .and_then(|h| h.get("secondSubtitle"))
        .and_then(extract_text);

    let full_subtitle = if let Some(sec) = second_sub {
        if subtitle.is_empty() {
            sec
        } else {
            format!("{} • {}", subtitle, sec)
        }
    } else {
        subtitle
    };

    let description = header
        .and_then(|h| h.get("description"))
        .and_then(extract_text);

    let thumbnail_url = header
        .and_then(|h| {
            h.pointer("/thumbnail/croppedSquareThumbnailRenderer/thumbnail")
                .or_else(|| h.pointer("/thumbnail/musicThumbnailRenderer/thumbnail"))
                .or_else(|| h.get("thumbnail"))
        })
        .and_then(extract_thumbnail);

    let mut tracks = Vec::new();

    // Check shelf contents across standard YTM layouts
    let shelf_contents = body
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/musicShelfRenderer/contents")
        .or_else(|| body.pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/musicPlaylistShelfRenderer/contents"))
        .or_else(|| body.pointer("/contents/twoColumnBrowseResultsRenderer/secondaryContents/sectionListRenderer/contents/0/musicPlaylistShelfRenderer/contents"))
        .or_else(|| body.pointer("/contents/twoColumnBrowseResultsRenderer/secondaryContents/sectionListRenderer/contents/0/musicShelfRenderer/contents"))
        .and_then(|v| v.as_array());

    if let Some(items) = shelf_contents {
        for item in items {
            if let Some(mut t) = parse_music_responsive_list_item(item) {
                if t.album.is_empty() || t.album == "YouTube Music" {
                    t.album = title.clone();
                }
                if !tracks.iter().any(|existing: &rustic_core::api::Track| existing.id == t.id) {
                    tracks.push(t);
                }
            }
        }
    }

    let track_count = if tracks.is_empty() { None } else { Some(tracks.len()) };

    Ok(YtCollectionDetails {
        id: id.to_string(),
        title,
        subtitle: full_subtitle,
        description,
        thumbnail_url,
        track_count,
        tracks,
    })
}

/// Parses an artist browse response (`channel_id` / `UC...`) into `YtArtistDetails`.
pub fn parse_artist_details(body: &Value, channel_id: &str) -> Result<YtArtistDetails, SourceError> {
    let header = body.pointer("/header/musicVisualHeaderRenderer")
        .or_else(|| body.pointer("/header/musicHeaderRenderer"))
        .or_else(|| body.pointer("/header/musicImmersiveHeaderRenderer"));

    let name = header
        .and_then(|h| h.get("title"))
        .and_then(extract_text)
        .or_else(|| body.pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/musicResponsiveHeaderRenderer/title").and_then(extract_text))
        .unwrap_or_else(|| "Artist".into());

    let description = header
        .and_then(|h| h.get("description"))
        .and_then(extract_text);

    let subscribers = header
        .and_then(|h| {
            h.pointer("/subscriptionButton/subscribeButtonRenderer/subscriberCountText")
                .and_then(extract_text)
                .or_else(|| h.get("subtitle").and_then(extract_text))
        });

    let avatar_url = header
        .and_then(|h| {
            h.pointer("/thumbnail/musicThumbnailRenderer/thumbnail")
                .or_else(|| h.pointer("/foregroundThumbnail/musicThumbnailRenderer/thumbnail"))
                .or_else(|| h.get("thumbnail"))
        })
        .and_then(extract_thumbnail);

    let banner_url = header
        .and_then(|h| h.pointer("/banner/musicThumbnailRenderer/thumbnail"))
        .and_then(extract_thumbnail);

    let mut top_songs = Vec::new();
    let mut albums = Vec::new();
    let mut singles = Vec::new();
    let mut related_artists = Vec::new();

    let sections = body
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents")
        .or_else(|| body.pointer("/contents/twoColumnBrowseResultsRenderer/secondaryContents/sectionListRenderer/contents"))
        .and_then(|v| v.as_array());

    if let Some(sections) = sections {
        for section in sections {
            // 1. Top Songs shelf
            if let Some(shelf) = section.get("musicShelfRenderer") {
                if let Some(contents) = shelf.get("contents").and_then(|v| v.as_array()) {
                    for item in contents {
                        if let Some(mut track) = parse_music_responsive_list_item(item) {
                            if track.artists.is_empty() || track.artists[0] == "Unknown Artist" {
                                track.artists = vec![name.clone()];
                            }
                            if !top_songs.iter().any(|t: &rustic_core::api::Track| t.id == track.id) {
                                top_songs.push(track);
                            }
                        }
                    }
                }
            }

            // 2. Carousels (Albums, Singles, Related Artists)
            if let Some(carousel) = section.get("musicCarouselShelfRenderer") {
                let carousel_title = carousel
                    .pointer("/header/musicCarouselShelfBasicHeaderRenderer/title")
                    .and_then(extract_text)
                    .unwrap_or_default()
                    .to_lowercase();

                if let Some(contents) = carousel.get("contents").and_then(|v| v.as_array()) {
                    for item in contents {
                        if let Some(two_row) = item.get("musicTwoRowItemRenderer") {
                            let browse_id = two_row
                                .pointer("/navigationEndpoint/browseEndpoint/browseId")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default();
                            let title = two_row.get("title").and_then(extract_text).unwrap_or_default();
                            let subtitle = two_row.get("subtitle").and_then(extract_text).unwrap_or_default();
                            let thumb = two_row
                                .pointer("/thumbnailRenderer/musicThumbnailRenderer/thumbnail")
                                .and_then(extract_thumbnail);

                            if browse_id.is_empty() {
                                continue;
                            }

                            if carousel_title.contains("fans") || carousel_title.contains("similar") || carousel_title.contains("related") || browse_id.starts_with("UC") {
                                related_artists.push(YtRelatedArtist {
                                    channel_id: browse_id.to_string(),
                                    name: title,
                                    subscribers: if subtitle.is_empty() { None } else { Some(subtitle) },
                                    thumbnail_url: thumb,
                                });
                            } else if carousel_title.contains("single") || carousel_title.contains("ep") || subtitle.to_lowercase().contains("single") || subtitle.to_lowercase().contains("ep") {
                                singles.push(YtAlbumItem {
                                    browse_id: browse_id.to_string(),
                                    title,
                                    year: extract_year_from_sub(&subtitle),
                                    thumbnail_url: thumb,
                                    explicit: false,
                                });
                            } else {
                                albums.push(YtAlbumItem {
                                    browse_id: browse_id.to_string(),
                                    title,
                                    year: extract_year_from_sub(&subtitle),
                                    thumbnail_url: thumb,
                                    explicit: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(YtArtistDetails {
        id: channel_id.to_string(),
        name,
        subscribers,
        description,
        avatar_url,
        banner_url,
        top_songs,
        albums,
        singles,
        related_artists,
    })
}

fn extract_year_from_sub(subtitle: &str) -> Option<String> {
    for part in subtitle.split('•') {
        let trimmed = part.trim();
        if trimmed.len() == 4 && trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Parses followed / library artists from FEmusic_library_corpus_artists.
pub fn parse_library_artists(body: &Value) -> Result<Vec<LibraryArtist>, SourceError> {
    let mut artists = Vec::new();
    let items = body
        .pointer("/contents/singleColumnBrowseResultsRenderer/tabs/0/tabRenderer/content/sectionListRenderer/contents/0/musicShelfRenderer/contents")
        .and_then(|v| v.as_array());

    if let Some(items) = items {
        for item in items {
            if let Some(ren) = item.get("musicResponsiveListItemRenderer") {
                let arid = ren
                    .pointer("/navigationEndpoint/browseEndpoint/browseId")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if arid.is_empty() {
                    continue;
                }
                let fcs = ren.get("flexColumns").and_then(|v| v.as_array());
                let name = fcs
                    .and_then(|cols| cols.first())
                    .and_then(|c| c.pointer("/musicResponsiveListItemFlexColumnRenderer/text"))
                    .and_then(extract_text)
                    .unwrap_or_default();
                let subscribers = fcs
                    .and_then(|cols| cols.get(1))
                    .and_then(|c| c.pointer("/musicResponsiveListItemFlexColumnRenderer/text"))
                    .and_then(extract_text);
                let thumbnail_url = ren
                    .pointer("/thumbnail/musicThumbnailRenderer/thumbnail")
                    .and_then(extract_thumbnail);

                artists.push(LibraryArtist {
                    id: arid.to_string(),
                    name,
                    subscribers,
                    thumbnail_url,
                });
            }
        }
    }

    Ok(artists)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_text_both_formats() {
        let runs_val = json!({ "runs": [{ "text": "Hello " }, { "text": "World" }] });
        assert_eq!(extract_text(&runs_val), Some("Hello World".into()));

        let simple_val = json!({ "simpleText": "Simple String" });
        assert_eq!(extract_text(&simple_val), Some("Simple String".into()));
    }

    #[test]
    fn test_parse_browse_feed_minimal() {
        let mock_feed = json!({
            "contents": {
                "singleColumnBrowseResultsRenderer": {
                    "tabs": [{
                        "tabRenderer": {
                            "content": {
                                "sectionListRenderer": {
                                    "contents": [{
                                        "musicCarouselShelfRenderer": {
                                            "header": {
                                                "musicCarouselShelfBasicHeaderRenderer": {
                                                    "title": { "runs": [{ "text": "Listen again" }] },
                                                    "strapline": { "runs": [{ "text": "KETAN" }] }
                                                }
                                            },
                                            "contents": [{
                                                "musicTwoRowItemRenderer": {
                                                    "title": { "runs": [{ "text": "folklore" }] },
                                                    "subtitle": { "runs": [{ "text": "Taylor Swift" }] },
                                                    "navigationEndpoint": {
                                                        "browseEndpoint": {
                                                            "browseId": "MPREb_123",
                                                            "browseEndpointContextSupportedConfigs": {
                                                                "browseEndpointContextMusicConfig": {
                                                                    "pageType": "MUSIC_PAGE_TYPE_ALBUM"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }]
                                        }
                                    }]
                                }
                            }
                        }
                    }]
                }
            }
        });

        let feed = parse_browse_feed(&mock_feed).expect("Should parse mock feed");
        assert_eq!(feed.sections.len(), 1);
        let sec = &feed.sections[0];
        assert_eq!(sec.title, "Listen again");
        assert_eq!(sec.subtitle.as_deref(), Some("KETAN"));
        assert_eq!(sec.items.len(), 1);
        assert_eq!(sec.items[0].title, "folklore");
        assert_eq!(sec.items[0].item_type, ShelfItemType::Album);
    }

    #[test]
    fn test_parse_search_results_with_item_sections() {
        let mock_search = json!({
            "contents": {
                "tabbedSearchResultsRenderer": {
                    "tabs": [{
                        "tabRenderer": {
                            "content": {
                                "sectionListRenderer": {
                                    "contents": [
                                        {
                                            "itemSectionRenderer": {
                                                "contents": [
                                                    {
                                                        "musicResponsiveListItemRenderer": {
                                                            "playlistItemData": { "videoId": "vid_abc123" },
                                                            "flexColumns": [
                                                                {
                                                                    "musicResponsiveListItemFlexColumnRenderer": {
                                                                        "text": { "runs": [{ "text": "Yellow" }] }
                                                                    }
                                                                },
                                                                {
                                                                    "musicResponsiveListItemFlexColumnRenderer": {
                                                                        "text": { "runs": [{ "text": "Coldplay" }, { "text": "4:29" }] }
                                                                    }
                                                                }
                                                            ],
                                                            "thumbnail": {
                                                                "musicThumbnailRenderer": {
                                                                    "thumbnail": {
                                                                        "thumbnails": [{ "url": "https://example.com/yellow.jpg" }]
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                ]
                                            }
                                        }
                                    ]
                                }
                            }
                        }
                    }]
                }
            }
        });

        let tracks = parse_search_results(&mock_search).expect("Should parse search results");
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].id, "vid_abc123");
        assert_eq!(tracks[0].title, "Yellow");
        assert_eq!(tracks[0].artists, vec!["Coldplay".to_string()]);
        assert_eq!(tracks[0].duration_secs, 4 * 60 + 29);
        assert_eq!(tracks[0].artwork_url.as_deref(), Some("https://example.com/yellow.jpg"));
    }

    #[test]
    fn test_parse_search_suggestions() {
        let mock_suggestions = json!({
            "contents": [
                {
                    "searchSuggestionsSectionRenderer": {
                        "contents": [
                            {
                                "searchSuggestionRenderer": {
                                    "suggestion": {
                                        "runs": [
                                            { "text": "cold", "bold": true },
                                            { "text": "play" }
                                        ]
                                    }
                                }
                            },
                            {
                                "searchSuggestionRenderer": {
                                    "suggestion": {
                                        "runs": [
                                            { "text": "coldplay yellow" }
                                        ]
                                    }
                                }
                            }
                        ]
                    }
                }
            ]
        });

        let suggestions = parse_search_suggestions(&mock_suggestions).expect("Should parse suggestions");
        assert_eq!(suggestions, vec!["coldplay".to_string(), "coldplay yellow".to_string()]);
    }

    #[test]
    fn test_normalize_thumbnail_url() {
        let u1 = "https://yt3.googleusercontent.com/kIG_Ks5fsPd9dHNttIv9_v=w226-h226-l90-rj";
        assert_eq!(normalize_thumbnail_url(u1), "https://yt3.googleusercontent.com/kIG_Ks5fsPd9dHNttIv9_v=w544-h544-l90-rj");
        let u2 = "https://lh3.googleusercontent.com/test_id=s120-c-k-c0x00ffffff-no-rj";
        assert_eq!(normalize_thumbnail_url(u2), "https://lh3.googleusercontent.com/test_id=w544-h544-l90-rj");
        let u3 = "https://i.ytimg.com/vi/12345/hq720.jpg";
        assert_eq!(normalize_thumbnail_url(u3), "https://i.ytimg.com/vi/12345/hq720.jpg");
    }

    #[test]
    fn test_parse_library_playlists() {
        let mock = json!({
            "contents": {
                "singleColumnBrowseResultsRenderer": {
                    "tabs": [{
                        "tabRenderer": {
                            "content": {
                                "sectionListRenderer": {
                                    "contents": [{
                                        "gridRenderer": {
                                            "items": [
                                                {
                                                    "musicTwoRowItemRenderer": {
                                                        "title": { "runs": [{ "text": "New playlist" }] },
                                                        "navigationEndpoint": { "browseEndpoint": { "browseId": "" } }
                                                    }
                                                },
                                                {
                                                    "musicTwoRowItemRenderer": {
                                                        "title": { "runs": [{ "text": "Slow Chill" }] },
                                                        "subtitle": { "runs": [{ "text": "Ketan • 229 tracks" }] },
                                                        "navigationEndpoint": { "browseEndpoint": { "browseId": "VLPL123" } },
                                                        "thumbnailRenderer": {
                                                            "musicThumbnailRenderer": {
                                                                "thumbnail": {
                                                                    "thumbnails": [{ "url": "https://yt3.googleusercontent.com/thumb=w226-h226-l90-rj" }]
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            ]
                                        }
                                    }]
                                }
                            }
                        }
                    }]
                }
            }
        });

        let playlists = parse_library_playlists(&mock).expect("Should parse playlists");
        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].id, "VLPL123");
        assert_eq!(playlists[0].title, "Slow Chill");
        assert_eq!(playlists[0].track_count, Some(229));
        assert_eq!(playlists[0].thumbnail_url.as_deref(), Some("https://yt3.googleusercontent.com/thumb=w544-h544-l90-rj"));
    }

    #[test]
    fn test_parse_library_artists() {
        let mock = json!({
            "contents": {
                "singleColumnBrowseResultsRenderer": {
                    "tabs": [{
                        "tabRenderer": {
                            "content": {
                                "sectionListRenderer": {
                                    "contents": [{
                                        "musicShelfRenderer": {
                                            "contents": [
                                                {
                                                    "musicResponsiveListItemRenderer": {
                                                        "navigationEndpoint": { "browseEndpoint": { "browseId": "UC123" } },
                                                        "flexColumns": [
                                                            {
                                                                "musicResponsiveListItemFlexColumnRenderer": {
                                                                    "text": { "runs": [{ "text": "Hanumankind" }] }
                                                                }
                                                            },
                                                            {
                                                                "musicResponsiveListItemFlexColumnRenderer": {
                                                                    "text": { "runs": [{ "text": "2.84M subscribers" }] }
                                                                }
                                                            }
                                                        ],
                                                        "thumbnail": {
                                                            "musicThumbnailRenderer": {
                                                                "thumbnail": {
                                                                    "thumbnails": [{ "url": "https://lh3.googleusercontent.com/art=w120-h120" }]
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            ]
                                        }
                                    }]
                                }
                            }
                        }
                    }]
                }
            }
        });

        let artists = parse_library_artists(&mock).expect("Should parse artists");
        assert_eq!(artists.len(), 1);
        assert_eq!(artists[0].id, "UC123");
        assert_eq!(artists[0].name, "Hanumankind");
        assert_eq!(artists[0].subscribers.as_deref(), Some("2.84M subscribers"));
        assert_eq!(artists[0].thumbnail_url.as_deref(), Some("https://lh3.googleusercontent.com/art=w544-h544-l90-rj"));
    }

    #[test]
    fn test_parse_artist_details() {
        let mock = json!({
            "header": {
                "musicVisualHeaderRenderer": {
                    "title": { "runs": [{ "text": "Taylor Swift" }] },
                    "subscriptionButton": {
                        "subscribeButtonRenderer": {
                            "subscriberCountText": { "runs": [{ "text": "58.2M subscribers" }] }
                        }
                    },
                    "description": { "runs": [{ "text": "Taylor Alison Swift is an American singer-songwriter." }] },
                    "thumbnail": {
                        "musicThumbnailRenderer": {
                            "thumbnail": {
                                "thumbnails": [{ "url": "https://lh3.googleusercontent.com/avatar=w300-h300" }]
                            }
                        }
                    }
                }
            },
            "contents": {
                "singleColumnBrowseResultsRenderer": {
                    "tabs": [{
                        "tabRenderer": {
                            "content": {
                                "sectionListRenderer": {
                                    "contents": [
                                        {
                                            "musicShelfRenderer": {
                                                "title": { "runs": [{ "text": "Songs" }] },
                                                "contents": [
                                                    {
                                                        "musicResponsiveListItemRenderer": {
                                                            "playlistItemData": { "videoId": "video123" },
                                                            "flexColumns": [
                                                                {
                                                                    "musicResponsiveListItemFlexColumnRenderer": {
                                                                        "text": { "runs": [{ "text": "Cruel Summer" }] }
                                                                    }
                                                                },
                                                                {
                                                                    "musicResponsiveListItemFlexColumnRenderer": {
                                                                        "text": { "runs": [{ "text": "Taylor Swift" }, { "text": " • " }, { "text": "Lover" }] }
                                                                    }
                                                                }
                                                            ],
                                                            "fixedColumns": [
                                                                {
                                                                    "musicResponsiveListItemFixedColumnRenderer": {
                                                                        "text": { "runs": [{ "text": "2:58" }] }
                                                                    }
                                                                }
                                                            ],
                                                            "thumbnail": {
                                                                "musicThumbnailRenderer": {
                                                                    "thumbnail": {
                                                                        "thumbnails": [{ "url": "https://lh3.googleusercontent.com/song=w120-h120" }]
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                ]
                                            }
                                        },
                                        {
                                            "musicCarouselShelfRenderer": {
                                                "header": {
                                                    "musicCarouselShelfBasicHeaderRenderer": {
                                                        "title": { "runs": [{ "text": "Albums" }] }
                                                    }
                                                },
                                                "contents": [
                                                    {
                                                        "musicTwoRowItemRenderer": {
                                                            "navigationEndpoint": {
                                                                "browseEndpoint": { "browseId": "MPREb_album123" }
                                                            },
                                                            "title": { "runs": [{ "text": "1989 (Taylor's Version)" }] },
                                                            "subtitle": { "runs": [{ "text": "Album • 2023" }] },
                                                            "thumbnailRenderer": {
                                                                "musicThumbnailRenderer": {
                                                                    "thumbnail": {
                                                                        "thumbnails": [{ "url": "https://lh3.googleusercontent.com/alb=w300-h300" }]
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                ]
                                            }
                                        },
                                        {
                                            "musicCarouselShelfRenderer": {
                                                "header": {
                                                    "musicCarouselShelfBasicHeaderRenderer": {
                                                        "title": { "runs": [{ "text": "Singles" }] }
                                                    }
                                                },
                                                "contents": [
                                                    {
                                                        "musicTwoRowItemRenderer": {
                                                            "navigationEndpoint": {
                                                                "browseEndpoint": { "browseId": "MPREb_single123" }
                                                            },
                                                            "title": { "runs": [{ "text": "Anti-Hero" }] },
                                                            "subtitle": { "runs": [{ "text": "Single • 2022" }] },
                                                            "thumbnailRenderer": {
                                                                "musicThumbnailRenderer": {
                                                                    "thumbnail": {
                                                                        "thumbnails": [{ "url": "https://lh3.googleusercontent.com/sgl=w300-h300" }]
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                ]
                                            }
                                        },
                                        {
                                            "musicCarouselShelfRenderer": {
                                                "header": {
                                                    "musicCarouselShelfBasicHeaderRenderer": {
                                                        "title": { "runs": [{ "text": "Fans might also like" }] }
                                                    }
                                                },
                                                "contents": [
                                                    {
                                                        "musicTwoRowItemRenderer": {
                                                            "navigationEndpoint": {
                                                                "browseEndpoint": { "browseId": "UC_olivia" }
                                                            },
                                                            "title": { "runs": [{ "text": "Olivia Rodrigo" }] },
                                                            "subtitle": { "runs": [{ "text": "10.4M subscribers" }] },
                                                            "thumbnailRenderer": {
                                                                "musicThumbnailRenderer": {
                                                                    "thumbnail": {
                                                                        "thumbnails": [{ "url": "https://lh3.googleusercontent.com/olivia=w300-h300" }]
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                ]
                                            }
                                        }
                                    ]
                                }
                            }
                        }
                    }]
                }
            }
        });

        let details = parse_artist_details(&mock, "UC_taylor").expect("Should parse artist details");
        assert_eq!(details.id, "UC_taylor");
        assert_eq!(details.name, "Taylor Swift");
        assert_eq!(details.subscribers.as_deref(), Some("58.2M subscribers"));
        assert!(details.description.as_deref().unwrap().contains("Alison Swift"));
        assert_eq!(details.avatar_url.as_deref(), Some("https://lh3.googleusercontent.com/avatar=w544-h544-l90-rj"));

        // Top songs
        assert_eq!(details.top_songs.len(), 1);
        assert_eq!(details.top_songs[0].id, "video123");
        assert_eq!(details.top_songs[0].title, "Cruel Summer");
        assert_eq!(details.top_songs[0].duration_secs, 178);

        // Albums
        assert_eq!(details.albums.len(), 1);
        assert_eq!(details.albums[0].browse_id, "MPREb_album123");
        assert_eq!(details.albums[0].title, "1989 (Taylor's Version)");
        assert_eq!(details.albums[0].year.as_deref(), Some("2023"));

        // Singles
        assert_eq!(details.singles.len(), 1);
        assert_eq!(details.singles[0].browse_id, "MPREb_single123");
        assert_eq!(details.singles[0].title, "Anti-Hero");
        assert_eq!(details.singles[0].year.as_deref(), Some("2022"));

        // Related artists
        assert_eq!(details.related_artists.len(), 1);
        assert_eq!(details.related_artists[0].channel_id, "UC_olivia");
        assert_eq!(details.related_artists[0].name, "Olivia Rodrigo");
        assert_eq!(details.related_artists[0].subscribers.as_deref(), Some("10.4M subscribers"));
    }
}
