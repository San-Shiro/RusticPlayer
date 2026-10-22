use gpui::prelude::*;
use gpui::*;
use crate::atoms::{Text, TextVariant};
use crate::tokens::{Radii, Spacing, Theme};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AudioSource {
    Local,
    YouTubeMusic,
    Spotify,
}

#[derive(IntoElement)]
pub struct SourceChip {
    source: AudioSource,
}

impl SourceChip {
    pub fn new(source: AudioSource) -> Self {
        Self { source }
    }

    fn bg_color(&self) -> Rgba {
        match self.source {
            AudioSource::Local => rgba(0x6200EEff), // Primary color placeholder
            AudioSource::YouTubeMusic => rgba(0xFF0000ff), // YTM Red
            AudioSource::Spotify => rgba(0x1DB954ff), // Spotify Green
        }
    }

    fn label(&self) -> &'static str {
        match self.source {
            AudioSource::Local => "Local",
            AudioSource::YouTubeMusic => "YTM",
            AudioSource::Spotify => "Spotify",
        }
    }
}

impl RenderOnce for SourceChip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;
        
        div()
            .bg(self.bg_color())
            .rounded(Radii::full())
            .px(Spacing::s2())
            .py(px(2.0))
            .child(
                Text::new(self.label())
                    .variant(TextVariant::Caption)
                    .color(palette.background) // White/dark depending on theme
            )
    }
}
