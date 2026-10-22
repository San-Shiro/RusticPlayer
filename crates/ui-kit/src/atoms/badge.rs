use gpui::prelude::*;
use gpui::*;
use crate::atoms::{Text, TextVariant};
use crate::tokens::{Radii, Spacing, Theme};

#[derive(IntoElement)]
pub struct Badge {
    text: SharedString,
}

impl Badge {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self { text: text.into() }
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;
        
        div()
            .bg(palette.primary)
            .rounded(Radii::full())
            .px(Spacing::s2())
            .py(px(2.0))
            .child(
                Text::new(self.text)
                    .variant(TextVariant::Caption)
                    .color(palette.background) // Contrast
            )
    }
}
