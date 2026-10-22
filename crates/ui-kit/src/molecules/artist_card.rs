use gpui::prelude::*;
use gpui::*;
use crate::atoms::{Artwork, Text, TextVariant};
use crate::tokens::{Radii, Spacing, Theme};

#[derive(IntoElement)]
pub struct ArtistCard {
    name: SharedString,
    artwork: Option<ImageSource>,
}

impl ArtistCard {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            artwork: None,
        }
    }

    pub fn artwork(mut self, src: impl Into<ImageSource>) -> Self {
        self.artwork = Some(src.into());
        self
    }
}

impl RenderOnce for ArtistCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(Spacing::s3())
            .w(px(160.0))
            .p(Spacing::s2())
            .rounded(Radii::base())
            .hover(|s| s.bg(palette.surface_hover))
            .cursor_pointer()
            .child(
                // Artwork 1:1 ratio, fully rounded for artist
                if let Some(src) = self.artwork {
                    Artwork::new(px(144.0)).src(src).rounded(Radii::full())
                } else {
                    Artwork::new(px(144.0)).rounded(Radii::full())
                }
            )
            .child(
                Text::new(self.name)
                    .variant(TextVariant::Body)
                    .color(palette.text)
            )
    }
}
