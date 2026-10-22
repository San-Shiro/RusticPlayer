use gpui::prelude::*;
use gpui::*;
use crate::tokens::Radii;

#[derive(IntoElement)]
pub struct EqualizerBars {
    size: Pixels,
    color: Rgba,
    is_playing: bool,
}

impl EqualizerBars {
    pub fn new() -> Self {
        Self {
            size: px(16.0),
            color: rgb(0xD89A5B),
            is_playing: true,
        }
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Rgba) -> Self {
        self.color = color;
        self
    }

    pub fn is_playing(mut self, is_playing: bool) -> Self {
        self.is_playing = is_playing;
        self
    }
}

impl Default for EqualizerBars {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for EqualizerBars {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let bar_width = px(2.5);
        let max_height = self.size;
        
        let scales = if self.is_playing {
            // Self-animating smooth phase offsets via instant
            window.request_animation_frame();
            let elapsed = crate::anim_epoch().elapsed().as_secs_f32();
            let b1 = (0.35 + 0.60 * (elapsed * 6.0).sin().abs()).clamp(0.2, 1.0);
            let b2 = (0.30 + 0.65 * (elapsed * 8.5 + 1.2).sin().abs()).clamp(0.2, 1.0);
            let b3 = (0.25 + 0.70 * (elapsed * 5.2 + 2.5).sin().abs()).clamp(0.2, 1.0);
            let b4 = (0.35 + 0.60 * (elapsed * 7.1 + 3.8).sin().abs()).clamp(0.2, 1.0);
            [b1, b2, b3, b4]
        } else {
            [0.25, 0.25, 0.25, 0.25]
        };

        div()
            .flex()
            .items_end()
            .justify_between()
            .w(self.size)
            .h(self.size)
            .gap(px(2.0))
            .children(scales.iter().map(|&scale| {
                let h = max_height * scale;
                div()
                    .w(bar_width)
                    .h(h)
                    .bg(self.color)
                    .rounded(Radii::full())
            }))
    }
}
