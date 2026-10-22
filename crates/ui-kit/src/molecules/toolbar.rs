use gpui::prelude::*;
use gpui::*;
use crate::atoms::{Button, ButtonVariant, Slider};
use crate::tokens::{Elevation, Spacing, Theme};

#[derive(IntoElement)]
pub struct Toolbar {
    is_playing: bool,
    progress: f32,
    volume: f32,
    on_play_pause: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_next: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_prev: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            progress: 0.0,
            volume: 1.0,
            on_play_pause: None,
            on_next: None,
            on_prev: None,
        }
    }

    pub fn is_playing(mut self, playing: bool) -> Self {
        self.is_playing = playing;
        self
    }

    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }

    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
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

impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        let play_label = if self.is_playing { "Pause" } else { "Play" };

        let mut play_btn = Button::new(play_label).variant(ButtonVariant::Primary);
        if let Some(on_play_pause) = self.on_play_pause {
            play_btn = play_btn.on_click(on_play_pause);
        }

        let mut prev_btn = Button::new("Prev").variant(ButtonVariant::Ghost);
        if let Some(on_prev) = self.on_prev {
            prev_btn = prev_btn.on_click(on_prev);
        }

        let mut next_btn = Button::new("Next").variant(ButtonVariant::Ghost);
        if let Some(on_next) = self.on_next {
            next_btn = next_btn.on_click(on_next);
        }

        div()
            .w_full()
            .h(px(80.0))
            .bg(palette.surface)
            .border_t_1()
            .border_color(palette.border)
            .shadow(vec![Elevation::md()])
            .flex()
            .items_center()
            .justify_between()
            .px(Spacing::s5())
            .child(
                // Left: Track Info Placeholder
                div()
                    .w(px(200.0))
                    .child("Track Info")
                    .text_color(palette.text)
            )
            .child(
                // Center: Controls
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(Spacing::s2())
                    .w(px(400.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(Spacing::s4())
                            .child(prev_btn)
                            .child(play_btn)
                            .child(next_btn)
                    )
                    .child(
                        Slider::new(self.progress)
                    )
            )
            .child(
                // Right: Volume Controls
                div()
                    .flex()
                    .items_center()
                    .w(px(200.0))
                    .child(
                        Slider::new(self.volume)
                    )
            )
    }
}
