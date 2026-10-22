use gpui::prelude::*;
use gpui::*;
use crate::molecules::section_header::SectionHeader;
use crate::tokens::Spacing;

#[derive(IntoElement)]
pub struct GalleryView {
    title: SharedString,
    children: Vec<AnyElement>,
    show_all: bool,
}

impl GalleryView {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            children: Vec::new(),
            show_all: true,
        }
    }

    pub fn show_all(mut self, show: bool) -> Self {
        self.show_all = show;
        self
    }
}

impl ParentElement for GalleryView {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for GalleryView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(Spacing::s4())
            .w_full()
            .child(SectionHeader::new(self.title.clone()).show_all(self.show_all))
            .child(
                div()
                    .id(self.title)
                    .flex()
                    .flex_row()
                    .gap(Spacing::s4())
                    .overflow_x_scroll() // Enable horizontal scrolling
                    .pb(Spacing::s4()) // Space for scrollbar or shadow
                    .children(self.children)
            )
    }
}
