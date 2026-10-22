use std::sync::Arc;
use std::time::Duration;
use gpui::prelude::*;
use gpui::*;
use crate::tokens::Radii;

/// Reference to a decoded video frame with timestamp and renderable image.
#[derive(Clone)]
pub struct VideoFrameRef {
    pub pts: Duration,
    pub image: Arc<RenderImage>,
}

/// Decoupled frame provider trait for video playback engines.
pub trait VideoFrameProvider: Send + Sync {
    fn latest_frame(&self, clock: Duration) -> Option<VideoFrameRef>;
}

#[derive(IntoElement)]
pub struct VideoSurface {
    provider: Option<Arc<dyn VideoFrameProvider>>,
    current_clock: Duration,
    fallback_artwork: Option<ImageSource>,
    width: Option<Pixels>,
    height: Option<Pixels>,
    rounded: Pixels,
}

impl VideoSurface {
    pub fn new(current_clock: Duration) -> Self {
        Self {
            provider: None,
            current_clock,
            fallback_artwork: None,
            width: None,
            height: None,
            rounded: Radii::base(),
        }
    }

    pub fn provider(mut self, provider: Arc<dyn VideoFrameProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn fallback_artwork(mut self, src: impl Into<ImageSource>) -> Self {
        self.fallback_artwork = Some(src.into());
        self
    }

    pub fn width(mut self, w: impl Into<Pixels>) -> Self {
        self.width = Some(w.into());
        self
    }

    pub fn height(mut self, h: impl Into<Pixels>) -> Self {
        self.height = Some(h.into());
        self
    }

    pub fn rounded(mut self, r: impl Into<Pixels>) -> Self {
        self.rounded = r.into();
        self
    }
}

impl RenderOnce for VideoSurface {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut frame_img: Option<Arc<RenderImage>> = None;
        if let Some(ref p) = self.provider {
            if let Some(frame) = p.latest_frame(self.current_clock) {
                frame_img = Some(frame.image);
            }
        }

        let mut container = div()
            .rounded(self.rounded)
            .overflow_hidden()
            .bg(cx.global::<crate::tokens::Theme>().palette.surface_hover);

        if let Some(w) = self.width {
            container = container.w(w);
        } else {
            container = container.w_full();
        }

        if let Some(h) = self.height {
            container = container.h(h);
        } else {
            container = container.h_full();
        }

        if let Some(render_img) = frame_img {
            container.child(
                img(ImageSource::Render(render_img))
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Cover)
            )
        } else if let Some(fallback) = self.fallback_artwork {
            container.child(
                img(fallback)
                    .w_full()
                    .h_full()
                    .object_fit(ObjectFit::Cover)
            )
        } else {
            container.child(
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        crate::icon::Icon::new("icons/music.svg")
                            .size(px(24.0))
                            .color(cx.global::<crate::tokens::Theme>().palette.text_muted)
                    )
            )
        }
    }
}
