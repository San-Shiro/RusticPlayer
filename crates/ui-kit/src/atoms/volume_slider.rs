use gpui::prelude::*;
use gpui::*;
use crate::atoms::Slider;

#[derive(IntoElement)]
pub struct VolumeSlider {
    volume: f32, // 0.0 to 1.0
    on_change: Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>,
}

impl VolumeSlider {
    pub fn new(volume: f32) -> Self {
        Self {
            volume: volume.clamp(0.0, 1.0),
            on_change: None,
        }
    }

    pub fn on_change(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for VolumeSlider {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut slider = Slider::new(self.volume)
            .id("volume_slider_atom")
            .w_full()
            .h(px(18.0));

        if let Some(on_change) = self.on_change {
            slider = slider.on_change(move |v, win, app| on_change(v, win, app));
        }

        div()
            .w_full()
            .flex()
            .items_center()
            .child(slider)
    }
}
