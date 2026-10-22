use gpui::prelude::*;
use gpui::*;
use crate::tokens::Radii;

#[derive(IntoElement)]
pub struct Artwork {
    image: Option<ImageSource>,
    size: Pixels,
    rounded: Pixels,
}

impl Artwork {
    pub fn new(size: impl Into<Pixels>) -> Self {
        Self {
            image: None,
            size: size.into(),
            rounded: Radii::base(),
        }
    }

    pub fn src(mut self, src: impl Into<ImageSource>) -> Self {
        self.image = Some(src.into());
        self
    }

    pub fn rounded(mut self, rounded: impl Into<Pixels>) -> Self {
        self.rounded = rounded.into();
        self
    }
}

impl RenderOnce for Artwork {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let container = div()
            .w(self.size)
            .h(self.size)
            .rounded(self.rounded)
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
                div()
                    .size_full()
                    .bg(cx.global::<crate::tokens::Theme>().palette.surface_hover)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        crate::icon::Icon::new("icons/music.svg")
                            .size(self.size / 3.0)
                            .color(cx.global::<crate::tokens::Theme>().palette.text_muted)
                    )
            )
        }
    }
}
