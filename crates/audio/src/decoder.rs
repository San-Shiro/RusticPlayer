//! Symphonia-backed decoder: probe -> decode -> rubato resample -> f32 interleaved.
//!
//! All errors are mapped to `AudioError`; decoding arbitrary or corrupt input
//! returns `Err`, never panics. Position bookkeeping is in device-rate frames
//! and is consumed through the ring buffer by the cpal callback; the decode
//! thread only produces.

use rubato::Resampler;
use symphonia::core::codecs::{Decoder as SymDecoder, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::units::Time;

use crate::error::{AudioError, Result};
use crate::source::MediaSource;

struct SymSource(Box<dyn MediaSource>);

impl std::io::Read for SymSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl std::io::Seek for SymSource {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.0.seek(pos)
    }
}

impl symphonia::core::io::MediaSource for SymSource {
    fn is_seekable(&self) -> bool {
        true
    }
    fn byte_len(&self) -> Option<u64> {
        self.0.len()
    }
}

/// Decoder output layout. Samples are always interleaved f32 at
/// `device_rate * channels`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioFormat {
    pub sample_rate: u32,
    pub channels: usize,
}

/// One decoded chunk, already resampled to device rate and interleaved.
#[derive(Debug, Clone)]
pub struct DecodedBlock {
    pub samples: Vec<f32>,
    /// Starting frame index in the *device* domain purely for tracing; the
    /// source of truth for position is samples consumed by the callback.
    pub frame_offset: u64,
}

/// Symphonia-backed decoder over our `MediaSource`.
pub struct DecoderImpl {
    reader: Box<dyn FormatReader>,
    codec: Box<dyn SymDecoder>,
    pub track_id: u32,
    format: AudioFormat,
    /// Sample rate of the source stream before any resampling.
    src_rate: u32,
    /// Total source frames read since the last seek.
    frames_consumed: u64,
    /// Cached value for `duration_secs`.
    duration_secs: Option<f64>,
    /// Lazy resampler; created when a decoded packet's length is known.
    resampler: Option<rubato::FftFixedInOut<f32>>,
    resample_in: Vec<Vec<f32>>,
    eos: bool,
}

impl DecoderImpl {
    /// Probe and open a `MediaSource`, producing device-rate f32 output.
    /// `device_rate` is the rate the cpal callback expects.
    pub fn open(source: Box<dyn MediaSource>, device_rate: u32, device_channels: usize) -> Result<Self> {
        let mss = MediaSourceStream::new(Box::new(SymSource(source)), Default::default());
        let hint = symphonia::core::probe::Hint::new();

        // The umbrella crate keeps a single default probe + default codec
        // registry, populated from cargo features.
        let reader = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &Default::default())
            .map_err(|e| AudioError::Decode(format!("probe: {}", e)))?
            .format;

        let (track, codec) = reader
            .tracks()
            .iter()
            .filter(|t| {
                t.codec_params.codec != CODEC_TYPE_NULL
                    && t.codec_params.sample_rate.is_some()
            })
            .find_map(|t| {
                let codec = symphonia::default::get_codecs()
                    .make(&t.codec_params, &DecoderOptions::default())
                    .ok()?;
                Some((t.clone(), codec))
            })
            .ok_or(AudioError::NoAudioTrack)?;

        let src_rate = track
            .codec_params
            .sample_rate
            .ok_or_else(|| AudioError::UnsupportedFormat("sample rate unknown".into()))?;

        let duration_secs = track.codec_params.time_base.zip(track.codec_params.n_frames).map(|(tb, frames)| {
            tb.calc_time(frames).seconds as f64 + tb.calc_time(frames).frac
        });

        let target_channels = if device_channels == 0 { 2 } else { device_channels };

        Ok(Self {
            codec,
            reader,
            track_id: track.id,
            format: AudioFormat {
                sample_rate: device_rate,
                channels: target_channels,
            },
            src_rate,
            frames_consumed: 0,
            duration_secs,
            resampler: None,
            resample_in: Vec::new(),
            eos: false,
        })
    }

    pub fn format(&self) -> AudioFormat {
        self.format
    }

    pub fn duration_secs(&self) -> Option<f64> {
        self.duration_secs
    }

    pub fn frames_consumed(&self) -> u64 {
        self.frames_consumed
    }

    /// Decode the next packet into f32 interleaved samples at device rate.
    /// Returns `Ok(None)` at end of stream. Corrupt packets are skipped.
    pub fn next_block(&mut self) -> Result<Option<DecodedBlock>> {
        if self.eos && (self.resample_in.is_empty() || self.resample_in[0].is_empty()) {
            return Ok(None);
        }

        loop {
            // If at EOS, flush remaining resampler input buffer
            if self.eos {
                if !self.resample_in.is_empty() && !self.resample_in[0].is_empty() {
                    let samples = self.flush_resample()?;
                    let samples = convert_channels(&samples, self.resample_in.len(), self.format.channels);
                    if !samples.is_empty() {
                        return Ok(Some(DecodedBlock {
                            samples,
                            frame_offset: self.frames_consumed,
                        }));
                    }
                }
                return Ok(None);
            }

            let packet = match self.reader.next_packet() {
                Ok(p) => p,
                Err(symphonia::core::errors::Error::IoError(_)) => {
                    self.eos = true;
                    continue;
                }
                Err(symphonia::core::errors::Error::ResetRequired) => {
                    continue;
                }
                Err(e) => return Err(AudioError::Decode(format!("demux: {}", e))),
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            let (frames_in, channels_in, samples) = {
                let buf = match self.codec.decode(&packet) {
                    Ok(b) => b,
                    // A corrupt packet should be skipped, not fatal.
                    Err(symphonia::core::errors::Error::DecodeError(_)) => {
                        continue;
                    }
                    Err(e) => return Err(AudioError::Decode(format!("decode: {}", e))),
                };

                let nframes = buf.frames();
                let spec = *buf.spec();
                
                let mut sample_buf = symphonia::core::audio::SampleBuffer::<f32>::new(buf.capacity() as u64, spec);
                sample_buf.copy_interleaved_ref(buf);
                let out = sample_buf.samples().to_vec();
                
                (nframes, spec.channels.count(), out)
            };

            self.frames_consumed += frames_in as u64;

            // Resample if the container rate differs from the device rate.
            let samples = if self.src_rate == self.format.sample_rate {
                samples
            } else {
                self.resample(&samples, channels_in)?
            };

            if samples.is_empty() {
                // Buffer filling in progress
                continue;
            }

            // Convert channel count to device channels (e.g. mono -> stereo).
            let samples = convert_channels(&samples, channels_in, self.format.channels);

            return Ok(Some(DecodedBlock {
                samples,
                frame_offset: self.frames_consumed,
            }));
        }
    }

    fn resample(&mut self, samples: &[f32], channels: usize) -> Result<Vec<f32>> {
        if self.resample_in.len() != channels {
            self.resample_in = vec![Vec::new(); channels];
        }

        for (i, &s) in samples.iter().enumerate() {
            self.resample_in[i % channels].push(s);
        }

        if self.resampler.is_none() {
            self.resampler = Some(
                rubato::FftFixedInOut::<f32>::new(
                    self.src_rate as usize,
                    self.format.sample_rate as usize,
                    1024,
                    channels,
                )
                .map_err(|e| AudioError::Decode(format!("resampler init: {}", e)))?,
            );
        }
        let res = self.resampler.as_mut().unwrap();
        let needed = res.input_frames_next();
        let mut all_resampled = Vec::new();

        while self.resample_in[0].len() >= needed {
            let chunk: Vec<Vec<f32>> = (0..channels)
                .map(|ch| self.resample_in[ch].drain(..needed).collect())
                .collect();
            let output_chan = res.output_frames_next();
            let mut output: Vec<Vec<f32>> = (0..channels).map(|_| vec![0.0; output_chan]).collect();
            let out_frames = res
                .process_into_buffer(&chunk, &mut output_refs_mut(&mut output), None)
                .map_err(|e| AudioError::Decode(format!("resample: {}", e)))?
                .1;
            all_resampled.extend(interleave(&output, out_frames));
        }

        Ok(all_resampled)
    }

    fn flush_resample(&mut self) -> Result<Vec<f32>> {
        if self.resample_in.is_empty() || self.resample_in[0].is_empty() {
            return Ok(Vec::new());
        }
        let channels = self.resample_in.len();
        if let Some(res) = self.resampler.as_mut() {
            let needed = res.input_frames_next();
            let available = self.resample_in[0].len();
            if available > 0 {
                let chunk: Vec<Vec<f32>> = (0..channels)
                    .map(|ch| {
                        let mut v = std::mem::take(&mut self.resample_in[ch]);
                        v.resize(needed, 0.0);
                        v
                    })
                    .collect();
                let output_chan = res.output_frames_next();
                let mut output: Vec<Vec<f32>> = (0..channels).map(|_| vec![0.0; output_chan]).collect();
                let out_frames = res
                    .process_into_buffer(&chunk, &mut output_refs_mut(&mut output), None)
                    .map_err(|e| AudioError::Decode(format!("resample flush: {}", e)))?
                    .1;
                let valid_out = ((available as f64 / needed as f64) * out_frames as f64).round() as usize;
                return Ok(interleave(&output, valid_out));
            }
        }
        self.resample_in.clear();
        Ok(Vec::new())
    }

    /// Seek the reader + decoder.
    pub fn seek(&mut self, position_seconds: f64) -> Result<()> {
        if position_seconds < 0.0 {
            return Err(AudioError::InvalidSeek {
                position_ms: 0,
                duration_ms: self.duration_secs.unwrap_or(0.0) as u64 * 1000,
            });
        }
        let time = Time::new(position_seconds as u64, position_seconds.fract());
        self.reader
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| AudioError::Decode(format!("seek: {}", e)))?;
        self.codec.reset();
        self.frames_consumed = (position_seconds * self.src_rate as f64) as u64;
        self.eos = false;
        self.resampler = None;
        self.resample_in.clear();
        Ok(())
    }
}

pub type Decoder = DecoderImpl;

fn interleave(input: &[Vec<f32>], frames: usize) -> Vec<f32> {
    let channels = input.len();
    let mut out = Vec::with_capacity(frames * channels);
    for f in 0..frames {
        for ch in input.iter() {
            out.push(ch.get(f).copied().unwrap_or(0.0));
        }
    }
    out
}

fn output_refs_mut(output: &mut [Vec<f32>]) -> Vec<&mut [f32]> {
    output.iter_mut().map(|v| v.as_mut_slice()).collect()
}

fn convert_channels(samples: &[f32], from: usize, to: usize) -> Vec<f32> {
    if from == to || from == 0 || to == 0 {
        return samples.to_vec();
    }
    let frames = samples.len() / from;
    let mut out = Vec::with_capacity(frames * to);
    if from == 1 && to == 2 {
        for &s in samples {
            out.push(s);
            out.push(s);
        }
    } else if from == 2 && to == 1 {
        for chunk in samples.chunks_exact(2) {
            out.push((chunk[0] + chunk[1]) * 0.5);
        }
    } else {
        for f in 0..frames {
            for c in 0..to {
                if c < from {
                    out.push(samples[f * from + c]);
                } else {
                    out.push(0.0);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::FileSource;

    #[test]
    fn test_decode_real_dummy_mp4() {
        let mut path = std::path::PathBuf::from(r"dummy_library\The Mountain Pines\Dust & Timber\01 - Campfire Embers.mp4");
        if !path.exists() {
            path = std::path::PathBuf::from(r"../../dummy_library/The Mountain Pines/Dust & Timber/01 - Campfire Embers.mp4");
        }
        if !path.exists() {
            panic!("dummy file not found at {:?}", path);
        }
        let source = Box::new(FileSource::open(&path).expect("FileSource open failed"));
        let mut decoder = DecoderImpl::open(source, 44100, 2).expect("Decoder open failed");
        assert_eq!(decoder.format().channels, 2);
        assert_eq!(decoder.format().sample_rate, 44100);
        assert!(decoder.duration_secs().is_some());
        let block = decoder.next_block().expect("next_block failed");
        assert!(block.is_some(), "expected at least one decoded block");
        let samples = block.unwrap().samples;
        assert!(!samples.is_empty(), "expected samples in block");
        // Channels is 2, so samples.len() should be an even number
        assert_eq!(samples.len() % 2, 0);
        println!("Successfully decoded {} stereo samples (duration {:?}s)", samples.len(), decoder.duration_secs());
    }
}

