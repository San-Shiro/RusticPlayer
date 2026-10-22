use gpui::prelude::*;
use gpui::*;
use crate::tokens::{Theme, TypeScale};

#[derive(Default, Clone, Copy)]
pub enum TextVariant {
    #[default]
    Body,
    H1,
    H2,
    Caption,
}

#[derive(IntoElement)]
pub struct Text {
    variant: TextVariant,
    text: SharedString,
    color: Option<Rgba>,
    muted: bool,
}

impl Text {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            variant: TextVariant::Body,
            text: text.into(),
            color: None,
            muted: false,
        }
    }

    pub fn variant(mut self, variant: TextVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn color(mut self, color: Rgba) -> Self {
        self.color = Some(color);
        self
    }

    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }
}

impl RenderOnce for Text {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        let (size, weight) = match self.variant {
            TextVariant::H1 => (TypeScale::h1(), FontWeight::BOLD),
            TextVariant::H2 => (TypeScale::h2(), FontWeight::SEMIBOLD),
            TextVariant::Body => (TypeScale::body(), FontWeight::NORMAL),
            TextVariant::Caption => (TypeScale::caption(), FontWeight::NORMAL),
        };

        let text_color = if let Some(c) = self.color {
            c
        } else if self.muted {
            palette.text_muted
        } else {
            palette.text
        };

        div()
            .text_size(px(size))
            .font_weight(weight)
            .text_color(text_color)
            .child(self.text)
    }
}
