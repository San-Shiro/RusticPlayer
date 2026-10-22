use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

pub type TrackId = String;
pub type PeerId = String;

pub const API_VERSION: &str = "1.0.0";
pub const PROTOCOL_VERSION: u32 = 1;

// ---- playback primitives ----

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum PlayContext {
    Album(String),
    Artist(String),
    Playlist(String),
    AllTracks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub enum RepeatMode {
    #[default]
    Off,
    All,
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
    Buffering,
}

// ---- tracks & sources ----

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TrackSource {
    Local(PathBuf),
    YouTubeMusic { video_id: String },
    Spotify { spotify_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration_secs: u32,
    pub source: TrackSource,
    pub isrc: Option<String>,
    pub artwork_url: Option<String>,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Playlist {
    pub id: String,
    pub provider: String,
    pub name: String,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResolvedAudio {
    pub stream_url: String,
    pub codec: String,
    pub bitrate_kbps: u32,
    pub expires_at_unix_secs: Option<u64>,
}

// ---- auth ----

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AuthFlow {
    Pkce,
    BrowserCookie,
    AccessToken(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuthState {
    pub provider: String,
    pub logged_in: bool,
    pub account_name: Option<String>,
    #[serde(default)]
    pub account_avatar_url: Option<String>,
    #[serde(default)]
    pub account_handle: Option<String>,
}

// ---- commands ----

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Command {
    pub origin: Option<PeerId>,
    pub action: CommandAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum CommandAction {
    // Transport
    Play,
    Pause,
    TogglePlay,
    Stop,
    Next,
    Previous,
    Seek(Duration),
    SetVolume(f32),

    // Resolution & Online Sources
    ResolveAndPlay(TrackId),
    Login {
        provider: String,
    },
    ImportPlaylist {
        provider: String,
        external_id: String,
    },

    // Queue
    PlayTrack(TrackId),
    PlayContext {
        context: PlayContext,
        start_at: usize,
    },
    EnqueueNext(Vec<TrackId>),
    EnqueueEnd(Vec<TrackId>),
    RemoveFromQueue(usize),
    MoveInQueue {
        from: usize,
        to: usize,
    },
    SetShuffle(bool),
    SetRepeat(RepeatMode),

    // --- 3-Lane Queue Commands ---
    PlayCollection {
        tracks: Vec<Track>,
        start_index: usize,
        title: String,
        id: String,
        shuffle: bool,
    },
    PlaySingleTrack(Track),
    EnqueueTrackNext(Track),
    EnqueueTrackEnd(Track),
    RemoveUserQueueItem(usize),
    ClearUserQueue,
    MoveUserQueueItem {
        from: usize,
        to: usize,
    },
    PromoteRadioItem(usize),
    PlayRadioItem(usize),
    JumpToQueueIndex(usize),
    JumpToTrackId(String),
    ToggleAutoplay(bool),
    IngestRadioTracks(Vec<Track>),
    SetUpcomingTracks(Vec<Track>),

    // Library
    ScanFolders(Vec<PathBuf>),
    RescanChanged,
    Search {
        query: String,
        limit: usize,
    },

    // Diagnostics & Hardware
    ReconnectAudio,
}

// ---- events ----

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResults {
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ErrorKind {
    Network,
    Auth,
    Resolve,
    Decode,
    Library,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EngineError {
    pub kind: ErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MultiLaneQueueSnapshot {
    pub current_track: Option<Track>,
    #[serde(default)]
    pub actual_queue: Vec<Track>,
    #[serde(default)]
    pub cursor: usize,
    pub user_queue: Vec<Track>,
    pub context_title: Option<String>,
    pub context_id: Option<String>,
    pub context_upcoming: Vec<Track>,
    pub radio_upcoming: Vec<Track>,
    pub history: Vec<Track>,
    pub autoplay_enabled: bool,
    pub is_shuffled: bool,
    pub generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Event {
    PlaybackStateChanged(PlaybackState),
    PositionChanged {
        position_secs: f64,
        duration_secs: f64,
    },
    TrackChanged(Option<Track>),
    Resolved {
        track_id: TrackId,
        audio: ResolvedAudio,
    },
    VolumeChanged(f32),

    QueueChanged(Vec<Track>),
    QueueUpdated(Box<MultiLaneQueueSnapshot>),
    ShuffleChanged(bool),
    RepeatChanged(RepeatMode),

    AuthStateChanged {
        provider: String,
        logged_in: bool,
    },
    LyricsChanged {
        track_id: TrackId,
        lrc_content: Option<String>,
    },
    JamSessionChanged {
        session_id: Option<String>,
        peers: Vec<PeerId>,
    },

    ScanProgress {
        done: usize,
        total: usize,
    },
    ScanComplete {
        added: usize,
        updated: usize,
        removed: usize,
    },
    LibraryChanged,
    SearchResults {
        query: String,
        results: SearchResults,
    },
    TrackEnded {
        track_id: TrackId,
    },

    Error(EngineError),
}
