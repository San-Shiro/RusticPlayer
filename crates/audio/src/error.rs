//! Error types for the audio crate. No user-facing panics.

use thiserror::Error;

/// The one error type the audio crate surfaces. Every fallible operation
/// returns this so the engine's `Event::Error(EngineError)` mapping stays
/// mechanical and exhaustive.
#[derive(Debug, Error)]
pub enum AudioError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("decode error: {0}")]
    Decode(String),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("no audio track found in source")]
    NoAudioTrack,

    #[error("network error: {0}")]
    Network(String),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("invalid seek position {position_ms}ms (duration {duration_ms}ms)")]
    InvalidSeek { position_ms: u64, duration_ms: u64 },

    #[error("output device error: {0}")]
    OutputDevice(String),

    #[error("stream underrun")]
    Underrun,

    #[error("cache error: {0}")]
    Cache(String),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, AudioError>;
