use gpui::prelude::*;
use gpui::*;
use crate::tokens::{Elevation, Radii, Spacing, Theme, TypeScale};

#[derive(Default, Clone, Copy)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
}

#[derive(IntoElement)]
pub struct Button {
    variant: ButtonVariant,
    label: SharedString,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    width: Option<Length>,
}

impl Button {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            variant: ButtonVariant::Primary,
            label: label.into(),
            on_click: None,
            width: None,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn w_full(mut self) -> Self {
        self.width = Some(Length::Definite(DefiniteLength::Fraction(1.0)));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        let mut button_div = div()
            .flex()
            .items_center()
            .justify_center()
            .rounded(Radii::base())
            .px(Spacing::s4())
            .py(Spacing::s2())
            .cursor_pointer();

        if let Some(w) = self.width {
            button_div = button_div.w(w);
        }

        let button_div = match self.variant {
            ButtonVariant::Primary => button_div
                .bg(palette.primary)
                .text_color(palette.background)
                .hover(|s| s.bg(palette.primary_hover))
                .shadow(vec![Elevation::sm()]),
            ButtonVariant::Secondary => button_div
                .bg(palette.surface)
                .border(px(1.0))
                .border_color(palette.border)
                .text_color(palette.text)
                .hover(|s| s.bg(palette.surface_hover)),
            ButtonVariant::Ghost => button_div
                .bg(transparent_black())
                .text_color(palette.text)
                .hover(|s| s.bg(palette.surface_hover)),
        };

        let mut el = button_div.id(SharedString::from(format!("btn_{}", self.label))).child(
            div()
                .text_size(px(TypeScale::body()))
                .font_weight(FontWeight::MEDIUM)
                .child(self.label.clone()),
        );

        if let Some(on_click) = self.on_click {
            el = el.on_click(on_click);
        }

        el
    }
}
