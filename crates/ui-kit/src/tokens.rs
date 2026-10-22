//! UI Design System tokens: Palette, Typography, Spacing, Radii, Elevation, Motion.

use gpui::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreset {
    Noir,
    Cyberpunk,
    Nord,
    Emerald,
    Oled,
    LightStudio,
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Noir => "Noir & Brass",
            Self::Cyberpunk => "Cyberpunk Neon",
            Self::Nord => "Nordic Frost",
            Self::Emerald => "Emerald Forest",
            Self::Oled => "Midnight OLED",
            Self::LightStudio => "Studio Light",
        }
    }

    pub fn is_dark(&self) -> bool {
        !matches!(self, Self::LightStudio)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub background: Rgba,
    pub titlebar: Rgba,
    pub surface: Rgba,
    pub surface_hover: Rgba,
    pub surface_elevated: Rgba,
    pub primary: Rgba,
    pub primary_hover: Rgba,
    pub on_primary: Rgba,
    pub accent: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub border: Rgba,
    pub border_subtle: Rgba,
    pub close_hover: Rgba,
    pub close_active: Rgba,
}

impl Palette {
    pub fn for_preset(preset: ThemePreset, enable_blur: bool) -> Self {
        match preset {
            ThemePreset::Noir => Self::noir(enable_blur),
            ThemePreset::Cyberpunk => Self::cyberpunk(enable_blur),
            ThemePreset::Nord => Self::nord(enable_blur),
            ThemePreset::Emerald => Self::emerald(enable_blur),
            ThemePreset::Oled => Self::oled(enable_blur),
            ThemePreset::LightStudio => Self::light_studio(enable_blur),
        }
    }

    pub fn noir(enable_blur: bool) -> Self {
        let (bg, surf, title, border_subtle) = if enable_blur {
            (rgba(0x12131666), rgba(0x1E202866), rgba(0x0E0F1288), rgba(0xFFFFFF18))
        } else {
            (rgba(0x121316FF), rgba(0x181A1FFF), rgba(0x101114FF), rgba(0x1F232BFF))
        };
        Self {
            background: bg,
            titlebar: title,
            surface: surf,
            surface_hover: rgba(0x23262EFF),
            surface_elevated: rgba(0x282C35FF),
            primary: rgba(0xE5A93CFF),       // Warm brass gold
            primary_hover: rgba(0xF5BA4EFF),
            on_primary: rgba(0x000000FF),
            accent: rgba(0xD97706FF),
            text: rgba(0xF3F4F6FF),
            text_muted: rgba(0x9CA3AFFF),
            border: rgba(0x2A2E38FF),
            border_subtle,
            close_hover: rgba(0xE11D48FF),
            close_active: rgba(0xBE123CFF),
        }
    }

    pub fn cyberpunk(enable_blur: bool) -> Self {
        let (bg, surf, title, border_subtle) = if enable_blur {
            (rgba(0x0A081666), rgba(0x15102866), rgba(0x07051088), rgba(0xA855F730))
        } else {
            (rgba(0x0C0916FF), rgba(0x141024FF), rgba(0x090710FF), rgba(0x231F54FF))
        };
        Self {
            background: bg,
            titlebar: title,
            surface: surf,
            surface_hover: rgba(0x201A38FF),
            surface_elevated: rgba(0x2A224AFF),
            primary: rgba(0xA855F7FF),       // Vivid Neon Violet
            primary_hover: rgba(0xC084FCFF),
            on_primary: rgba(0xFFFFFFFF),
            accent: rgba(0x06B6D4FF),        // Electric Cyan
            text: rgba(0xFAF5FFFF),
            text_muted: rgba(0xA8A29EFF),
            border: rgba(0x312E81FF),
            border_subtle,
            close_hover: rgba(0xF43F5EFF),
            close_active: rgba(0xE11D48FF),
        }
    }

    pub fn nord(enable_blur: bool) -> Self {
        let (bg, surf, title, border_subtle) = if enable_blur {
            (rgba(0x20253070), rgba(0x2E344070), rgba(0x1A1E2688), rgba(0xECEFF418))
        } else {
            (rgba(0x242933FF), rgba(0x2E3440FF), rgba(0x1E222AFF), rgba(0x3B4252FF))
        };
        Self {
            background: bg,
            titlebar: title,
            surface: surf,
            surface_hover: rgba(0x3B4252FF),
            surface_elevated: rgba(0x434C5EFF),
            primary: rgba(0x88C0D0FF),       // Frost blue
            primary_hover: rgba(0x81A1C1FF),
            on_primary: rgba(0x1E222AFF),
            accent: rgba(0x5E81ACFF),
            text: rgba(0xECEFF4FF),
            text_muted: rgba(0xD8DEE9FF),
            border: rgba(0x4C566AFF),
            border_subtle,
            close_hover: rgba(0xBF616AFF),
            close_active: rgba(0xA8525BFF),
        }
    }

    pub fn emerald(enable_blur: bool) -> Self {
        let (bg, surf, title, border_subtle) = if enable_blur {
            (rgba(0x06120A66), rgba(0x0E201666), rgba(0x040D0788), rgba(0x10B98125))
        } else {
            (rgba(0x091610FF), rgba(0x10241AFF), rgba(0x07110CFF), rgba(0x133021FF))
        };
        Self {
            background: bg,
            titlebar: title,
            surface: surf,
            surface_hover: rgba(0x173827FF),
            surface_elevated: rgba(0x1F4732FF),
            primary: rgba(0x10B981FF),       // Radiant Emerald
            primary_hover: rgba(0x34D399FF),
            on_primary: rgba(0x041A0EFF),
            accent: rgba(0x059669FF),
            text: rgba(0xECFDF5FF),
            text_muted: rgba(0xA7F3D0FF),
            border: rgba(0x1F4732FF),
            border_subtle,
            close_hover: rgba(0xF43F5EFF),
            close_active: rgba(0xE11D48FF),
        }
    }

    pub fn oled(enable_blur: bool) -> Self {
        let (bg, surf, title, border_subtle) = if enable_blur {
            (rgba(0x00000088), rgba(0x08080888), rgba(0x00000099), rgba(0xFFFFFF20))
        } else {
            (rgba(0x000000FF), rgba(0x0A0A0AFF), rgba(0x000000FF), rgba(0x1A1A1AFF))
        };
        Self {
            background: bg,
            titlebar: title,
            surface: surf,
            surface_hover: rgba(0x1C1C1CFF),
            surface_elevated: rgba(0x262626FF),
            primary: rgba(0xFFFFFFFF),       // High contrast diamond white
            primary_hover: rgba(0xE5E5E5FF),
            on_primary: rgba(0x000000FF),
            accent: rgba(0xA855F7FF),
            text: rgba(0xFFFFFFFF),
            text_muted: rgba(0xA3A3A3FF),
            border: rgba(0x2E2E2EFF),
            border_subtle,
            close_hover: rgba(0xDC2626FF),
            close_active: rgba(0xB91C1CFF),
        }
    }

    pub fn light_studio(enable_blur: bool) -> Self {
        let (bg, surf, title, border_subtle) = if enable_blur {
            (rgba(0xF8FAFC77), rgba(0xFFFFFF77), rgba(0xF1F5F988), rgba(0x00000010))
        } else {
            (rgba(0xF8FAFCFF), rgba(0xFFFFFFFF), rgba(0xF1F5F9FF), rgba(0xE2E8F0FF))
        };
        Self {
            background: bg,
            titlebar: title,
            surface: surf,
            surface_hover: rgba(0xF1F5F9FF),
            surface_elevated: rgba(0xE2E8F0FF),
            primary: rgba(0x4F46E5FF),       // Vibrant modern Indigo
            primary_hover: rgba(0x4338CAFF),
            on_primary: rgba(0xFFFFFFFF),
            accent: rgba(0x06B6D4FF),
            text: rgba(0x0F172AFF),
            text_muted: rgba(0x64748BFF),
            border: rgba(0xE2E8F0FF),
            border_subtle,
            close_hover: rgba(0xE11D48FF),
            close_active: rgba(0xBE123CFF),
        }
    }

    pub fn dark() -> Self {
        Self::noir(false)
    }

    pub fn light() -> Self {
        Self::light_studio(false)
    }
}

pub struct TypeScale;
impl TypeScale {
    pub fn h1() -> f32 { 32.0 }
    pub fn h2() -> f32 { 24.0 }
    pub fn body() -> f32 { 15.0 }
    pub fn caption() -> f32 { 12.0 }
}

pub struct Spacing;
impl Spacing {
    pub fn s1() -> Pixels { px(4.0) }
    pub fn s2() -> Pixels { px(8.0) }
    pub fn s3() -> Pixels { px(12.0) }
    pub fn s4() -> Pixels { px(16.0) }
    pub fn s5() -> Pixels { px(24.0) }
    pub fn s6() -> Pixels { px(32.0) }
}

pub struct Radii;
impl Radii {
    pub fn small() -> Pixels { px(4.0) }
    pub fn base() -> Pixels { px(8.0) }
    pub fn large() -> Pixels { px(14.0) }
    pub fn full() -> Pixels { px(9999.0) }
}

pub struct Elevation;
impl Elevation {
    pub fn sm() -> gpui::BoxShadow {
        BoxShadow {
            color: rgba(0x0000001A).into(),
            offset: point(px(0.0), px(1.0)),
            blur_radius: px(2.0),
            spread_radius: px(0.0),
            inset: false,
        }
    }
    pub fn md() -> gpui::BoxShadow {
        BoxShadow {
            color: rgba(0x00000026).into(),
            offset: point(px(0.0), px(4.0)),
            blur_radius: px(6.0),
            spread_radius: px(0.0),
            inset: false,
        }
    }
    pub fn lg() -> gpui::BoxShadow {
        BoxShadow {
            color: rgba(0x00000055).into(),
            offset: point(px(0.0), px(12.0)),
            blur_radius: px(28.0),
            spread_radius: px(0.0),
            inset: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeConfig {
    pub preset: ThemePreset,
    pub enable_blur: bool,
    pub opacity: f32,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            preset: ThemePreset::Cyberpunk,
            enable_blur: true,
            opacity: 0.92,
        }
    }
}

/// A global theme providing the active Palette and configuration.
#[derive(Clone)]
pub struct Theme {
    pub preset: ThemePreset,
    pub palette: Palette,
    pub config: ThemeConfig,
}

impl Theme {
    pub fn new(preset: ThemePreset) -> Self {
        Self::from_config(ThemeConfig {
            preset,
            enable_blur: false,
            opacity: 1.0,
        })
    }

    pub fn from_config(config: ThemeConfig) -> Self {
        Self {
            preset: config.preset,
            palette: Palette::for_preset(config.preset, config.enable_blur),
            config,
        }
    }

    pub fn with_palette(palette: Palette) -> Self {
        Self {
            preset: ThemePreset::Noir,
            palette,
            config: ThemeConfig::default(),
        }
    }

    pub fn set_preset(&mut self, preset: ThemePreset) {
        self.config.preset = preset;
        self.preset = preset;
        self.palette = Palette::for_preset(preset, self.config.enable_blur);
    }

    pub fn set_blur(&mut self, enable: bool) {
        self.config.enable_blur = enable;
        self.palette = Palette::for_preset(self.preset, enable);
    }
}

impl Global for Theme {}
