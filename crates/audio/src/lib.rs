//! Rustic Player audio engine.
//!
//! Decode (symphonia) → resample (rubato) → rtrb ring → cpal callback. All
//! position bookkeeping derives from samples consumed by the real-time
//! callback, never wall clock, so the Jam controller can rely on it later.
//!
//! Layering: this crate owns *playback*; it knows nothing about the library,
//! providers, or UI. Everything crosses the engine boundary as `Command`/`Event`.

pub mod crossfade;
pub mod decoder;
pub mod error;
pub mod http_source;
pub mod output;
pub mod player;
pub mod replaygain;
pub mod source;
pub mod stream_cache;

// Convenience re-exports for the common case.
pub use decoder::{AudioFormat, DecodedBlock, Decoder};
pub use error::{AudioError, Result};
pub use output::{AudioDiagnostics, OutputDevice, OutputStream, RING_BUFFER_MS, VOLUME_RAMP_MS, ring_buffer};
pub use player::{PlaybackState, Player, PlayerCommand, PlayerEvent};
pub use replaygain::{Gain, db_to_linear, parse_db};
pub use source::{CHUNK_SIZE, FileSource, HttpSource, MediaSource};
pub use stream_cache::{CacheStats, StreamCache, StreamMeta};


