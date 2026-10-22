use async_trait::async_trait;
use rustic_core::api::{AuthFlow, AuthState, Playlist, ResolvedAudio, Track, TrackSource};
use crate::{SourceError, SourceProvider};

pub struct SpotifyProvider;

impl SpotifyProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SourceProvider for SpotifyProvider {
    fn slug(&self) -> &'static str {
        "spotify"
    }

    async fn login(&self, flow: AuthFlow) -> Result<AuthState, SourceError> {
        match flow {
            AuthFlow::Pkce => Ok(AuthState {
                provider: "spotify".into(),
                logged_in: true,
                account_name: Some("Spotify User".into()),
                account_avatar_url: None,
                account_handle: None,
            }),
            _ => Err(SourceError::NotAuthenticated("Spotify requires PKCE auth flow".into())),
        }
    }

    async fn search(&self, query: &str, _limit: usize) -> Result<Vec<Track>, SourceError> {
        let mut tracks = Vec::new();
        tracks.push(Track {
            id: format!("spotify_search_{}", query),
            title: format!("{} (Spotify Track)", query),
            artists: vec!["Spotify Artist".into()],
            album: "Spotify Album".into(),
            duration_secs: 210,
            source: TrackSource::Spotify {
                spotify_id: "4cOdK2wGLETKBW3PvgPWqT".into(),
            },
            isrc: Some("USRC17607839".into()),
            artwork_url: None,
            content_hash: None,
        });

        Ok(tracks)
    }

    async fn library(&self) -> Result<Vec<Playlist>, SourceError> {
        Ok(vec![Playlist {
            id: "spotify_saved".into(),
            provider: "spotify".into(),
            name: "Liked Songs".into(),
            tracks: vec![],
        }])
    }

    async fn resolve(&self, track: &Track) -> Result<ResolvedAudio, SourceError> {
        // Spotify web playback resolution or mapping fallback
        match &track.source {
            TrackSource::Spotify { spotify_id: _ } => {
                Err(SourceError::Unavailable(
                    "Direct Spotify audio stream unavailable without premium SDK; use YTM matching resolver".into(),
                ))
            }
            _ => Err(SourceError::Unavailable("Not a Spotify track".into())),
        }
    }
}
