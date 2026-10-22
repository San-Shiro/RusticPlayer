use gpui::prelude::*;
use gpui::*;
use crate::tokens::Theme;

#[derive(IntoElement)]
pub struct Icon {
    path: SharedString,
    color: Option<Rgba>,
    size: Option<Pixels>,
}

impl Icon {
    pub fn new(path: impl Into<SharedString>) -> Self {
        Self {
            path: path.into(),
            color: None,
            size: None,
        }
    }

    pub fn color(mut self, color: Rgba) -> Self {
        self.color = Some(color);
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;
        
        let color = self.color.unwrap_or(palette.text);
        let size = self.size.unwrap_or(px(16.0));

        svg()
            .path(self.path)
            .size(size)
            .text_color(color)
    }
}
