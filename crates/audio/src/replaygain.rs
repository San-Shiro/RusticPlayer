//! ReplayGain support. Plain gain staging applied linearly in the mixer.
//!
//! Tag reference (EBU R128 derived): a gain of `0 dB` is reference level;
//! positive values attenuate. We normalize to a *linear multiplier* once, so
//! the audio callback only sees a plain f32.

/// Linear gain multiplier to reach reference loudness.
pub type Gain = f32;

/// Parse a ReplayGain track gain string (e.g. `-7.41 dB`) and return the
/// linear multiplier. Returns `1.0` for missing/unparseable values (no-op).
pub fn parse_db(db: &str) -> Gain {
    let trimmed = db.trim().trim_end_matches("dB").trim();
    match trimmed.parse::<f32>() {
        Ok(v) => db_to_linear(v),
        Err(_) => 1.0,
    }
}

/// Convert dB → linear gain.
#[inline]
pub fn db_to_linear(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_values() {
        assert!((parse_db("-6.02 dB") - 0.5).abs() < 1e-3);
        assert!((parse_db("0.0 dB") - 1.0).abs() < 1e-6);
        assert!((parse_db("+3.5") - db_to_linear(3.5)).abs() < 1e-6);
    }

    #[test]
    fn missing_becomes_unity() {
        assert_eq!(parse_db(""), 1.0);
        assert_eq!(parse_db("not-a-number"), 1.0);
    }
}
