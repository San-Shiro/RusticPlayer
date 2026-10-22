use gpui::prelude::*;
use gpui::*;
use crate::atoms::{Artwork, Text, TextVariant};
use crate::tokens::{Radii, Spacing, Theme};

#[derive(IntoElement)]
pub struct PlaylistCard {
    title: SharedString,
    artwork: Option<ImageSource>,
}

impl PlaylistCard {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            artwork: None,
        }
    }

    pub fn artwork(mut self, src: impl Into<ImageSource>) -> Self {
        self.artwork = Some(src.into());
        self
    }
}

impl RenderOnce for PlaylistCard {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        div()
            .flex()
            .flex_col()
            .gap(Spacing::s2())
            .w(px(160.0))
            .p(Spacing::s2())
            .rounded(Radii::base())
            .hover(|s| s.bg(palette.surface_hover))
            .cursor_pointer()
            .child(
                // Artwork 1:1 ratio
                if let Some(src) = self.artwork {
                    Artwork::new(px(144.0)).src(src).rounded(Radii::base())
                } else {
                    Artwork::new(px(144.0)).rounded(Radii::base())
                }
            )
            .child(
                Text::new(self.title)
                    .variant(TextVariant::Body)
                    .color(palette.text)
            )
    }
}
