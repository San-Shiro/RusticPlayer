use gpui::prelude::*;
use gpui::*;
use crate::icon::Icon;
use crate::tokens::{Radii, Theme};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IconButtonVariant {
    Primary,
    Ghost,
}

#[derive(IntoElement)]
pub struct IconButton {
    icon: SharedString,
    variant: IconButtonVariant,
    size: Pixels,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl IconButton {
    pub fn new(icon: impl Into<SharedString>) -> Self {
        Self {
            icon: icon.into(),
            variant: IconButtonVariant::Ghost,
            size: px(20.0),
            on_click: None,
        }
    }

    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        let (bg, hover_bg) = match self.variant {
            IconButtonVariant::Primary => (palette.primary, palette.primary_hover),
            IconButtonVariant::Ghost => (rgba(0x00000000), palette.surface_hover),
        };

        let icon_color = match self.variant {
            IconButtonVariant::Primary => palette.on_primary,
            IconButtonVariant::Ghost => palette.text,
        };

        let mut btn = div()
            .id(SharedString::from(format!("icon_btn_{}", self.icon)))
            .flex()
            .items_center()
            .justify_center()
            .w(self.size + px(16.0))
            .h(self.size + px(16.0))
            .rounded(Radii::full())
            .bg(bg)
            .hover(|s| s.bg(hover_bg))
            .active(|s| s.opacity(0.7))
            .cursor_pointer()
            .child(
                Icon::new(self.icon.clone())
                    .size(self.size)
                    .color(icon_color)
            );

        if let Some(h) = self.on_click {
            btn = btn.on_click(h);
        }

        btn
    }
}
