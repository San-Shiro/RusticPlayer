use gpui::prelude::*;
use gpui::*;
use crate::icon::Icon;
use crate::tokens::{Radii, Theme};

struct NavItemDef {
    id: &'static str,
    label: &'static str,
    icon: &'static str,
}

const NAV_ITEMS: [NavItemDef; 4] = [
    NavItemDef { id: "Home", label: "Home", icon: "icons/music.svg" },
    NavItemDef { id: "Explore", label: "Explore", icon: "icons/disc.svg" },
    NavItemDef { id: "Library", label: "Library", icon: "icons/library.svg" },
    NavItemDef { id: "Settings", label: "Settings", icon: "icons/settings.svg" },
];

#[derive(IntoElement)]
pub struct Sidebar {
    active_item: SharedString,
    collapsed: bool,
    on_select: Option<Box<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>,
}

impl Sidebar {
    pub fn new(active_item: impl Into<SharedString>) -> Self {
        Self {
            active_item: active_item.into(),
            collapsed: false,
            on_select: None,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.collapsed = collapsed;
        self
    }

    pub fn on_select(mut self, handler: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Sidebar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let _theme = cx.global::<Theme>();
        let handler = self.on_select.map(std::rc::Rc::new);

        if self.collapsed {
            let mut list = div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(14.0))
                .pt(px(16.0));

            for item in NAV_ITEMS {
                let is_active = self.active_item.as_ref() == item.id;
                let id_shared: SharedString = item.id.into();
                let id_clone = id_shared.clone();

                let icon_color = if is_active {
                    rgb(0xFFFFFF)
                } else {
                    rgb(0xAAAAAA)
                };

                let text_color = if is_active {
                    rgb(0xFFFFFF)
                } else {
                    rgb(0xAAAAAA)
                };

                let bg = if is_active {
                    rgba(0xFFFFFF1A)
                } else {
                    rgba(0x00000000)
                };

                let mut btn = div()
                    .id(SharedString::from(format!("nav_item_{}", item.id)))
                    .w(px(64.0))
                    .h(px(54.0))
                    .rounded(Radii::base())
                    .bg(bg)
                    .hover(|s| s.bg(rgba(0xFFFFFF15)))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(4.0))
                    .cursor_pointer()
                    .child(
                        Icon::new(item.icon)
                            .size(px(22.0))
                            .color(icon_color)
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                            .text_color(text_color)
                            .child(item.label)
                    );

                if let Some(h) = handler.clone() {
                    btn = btn.on_click(move |_ev, window, app| {
                        h(id_clone.clone(), window, app);
                    });
                }

                list = list.child(btn);
            }

            return div()
                .id("ytm_slim_sidebar")
                .w(px(72.0))
                .h_full()
                .bg(rgba(0x00000000))
                .child(list);
        }

        // Expanded 240px Navigation Drawer
        let mut list = div()
            .flex()
            .flex_col()
            .w_full()
            .gap(px(4.0));

        for item in NAV_ITEMS {
            let is_active = self.active_item.as_ref() == item.id;
            let id_shared: SharedString = item.id.into();
            let id_clone = id_shared.clone();

            let icon_color = if is_active {
                rgb(0xFFFFFF)
            } else {
                rgb(0xAAAAAA)
            };

            let text_color = if is_active {
                rgb(0xFFFFFF)
            } else {
                rgb(0xAAAAAA)
            };

            let bg = if is_active {
                rgba(0xFFFFFF1A)
            } else {
                rgba(0x00000000)
            };

            let mut btn = div()
                .id(SharedString::from(format!("nav_drawer_item_{}", item.id)))
                .w_full()
                .h(px(44.0))
                .px(px(14.0))
                .rounded(Radii::base())
                .bg(bg)
                .hover(|s| s.bg(rgba(0xFFFFFF15)))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(16.0))
                .cursor_pointer()
                .child(
                    Icon::new(item.icon)
                        .size(px(20.0))
                        .color(icon_color)
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(if is_active { FontWeight::SEMIBOLD } else { FontWeight::NORMAL })
                        .text_color(text_color)
                        .child(item.label)
                );

            if let Some(h) = handler.clone() {
                btn = btn.on_click(move |_ev, window, app| {
                    h(id_clone.clone(), window, app);
                });
            }

            list = list.child(btn);
        }

        div()
            .id("ytm_expanded_sidebar")
            .w(px(240.0))
            .h_full()
            .bg(rgba(0x00000000))
            .flex()
            .flex_col()
            .p(px(12.0))
            .child(list)
    }
}

#[derive(IntoElement)]
pub struct BottomNav {
    active_item: SharedString,
    on_select: Option<Box<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>,
}

impl BottomNav {
    pub fn new(active_item: impl Into<SharedString>) -> Self {
        Self {
            active_item: active_item.into(),
            on_select: None,
        }
    }

    pub fn on_select(mut self, handler: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for BottomNav {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let handler = self.on_select.map(std::rc::Rc::new);
        let items = [
            ("Home", "icons/home.svg"),
            ("Explore", "icons/compass.svg"),
            ("Library", "icons/library.svg"),
        ];

        let mut row = div()
            .id("ytm_mobile_bottom_nav")
            .flex()
            .flex_row()
            .items_center()
            .justify_around()
            .w_full()
            .h(px(56.0))
            .bg(rgba(0x181818EE))
            .border_t_1()
            .border_color(rgba(0xFFFFFF15));

        for (label, icon) in items {
            let is_active = self.active_item.as_ref() == label;
            let id_shared: SharedString = label.into();
            let id_clone = id_shared.clone();

            let icon_color = if is_active { rgb(0xFFFFFF) } else { rgb(0x888888) };
            let text_color = if is_active { rgb(0xFFFFFF) } else { rgb(0x888888) };

            let mut btn = div()
                .id(SharedString::from(format!("bottom_nav_tab_{}", label)))
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(3.0))
                .flex_1()
                .h_full()
                .cursor_pointer()
                .child(Icon::new(icon).size(px(20.0)).color(icon_color))
                .child(
                    div()
                        .text_xs()
                        .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::NORMAL })
                        .text_color(text_color)
                        .child(label)
                );

            if let Some(h) = handler.clone() {
                btn = btn.on_click(move |_ev, window, app| {
                    h(id_clone.clone(), window, app);
                });
            }

            row = row.child(btn);
        }

        row
    }
}
