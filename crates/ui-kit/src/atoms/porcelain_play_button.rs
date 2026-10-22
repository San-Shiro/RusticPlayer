use gpui::prelude::*;
use gpui::*;
use crate::icon::Icon;
use crate::tokens::Radii;

#[derive(IntoElement)]
pub struct PorcelainPlayButton {
    id: ElementId,
    is_playing: bool,
    is_buffering: bool,
    size: Pixels,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl PorcelainPlayButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            is_playing: false,
            is_buffering: false,
            size: px(52.0),
            on_click: None,
        }
    }

    pub fn is_playing(mut self, playing: bool) -> Self {
        self.is_playing = playing;
        self
    }

    pub fn is_buffering(mut self, buffering: bool) -> Self {
        self.is_buffering = buffering;
        self
    }

    pub fn size(mut self, size: Pixels) -> Self {
        self.size = size;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PorcelainPlayButton {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let icon_size = px((self.size.to_f64() * 0.44) as f32);
        
        let mut btn = div()
            .id(self.id)
            .size(self.size)
            .rounded(Radii::full())
            .bg(rgb(0xF7F4EF))
            .border_1()
            .border_color(rgba(0xFFFFFFD0))
            .shadow(vec![
                BoxShadow {
                    color: rgba(0x00000073).into(),
                    offset: point(px(0.0), px(4.0)),
                    blur_radius: px(12.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
                BoxShadow {
                    color: rgba(0x00000040).into(),
                    offset: point(px(0.0), px(1.0)),
                    blur_radius: px(3.0),
                    spread_radius: px(0.0),
                    inset: false,
                },
            ])
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .hover(|s| s.bg(rgb(0xFFFFFF)))
            .active(|s| s.bg(rgb(0xEDE9E1)));

        if let Some(h) = self.on_click {
            btn = btn.on_click(move |ev, win, cx| {
                h(ev, win, cx);
            });
        }

        if self.is_buffering {
            // Smooth pulse / frame animation when buffering
            window.request_animation_frame();
            let elapsed = crate::anim_epoch().elapsed().as_secs_f32();
            let pulse_opacity = 0.45 + 0.50 * (elapsed * 5.0).sin().abs();
            btn.child(
                div()
                    .size(icon_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .opacity(pulse_opacity)
                    .child(
                        Icon::new("icons/loader.svg")
                            .size(icon_size)
                            .color(rgb(0x141312))
                    )
            )
        } else if self.is_playing {
            btn.child(
                Icon::new("icons/pause.svg")
                    .size(icon_size)
                    .color(rgb(0x141312))
            )
        } else {
            // Play icon with slight optical centering offset
            btn.child(
                div()
                    .ml(px(2.5))
                    .child(
                        Icon::new("icons/play.svg")
                            .size(icon_size)
                            .color(rgb(0x141312))
                    )
            )
        }
    }
}
