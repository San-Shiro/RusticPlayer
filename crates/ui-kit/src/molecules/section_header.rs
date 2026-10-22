use gpui::prelude::*;
use gpui::*;
use crate::atoms::{Button, ButtonVariant, Text, TextVariant};
use crate::tokens::{Spacing, Theme};

#[derive(IntoElement)]
pub struct SectionHeader {
    title: SharedString,
    show_all: bool,
    on_show_all: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl SectionHeader {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            show_all: true,
            on_show_all: None,
        }
    }

    pub fn show_all(mut self, show: bool) -> Self {
        self.show_all = show;
        self
    }

    pub fn on_show_all(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_show_all = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SectionHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        let mut container = div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .pb(Spacing::s2())
            .border_b_1()
            .border_color(palette.border)
            .child(
                Text::new(self.title)
                    .variant(TextVariant::H2)
                    .color(palette.text)
            );

        if self.show_all {
            let mut btn = Button::new("Show All").variant(ButtonVariant::Ghost);
            if let Some(h) = self.on_show_all {
                btn = btn.on_click(h);
            }
            container = container.child(btn);
        }

        container
    }
}
