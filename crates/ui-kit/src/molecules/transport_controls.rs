use gpui::prelude::*;
use gpui::*;
use crate::tokens::Spacing;

#[derive(IntoElement)]
pub struct TransportControls {
    is_playing: bool,
    on_play_pause: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_next: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_prev: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl TransportControls {
    pub fn new(is_playing: bool) -> Self {
        Self {
            is_playing,
            on_play_pause: None,
            on_next: None,
            on_prev: None,
        }
    }

    pub fn on_play_pause(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_play_pause = Some(Box::new(handler));
        self
    }

    pub fn on_next(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_next = Some(Box::new(handler));
        self
    }

    pub fn on_prev(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_prev = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TransportControls {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let play_icon = if self.is_playing {
            "icons/pause.svg"
        } else {
            "icons/play.svg"
        };

        let mut play_btn = crate::atoms::IconButton::new(play_icon)
            .size(px(24.0))
            .variant(crate::atoms::IconButtonVariant::Primary);

        if let Some(h) = self.on_play_pause {
            play_btn = play_btn.on_click(h);
        }

        let mut prev_btn = crate::atoms::IconButton::new("icons/skip-back.svg")
            .size(px(20.0))
            .variant(crate::atoms::IconButtonVariant::Ghost);

        if let Some(h) = self.on_prev {
            prev_btn = prev_btn.on_click(h);
        }

        let mut next_btn = crate::atoms::IconButton::new("icons/skip-forward.svg")
            .size(px(20.0))
            .variant(crate::atoms::IconButtonVariant::Ghost);

        if let Some(h) = self.on_next {
            next_btn = next_btn.on_click(h);
        }

        div()
            .flex()
            .items_center()
            .gap(Spacing::s4())
            .child(prev_btn)
            .child(play_btn)
            .child(next_btn)
    }
}
