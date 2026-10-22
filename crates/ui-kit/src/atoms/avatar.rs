use gpui::prelude::*;
use gpui::*;
use crate::atoms::{Text, TextVariant};
use crate::tokens::{Radii, Theme};

#[derive(IntoElement)]
pub struct Avatar {
    image: Option<ImageSource>,
    size: Pixels,
    fallback: SharedString,
}

impl Avatar {
    pub fn new(size: impl Into<Pixels>) -> Self {
        Self {
            image: None,
            size: size.into(),
            fallback: "?".into(),
        }
    }

    pub fn src(mut self, src: impl Into<ImageSource>) -> Self {
        self.image = Some(src.into());
        self
    }

    pub fn fallback(mut self, text: impl Into<SharedString>) -> Self {
        self.fallback = text.into();
        self
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        let container = div()
            .w(self.size)
            .h(self.size)
            .rounded(Radii::full())
            .bg(palette.surface_hover)
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden();

        if let Some(src) = self.image {
            container.child(
                img(src)
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Cover)
            )
        } else {
            container.child(
                Text::new(self.fallback)
                    .variant(TextVariant::Caption)
                    .color(palette.text_muted)
            )
        }
    }
}
