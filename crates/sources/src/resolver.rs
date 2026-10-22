use rustic_core::api::{ResolvedAudio, Track, TrackSource};
use std::sync::Arc;
use crate::{SourceError, SourceProvider};

pub struct SourceResolver {
    ytm: Arc<dyn SourceProvider>,
    spotify: Arc<dyn SourceProvider>,
}

impl SourceResolver {
    pub fn new(ytm: Arc<dyn SourceProvider>, spotify: Arc<dyn SourceProvider>) -> Self {
        Self { ytm, spotify }
    }

    pub async fn resolve(&self, track: &Track) -> Result<ResolvedAudio, SourceError> {
        match &track.source {
            TrackSource::Local(path) => Ok(ResolvedAudio {
                stream_url: path.to_string_lossy().to_string(),
                codec: "file".into(),
                bitrate_kbps: 320,
                expires_at_unix_secs: None,
            }),
            TrackSource::YouTubeMusic { .. } => self.ytm.resolve(track).await,
            TrackSource::Spotify { .. } => {
                // First try direct, if unavailable match with YTM
                match self.spotify.resolve(track).await {
                    Ok(res) => Ok(res),
                    Err(_) => {
                        let ytm_matches = self.ytm.search(&track.title, 1).await?;
                        if let Some(matched_track) = ytm_matches.first() {
                            self.ytm.resolve(matched_track).await
                        } else {
                            Err(SourceError::Unavailable("No YTM stream match found for Spotify track".into()))
                        }
                    }
                }
            }
        }
    }
}
