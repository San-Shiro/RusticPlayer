//! MediaSource abstraction: the [`MediaSource`] trait plus its concrete
//! implementations.
//!
//! [`FileSource`] wraps a local file; [`HttpSource`] (defined in
//! [`crate::http_source`] and re-exported here) provides a ranged, seekable,
//! range-buffered view over an HTTP URL for symphonia's synchronous
//! `MediaSource` trait.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

use crate::error::Result;

pub use crate::http_source::HttpSource;

/// Chunk size for HTTP range reads. 256KB reduces round-trips during
/// ISOMP4 header/atom probing and buffers ~16s of AAC audio per fetch.
pub const CHUNK_SIZE: u64 = 256 * 1024;

/// A logical audio input. Implemented by local files and ranged HTTP.
///
/// The trait is synchronous because symphonia's `MediaSource` is synchronous.
/// See [`crate::http_source::HttpSource`] for the ranged HTTP implementation.
pub trait MediaSource: Read + Seek + Send + Sync {
    /// Total byte length of the resource, if known (content-length).
    fn len(&self) -> Option<u64>;

    /// True when the source is a network stream (drives cache prebuffering).
    fn is_remote(&self) -> bool {
        false
    }
}

/// Local file-backed source.
pub struct FileSource {
    file: File,
    len: u64,
}

impl FileSource {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let file = File::open(path.into())?;
        let len = file.metadata()?.len();
        Ok(Self { file, len })
    }
}

impl Read for FileSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl Seek for FileSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl MediaSource for FileSource {
    fn len(&self) -> Option<u64> {
        Some(self.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, SeekFrom};

    #[test]
    fn file_source_len_and_seek() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let tmp = tempfile::NamedTempFile::new()?;
        std::fs::write(tmp.path(), b"hello world")?;

        let mut src = FileSource::open(tmp.path())?;
        assert_eq!(src.len(), Some(11));

        let mut buf = [0u8; 5];
        src.read_exact(&mut buf)?;
        assert_eq!(&buf, b"hello");

        src.seek(SeekFrom::Start(6))?;
        let mut rest = Vec::new();
        src.read_to_end(&mut rest)?;
        assert_eq!(rest, b"world");
        Ok(())
    }
}
