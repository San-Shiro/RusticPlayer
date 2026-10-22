use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use image::GenericImageView;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hsl {
    pub h: f32, // 0.0 .. 360.0
    pub s: f32, // 0.0 .. 1.0
    pub l: f32, // 0.0 .. 1.0
}

impl Hsl {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let rf = r as f32 / 255.0;
        let gf = g as f32 / 255.0;
        let bf = b as f32 / 255.0;

        let max = rf.max(gf).max(bf);
        let min = rf.min(gf).min(bf);
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta.abs() < 1e-5 {
            return Self { h: 0.0, s: 0.0, l };
        }

        let s = if l > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };

        let mut h = if (max - rf).abs() < 1e-5 {
            (gf - bf) / delta + (if gf < bf { 6.0 } else { 0.0 })
        } else if (max - gf).abs() < 1e-5 {
            (bf - rf) / delta + 2.0
        } else {
            (rf - gf) / delta + 4.0
        };
        h *= 60.0;
        if h < 0.0 {
            h += 360.0;
        }

        Self { h, s, l }
    }

    pub fn to_gpui_rgba(self, alpha: f32) -> gpui::Rgba {
        let c = (1.0 - (2.0 * self.l - 1.0).abs()) * self.s;
        let hp = (self.h % 360.0) / 60.0;
        let x = c * (1.0 - ((hp % 2.0) - 1.0).abs());
        let (r1, g1, b1) = if hp >= 0.0 && hp < 1.0 {
            (c, x, 0.0)
        } else if hp >= 1.0 && hp < 2.0 {
            (x, c, 0.0)
        } else if hp >= 2.0 && hp < 3.0 {
            (0.0, c, x)
        } else if hp >= 3.0 && hp < 4.0 {
            (0.0, x, c)
        } else if hp >= 4.0 && hp < 5.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let m = self.l - c / 2.0;
        gpui::Rgba {
            r: (r1 + m).clamp(0.0, 1.0),
            g: (g1 + m).clamp(0.0, 1.0),
            b: (b1 + m).clamp(0.0, 1.0),
            a: alpha.clamp(0.0, 1.0),
        }
    }
}

#[derive(Default)]
struct ColorBin {
    count: usize,
    sum_h: f32,
    sum_s: f32,
    sum_l: f32,
}

#[derive(Clone)]
pub struct PaletteCache {
    cache: Arc<Mutex<HashMap<String, (gpui::Rgba, gpui::Rgba)>>>,
    ambient_cache: Arc<Mutex<HashMap<String, String>>>,
}

impl PaletteCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ambient_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns the default dark ambient palette (deep warm amber c1, deep cool slate c2).
    pub fn default_palette() -> (gpui::Rgba, gpui::Rgba) {
        let c1 = Hsl { h: 32.0, s: 0.45, l: 0.22 }.to_gpui_rgba(1.0);
        let c2 = Hsl { h: 215.0, s: 0.32, l: 0.14 }.to_gpui_rgba(1.0);
        (c1, c2)
    }

    /// Checks if the palette for a given path/key is already cached.
    pub fn get_cached(&self, key: &str) -> Option<(gpui::Rgba, gpui::Rgba)> {
        let guard = self.cache.lock().unwrap();
        guard.get(key).copied()
    }

    /// Synchronously extracts dominant ambient colors from an image file on disk.
    /// Clamps luminance to dark moody levels (c1: 0.18-0.28, c2: 0.10-0.20)
    /// and saturation (c1: 0.35-0.65, c2: 0.25-0.50) so text remains crisp and legible.
    pub fn extract_from_file_sync(path: &Path) -> (gpui::Rgba, gpui::Rgba) {
        let Ok(img) = image::open(path) else {
            return Self::default_palette();
        };

        // Resize down to 64x64 for lightning-fast analysis
        let thumb = img.thumbnail(64, 64);
        let (width, height) = thumb.dimensions();
        if width == 0 || height == 0 {
            return Self::default_palette();
        }

        // 12 hue bins (30 degrees each)
        let mut bins: Vec<ColorBin> = (0..12).map(|_| ColorBin::default()).collect();
        let mut grayscale_bin = ColorBin::default();

        for y in 0..height {
            for x in 0..width {
                let pixel = thumb.get_pixel(x, y);
                let hsl = Hsl::from_rgb(pixel[0], pixel[1], pixel[2]);

                // Filter out blown-out whites and pitch blacks
                if hsl.l < 0.06 || hsl.l > 0.94 {
                    continue;
                }

                if hsl.s < 0.12 {
                    grayscale_bin.count += 1;
                    grayscale_bin.sum_s += hsl.s;
                    grayscale_bin.sum_l += hsl.l;
                } else {
                    let bin_idx = ((hsl.h / 30.0) as usize).min(11);
                    bins[bin_idx].count += 1;
                    bins[bin_idx].sum_h += hsl.h;
                    bins[bin_idx].sum_s += hsl.s;
                    bins[bin_idx].sum_l += hsl.l;
                }
            }
        }

        // Rank colored bins by vibrancy score
        let mut scored_bins: Vec<(usize, f32)> = bins
            .iter()
            .enumerate()
            .filter(|(_, b)| b.count > 0)
            .map(|(idx, b)| {
                let avg_s = b.sum_s / b.count as f32;
                let score = (avg_s * 1.5 + 0.5) * (b.count as f32).sqrt();
                (idx, score)
            })
            .collect();

        scored_bins.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if scored_bins.is_empty() {
            // Primarily monochrome or grayscale cover
            if grayscale_bin.count > 0 {
                let avg_l = (grayscale_bin.sum_l / grayscale_bin.count as f32).clamp(0.16, 0.28);
                let c1 = Hsl { h: 220.0, s: 0.10, l: avg_l }.to_gpui_rgba(1.0);
                let c2 = Hsl { h: 220.0, s: 0.08, l: (avg_l * 0.65).clamp(0.10, 0.18) }.to_gpui_rgba(1.0);
                return (c1, c2);
            }
            return Self::default_palette();
        }

        // Primary dominant color
        let best_bin_idx = scored_bins[0].0;
        let b1 = &bins[best_bin_idx];
        let h1 = b1.sum_h / b1.count as f32;
        let s1 = (b1.sum_s / b1.count as f32).clamp(0.35, 0.65);
        let l1 = (b1.sum_l / b1.count as f32).clamp(0.18, 0.28);
        let c1 = Hsl { h: h1, s: s1, l: l1 }.to_gpui_rgba(1.0);

        // Secondary accent color: select another distinct hue bucket
        let c2 = if let Some(&(sec_idx, _)) = scored_bins.iter().skip(1).find(|(idx, _)| {
            let dist = (*idx as isize - best_bin_idx as isize).abs();
            let wrap_dist = dist.min(12 - dist);
            wrap_dist >= 2 // At least 60 degrees apart
        }) {
            let b2 = &bins[sec_idx];
            let h2 = b2.sum_h / b2.count as f32;
            let s2 = (b2.sum_s / b2.count as f32).clamp(0.25, 0.50);
            let l2 = (b2.sum_l / b2.count as f32).clamp(0.10, 0.20);
            Hsl { h: h2, s: s2, l: l2 }.to_gpui_rgba(1.0)
        } else {
            // Shift hue by +45 degrees and make moodier/darker
            let h2 = (h1 + 45.0) % 360.0;
            let s2 = (s1 * 0.75).clamp(0.25, 0.45);
            let l2 = (l1 * 0.65).clamp(0.10, 0.18);
            Hsl { h: h2, s: s2, l: l2 }.to_gpui_rgba(1.0)
        };

        (c1, c2)
    }

    /// Checks if the ambient blur image path for a given key is cached.
    pub fn get_ambient_image(&self, key: &str) -> Option<String> {
        let guard = self.ambient_cache.lock().unwrap();
        guard.get(key).cloned()
    }

    /// Asynchronously extracts the palette and generates the dynamic ambient blur image.
    pub async fn extract(&self, path_str: String) -> ((gpui::Rgba, gpui::Rgba), Option<String>) {
        let cached_pal = self.get_cached(&path_str);
        let cached_amb = self.get_ambient_image(&path_str);
        if let (Some(pal), amb) = (cached_pal, cached_amb) {
            return (pal, amb);
        }

        let cache = self.cache.clone();
        let ambient_cache = self.ambient_cache.clone();
        let path = PathBuf::from(path_str.clone());
        let key = path_str;

        let (palette, ambient_path) = tokio::task::spawn_blocking(move || {
            let pal = Self::extract_from_file_sync(&path);
            let ambient = Self::generate_and_cache_ambient(&path, pal);
            (pal, ambient)
        }).await.unwrap_or_else(|_| (Self::default_palette(), None));

        {
            let mut guard = cache.lock().unwrap();
            guard.insert(key.clone(), palette);
        }
        if let Some(ref amb) = ambient_path {
            let mut guard = ambient_cache.lock().unwrap();
            guard.insert(key, amb.clone());
        }

        (palette, ambient_path)
    }

    /// Generates and caches a butter-smooth Gaussian ambient blur texture on disk for an artwork.
    pub fn generate_and_cache_ambient(path: &Path, (c1, c2): (gpui::Rgba, gpui::Rgba)) -> Option<String> {
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        hasher.update(path.to_string_lossy().as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        let rel_path = format!("cache/ambient/{}.jpg", hash);
        let out_path = PathBuf::from(&rel_path);

        if out_path.exists() {
            return Some(rel_path);
        }

        if let Err(e) = Self::generate_ambient_blur(c1, c2, &out_path) {
            tracing::warn!("Failed to generate ambient blur image: {:?}", e);
            return None;
        }

        Some(rel_path)
    }

    /// Generates a genuine Gaussian-blurred dual-aura texture from primary and secondary colors.
    pub fn generate_ambient_blur(
        c1: gpui::Rgba,
        c2: gpui::Rgba,
        out_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let width = 320u32;
        let height = 200u32;
        let mut img = image::RgbImage::new(width, height);

        let c1_r = (c1.r * 255.0) as f32;
        let c1_g = (c1.g * 255.0) as f32;
        let c1_b = (c1.b * 255.0) as f32;

        let c2_r = (c2.r * 255.0) as f32;
        let c2_g = (c2.g * 255.0) as f32;
        let c2_b = (c2.b * 255.0) as f32;

        let cx1 = 70.0f32;
        let cy1 = 35.0f32;
        let rad1 = 110.0f32;

        let cx2 = 250.0f32;
        let cy2 = 40.0f32;
        let rad2 = 95.0f32;

        for y in 0..height {
            let yf = y as f32;
            for x in 0..width {
                let xf = x as f32;

                let d1 = ((xf - cx1).powi(2) + (yf - cy1).powi(2)).sqrt();
                let factor1 = if d1 < rad1 {
                    let t = 1.0 - (d1 / rad1);
                    t * t
                } else {
                    0.0
                };

                let d2 = ((xf - cx2).powi(2) + (yf - cy2).powi(2)).sqrt();
                let factor2 = if d2 < rad2 {
                    let t = 1.0 - (d2 / rad2);
                    t * t
                } else {
                    0.0
                };

                let r = (3.0 + c1_r * factor1 + c2_r * factor2).min(255.0) as u8;
                let g = (3.0 + c1_g * factor1 + c2_g * factor2).min(255.0) as u8;
                let b = (3.0 + c1_b * factor1 + c2_b * factor2).min(255.0) as u8;

                img.put_pixel(x, y, image::Rgb([r, g, b]));
            }
        }

        let blurred = image::imageops::blur(&img, 24.0);

        if let Some(parent) = out_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        blurred.save(out_path)?;
        Ok(())
    }

    /// Manually inserts a palette into the cache for a key.
    pub fn insert(&self, key: String, palette: (gpui::Rgba, gpui::Rgba)) {
        let mut guard = self.cache.lock().unwrap();
        guard.insert(key, palette);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsl_from_rgb_pure_colors() {
        // Red
        let red = Hsl::from_rgb(255, 0, 0);
        assert!((red.h - 0.0).abs() < 1e-3);
        assert!((red.s - 1.0).abs() < 1e-3);
        assert!((red.l - 0.5).abs() < 1e-3);

        // Green
        let green = Hsl::from_rgb(0, 255, 0);
        assert!((green.h - 120.0).abs() < 1e-3);
        assert!((green.s - 1.0).abs() < 1e-3);
        assert!((green.l - 0.5).abs() < 1e-3);

        // Blue
        let blue = Hsl::from_rgb(0, 0, 255);
        assert!((blue.h - 240.0).abs() < 1e-3);
        assert!((blue.s - 1.0).abs() < 1e-3);
        assert!((blue.l - 0.5).abs() < 1e-3);
    }

    #[test]
    fn test_hsl_to_gpui_rgba_conversion() {
        let hsl = Hsl { h: 0.0, s: 1.0, l: 0.5 };
        let rgba = hsl.to_gpui_rgba(1.0);
        assert!((rgba.r - 1.0).abs() < 1e-3);
        assert!(rgba.g.abs() < 1e-3);
        assert!(rgba.b.abs() < 1e-3);
        assert!((rgba.a - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_default_palette_clamping() {
        let (c1, c2) = PaletteCache::default_palette();
        // Luminance of c1 should be dark/moody (< 0.40) and non-zero
        assert!(c1.r > 0.05 && c1.r < 0.60);
        assert!(c2.b > 0.05 && c2.b < 0.40);
    }

    #[test]
    fn test_palette_cache_insertion() {
        let cache = PaletteCache::new();
        assert!(cache.get_cached("test_track").is_none());

        let (c1, c2) = PaletteCache::default_palette();
        cache.insert("test_track".to_string(), (c1, c2));

        let retrieved = cache.get_cached("test_track");
        assert!(retrieved.is_some());
        let (r1, r2) = retrieved.unwrap();
        assert_eq!(r1, c1);
        assert_eq!(r2, c2);
    }

    #[test]
    fn test_imageops_blur_compilation() {
        let img = image::RgbImage::new(16, 16);
        let blurred = image::imageops::blur(&img, 2.0);
        assert_eq!(blurred.width(), 16);
    }

    #[test]
    fn test_generate_ambient_blur() {
        let (c1, c2) = PaletteCache::default_palette();
        let out_dir = std::env::temp_dir().join("rustic_test_blur.jpg");
        let res = PaletteCache::generate_ambient_blur(c1, c2, &out_dir);
        assert!(res.is_ok());
        assert!(out_dir.exists());
        let _ = std::fs::remove_file(&out_dir);
    }
}
