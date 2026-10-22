use regex::Regex;

pub struct MatchScore;

impl MatchScore {
    pub fn calculate(
        isrc_a: Option<&str>,
        isrc_b: Option<&str>,
        title_a: &str,
        title_b: &str,
        duration_a_secs: u32,
        duration_b_secs: u32,
    ) -> f32 {
        // 1. ISRC Exact Match
        if let (Some(a), Some(b)) = (isrc_a, isrc_b) {
            if !a.is_empty() && a.eq_ignore_ascii_case(b) {
                return 1.0;
            }
        }

        // 2. Clean titles
        let re = Regex::new(r"(?i)\s*[\(\[](official\s*)?(video|audio|lyric video|visualizer)[\)\]]").unwrap();
        let clean_a = re.replace_all(title_a, "").trim().to_lowercase();
        let clean_b = re.replace_all(title_b, "").trim().to_lowercase();

        let title_sim = if clean_a == clean_b {
            0.8
        } else if clean_a.contains(&clean_b) || clean_b.contains(&clean_a) {
            0.5
        } else {
            0.0
        };

        // 3. Duration delta penalty (+/- 3 seconds tolerance)
        let diff = (duration_a_secs as i32 - duration_b_secs as i32).abs();
        let duration_penalty = if diff <= 3 {
            0.2
        } else if diff <= 10 {
            0.1
        } else {
            0.0
        };

        f32::min(title_sim + duration_penalty, 1.0)
    }
}
