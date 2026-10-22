use gpui::prelude::*;
use gpui::*;
use crate::atoms::{Text, TextVariant};

#[derive(IntoElement)]
pub struct Marquee {
    text: SharedString,
    // Typically requires animation frame state, but stubbed as static text for MVP.
}

impl Marquee {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
        }
    }
}

impl RenderOnce for Marquee {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        // A true marquee would use a translation transform animated over time,
        // and clip its contents. For now, it just renders standard text with overflow hiding.
        div()
            .overflow_hidden()
            .whitespace_nowrap()
            .child(
                Text::new(self.text)
                    .variant(TextVariant::Body)
            )
    }
}
