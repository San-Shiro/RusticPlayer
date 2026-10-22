use gpui::prelude::*;
use gpui::*;
use crate::tokens::{Radii, Theme};

#[derive(IntoElement)]
pub struct Skeleton {
    width: Length,
    height: Length,
    rounded: Pixels,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            width: Length::Definite(DefiniteLength::Fraction(1.0)),
            height: Length::Definite(DefiniteLength::Fraction(1.0)),
            rounded: Radii::base(),
        }
    }

    pub fn w(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn h(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn rounded(mut self, rounded: impl Into<Pixels>) -> Self {
        self.rounded = rounded.into();
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;
        
        div()
            .w(self.width)
            .h(self.height)
            .bg(palette.surface_hover) // Placeholder color
            .rounded(self.rounded)
    }
}
