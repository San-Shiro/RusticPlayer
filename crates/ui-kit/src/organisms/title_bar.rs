use gpui::prelude::*;
use gpui::*;
use crate::tokens::{Radii, Spacing, Theme};

#[derive(IntoElement)]
pub struct TitleBar {
    #[allow(dead_code)]
    title: SharedString,
    subtitle: Option<SharedString>,
    can_go_back: bool,
    show_player_toggle: bool,
    is_video_mode: bool,
    is_mobile: bool,
    on_home: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_toggle_video_mode: Option<Box<dyn Fn(bool, &mut Window, &mut App) + 'static>>,
    on_back: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_toggle_sidebar: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_toggle_theme: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_toggle_blur: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_toggle_debug: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_search_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    search_query: Option<SharedString>,
    is_search_active: bool,
    on_search_clear: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    is_logged_in: bool,
    account_name: Option<SharedString>,
    account_avatar: Option<SharedString>,
    account_handle: Option<SharedString>,
    on_sign_in: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl TitleBar {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            can_go_back: false,
            show_player_toggle: false,
            is_video_mode: false,
            is_mobile: false,
            on_home: None,
            on_toggle_video_mode: None,
            on_back: None,
            on_toggle_sidebar: None,
            on_toggle_theme: None,
            on_toggle_blur: None,
            on_toggle_debug: None,
            on_search_click: None,
            search_query: None,
            is_search_active: false,
            on_search_clear: None,
            is_logged_in: false,
            account_name: None,
            account_avatar: None,
            account_handle: None,
            on_sign_in: None,
        }
    }

    pub fn is_mobile(mut self, mobile: bool) -> Self {
        self.is_mobile = mobile;
        self
    }

    pub fn can_go_back(mut self, can: bool) -> Self {
        self.can_go_back = can;
        self
    }

    pub fn on_home(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_home = Some(Box::new(handler));
        self
    }

    pub fn on_back(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(Box::new(handler));
        self
    }

    pub fn subtitle(mut self, sub: impl Into<SharedString>) -> Self {
        self.subtitle = Some(sub.into());
        self
    }

    pub fn show_player_toggle(mut self, show: bool) -> Self {
        self.show_player_toggle = show;
        self
    }

    pub fn is_video_mode(mut self, is_video: bool) -> Self {
        self.is_video_mode = is_video;
        self
    }

    pub fn on_toggle_video_mode(mut self, handler: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_video_mode = Some(Box::new(handler));
        self
    }

    pub fn on_toggle_sidebar(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_sidebar = Some(Box::new(handler));
        self
    }

    pub fn on_toggle_theme(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_theme = Some(Box::new(handler));
        self
    }

    pub fn on_toggle_blur(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_blur = Some(Box::new(handler));
        self
    }

    pub fn on_toggle_debug(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_debug = Some(Box::new(handler));
        self
    }

    pub fn on_search_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_search_click = Some(Box::new(handler));
        self
    }

    pub fn search_query(mut self, q: impl Into<SharedString>) -> Self {
        self.search_query = Some(q.into());
        self
    }

    pub fn is_search_active(mut self, active: bool) -> Self {
        self.is_search_active = active;
        self
    }

    pub fn on_search_clear(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_search_clear = Some(Box::new(handler));
        self
    }

    pub fn is_logged_in(mut self, logged_in: bool) -> Self {
        self.is_logged_in = logged_in;
        self
    }

    pub fn account_name(mut self, name: impl Into<SharedString>) -> Self {
        self.account_name = Some(name.into());
        self
    }

    pub fn account_avatar(mut self, avatar: Option<impl Into<SharedString>>) -> Self {
        self.account_avatar = avatar.map(|a| a.into());
        self
    }

    pub fn account_handle(mut self, handle: Option<impl Into<SharedString>>) -> Self {
        self.account_handle = handle.map(|h| h.into());
        self
    }

    pub fn on_sign_in(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_sign_in = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for TitleBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;
        let is_maximized = window.is_maximized();
        let on_search_click = self.on_search_click.map(std::rc::Rc::new);
        let on_home = self.on_home.map(std::rc::Rc::new);

        // 1. Left Cluster: Hamburger, YTM Red Circular Badge + "Music", Nav Arrows
        let mut left_cluster = div()
            .flex()
            .items_center()
            .gap(Spacing::s3())
            .pl(px(16.0));

        // Hamburger Menu Toggle
        if let Some(h) = self.on_toggle_sidebar {
            left_cluster = left_cluster.child(
                div()
                    .id("header_hamburger_btn")
                    .occlude()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(3.0))
                    .size(px(32.0))
                    .p(px(6.0))
                    .rounded(Radii::full())
                    .cursor_pointer()
                    .hover(|s| s.bg(rgba(0xFFFFFF15)))
                    .on_click(h)
                    .child(div().w(px(18.0)).h(px(2.0)).bg(palette.text))
                    .child(div().w(px(18.0)).h(px(2.0)).bg(palette.text))
                    .child(div().w(px(18.0)).h(px(2.0)).bg(palette.text))
            );
        }

        // YouTube Music Brand: Authentic vinyl record badge + "Music" (Clickable -> Home)
        let mut logo_btn = div()
            .id("header_brand_logo")
            .occlude()
            .flex()
            .items_center()
            .gap(px(6.0))
            .cursor_pointer()
            .hover(|s| s.opacity(0.85))
            .child(
                crate::icon::Icon::new("icons/ytmusic_logo.svg")
                    .size(px(22.0))
            )
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgb(0xFFFFFF))
                    .child("Music")
            );

        if let Some(ref h) = on_home {
            let h = h.clone();
            logo_btn = logo_btn.on_click(move |ev, win, app| h(ev, win, app));
        }

        left_cluster = left_cluster.child(logo_btn);

        // Nav Arrows: Back & Forward
        let mut back_btn = div()
            .id("header_back_btn")
            .occlude()
            .size(px(26.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(if self.can_go_back { rgb(0xFFFFFF) } else { rgba(0xFFFFFF44) })
                    .child("‹")
            );

        if self.can_go_back {
            if let Some(h) = self.on_back {
                back_btn = back_btn.on_click(h);
            }
        }

        let forward_btn = div()
            .id("header_forward_btn")
            .occlude()
            .size(px(26.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(rgba(0xFFFFFF44))
                    .child("›")
            );

        left_cluster = left_cluster.child(
            div()
                .flex()
                .items_center()
                .gap(px(2.0))
                .child(back_btn)
                .child(forward_btn)
        );

        // 2. Center Section: Search Capsule (Always present, authentic YouTube Music design)
        let has_query = self.search_query.as_ref().map(|q| !q.is_empty()).unwrap_or(false);
        let mut search_capsule = div()
            .id("header_search_capsule")
            .occlude()
            .flex_1()
            .max_w(px(460.0))
            .h(px(32.0))
            .px(px(12.0))
            .rounded(Radii::full())
            .bg(if self.is_search_active { rgb(0x282828) } else { rgb(0x212121) })
            .border_1()
            .border_color(if self.is_search_active { rgb(0xFF0000) } else { rgba(0xFFFFFF14) })
            .flex()
            .items_center()
            .justify_between()
            .gap(px(10.0))
            .cursor_pointer()
            .hover(|s| s.bg(rgb(0x282828)))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .overflow_hidden()
                    .child(
                        crate::icon::Icon::new("icons/search.svg")
                            .size(px(15.0))
                            .color(if self.is_search_active { rgb(0xFF0000) } else { rgb(0xAAAAAA) })
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(if has_query { rgb(0xFFFFFF) } else { rgb(0x888888) })
                            .child(
                                self.search_query
                                    .clone()
                                    .unwrap_or_else(|| "Search songs, albums, artists, podcasts".into())
                            )
                    )
            );

        if has_query {
            if let Some(on_clear) = self.on_search_clear {
                search_capsule = search_capsule.child(
                    div()
                        .id("header_search_clear_btn")
                        .occlude()
                        .size(px(18.0))
                        .rounded(Radii::full())
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(|s| s.bg(rgba(0xFFFFFF20)))
                        .on_click(move |ev, win, app| on_clear(ev, win, app))
                        .child(
                            crate::icon::Icon::new("icons/x.svg")
                                .size(px(10.0))
                                .color(rgb(0xAAAAAA))
                        )
                );
            }
        }

        if let Some(ref h) = on_search_click {
            let h = h.clone();
            search_capsule = search_capsule.on_click(move |ev, win, app| h(ev, win, app));
        }

        let center_section = search_capsule.into_any_element();

        // 3. Right Cluster: Cast & Window Buttons
        let mut right_cluster = div()
            .flex()
            .items_center()
            .gap(Spacing::s2())
            .pr(px(8.0));

        // Sign-In / Account Button
        if self.is_logged_in {
            let initial = self
                .account_name
                .as_ref()
                .and_then(|n| n.chars().next())
                .unwrap_or('U')
                .to_uppercase()
                .to_string();
            let mut acc_btn = div()
                .id("header_account_btn")
                .occlude()
                .size(px(28.0))
                .rounded(Radii::full())
                .overflow_hidden()
                .bg(rgb(0x065FD4))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.opacity(0.85));

            if let Some(ref path) = self.account_avatar {
                acc_btn = acc_btn.child(
                    img(path.clone())
                        .size_full()
                        .object_fit(gpui::ObjectFit::Cover)
                );
            } else {
                acc_btn = acc_btn.child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xFFFFFF))
                        .child(initial)
                );
            }
            if let Some(h) = self.on_sign_in {
                acc_btn = acc_btn.on_click(h);
            }
            right_cluster = right_cluster.child(acc_btn);
        } else {
            let mut sign_in_btn = div()
                .id("header_sign_in_btn")
                .occlude()
                .h(px(28.0))
                .px(px(12.0))
                .rounded(Radii::full())
                .border_1()
                .border_color(rgba(0xFFFFFF20))
                .bg(rgba(0xFFFFFF08))
                .flex()
                .items_center()
                .gap(px(6.0))
                .cursor_pointer()
                .hover(|s| s.bg(rgba(0xFFFFFF18)))
                .child(
                    crate::icon::Icon::new("icons/user.svg")
                        .size(px(14.0))
                        .color(rgb(0x3EA6FF))
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0x3EA6FF))
                        .child("Sign in")
                );
            if let Some(h) = self.on_sign_in {
                sign_in_btn = sign_in_btn.on_click(h);
            }
            right_cluster = right_cluster.child(sign_in_btn);
        }

        // Cast Icon (Authentic Google Cast SVG)
        right_cluster = right_cluster.child(
            div()
                .id("header_cast_btn")
                .occlude()
                .size(px(30.0))
                .rounded(Radii::full())
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(|s| s.bg(rgba(0xFFFFFF15)))
                .child(
                    crate::icon::Icon::new("icons/cast.svg")
                        .size(px(17.0))
                        .color(rgb(0xCCCCCC))
                )
        );

        // Window buttons (Min, Max, Close)
        let min_btn = div()
            .id("window_min_btn")
            .flex()
            .items_center()
            .justify_center()
            .w(px(38.0))
            .h_full()
            .cursor_pointer()
            .occlude()
            .window_control_area(WindowControlArea::Min)
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .on_click(|_ev, window, _cx| {
                window.minimize_window();
            })
            .child(
                crate::icon::Icon::new("icons/minus.svg")
                    .size(px(11.0))
                    .color(palette.text_muted)
            );

        let max_icon = if is_maximized { "icons/minimize.svg" } else { "icons/maximize.svg" };
        let max_btn = div()
            .id("window_max_btn")
            .flex()
            .items_center()
            .justify_center()
            .w(px(38.0))
            .h_full()
            .cursor_pointer()
            .occlude()
            .window_control_area(WindowControlArea::Max)
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .on_click(|_ev, window, _cx| {
                window.zoom_window();
            })
            .child(
                crate::icon::Icon::new(max_icon)
                    .size(px(11.0))
                    .color(palette.text_muted)
            );

        let close_btn = div()
            .id("window_close_btn")
            .flex()
            .items_center()
            .justify_center()
            .w(px(42.0))
            .h_full()
            .cursor_pointer()
            .occlude()
            .window_control_area(WindowControlArea::Close)
            .hover(|s| s.bg(rgb(0xE81123)))
            .active(|s| s.bg(rgb(0xF1707A)))
            .on_click(|_ev, _window, cx| {
                cx.quit();
            })
            .child(
                crate::icon::Icon::new("icons/x.svg")
                    .size(px(12.0))
                    .color(palette.text_muted)
            );

        if self.is_mobile {
            let mut mob_logo = div()
                .id("mob_header_brand_logo")
                .occlude()
                .flex()
                .items_center()
                .gap(px(10.0))
                .cursor_pointer()
                .hover(|s| s.opacity(0.85))
                .child(
                    div()
                        .size(px(20.0))
                        .rounded(Radii::full())
                        .bg(rgb(0xFF0000))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .child("▶")
                        )
                )
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xFFFFFF))
                        .child("Music")
                );

            if let Some(ref h) = on_home {
                let h = h.clone();
                mob_logo = mob_logo.on_click(move |ev, win, app| h(ev, win, app));
            }

            return div()
                .id("ytm_mobile_header_bar")
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .h(px(46.0))
                .px(px(16.0))
                .bg(rgba(0x00000000))
                .window_control_area(WindowControlArea::Drag)
                .child(mob_logo)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(14.0))
                        .child(
                            crate::icon::Icon::new("icons/cast.svg")
                                .size(px(18.0))
                                .color(rgb(0xCCCCCC))
                        )
                        .child({
                            let mut mob_search = div()
                                .id("mob_search_btn")
                                .occlude()
                                .cursor_pointer()
                                .child(
                                    crate::icon::Icon::new("icons/search.svg")
                                        .size(px(18.0))
                                        .color(rgb(0xCCCCCC))
                                );
                            if let Some(ref h) = on_search_click {
                                let h = h.clone();
                                mob_search = mob_search.on_click(move |ev, win, app| h(ev, win, app));
                            }
                            mob_search
                        })
                );
        }

        div()
            .id("ytm_header_bar")
            .flex_shrink_0()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(46.0))
            .bg(rgba(0x00000000))
            .window_control_area(WindowControlArea::Drag)
            .on_click(|ev, window, _cx| {
                if ev.click_count() == 2 {
                    window.zoom_window();
                }
            })
            .child(left_cluster)
            .child(center_section)
            .child(
                div()
                    .flex()
                    .items_center()
                    .h_full()
                    .child(right_cluster)
                    .child(min_btn)
                    .child(max_btn)
                    .child(close_btn)
            )
    }
}
