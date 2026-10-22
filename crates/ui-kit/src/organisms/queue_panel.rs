use gpui::prelude::*;
use gpui::*;
use crate::atoms::{EqualizerBars, Text, TextVariant};
use crate::icon::Icon;
use crate::tokens::{Radii, Spacing, Theme};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueueGroup {
    History,
    Current,
    UpNext,
    Suggestions,
}

#[derive(Clone, Debug)]
pub struct QueueItemViewData {
    pub id: SharedString,
    pub title: SharedString,
    pub artist: SharedString,
    pub duration: Option<SharedString>,
    pub thumbnail_path: Option<SharedString>,
    pub is_active: bool,
    pub lane: SharedString,
    pub group: QueueGroup,
}

#[derive(IntoElement)]
pub struct QueuePanel {
    items: Vec<QueueItemViewData>,
    on_select: Option<Box<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
    on_move_up: Option<Box<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
    on_move_down: Option<Box<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
    on_remove: Option<Box<dyn Fn(usize, &mut Window, &mut App) + 'static>>,
    on_close: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl QueuePanel {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            on_select: None,
            on_move_up: None,
            on_move_down: None,
            on_remove: None,
            on_close: None,
        }
    }

    pub fn items(mut self, items: Vec<QueueItemViewData>) -> Self {
        self.items = items;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }

    pub fn on_move_up(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_move_up = Some(Box::new(handler));
        self
    }

    pub fn on_move_down(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_move_down = Some(Box::new(handler));
        self
    }

    pub fn on_remove(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Box::new(handler));
        self
    }

    pub fn on_close(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }
}

impl Default for QueuePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for QueuePanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;
        let num_items = self.items.len();

        let on_select = self.on_select.map(std::rc::Rc::new);
        let on_move_up = self.on_move_up.map(std::rc::Rc::new);
        let on_move_down = self.on_move_down.map(std::rc::Rc::new);
        let on_remove = self.on_remove.map(std::rc::Rc::new);

        let mut list = div()
            .id("queue_scroll_list")
            .flex()
            .flex_col()
            .gap(Spacing::s1())
            .overflow_y_scroll()
            .flex_1();

        let mut current_section: Option<QueueGroup> = None;

        for (idx, item) in self.items.into_iter().enumerate() {
            if current_section != Some(item.group) {
                current_section = Some(item.group);
                let section_title = match item.group {
                    QueueGroup::History => "History",
                    QueueGroup::Current => "Playing Now",
                    QueueGroup::UpNext => "Up Next",
                    QueueGroup::Suggestions => "Autoplay Recommendations",
                };
                list = list.child(
                    div()
                        .pt(if idx == 0 { px(2.0) } else { px(12.0) })
                        .pb(px(4.0))
                        .px(Spacing::s2())
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x888888))
                        .child(section_title)
                );
            }

            let on_sel = on_select.clone();
            let on_up = on_move_up.clone();
            let on_dn = on_move_down.clone();
            let on_rem = on_remove.clone();

            let is_history = item.group == QueueGroup::History;
            let is_up_next = item.group == QueueGroup::UpNext;
            let is_suggestion = item.group == QueueGroup::Suggestions;

            let row_bg = if item.is_active {
                rgba(0xFFFFFF15)
            } else {
                rgba(0x00000000)
            };

            let row_opacity = if is_history { 0.55 } else { 1.0 };

            let mut row = div()
                .id(ElementId::Name(format!("queue_row_{}", idx).into()))
                .flex()
                .items_center()
                .justify_between()
                .p(Spacing::s2())
                .rounded(Radii::small())
                .bg(row_bg)
                .opacity(row_opacity)
                .hover(|s| s.bg(rgba(0xFFFFFF18)))
                .cursor_pointer();

            // Left side (Thumbnail + Title + Artist)
            let mut left = div()
                .id(ElementId::Name(format!("queue_left_{}", idx).into()))
                .flex()
                .items_center()
                .gap(Spacing::s3())
                .flex_1()
                .min_w_0();

            if let Some(h) = on_sel {
                left = left.on_click(move |_ev, win, app| {
                    h(idx, win, app);
                });
            }

            // Thumbnail or active playing indicator
            let thumb = if let Some(ref path) = item.thumbnail_path {
                div()
                    .size(px(40.0))
                    .rounded(Radii::small())
                    .overflow_hidden()
                    .flex_shrink_0()
                    .child(
                        img(path.clone())
                            .size_full()
                            .object_fit(ObjectFit::Cover)
                    )
            } else {
                div()
                    .size(px(40.0))
                    .rounded(Radii::small())
                    .bg(rgb(0x282828))
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_shrink_0()
                    .child(div().text_xs().text_color(rgb(0x717171)).child("🎵"))
            };

            left = left.child(thumb);

            left = left.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(1.0))
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(if item.is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                                    .text_color(if item.is_active { rgb(0xFFFFFF) } else { rgb(0xDDDDDD) })
                                    .truncate()
                                    .child(item.title)
                            )
                            .children(if item.is_active {
                                Some(EqualizerBars::new().size(px(12.0)).color(rgb(0x3EA6FF)))
                            } else {
                                None
                            })
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x888888))
                                    .truncate()
                                    .child(item.artist)
                            )
                            .child(div().text_xs().text_color(rgb(0x555555)).child("•"))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(if is_suggestion { rgb(0x00D084) } else { rgb(0x3EA6FF) })
                                    .child(item.lane)
                            )
                    )
            );

            row = row.child(left);

            // Right side (Move Up, Move Down, Remove controls for UpNext only)
            let mut right = div()
                .flex()
                .items_center()
                .gap(px(4.0))
                .flex_shrink_0();

            if is_up_next {
                if let Some(h) = on_up {
                    right = right.child(
                        div()
                            .id(ElementId::Name(format!("queue_up_{}", idx).into()))
                            .size(px(24.0))
                            .rounded(Radii::full())
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xFFFFFF20)))
                            .on_click(move |_ev, win, app| {
                                h(idx, win, app);
                            })
                            .child(Icon::new("icons/chevron-up.svg").size(px(14.0)).color(rgb(0xAAAAAA)))
                    );
                }

                if let Some(h) = on_dn {
                    right = right.child(
                        div()
                            .id(ElementId::Name(format!("queue_dn_{}", idx).into()))
                            .size(px(24.0))
                            .rounded(Radii::full())
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xFFFFFF20)))
                            .on_click(move |_ev, win, app| {
                                h(idx, win, app);
                            })
                            .child(Icon::new("icons/chevron-down.svg").size(px(14.0)).color(rgb(0xAAAAAA)))
                    );
                }

                if let Some(h) = on_rem {
                    right = right.child(
                        div()
                            .id(ElementId::Name(format!("queue_rem_{}", idx).into()))
                            .size(px(24.0))
                            .rounded(Radii::full())
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xFF000030)))
                            .on_click(move |_ev, win, app| {
                                h(idx, win, app);
                            })
                            .child(Icon::new("icons/x.svg").size(px(12.0)).color(rgb(0x888888)))
                    );
                }
            } else if is_suggestion {
                right = right.child(
                    div()
                        .px(Spacing::s2())
                        .py(px(2.0))
                        .rounded(Radii::full())
                        .bg(rgba(0x00D08420))
                        .text_xs()
                        .text_color(rgb(0x00D084))
                        .child("Auto")
                );
            }

            row = row.child(right);
            list = list.child(row);
        }

        let mut header = div()
            .flex()
            .items_center()
            .justify_between()
            .pb(Spacing::s3())
            .border_b_1()
            .border_color(rgba(0xFFFFFF15))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(Spacing::s2())
                    .child(
                        Text::new("Queue")
                            .variant(TextVariant::H2)
                    )
                    .child(
                        div()
                            .px(px(8.0))
                            .py(px(2.0))
                            .rounded(Radii::full())
                            .bg(rgba(0xFFFFFF15))
                            .text_xs()
                            .text_color(rgb(0xAAAAAA))
                            .child(format!("{} tracks", num_items))
                    )
            );

        if let Some(h) = self.on_close {
            header = header.child(
                div()
                    .id("queue_close_btn")
                    .size(px(28.0))
                    .rounded(Radii::full())
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgba(0xFFFFFF15)))
                    .on_click(h)
                    .child(Icon::new("icons/x.svg").size(px(14.0)).color(rgb(0xAAAAAA)))
            );
        }

        div()
            .w(px(360.0))
            .h_full()
            .bg(palette.surface)
            .border_l_1()
            .border_color(palette.border)
            .p(Spacing::s4())
            .flex()
            .flex_col()
            .child(header)
            .child(list)
    }
}
