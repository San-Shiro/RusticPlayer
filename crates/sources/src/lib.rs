use async_trait::async_trait;
use rustic_core::api::{AuthFlow, AuthState, Playlist, ResolvedAudio, Track};

pub mod ytmusic;
pub mod spotify;
pub mod matching;
pub mod resolver;

pub use ytmusic::YtMusicProvider;
pub use ytmusic::types::*;
pub use spotify::SpotifyProvider;
pub use matching::MatchScore;
pub use resolver::SourceResolver;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("not authenticated with {0}")]
    NotAuthenticated(String),
    #[error("network error: {0}")]
    Network(String),
    #[error("track unavailable: {0}")]
    Unavailable(String),
    #[error("parse error: {0}")]
    ParseError(String),
}

#[async_trait]
pub trait SourceProvider: Send + Sync {
    fn slug(&self) -> &'static str;
    async fn login(&self, flow: AuthFlow) -> Result<AuthState, SourceError>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<Track>, SourceError>;
    async fn library(&self) -> Result<Vec<Playlist>, SourceError>;
    async fn resolve(&self, track: &Track) -> Result<ResolvedAudio, SourceError>;
}
