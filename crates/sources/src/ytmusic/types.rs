use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodChip {
    pub title: String,
    pub params: Option<String>,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseFeed {
    pub title: String,
    pub chips: Vec<MoodChip>,
    pub sections: Vec<BrowseSection>,
    pub continuation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseSection {
    pub title: String,
    pub subtitle: Option<String>,
    pub items: Vec<ShelfItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShelfItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub thumbnail_url: Option<String>,
    pub item_type: ShelfItemType,
    pub endpoint_id: String,
    pub duration_secs: Option<u64>,
    pub is_explicit: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShelfItemType {
    Track,
    Album,
    Artist,
    Playlist,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSnapshot {
    pub current_track: Option<QueueTrackItem>,
    pub upcoming: Vec<QueueTrackItem>,
    pub autoplay_enabled: bool,
    pub source_playlist_title: Option<String>,
    #[serde(default)]
    pub history: Vec<QueueTrackItem>,
    #[serde(default)]
    pub continuation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueTrackItem {
    pub video_id: String,
    pub title: String,
    pub artist: String,
    pub duration_str: String,
    pub thumbnail_url: Option<String>,
    pub is_active: bool,
}

impl QueueTrackItem {
    pub fn parse_duration_secs(&self) -> u32 {
        let parts: Vec<&str> = self.duration_str.split(':').collect();
        match parts.len() {
            2 => parts[0].trim().parse::<u32>().unwrap_or(0) * 60 + parts[1].trim().parse::<u32>().unwrap_or(0),
            3 => parts[0].trim().parse::<u32>().unwrap_or(0) * 3600 + parts[1].trim().parse::<u32>().unwrap_or(0) * 60 + parts[2].trim().parse::<u32>().unwrap_or(0),
            _ => 180,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsDocument {
    pub plain_text: Option<String>,
    pub lines: Vec<LyricsLine>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricsLine {
    pub text: String,
    pub start_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LibraryItemType {
    Playlist,
    Album,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub item_type: LibraryItemType,
    pub thumbnail_url: Option<String>,
    pub track_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryArtist {
    pub id: String,
    pub name: String,
    pub subscribers: Option<String>,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtCollectionDetails {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub description: Option<String>,
    pub thumbnail_url: Option<String>,
    pub track_count: Option<usize>,
    pub tracks: Vec<rustic_core::api::Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtArtistDetails {
    pub id: String,
    pub name: String,
    pub subscribers: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub top_songs: Vec<rustic_core::api::Track>,
    pub albums: Vec<YtAlbumItem>,
    pub singles: Vec<YtAlbumItem>,
    pub related_artists: Vec<YtRelatedArtist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtAlbumItem {
    pub browse_id: String,
    pub title: String,
    pub year: Option<String>,
    pub thumbnail_url: Option<String>,
    pub explicit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YtRelatedArtist {
    pub channel_id: String,
    pub name: String,
    pub subscribers: Option<String>,
    pub thumbnail_url: Option<String>,
}

