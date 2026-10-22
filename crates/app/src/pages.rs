use gpui::prelude::*;
use gpui::*;
use ui_kit::tokens::{Theme, ThemePreset, Palette, Spacing, Radii};
use ui_kit::atoms::{Button, ButtonVariant, Text, TextVariant, Artwork};
use rustic_core::api::{Command, CommandAction, PlaybackState};
use crate::engine_bridge::{AppState, Route, Loadable, PlayerTab, LibraryTab};

pub struct HomePage {
    app_state: Entity<AppState>,
    #[allow(dead_code)]
    last_album_count: usize,
    #[allow(dead_code)]
    last_artist_count: usize,
    selected_mood: String,
}

impl HomePage {
    pub fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        let (album_count, artist_count) = {
            let s = app_state.read(cx);
            (s.albums.len(), s.artists.len())
        };
        cx.observe(&app_state, |_this, _state, cx| {
            cx.notify();
        }).detach();
        Self {
            app_state,
            last_album_count: album_count,
            last_artist_count: artist_count,
            selected_mood: "".to_string(),
        }
    }
}

impl Render for HomePage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _palette = Palette::dark();
        let state = self.app_state.read(cx);

        // 1. Mood filter chips strip (Figma: 32px height, #212121 bg, 12px text)
        let chips_data: Vec<(String, Option<String>)> = match &state.home_feed {
            Loadable::Ready(feed) if !feed.chips.is_empty() => {
                feed.chips.iter().map(|c| (c.title.clone(), c.params.clone())).collect()
            }
            _ => vec![
                ("Podcasts".into(), None),
                ("Romance".into(), None),
                ("Feel good".into(), None),
                ("Relax".into(), None),
                ("Party".into(), None),
                ("Sad".into(), None),
                ("Energize".into(), None),
                ("Sleep".into(), None),
                ("Workout".into(), None),
                ("Commute".into(), None),
                ("Focus".into(), None),
            ],
        };

        let chips_row = div()
            .id("mood_chips_row")
            .flex()
            .flex_row()
            .items_center()
            .gap(Spacing::s2())
            .overflow_x_scroll()
            .py(px(4.0))
            .children(chips_data.into_iter().map(|(chip_title, chip_params)| {
                let is_selected = self.selected_mood == chip_title;
                let c_title = chip_title.clone();
                let c_params = chip_params.clone();
                div()
                    .id(SharedString::from(format!("chip_{}", chip_title)))
                    .px(px(14.0))
                    .py(px(6.0))
                    .rounded(Radii::full())
                    .border_1()
                    .border_color(if is_selected { rgba(0xFFFFFF33) } else { rgba(0xFFFFFF14) })
                    .bg(if is_selected { rgb(0x383838) } else { rgb(0x212121) })
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x303030)))
                    .on_click(cx.listener({
                        let c = c_title.clone();
                        let p = c_params.clone();
                        let app_state_clone = self.app_state.clone();
                        move |this, _ev, _win, cx| {
                            let (target_mood, target_params) = if this.selected_mood == c {
                                ("".to_string(), None)
                            } else {
                                (c.clone(), p.clone())
                            };
                            this.selected_mood = target_mood.clone();
                            app_state_clone.update(cx, |s, cx| {
                                s.load_home_feed_for_mood(&target_mood, target_params, cx);
                            });
                            cx.notify();
                        }
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if is_selected { rgb(0xFFFFFF) } else { rgb(0xEEEEEE) })
                            .child(chip_title)
                    )
            }));

        // 2. Exact Listen Again Cards from Figma
        let app_state_for_folklore = self.app_state.clone();
        let app_state_for_slowchill = self.app_state.clone();
        let app_state_for_player = self.app_state.clone();

        let listen_again_cards = div()
            .id("listen_again_cards_row")
            .flex()
            .flex_row()
            .gap(px(20.0))
            .overflow_x_scroll()
            // Card 1: folklore (Taylor Swift) -> opens Album View
            .child(
                div()
                    .id("card_folklore")
                    .flex()
                    .flex_col()
                    .w(px(220.0))
                    .cursor_pointer()
                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                        app_state_for_folklore.update(cx, |s, cx| {
                            s.open_album("MPREb_9u6q97B0kQ6", cx);
                        });
                    }))
                    .child(
                        div()
                            .size(px(220.0))
                            .rounded(px(8.0))
                            .overflow_hidden()
                            .child(
                                Artwork::new(px(220.0))
                                    .src(SharedString::from("assets/artwork/folklore.jpg"))
                                    .rounded(px(8.0))
                            )
                    )
                    .child(
                        div()
                            .mt(px(8.0))
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("folklore")
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .size(px(14.0))
                                            .rounded(px(2.0))
                                            .bg(rgb(0x444444))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child("E")
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0xAAAAAA))
                                            .child("Album • Taylor Swift")
                                    )
                            )
                    )
            )
            // Card 2: Cupid (Twin Version)
            .child(
                div()
                    .id("card_cupid")
                    .flex()
                    .flex_col()
                    .w(px(260.0))
                    .cursor_pointer()
                    .child(
                        div()
                            .relative()
                            .w(px(260.0))
                            .h(px(220.0))
                            .rounded(px(8.0))
                            .overflow_hidden()
                            .bg(rgb(0x1F1424))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Artwork::new(px(260.0))
                                    .src(SharedString::from("assets/artwork/golden_hour.jpg"))
                                    .rounded(px(8.0))
                            )
                            .child(
                                div()
                                    .absolute()
                                    .size(px(48.0))
                                    .rounded(Radii::full())
                                    .bg(rgba(0x00000099))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xFFFFFF))
                                            .child("▶")
                                    )
                            )
                            .child(
                                div()
                                    .absolute()
                                    .bottom(px(8.0))
                                    .left(px(8.0))
                                    .px(px(6.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .bg(rgba(0xBE185DCC))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(rgb(0xFFFFFF))
                                            .child("Cupid (Twin Ver.)")
                                    )
                            )
                    )
                    .child(
                        div()
                            .mt(px(8.0))
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("Cupid (Twin Version)")
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xAAAAAA))
                                    .child("FIFTY FIFTY • 94M views")
                            )
                    )
            )
            // Card 3: NOT CUTE ANYMORE
            .child(
                div()
                    .id("card_illit")
                    .flex()
                    .flex_col()
                    .w(px(180.0))
                    .cursor_pointer()
                    .child(
                        div()
                            .relative()
                            .w(px(180.0))
                            .h(px(220.0))
                            .rounded(px(8.0))
                            .overflow_hidden()
                            .bg(rgb(0x2A1526))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Artwork::new(px(180.0))
                                    .src(SharedString::from("assets/artwork/velvet_midnight.jpg"))
                                    .rounded(px(8.0))
                            )
                            .child(
                                div()
                                    .absolute()
                                    .size(px(44.0))
                                    .rounded(Radii::full())
                                    .bg(rgba(0x00000099))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xFFFFFF))
                                            .child("▶")
                                    )
                            )
                    )
                    .child(
                        div()
                            .mt(px(8.0))
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("NOT CUTE ANYMORE")
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xAAAAAA))
                                    .child("Song • ILLIT")
                            )
                    )
            )
            // Card 4: Barsaat
            .child(
                div()
                    .id("card_barsaat")
                    .flex()
                    .flex_col()
                    .w(px(260.0))
                    .cursor_pointer()
                    .child(
                        div()
                            .relative()
                            .w(px(260.0))
                            .h(px(220.0))
                            .rounded(px(8.0))
                            .overflow_hidden()
                            .bg(rgb(0x3B1212))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xEF4444))
                                    .child("बरसात")
                            )
                            .child(
                                div()
                                    .absolute()
                                    .size(px(48.0))
                                    .rounded(Radii::full())
                                    .bg(rgba(0x00000099))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xFFFFFF))
                                            .child("▶")
                                    )
                            )
                    )
                    .child(
                        div()
                            .mt(px(8.0))
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("Barsaat")
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xAAAAAA))
                                    .child("Banjaare & Roni • 48M views")
                            )
                    )
            )
            // Card 5: Night Song (Real)
            .child(
                div()
                    .id("card_nightsong")
                    .flex()
                    .flex_col()
                    .w(px(180.0))
                    .cursor_pointer()
                    .child(
                        div()
                            .relative()
                            .w(px(180.0))
                            .h(px(220.0))
                            .rounded(px(8.0))
                            .overflow_hidden()
                            .bg(rgb(0x111827))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                Artwork::new(px(180.0))
                                    .src(SharedString::from("assets/artwork/tokyo_1984.jpg"))
                                    .rounded(px(8.0))
                            )
                    )
                    .child(
                        div()
                            .mt(px(8.0))
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("Night Song (Real)")
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xAAAAAA))
                                    .child("Album • Nusrat Fateh Ali Khan")
                            )
                    )
            );

        // 3. Section "Forgotten favorites" from Figma
        let forgotten_favorites = div()
            .id("forgotten_favorites_row")
            .flex()
            .flex_row()
            .gap(px(20.0))
            .overflow_x_scroll()
            // Card 1: Slow Chill (opens Playlist View)
            .child(
                div()
                    .id("card_slow_chill")
                    .w(px(280.0))
                    .h(px(180.0))
                    .rounded(px(8.0))
                    .bg(rgb(0x1E1B4B))
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .justify_between()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x2E2A72)))
                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                        app_state_for_slowchill.update(cx, |s, cx| {
                            s.open_playlist(1, cx);
                        });
                    }))
                    .child(
                        div()
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgba(0xFFFFFF1F))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(div().text_sm().text_color(rgb(0xFFFFFF)).child("▶"))
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("Slow Chill"))
                            .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Playlist • Private • 2026"))
                    )
            )
            // Card 2: Rehbara (opens Fullscreen Player)
            .child(
                div()
                    .id("card_rehbara_player")
                    .w(px(280.0))
                    .h(px(180.0))
                    .rounded(px(8.0))
                    .bg(rgb(0x4A3B2C))
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .justify_between()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x5C4A38)))
                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                        app_state_for_player.update(cx, |s, cx| {
                            s.open_player(cx);
                        });
                    }))
                    .child(
                        div()
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgba(0xFFFFFF1F))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(div().text_sm().text_color(rgb(0xFFFFFF)).child("▶"))
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("रहबरा • Rehbara"))
                            .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Abhijeet Srivastava & Shayra Apoorva • 2026"))
                    )
            )
            .child(
                div()
                    .w(px(280.0))
                    .h(px(180.0))
                    .rounded(px(8.0))
                    .bg(rgb(0x1F2937))
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .justify_between()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x283548)))
                    .child(
                        div()
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgba(0xFFFFFF1F))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(div().text_sm().text_color(rgb(0xFFFFFF)).child("▶"))
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("Quiet Waves"))
                            .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Lofi • 2023"))
                    )
            )
            .child(
                div()
                    .w(px(280.0))
                    .h(px(180.0))
                    .rounded(px(8.0))
                    .bg(rgb(0x374151))
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .justify_between()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x434E62)))
                    .child(
                        div()
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgba(0xFFFFFF1F))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(div().text_sm().text_color(rgb(0xFFFFFF)).child("▶"))
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("Midnight City"))
                            .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Synth • 2022"))
                    )
            )
            .child(
                div()
                    .w(px(280.0))
                    .h(px(180.0))
                    .rounded(px(8.0))
                    .bg(rgb(0x4B5563))
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .justify_between()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x576374)))
                    .child(
                        div()
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgba(0xFFFFFF1F))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(div().text_sm().text_color(rgb(0xFFFFFF)).child("▶"))
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("Golden Nostalgia"))
                            .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Indie • 2021"))
                    )
            )
            .child(
                div()
                    .w(px(280.0))
                    .h(px(180.0))
                    .rounded(px(8.0))
                    .bg(rgb(0x1E293B))
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .justify_between()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x28374F)))
                    .child(
                        div()
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgba(0xFFFFFF1F))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(div().text_sm().text_color(rgb(0xFFFFFF)).child("▶"))
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("Rainy Day Chill"))
                            .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Acoustic • 2020"))
                    )
            );



        let mut home_container = div()
            .id("home-scroll")
            .flex()
            .flex_col()
            .size_full()
            .min_h_0()
            .bg(rgba(0x00000000))
            .p(px(24.0))
            .pb(px(108.0))
            .gap(px(28.0))
            .overflow_y_scroll()
            .child(
                div()
                    .relative()
                    .w_full()
                    .child(chips_row)
            );

        match &state.home_feed {
            Loadable::Ready(feed) if !feed.sections.is_empty() => {
                for (s_idx, section) in feed.sections.iter().enumerate() {
                    let section_title = section.title.clone();
                    let section_subtitle = section.subtitle.clone();

                    let section_row = div()
                        .id(SharedString::from(format!("feed_section_row_{}", s_idx)))
                        .flex()
                        .flex_row()
                        .gap(px(20.0))
                        .overflow_x_scroll()
                        .children(section.items.iter().enumerate().map(|(item_idx, item)| {
                            let app_state_clone = self.app_state.clone();
                            let item_title = item.title.clone();
                            let item_sub = item.subtitle.clone();
                            let thumb_url = item.thumbnail_url.clone();
                            let item_type = item.item_type;
                            let endpoint_id = item.endpoint_id.clone();
                            let is_explicit = item.is_explicit;

                            div()
                                .id(SharedString::from(format!("feed_item_{}_{}", s_idx, item_idx)))
                                .flex()
                                .flex_col()
                                .w(px(220.0))
                                .min_w(px(220.0))
                                .cursor_pointer()
                                .on_click(cx.listener({
                                    let endpoint_id = endpoint_id.clone();
                                    let item_title = item_title.clone();
                                    let item_sub = item_sub.clone();
                                    let thumb_url = thumb_url.clone();
                                    let sec_title = section_title.clone();
                                    move |_this, _ev, _win, cx| {
                                        app_state_clone.update(cx, |s, cx| {
                                            match item_type {
                                                sources::ShelfItemType::Track => {
                                                    s.play_youtube_track(
                                                        endpoint_id.clone(),
                                                        item_title.clone(),
                                                        item_sub.clone(),
                                                        thumb_url.clone(),
                                                        Some(sec_title.clone()),
                                                        cx,
                                                    );
                                                }
                                                sources::ShelfItemType::Album => {
                                                    s.open_album_detail(
                                                        endpoint_id.clone(),
                                                        item_title.clone(),
                                                        thumb_url.clone(),
                                                        cx,
                                                    );
                                                }
                                                sources::ShelfItemType::Artist => {
                                                    s.open_artist_detail(
                                                        endpoint_id.clone(),
                                                        item_title.clone(),
                                                        thumb_url.clone(),
                                                        cx,
                                                    );
                                                }
                                                _ => {
                                                    s.open_playlist_detail(
                                                        endpoint_id.clone(),
                                                        item_title.clone(),
                                                        thumb_url.clone(),
                                                        cx,
                                                    );
                                                }
                                            }
                                        });
                                    }
                                }))
                                .child(
                                    div()
                                        .relative()
                                        .size(px(220.0))
                                        .rounded(px(8.0))
                                        .overflow_hidden()
                                        .bg(rgb(0x1F1F1F))
                                        .child(
                                            if let Some(path) = state.thumbnail_manager.resolve_or_queue(&thumb_url, None) {
                                                Artwork::new(px(220.0))
                                                    .src(SharedString::from(path))
                                                    .rounded(px(8.0))
                                                    .into_any_element()
                                            } else {
                                                div()
                                                    .size_full()
                                                    .bg(rgb(0x282828))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        div()
                                                            .text_2xl()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(rgb(0x666666))
                                                            .child("♫")
                                                    )
                                                    .into_any_element()
                                            }
                                        )
                                        .child(
                                            div()
                                                .absolute()
                                                .size(px(44.0))
                                                .rounded(Radii::full())
                                                .bg(rgba(0x00000099))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .bottom(px(10.0))
                                                .right(px(10.0))
                                                .child(div().text_sm().text_color(rgb(0xFFFFFF)).child("▶"))
                                        )
                                )
                                .child(
                                    div()
                                        .mt(px(8.0))
                                        .flex()
                                        .flex_col()
                                        .gap(px(2.0))
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0xFFFFFF))
                                                .overflow_hidden()
                                                .child(item_title)
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(4.0))
                                                .children(if is_explicit {
                                                    Some(
                                                        div()
                                                            .size(px(14.0))
                                                            .rounded(px(2.0))
                                                            .bg(rgb(0x444444))
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(rgb(0xFFFFFF))
                                                                    .child("E")
                                                            )
                                                    )
                                                } else {
                                                    None
                                                })
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xAAAAAA))
                                                        .overflow_hidden()
                                                        .child(item_sub)
                                                )
                                        )
                                )
                        }));

                    home_container = home_container.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .children(section_subtitle.map(|sub| {
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(rgb(0xAAAAAA))
                                                    .child(sub)
                                            }))
                                            .child(
                                                div()
                                                    .text_2xl()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child(section_title)
                                            )
                                    )
                            )
                            .child(section_row)
                    );
                }
            }
            Loadable::Loading => {
                home_container = home_container.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(28.0))
                        .children((0..2).map(|_| {
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(16.0))
                                .child(
                                    div()
                                        .w(px(220.0))
                                        .h(px(24.0))
                                        .rounded(px(4.0))
                                        .bg(rgb(0x222222))
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap(px(20.0))
                                        .overflow_hidden()
                                        .children((0..5).map(|_| {
                                            div()
                                                .flex()
                                                .flex_col()
                                                .w(px(220.0))
                                                .min_w(px(220.0))
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .size(px(220.0))
                                                        .rounded(px(8.0))
                                                        .bg(rgb(0x1C1C1C))
                                                )
                                                .child(
                                                    div()
                                                        .w(px(160.0))
                                                        .h(px(16.0))
                                                        .rounded(px(4.0))
                                                        .bg(rgb(0x242424))
                                                )
                                                .child(
                                                    div()
                                                        .w(px(110.0))
                                                        .h(px(12.0))
                                                        .rounded(px(4.0))
                                                        .bg(rgb(0x1E1E1E))
                                                )
                                        }))
                                )
                        }))
                );
            }
            Loadable::Error(err) => {
                let app_state_retry = self.app_state.clone();
                home_container = home_container
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .p(px(12.0))
                            .rounded(px(8.0))
                            .bg(rgba(0xEF444422))
                            .border_1()
                            .border_color(rgba(0xEF444455))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xFCA5A5))
                                            .child("Live YouTube Music Feed Offline")
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0xAAAAAA))
                                            .child(err.clone())
                                    )
                            )
                            .child(
                                div()
                                    .id("retry_home_feed_btn")
                                    .px(px(14.0))
                                    .py(px(6.0))
                                    .rounded(Radii::full())
                                    .bg(rgb(0x333333))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x444444)))
                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                        app_state_retry.update(cx, |s, cx| {
                                            s.load_home_feed(cx);
                                        });
                                    }))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xFFFFFF))
                                            .child("Retry")
                                    )
                            )
                    )
                    // Fallback to offline / curated sections
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("Listen again")
                            )
                            .child(listen_again_cards)
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(14.0))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("Forgotten favorites")
                            )
                            .child(forgotten_favorites)
                    );
            }
            _ => {
                // Idle / Fallback
                home_container = home_container
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(12.0))
                                            .child(
                                                div()
                                                    .size(px(36.0))
                                                    .rounded(Radii::full())
                                                    .bg(rgb(0x0084C7))
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(rgb(0xFFFFFF))
                                                            .child("K")
                                                    )
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(FontWeight::SEMIBOLD)
                                                            .text_color(rgb(0xAAAAAA))
                                                            .child("KETAN")
                                                    )
                                                    .child(
                                                        div()
                                                            .text_2xl()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(rgb(0xFFFFFF))
                                                            .child("Listen again")
                                                    )
                                            )
                                    )
                            )
                            .child(listen_again_cards)
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(14.0))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("Forgotten favorites")
                            )
                            .child(forgotten_favorites)
                    );
            }
        }

        home_container
    }
}

pub struct LibraryPage {
    app_state: Entity<AppState>,
}

impl LibraryPage {
    pub fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        cx.observe(&app_state, |_this, _state, cx| {
            cx.notify();
        }).detach();
        Self {
            app_state,
        }
    }
}

impl Render for LibraryPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.global::<Theme>().palette;
        let state = self.app_state.read(cx);

        // Filter chips for YouTube Music library sections
        let tabs = [
            (LibraryTab::Playlists, "Playlists"),
            (LibraryTab::Songs, "Liked Songs"),
            (LibraryTab::Albums, "Albums"),
            (LibraryTab::Artists, "Artists"),
            (LibraryTab::Downloads, "Downloads"),
        ];

        let chips_row = div()
            .flex()
            .flex_row()
            .gap(px(10.0))
            .items_center()
            .children(tabs.iter().map(|(tab, label)| {
                let is_selected = state.library_tab == *tab;
                let t = *tab;
                div()
                    .id(SharedString::from(format!("lib_chip_{}", label)))
                    .px(px(16.0))
                    .py(px(6.0))
                    .rounded(Radii::full())
                    .cursor_pointer()
                    .bg(if is_selected { rgb(0xFFFFFF) } else { rgba(0xFFFFFF14) })
                    .border_1()
                    .border_color(if is_selected { rgb(0xFFFFFF) } else { rgba(0xFFFFFF1A) })
                    .hover(|s| if is_selected { s } else { s.bg(rgba(0xFFFFFF26)) })
                    .on_click(cx.listener(move |this, _ev, _win, cx| {
                        this.app_state.update(cx, |s, cx| {
                            s.set_library_tab(t, cx);
                        });
                    }))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(if is_selected { FontWeight::BOLD } else { FontWeight::NORMAL })
                            .text_color(if is_selected { rgb(0x0F0F0F) } else { rgb(0xF1F1F1) })
                            .child(*label)
                    )
            }));

        let content = match state.library_tab {
            LibraryTab::Playlists => {
                match &state.library_playlists {
                    Loadable::Loading => {
                        div().p(px(24.0)).child(Text::new("Loading playlists from YouTube Music...").variant(TextVariant::Body)).into_any_element()
                    }
                    Loadable::Error(err) => {
                        let app_state = self.app_state.clone();
                        div()
                            .p(px(24.0))
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(Text::new(format!("Failed to load playlists: {}", err)).variant(TextVariant::Body))
                            .child(
                                Button::new("Retry")
                                    .variant(ButtonVariant::Primary)
                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                        app_state.update(cx, |s, cx| s.load_library(cx));
                                    }))
                            )
                            .into_any_element()
                    }
                    Loadable::Ready(items) => {
                        if items.is_empty() {
                            div().p(px(24.0)).child(Text::new("No playlists found in your YouTube Music library.").variant(TextVariant::Body)).into_any_element()
                        } else {
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(20.0))
                                .children(items.iter().enumerate().map(|(idx, item)| {
                                    let app_state = self.app_state.clone();
                                    let id = item.id.clone();
                                    let title = item.title.clone();
                                    let thumb_url = item.thumbnail_url.clone();
                                    let track_count = item.track_count;
                                    let subtitle = if !item.subtitle.is_empty() {
                                        item.subtitle.clone()
                                    } else {
                                        track_count.map(|c| format!("{} tracks", c)).unwrap_or_else(|| "Playlist".into())
                                    };

                                    div()
                                        .id(SharedString::from(format!("lib_pl_{}", idx)))
                                        .flex()
                                        .flex_col()
                                        .w(px(200.0))
                                        .cursor_pointer()
                                        .on_click(cx.listener({
                                            let id = id.clone();
                                            let title = title.clone();
                                            let thumb_url = thumb_url.clone();
                                            move |_this, _ev, _win, cx| {
                                                app_state.update(cx, |s, cx| {
                                                    s.open_playlist_detail(id.clone(), title.clone(), thumb_url.clone(), cx);
                                                });
                                            }
                                        }))
                                        .child(
                                            div()
                                                .relative()
                                                .size(px(200.0))
                                                .rounded(px(8.0))
                                                .overflow_hidden()
                                                .bg(rgb(0x1F1F1F))
                                                .child(
                                                    if let Some(path) = state.thumbnail_manager.resolve_or_queue(&thumb_url, None) {
                                                        Artwork::new(px(200.0))
                                                            .src(SharedString::from(path))
                                                            .rounded(px(8.0))
                                                            .into_any_element()
                                                    } else {
                                                        div()
                                                            .size_full()
                                                            .bg(rgb(0x282828))
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .child(
                                                                div()
                                                                    .text_2xl()
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(rgb(0x666666))
                                                                    .child("♫")
                                                            )
                                                            .into_any_element()
                                                    }
                                                )
                                                .child(
                                                    div()
                                                        .absolute()
                                                        .size(px(40.0))
                                                        .rounded(Radii::full())
                                                        .bg(rgba(0x00000099))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .bottom(px(8.0))
                                                        .right(px(8.0))
                                                        .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("▶"))
                                                )
                                        )
                                        .child(
                                            div()
                                                .mt(px(8.0))
                                                .flex()
                                                .flex_col()
                                                .gap(px(2.0))
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0xFFFFFF))
                                                        .overflow_hidden()
                                                        .child(title)
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xAAAAAA))
                                                        .overflow_hidden()
                                                        .child(subtitle)
                                                )
                                        )
                                }))
                                .into_any_element()
                        }
                    }
                    _ => div().p(px(24.0)).child(Text::new("Ready to load playlists.").variant(TextVariant::Body)).into_any_element(),
                }
            }
            LibraryTab::Songs => {
                match &state.library_songs {
                    Loadable::Loading => {
                        div().p(px(24.0)).child(Text::new("Loading liked songs from YouTube Music...").variant(TextVariant::Body)).into_any_element()
                    }
                    Loadable::Error(err) => {
                        let app_state = self.app_state.clone();
                        div()
                            .p(px(24.0))
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(Text::new(format!("Failed to load liked songs: {}", err)).variant(TextVariant::Body))
                            .child(
                                Button::new("Retry")
                                    .variant(ButtonVariant::Primary)
                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                        app_state.update(cx, |s, cx| s.load_library(cx));
                                    }))
                            )
                            .into_any_element()
                    }
                    Loadable::Ready(songs) => {
                        if songs.is_empty() {
                            div().p(px(24.0)).child(Text::new("No liked songs found in your library.").variant(TextVariant::Body)).into_any_element()
                        } else {
                            let current_id = state.current_track.as_ref().map(|t| t.id.clone());
                            div()
                                .flex()
                                .flex_col()
                                .w_full()
                                .gap(px(4.0))
                                .children(songs.iter().enumerate().map(|(idx, track)| {
                                    let app_state = self.app_state.clone();
                                    let track_clone = track.clone();
                                    let is_active = current_id.as_ref() == Some(&track.id);
                                    let dur_secs = track.duration_secs;
                                    let dur_str = format!("{}:{:02}", dur_secs / 60, dur_secs % 60);
                                    let thumb_url = track.artwork_url.clone();
                                    let artists_str = track.artists.join(", ");

                                    div()
                                        .id(SharedString::from(format!("lib_song_{}", idx)))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px(px(12.0))
                                        .py(px(8.0))
                                        .rounded(px(6.0))
                                        .bg(if is_active { rgba(0xFFFFFF1A) } else { rgba(0x00000000) })
                                        .hover(|s| s.bg(rgba(0xFFFFFF14)))
                                        .cursor_pointer()
                                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                            let t = track_clone.clone();
                                            app_state.update(cx, |s, cx| {
                                                s.play_track_obj(t, cx);
                                            });
                                        }))
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(12.0))
                                                .flex_1()
                                                .child(
                                                    div()
                                                        .w(px(28.0))
                                                        .text_sm()
                                                        .text_color(if is_active { palette.primary } else { rgb(0x888888) })
                                                        .child(if is_active { "▶".to_string() } else { format!("{}", idx + 1) })
                                                )
                                                .child(
                                                    div()
                                                        .size(px(40.0))
                                                        .rounded(px(4.0))
                                                        .overflow_hidden()
                                                        .bg(rgb(0x1F1F1F))
                                                        .child(
                                                            if let Some(path) = state.thumbnail_manager.resolve_or_queue(&thumb_url, None) {
                                                                Artwork::new(px(40.0))
                                                                    .src(SharedString::from(path))
                                                                    .rounded(px(4.0))
                                                                    .into_any_element()
                                                            } else {
                                                                div().size_full().bg(rgb(0x282828)).into_any_element()
                                                            }
                                                        )
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_color(if is_active { palette.primary } else { rgb(0xFFFFFF) })
                                                                .child(track.title.clone())
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(0xAAAAAA))
                                                                .child(artists_str)
                                                        )
                                                )
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(24.0))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x888888))
                                                        .child(track.album.clone())
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xAAAAAA))
                                                        .child(dur_str)
                                                )
                                        )
                                }))
                                .when(state.library_songs_continuation.is_some(), |this| {
                                    let app_state_more = self.app_state.clone();
                                    let is_loading_more = state.is_loading_more_songs;
                                    this.child(
                                        div()
                                            .flex()
                                            .justify_center()
                                            .pt(px(16.0))
                                            .pb(px(24.0))
                                            .child(
                                                div()
                                                    .id("lib_load_more_btn")
                                                    .px(px(24.0))
                                                    .py(px(8.0))
                                                    .rounded(Radii::full())
                                                    .bg(if is_loading_more { rgb(0x222222) } else { rgba(0xFFFFFF14) })
                                                    .border_1()
                                                    .border_color(rgba(0xFFFFFF1A))
                                                    .cursor_pointer()
                                                    .hover(|s| s.bg(rgba(0xFFFFFF26)))
                                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                                        app_state_more.update(cx, |s, cx| {
                                                            s.load_more_library_songs(cx);
                                                        });
                                                    }))
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(rgb(0xFFFFFF))
                                                            .child(if is_loading_more { "Loading more..." } else { "Load More" })
                                                    )
                                            )
                                    )
                                })
                                .into_any_element()
                        }
                    }
                    _ => div().p(px(24.0)).child(Text::new("Ready to load liked songs.").variant(TextVariant::Body)).into_any_element(),
                }
            }
            LibraryTab::Albums => {
                match &state.library_albums {
                    Loadable::Loading => {
                        div().p(px(24.0)).child(Text::new("Loading albums from YouTube Music...").variant(TextVariant::Body)).into_any_element()
                    }
                    Loadable::Error(err) => {
                        let app_state = self.app_state.clone();
                        div()
                            .p(px(24.0))
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(Text::new(format!("Failed to load albums: {}", err)).variant(TextVariant::Body))
                            .child(
                                Button::new("Retry")
                                    .variant(ButtonVariant::Primary)
                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                        app_state.update(cx, |s, cx| s.load_library(cx));
                                    }))
                            )
                            .into_any_element()
                    }
                    Loadable::Ready(items) => {
                        if items.is_empty() {
                            div().p(px(24.0)).child(Text::new("No albums found in your YouTube Music library.").variant(TextVariant::Body)).into_any_element()
                        } else {
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(20.0))
                                .children(items.iter().enumerate().map(|(idx, item)| {
                                    let app_state = self.app_state.clone();
                                    let id = item.id.clone();
                                    let title = item.title.clone();
                                    let thumb_url = item.thumbnail_url.clone();
                                    let subtitle = if !item.subtitle.is_empty() {
                                        item.subtitle.clone()
                                    } else {
                                        "Album".into()
                                    };

                                    div()
                                        .id(SharedString::from(format!("lib_al_{}", idx)))
                                        .flex()
                                        .flex_col()
                                        .w(px(200.0))
                                        .cursor_pointer()
                                        .on_click(cx.listener({
                                            let id = id.clone();
                                            let title = title.clone();
                                            let thumb_url = thumb_url.clone();
                                            move |_this, _ev, _win, cx| {
                                                app_state.update(cx, |s, cx| {
                                                    s.open_album_detail(id.clone(), title.clone(), thumb_url.clone(), cx);
                                                });
                                            }
                                        }))
                                        .child(
                                            div()
                                                .relative()
                                                .size(px(200.0))
                                                .rounded(px(8.0))
                                                .overflow_hidden()
                                                .bg(rgb(0x1F1F1F))
                                                .child(
                                                    if let Some(path) = state.thumbnail_manager.resolve_or_queue(&thumb_url, None) {
                                                        Artwork::new(px(200.0))
                                                            .src(SharedString::from(path))
                                                            .rounded(px(8.0))
                                                            .into_any_element()
                                                    } else {
                                                        div()
                                                            .size_full()
                                                            .bg(rgb(0x282828))
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .child(
                                                                div()
                                                                    .text_2xl()
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(rgb(0x666666))
                                                                    .child("♫")
                                                            )
                                                            .into_any_element()
                                                    }
                                                )
                                                .child(
                                                    div()
                                                        .absolute()
                                                        .size(px(40.0))
                                                        .rounded(Radii::full())
                                                        .bg(rgba(0x00000099))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .bottom(px(8.0))
                                                        .right(px(8.0))
                                                        .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("▶"))
                                                )
                                        )
                                        .child(
                                            div()
                                                .mt(px(8.0))
                                                .flex()
                                                .flex_col()
                                                .gap(px(2.0))
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0xFFFFFF))
                                                        .overflow_hidden()
                                                        .child(title)
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xAAAAAA))
                                                        .overflow_hidden()
                                                        .child(subtitle)
                                                )
                                        )
                                }))
                                .into_any_element()
                        }
                    }
                    _ => div().p(px(24.0)).child(Text::new("Ready to load albums.").variant(TextVariant::Body)).into_any_element(),
                }
            }
            LibraryTab::Artists => {
                match &state.library_artists {
                    Loadable::Loading => {
                        div().p(px(24.0)).child(Text::new("Loading artists from YouTube Music...").variant(TextVariant::Body)).into_any_element()
                    }
                    Loadable::Error(err) => {
                        let app_state = self.app_state.clone();
                        div()
                            .p(px(24.0))
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(Text::new(format!("Failed to load artists: {}", err)).variant(TextVariant::Body))
                            .child(
                                Button::new("Retry")
                                    .variant(ButtonVariant::Primary)
                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                        app_state.update(cx, |s, cx| s.load_library(cx));
                                    }))
                            )
                            .into_any_element()
                    }
                    Loadable::Ready(artists) => {
                        if artists.is_empty() {
                            div().p(px(24.0)).child(Text::new("No followed artists found in your library.").variant(TextVariant::Body)).into_any_element()
                        } else {
                            div()
                                .flex()
                                .flex_wrap()
                                .gap(px(24.0))
                                .children(artists.iter().enumerate().map(|(idx, artist)| {
                                    let app_state = self.app_state.clone();
                                    let id = artist.id.clone();
                                    let name = artist.name.clone();
                                    let thumb_url = artist.thumbnail_url.clone();
                                    let sub_str = artist.subscribers.clone().unwrap_or_else(|| "Artist".into());

                                    div()
                                        .id(SharedString::from(format!("lib_art_{}", idx)))
                                        .flex()
                                        .flex_col()
                                        .w(px(160.0))
                                        .items_center()
                                        .cursor_pointer()
                                        .on_click(cx.listener({
                                            let id = id.clone();
                                            let name = name.clone();
                                            let thumb_url = thumb_url.clone();
                                            move |_this, _ev, _win, cx| {
                                                app_state.update(cx, |s, cx| {
                                                    s.open_artist_detail(id.clone(), name.clone(), thumb_url.clone(), cx);
                                                });
                                            }
                                        }))
                                        .child(
                                            div()
                                                .relative()
                                                .size(px(160.0))
                                                .rounded(Radii::full())
                                                .overflow_hidden()
                                                .bg(rgb(0x1F1F1F))
                                                .child(
                                                    if let Some(path) = state.thumbnail_manager.resolve_or_queue(&thumb_url, None) {
                                                        Artwork::new(px(160.0))
                                                            .src(SharedString::from(path))
                                                            .rounded(Radii::full())
                                                            .into_any_element()
                                                    } else {
                                                        div()
                                                            .size_full()
                                                            .bg(rgb(0x282828))
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .child(
                                                                div()
                                                                    .text_2xl()
                                                                    .font_weight(FontWeight::BOLD)
                                                                    .text_color(rgb(0x666666))
                                                                    .child("👤")
                                                            )
                                                            .into_any_element()
                                                    }
                                                )
                                                .child(
                                                    div()
                                                        .absolute()
                                                        .size(px(36.0))
                                                        .rounded(Radii::full())
                                                        .bg(rgba(0x00000099))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .bottom(px(6.0))
                                                        .right(px(6.0))
                                                        .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("▶"))
                                                )
                                        )
                                        .child(
                                            div()
                                                .mt(px(8.0))
                                                .flex()
                                                .flex_col()
                                                .items_center()
                                                .gap(px(2.0))
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0xFFFFFF))
                                                        .overflow_hidden()
                                                        .child(name)
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xAAAAAA))
                                                        .overflow_hidden()
                                                        .child(sub_str)
                                                )
                                        )
                                }))
                                .into_any_element()
                        }
                    }
                    _ => div().p(px(24.0)).child(Text::new("Ready to load artists.").variant(TextVariant::Body)).into_any_element(),
                }
            }
            LibraryTab::Downloads => {
                if state.library_downloads.is_empty() {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .py(px(60.0))
                        .gap(px(12.0))
                        .child(div().text_3xl().child("📥"))
                        .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("No offline downloads yet"))
                        .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Songs, albums, and playlists you download will appear here for instant offline playback."))
                        .into_any_element()
                } else {
                    let (total_tracks, total_bytes) = {
                        let dm = state.download_manager.read().unwrap();
                        (dm.get_total_tracks(), dm.get_total_bytes())
                    };
                    let size_mb = (total_bytes as f64) / (1024.0 * 1024.0);
                    let current_id = state.current_track.as_ref().map(|t| t.id.clone());

                    let app_state_play_all = self.app_state.clone();
                    let app_state_shuffle_all = self.app_state.clone();

                    let header_bar = div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pb(px(16.0))
                        .border_b_1()
                        .border_color(rgba(0xFFFFFF14))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xFFFFFF))
                                        .child(format!("{} Downloaded Tracks", total_tracks))
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x22C55E))
                                        .child(format!("{:.1} MB stored offline · Zero network latency", size_mb))
                                )
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .id("dl_btn_play_all")
                                        .px(px(16.0))
                                        .py(px(6.0))
                                        .rounded(Radii::full())
                                        .bg(rgb(0xFFFFFF))
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgb(0xEEEEEE)))
                                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                            app_state_play_all.update(cx, |s, cx| {
                                                s.play_downloaded_tracks(false, 0, cx);
                                            });
                                        }))
                                        .child(div().text_xs().text_color(rgb(0x000000)).child("▶"))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0x000000))
                                                .child("Play All")
                                        )
                                )
                                .child(
                                    div()
                                        .id("dl_btn_shuffle_all")
                                        .px(px(14.0))
                                        .py(px(6.0))
                                        .rounded(Radii::full())
                                        .bg(rgba(0xFFFFFF14))
                                        .border_1()
                                        .border_color(rgba(0xFFFFFF1A))
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgba(0xFFFFFF26)))
                                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                            app_state_shuffle_all.update(cx, |s, cx| {
                                                s.play_downloaded_tracks(true, 0, cx);
                                            });
                                        }))
                                        .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("🔀"))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0xFFFFFF))
                                                .child("Shuffle")
                                        )
                                )
                        );

                    let list = div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .gap(px(4.0))
                        .children(state.library_downloads.iter().enumerate().map(|(idx, track)| {
                            let app_state = self.app_state.clone();
                            let app_state_del = self.app_state.clone();
                            let track_id_del = track.id.clone();
                            let is_active = current_id.as_ref() == Some(&track.id);
                            let dur_secs = track.duration_secs;
                            let dur_str = format!("{}:{:02}", dur_secs / 60, dur_secs % 60);
                            let thumb_url = track.artwork_url.clone();
                            let artists_str = track.artists.join(", ");

                            div()
                                .id(SharedString::from(format!("lib_dl_{}", idx)))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px(px(12.0))
                                .py(px(8.0))
                                .rounded(px(6.0))
                                .bg(if is_active { rgba(0xFFFFFF1A) } else { rgba(0x00000000) })
                                .hover(|s| s.bg(rgba(0xFFFFFF14)))
                                .cursor_pointer()
                                .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                    app_state.update(cx, |s, cx| {
                                        s.play_downloaded_tracks(false, idx, cx);
                                    });
                                }))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(12.0))
                                        .flex_1()
                                        .child(
                                            div()
                                                .w(px(28.0))
                                                .text_sm()
                                                .text_color(if is_active { palette.primary } else { rgb(0x888888) })
                                                .child(if is_active { "▶".to_string() } else { format!("{}", idx + 1) })
                                        )
                                        .child(
                                            div()
                                                .size(px(40.0))
                                                .rounded(px(4.0))
                                                .overflow_hidden()
                                                .bg(rgb(0x1F1F1F))
                                                .child(
                                                    if let Some(path) = state.thumbnail_manager.resolve_or_queue(&thumb_url, None) {
                                                        Artwork::new(px(40.0))
                                                            .src(SharedString::from(path))
                                                            .rounded(px(4.0))
                                                            .into_any_element()
                                                    } else {
                                                        div().size_full().bg(rgb(0x282828)).into_any_element()
                                                    }
                                                )
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(if is_active { palette.primary } else { rgb(0xFFFFFF) })
                                                        .child(track.title.clone())
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xAAAAAA))
                                                        .child(artists_str)
                                                )
                                        )
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(20.0))
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(6.0))
                                                .px(px(8.0))
                                                .py(px(2.0))
                                                .rounded(Radii::full())
                                                .bg(rgba(0x22C55E1A))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x22C55E))
                                                        .child("✓ Local")
                                                )
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x888888))
                                                .child(track.album.clone())
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0xAAAAAA))
                                                .child(dur_str)
                                        )
                                        .child(
                                            div()
                                                .id(SharedString::from(format!("del_dl_btn_{}", idx)))
                                                .size(px(28.0))
                                                .rounded(Radii::full())
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgba(0xEF444422)))
                                                .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                                    let tid = track_id_del.clone();
                                                    app_state_del.update(cx, |s, cx| {
                                                        s.trigger_delete_download(&tid, cx);
                                                    });
                                                }))
                                                .child(div().text_xs().text_color(rgb(0x777777)).hover(|s| s.text_color(rgb(0xEF4444))).child("🗑"))
                                        )
                                )
                        }));

                    div()
                        .flex()
                        .flex_col()
                        .w_full()
                        .gap(px(16.0))
                        .child(header_bar)
                        .child(list)
                        .into_any_element()
                }
            }
        };

        div()
            .id("library-scroll")
            .flex()
            .flex_col()
            .size_full()
            .p(Spacing::s6())
            .pb(px(120.0))
            .overflow_y_scroll()
            .gap(Spacing::s5())
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(Text::new("Your Library").variant(TextVariant::H1))
                            .child(Text::new("YouTube Music Cloud Library").variant(TextVariant::Caption).muted())
                    )
                    .child(
                        Button::new("↻ Refresh")
                            .variant(ButtonVariant::Secondary)
                            .on_click(cx.listener(move |this, _ev, _win, cx| {
                                this.app_state.update(cx, |s, cx| {
                                    s.load_library(cx);
                                });
                            }))
                    )
            )
            .child(chips_row)
            .child(content)
    }
}

pub struct PlaylistsPage {
    app_state: Entity<AppState>,
    last_playlist_count: usize,
}

impl PlaylistsPage {
    pub fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        let count = app_state.read(cx).playlists.len();
        cx.observe(&app_state, |this, state, cx| {
            let s = state.read(cx);
            if s.playlists.len() != this.last_playlist_count {
                this.last_playlist_count = s.playlists.len();
                cx.notify();
            }
        }).detach();
        Self {
            app_state,
            last_playlist_count: count,
        }
    }
}

impl Render for PlaylistsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.global::<Theme>().palette;
        let state = self.app_state.read(cx);

        let mut playlist_cards: Vec<AnyElement> = Vec::new();
        if let Loadable::Ready(ref items) = state.library_playlists {
            for (idx, item) in items.iter().enumerate() {
                let app_state = self.app_state.clone();
                let id = item.id.clone();
                let title = item.title.clone();
                let thumb_url = item.thumbnail_url.clone();
                let subtitle = if !item.subtitle.is_empty() {
                    item.subtitle.clone()
                } else {
                    item.track_count.map(|c| format!("{} tracks", c)).unwrap_or_else(|| "Playlist".into())
                };

                playlist_cards.push(
                    div()
                        .id(SharedString::from(format!("pl_page_item_{}", idx)))
                        .flex()
                        .flex_col()
                        .w(px(200.0))
                        .cursor_pointer()
                        .on_click(cx.listener({
                            let id = id.clone();
                            let title = title.clone();
                            let thumb_url = thumb_url.clone();
                            move |_this, _ev, _win, cx| {
                                app_state.update(cx, |s, cx| {
                                    s.open_playlist_detail(id.clone(), title.clone(), thumb_url.clone(), cx);
                                });
                            }
                        }))
                        .child(
                            div()
                                .relative()
                                .size(px(200.0))
                                .rounded(px(8.0))
                                .overflow_hidden()
                                .bg(rgb(0x1F1F1F))
                                .child(
                                    if let Some(path) = state.thumbnail_manager.resolve_or_queue(&thumb_url, None) {
                                        Artwork::new(px(200.0))
                                            .src(SharedString::from(path))
                                            .rounded(px(8.0))
                                            .into_any_element()
                                    } else {
                                        div()
                                            .size_full()
                                            .bg(rgb(0x282828))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                div()
                                                    .text_2xl()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0x666666))
                                                    .child("♫")
                                            )
                                            .into_any_element()
                                    }
                                )
                                .child(
                                    div()
                                        .absolute()
                                        .size(px(40.0))
                                        .rounded(Radii::full())
                                        .bg(rgba(0x00000099))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .bottom(px(8.0))
                                        .right(px(8.0))
                                        .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("▶"))
                                )
                        )
                        .child(
                            div()
                                .mt(px(8.0))
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xFFFFFF))
                                        .overflow_hidden()
                                        .child(title)
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xAAAAAA))
                                        .overflow_hidden()
                                        .child(subtitle)
                                )
                        )
                        .into_any_element()
                );
            }
        }

        if playlist_cards.is_empty() {
            for pl in &state.playlists {
                playlist_cards.push(
                    div()
                        .flex()
                        .flex_col()
                        .p(Spacing::s5())
                        .rounded(Radii::large())
                        .bg(palette.surface)
                        .border_1()
                        .border_color(palette.border_subtle)
                        .hover(|s| s.bg(palette.surface_hover))
                        .cursor_pointer()
                        .w(px(220.0))
                        .gap(Spacing::s2())
                        .child(
                            div()
                                .size(px(60.0))
                                .rounded(Radii::base())
                                .bg(palette.surface_hover)
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    Text::new("♫")
                                        .variant(TextVariant::H1)
                                        .color(palette.primary)
                                )
                        )
                        .child(Text::new(pl.name.clone()).variant(TextVariant::H2))
                        .child(Text::new(format!("{} tracks", pl.track_count)).variant(TextVariant::Caption).muted())
                        .into_any_element()
                );
            }
        }

        div()
            .id("playlists-scroll")
            .flex()
            .flex_col()
            .size_full()
            .p(Spacing::s6())
            .pb(px(108.0))
            .overflow_y_scroll()
            .gap(Spacing::s5())
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(Text::new("Playlists").variant(TextVariant::H1))
                    .child(
                        Button::new("↻ Refresh")
                            .variant(ButtonVariant::Secondary)
                            .on_click(cx.listener(move |this, _ev, _win, cx| {
                                this.app_state.update(cx, |s, cx| {
                                    s.load_library(cx);
                                });
                            }))
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(Spacing::s4())
                    .children(playlist_cards)
            )
    }
}

pub struct SettingsPage {
    app_state: Entity<AppState>,
    last_preset: ThemePreset,
    last_blur: bool,
}

impl SettingsPage {
    pub fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        let (preset, blur) = {
            let s = app_state.read(cx);
            (s.theme_preset, s.enable_blur)
        };
        cx.observe(&app_state, |this, state, cx| {
            let s = state.read(cx);
            this.last_preset = s.theme_preset;
            this.last_blur = s.enable_blur;
            cx.notify();
        }).detach();
        Self {
            app_state,
            last_preset: preset,
            last_blur: blur,
        }
    }
}

impl Render for SettingsPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;
        let state = self.app_state.read(cx);

        let current_preset = state.theme_preset;
        let blur_enabled = state.enable_blur;

        let presets = [
            (ThemePreset::Cyberpunk, "Cyberpunk Neon"),
            (ThemePreset::Noir, "Noir & Brass"),
            (ThemePreset::Nord, "Nordic Frost"),
            (ThemePreset::Emerald, "Emerald Forest"),
            (ThemePreset::Oled, "Midnight OLED"),
            (ThemePreset::LightStudio, "Studio Light"),
        ];

        let mut preset_buttons = Vec::new();
        for (preset, name) in presets {
            let is_selected = preset == current_preset;
            let bg = if is_selected { palette.primary } else { palette.surface_hover };
            let fg = if is_selected {
                if preset.is_dark() { rgba(0x000000FF) } else { rgba(0xFFFFFFFF) }
            } else {
                palette.text
            };

            preset_buttons.push(
                div()
                    .id(SharedString::from(format!("preset_btn_{:?}", preset)))
                    .px(Spacing::s4())
                    .py(Spacing::s2())
                    .rounded(Radii::base())
                    .bg(bg)
                    .border_1()
                    .border_color(if is_selected { palette.primary } else { palette.border_subtle })
                    .cursor_pointer()
                    .on_click(cx.listener(move |this, _ev, _win, cx| {
                        this.app_state.update(cx, |s, cx| {
                            s.set_theme_preset(preset, cx);
                        });
                    }))
                    .child(
                        Text::new(name)
                            .variant(TextVariant::Body)
                            .color(fg)
                    )
            );
        }

        div()
            .id("settings-scroll")
            .flex()
            .flex_col()
            .size_full()
            .p(Spacing::s6())
            .pb(px(56.0))
            .gap(Spacing::s5())
            .overflow_y_scroll()
            .child(Text::new("Settings").variant(TextVariant::H1))
            // Theming & Appearance Card
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p(Spacing::s5())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border_subtle)
                    .gap(Spacing::s4())
                    .child(Text::new("Appearance & Theme Engine").variant(TextVariant::H2))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(Spacing::s2())
                            .child(Text::new("Theme Presets").variant(TextVariant::Caption).color(palette.text_muted))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap(Spacing::s2())
                                    .children(preset_buttons)
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p(Spacing::s3())
                            .rounded(Radii::base())
                            .bg(palette.surface_hover)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(Text::new("Glassmorphism & Acrylic Blur").variant(TextVariant::Body))
                                    .child(Text::new("Enables translucent surfaces with background mica/acrylic blur effect").variant(TextVariant::Caption).muted())
                            )
                            .child(
                                Button::new(if blur_enabled { "Blur: Enabled" } else { "Blur: Disabled" })
                                    .variant(if blur_enabled { ButtonVariant::Primary } else { ButtonVariant::Secondary })
                                    .on_click(cx.listener(|this, _ev, _win, cx| {
                                        this.app_state.update(cx, |s, cx| {
                                            s.toggle_blur(cx);
                                        });
                                    }))
                            )
                    )
            )
            // Playback & Audio Engine Card
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p(Spacing::s5())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border_subtle)
                    .gap(Spacing::s3())
                    .child(Text::new("Playback & Audio Engine").variant(TextVariant::H2))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p(Spacing::s3())
                            .rounded(Radii::base())
                            .bg(palette.surface_hover)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(Spacing::s1())
                                    .child(Text::new("Audio Output Device").variant(TextVariant::Body))
                                    .child(
                                        Text::new(
                                            state.audio_diagnostics.as_ref().map(|d| {
                                                format!("{} ({}Hz, {} channels)", d.device_name, d.sample_rate, d.channels)
                                            }).unwrap_or_else(|| "Default Audio Device".to_string())
                                        )
                                        .variant(TextVariant::Caption)
                                        .color(palette.text_muted)
                                    )
                            )
                            .child(
                                Button::new("Reconnect Audio")
                                    .variant(ButtonVariant::Secondary)
                                    .on_click(cx.listener(|this, _ev, _win, cx| {
                                        this.app_state.update(cx, |s, _cx| {
                                            s.reconnect_audio();
                                        });
                                    }))
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p(Spacing::s3())
                            .rounded(Radii::base())
                            .bg(palette.surface_hover)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(Text::new("Volume").variant(TextVariant::Body))
                                    .child(Text::new(format!("Current Master Output: {:.0}%", state.volume * 100.0)).variant(TextVariant::Caption).muted())
                            )
                    )
            )
            // YouTube Music Authentication Card
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p(Spacing::s5())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border_subtle)
                    .gap(Spacing::s3())
                    .child(Text::new("YouTube Music Authentication & Session").variant(TextVariant::H2))
                    .child(
                        div()
                            .p(Spacing::s3())
                            .rounded(Radii::base())
                            .bg(rgba(0xFFFFFF08))
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("ℹ️ How does YouTube Music work without signing in?")
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xAAAAAA))
                                    .child("RusticPlayer connects directly to YouTube Music's public InnerTube WEB_REMIX API in Guest Mode. Music search, streaming audio, radio generation, lyrics, and home feeds work immediately without requiring any login. Importing cookies.json is optional to unlock your personalized YouTube library, likes, and custom playlists.")
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p(Spacing::s3())
                            .rounded(Radii::base())
                            .bg(palette.surface_hover)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(Spacing::s1())
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(Spacing::s3())
                                            .children({
                                                let avatar_path = if state.auth_state.logged_in {
                                                    state.auth_state.account_avatar_url.as_ref().and_then(|u| {
                                                        state.thumbnail_manager.resolve_or_queue(&Some(u.clone()), None)
                                                    })
                                                } else {
                                                    None
                                                };
                                                avatar_path.map(|path| {
                                                    div()
                                                        .size(px(40.0))
                                                        .rounded(Radii::full())
                                                        .overflow_hidden()
                                                        .child(
                                                            img(SharedString::from(path))
                                                                .size_full()
                                                                .object_fit(ObjectFit::Cover)
                                                        )
                                                })
                                            })
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(2.0))
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap(Spacing::s2())
                                                            .child(
                                                                div()
                                                                    .size(px(8.0))
                                                                    .rounded(Radii::full())
                                                                    .bg(if state.auth_state.logged_in { rgb(0x22C55E) } else { rgb(0x38BDF8) })
                                                            )
                                                            .child(
                                                                Text::new(if state.auth_state.logged_in {
                                                                    "Connected & Authenticated"
                                                                } else {
                                                                    "Guest Mode (Direct InnerTube API)"
                                                                })
                                                                .variant(TextVariant::Body)
                                                            )
                                                    )
                                                    .child(
                                                        Text::new(if state.auth_state.logged_in {
                                                            let handle_suffix = state.auth_state.account_handle.as_deref().map(|h| format!(" ({})", h)).unwrap_or_default();
                                                            format!("Account: {}{} • Active Session", state.auth_state.account_name.as_deref().unwrap_or("Personal Account"), handle_suffix)
                                                        } else {
                                                            "Place a cookies.json in the application root folder to connect your personal YouTube account.".to_string()
                                                        })
                                                        .variant(TextVariant::Caption)
                                                        .color(palette.text_muted)
                                                    )
                                            )
                                    )
                            )
                            .child(
                                Button::new("Reload cookies.json")
                                    .variant(ButtonVariant::Primary)
                                    .on_click(cx.listener(|this, _ev, _win, cx| {
                                        this.app_state.update(cx, |s, cx| {
                                            s.reload_cookies(cx);
                                        });
                                    }))
                            )
                    )
            )
            // Music Folders Card
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p(Spacing::s5())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border_subtle)
                    .gap(Spacing::s3())
                    .child(Text::new("Local Music Folders").variant(TextVariant::H2))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p(Spacing::s3())
                            .rounded(Radii::base())
                            .bg(palette.surface_hover)
                            .child(Text::new("dummy_library/ (Synthetic Offline Test Audio)").variant(TextVariant::Body))
                            .child(
                                Button::new("Rescan Library")
                                    .variant(ButtonVariant::Primary)
                                    .on_click(cx.listener(|this, _ev, _win, cx| {
                                        this.app_state.update(cx, |s, cx| {
                                            s.reload_library();
                                            cx.notify();
                                        });
                                    }))
                            )
                    )
            )
            // Library Stats Card
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p(Spacing::s5())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border_subtle)
                    .gap(Spacing::s2())
                    .child(Text::new("Library Statistics").variant(TextVariant::H2))
                    .child(Text::new(format!("Total Local Tracks: {}", state.tracks.len())).variant(TextVariant::Body))
                    .child(Text::new(format!("Total Albums: {}", state.albums.len())).variant(TextVariant::Body))
                    .child(Text::new(format!("Total Artists: {}", state.artists.len())).variant(TextVariant::Body))
                    .child(Text::new(format!("Total Playlists: {}", state.playlists.len())).variant(TextVariant::Body))
                    .child(Text::new(format!("Liked Songs: {}", state.liked_tracks.len())).variant(TextVariant::Body))
            )
    }
}

// =========================================================================
// YTM 02 • ALBUM PAGE (folklore 2-Column View)
// =========================================================================
pub struct AlbumPage {
    app_state: Entity<AppState>,
}

impl AlbumPage {
    pub fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        cx.observe(&app_state, |_this, _state, cx| {
            cx.notify();
        }).detach();
        Self { app_state }
    }
}

impl Render for AlbumPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state.read(cx);

        if let Loadable::Ready(ref details) = state.active_album {
            let title = details.title.clone();
            let subtitle = details.subtitle.clone();
            let description = details.description.clone();
            let thumb_path = details.thumbnail_url.as_ref().and_then(|u| state.thumbnail_manager.resolve_or_queue(&Some(u.clone()), None));
            let current_track_key = state.current_track.as_ref().map(|t| t.id.clone());
            let is_playing = state.playback_state == PlaybackState::Playing;

            let app_state_play = self.app_state.clone();
            let app_state_shuffle = self.app_state.clone();
            let details_clone = details.clone();
            let details_for_shuffle = details.clone();
            let app_state_dl = self.app_state.clone();
            let details_for_dl = details.clone();

            let left_hero = div()
                .flex()
                .flex_col()
                .w(px(280.0))
                .min_w(px(260.0))
                .gap(px(16.0))
                .child(
                    div()
                        .size(px(260.0))
                        .rounded(px(8.0))
                        .overflow_hidden()
                        .bg(rgb(0x1F1F1F))
                        .shadow(vec![
                            gpui::BoxShadow {
                                color: rgba(0x00000099).into(),
                                offset: gpui::point(px(0.0), px(8.0)),
                                blur_radius: px(24.0),
                                spread_radius: px(4.0),
                                inset: false,
                            }
                        ])
                        .child(
                            if let Some(ref p) = thumb_path {
                                Artwork::new(px(260.0))
                                    .src(SharedString::from(p.clone()))
                                    .rounded(px(8.0))
                                    .into_any_element()
                            } else {
                                div()
                                    .size_full()
                                    .bg(rgb(0x282828))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(div().text_2xl().text_color(rgb(0x666666)).child("💽"))
                                    .into_any_element()
                            }
                        )
                )
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::EXTRA_BOLD)
                        .text_color(rgb(0xFFFFFF))
                        .child(title)
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(0xAAAAAA))
                        .child(subtitle)
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x717171))
                        .child(format!("{} songs", details.tracks.len()))
                )
                .when_some(description, |this, desc| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x888888))
                            .max_h(px(60.0))
                            .overflow_hidden()
                            .child(desc)
                    )
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .pt(px(4.0))
                        .child(
                            div()
                                .id("album_btn_play")
                                .px(px(20.0))
                                .py(px(8.0))
                                .rounded(Radii::full())
                                .bg(rgb(0xFFFFFF))
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0xEEEEEE)))
                                .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                    let d = details_clone.clone();
                                    app_state_play.update(cx, |s, cx| {
                                        s.play_collection(&d, false, cx);
                                    });
                                }))
                                .child(div().text_sm().text_color(rgb(0x000000)).child("▶"))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x000000))
                                        .child("Play")
                                )
                        )
                        .child(
                            div()
                                .id("album_btn_shuffle")
                                .px(px(16.0))
                                .py(px(8.0))
                                .rounded(Radii::full())
                                .bg(rgba(0xFFFFFF14))
                                .border_1()
                                .border_color(rgba(0xFFFFFF1A))
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgba(0xFFFFFF26)))
                                .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                    let d = details_for_shuffle.clone();
                                    app_state_shuffle.update(cx, |s, cx| {
                                        s.play_collection(&d, true, cx);
                                    });
                                }))
                                .child(div().text_sm().text_color(rgb(0xFFFFFF)).child("🔀"))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xFFFFFF))
                                        .child("Shuffle")
                                )
                        )
                        .child(
                            div()
                                .id("album_btn_download")
                                .size(px(36.0))
                                .rounded(Radii::full())
                                .bg(rgba(0xFFFFFF14))
                                .border_1()
                                .border_color(rgba(0xFFFFFF1A))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .hover(|s| s.bg(rgba(0xFFFFFF26)))
                                .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                    let d = details_for_dl.clone();
                                    app_state_dl.update(cx, |s, cx| {
                                        s.trigger_download_collection(d, cx);
                                    });
                                }))
                                .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("↓"))
                        )
                        .child(
                            div()
                                .id("album_btn_bookmark")
                                .size(px(36.0))
                                .rounded(Radii::full())
                                .bg(rgba(0xFFFFFF14))
                                .border_1()
                                .border_color(rgba(0xFFFFFF1A))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .hover(|s| s.bg(rgba(0xFFFFFF26)))
                                .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("🔖"))
                        )
                );

            let mut track_rows = Vec::new();
            for (idx, track) in details.tracks.iter().enumerate() {
                let is_cur = current_track_key.as_ref() == Some(&track.id);
                let dur_secs = track.duration_secs;
                let dur_str = format!("{}:{:02}", dur_secs / 60, dur_secs % 60);
                let artists_str = track.artists.join(", ");
                let app_state_track = self.app_state.clone();
                let details_track = details.clone();

                track_rows.push(
                    div()
                        .id(SharedString::from(format!("album_yt_track_{}", idx)))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(16.0))
                        .py(px(8.0))
                        .rounded(px(6.0))
                        .cursor_pointer()
                        .bg(if is_cur { rgba(0xFFFFFF1A) } else { rgba(0x00000000) })
                        .hover(|s| s.bg(rgba(0xFFFFFF14)))
                        .on_click(cx.listener({
                            let d = details_track.clone();
                            move |_this, _ev, _win, cx| {
                                app_state_track.update(cx, |s, cx| {
                                    s.play_collection_track(&d, idx, cx);
                                });
                            }
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(16.0))
                                .min_w_0()
                                .child(
                                    div()
                                        .w(px(24.0))
                                        .flex()
                                        .justify_center()
                                        .text_xs()
                                        .text_color(if is_cur { rgb(0xFFFFFF) } else { rgb(0x888888) })
                                        .child(SharedString::from(if is_cur && is_playing { "▶".to_string() } else { format!("{}", idx + 1) }))
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(2.0))
                                        .min_w_0()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(if is_cur { rgb(0xFFFFFF) } else { rgb(0xEEEEEE) })
                                                .child(track.title.clone())
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x888888))
                                                .child(artists_str)
                                        )
                                )
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(16.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if is_cur { rgb(0xFFFFFF) } else { rgb(0xAAAAAA) })
                                        .child(dur_str)
                                )
                        )
                );
            }

            return div()
                .id("album_page_container")
                .size_full()
                .min_h_0()
                .bg(rgba(0x00000000))
                .flex()
                .flex_row()
                .p(px(32.0))
                .pb(px(88.0))
                .gap(px(48.0))
                .overflow_y_scroll()
                .child(left_hero)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .children(track_rows)
                )
                .into_any_element();
        } else if let Loadable::Loading = state.active_album {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .p(px(32.0))
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(0xAAAAAA))
                        .child("Loading album from YouTube Music...")
                )
                .into_any_element();
        } else if let Loadable::Error(ref err) = state.active_album {
            let app_state_retry = self.app_state.clone();
            let route_aid = match &state.current_route {
                Route::Album(id) => id.clone(),
                _ => String::new(),
            };
            return div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(16.0))
                .p(px(32.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xEF4444))
                        .child(format!("Failed to load album: {}", err))
                )
                .child(
                    Button::new("Retry")
                        .variant(ButtonVariant::Primary)
                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                            let aid = route_aid.clone();
                            app_state_retry.update(cx, |s, cx| {
                                if !aid.is_empty() {
                                    s.open_album_detail(aid, String::new(), None, cx);
                                } else {
                                    s.load_library(cx);
                                }
                            });
                        }))
                )
                .into_any_element();
        }

        let album_id = match &state.current_route {
            Route::Album(id) if !id.is_empty() => Some(id.clone()),
            _ => None,
        };

        if let Some(aid) = album_id {
            let app_state_load = self.app_state.clone();
            let aid_clone = aid.clone();
            cx.spawn(move |_, cx: &mut gpui::AsyncApp| {
                let cx = cx.clone();
                async move {
                    let mut cx = cx.clone();
                    let _ = app_state_load.update(&mut cx, |s, cx| {
                        if matches!(s.active_album, Loadable::Idle) {
                            s.open_album_detail(aid_clone, String::new(), None, cx);
                        }
                    });
                }
            }).detach();

            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .p(px(32.0))
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(0xAAAAAA))
                        .child("Loading album from YouTube Music...")
                )
                .into_any_element()
        } else {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .p(px(32.0))
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(0xAAAAAA))
                        .child("No album selected")
                )
                .into_any_element()
        }
    }
}

/// Cleans raw artist/album subtitles by stripping YouTube statistic tokens (e.g. "114M views", "1M likes", "2.84M subscribers")
/// while strictly preserving genuine artist, band, album, and song names (such as "The View", "Like A Storm", "Birds of a Feather").
pub fn clean_artist_subtitle(raw: &str) -> String {
    fn is_stat_token(s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return false;
        }
        let lower = trimmed.to_lowercase();
        let stat_suffixes = ["views", "view", "likes", "like", "subscribers", "subscriber", "plays", "play", "watching"];
        for suffix in &stat_suffixes {
            if let Some(prefix) = lower.strip_suffix(suffix) {
                let prefix_trimmed = prefix.trim();
                if !prefix_trimmed.is_empty()
                    && prefix_trimmed.chars().any(|c| c.is_ascii_digit())
                    && prefix_trimmed.chars().all(|c| {
                        c.is_ascii_digit() || c == '.' || c == ',' || c == ' ' || c == 'k' || c == 'm' || c == 'b' || c == 't'
                    })
                {
                    return true;
                }
            }
        }
        false
    }

    if raw.is_empty() {
        return String::new();
    }

    if raw.contains('•') {
        let parts: Vec<&str> = raw.split('•').collect();
        let mut cleaned_parts = Vec::new();
        for p in parts {
            let p_trim = p.trim();
            if is_stat_token(p_trim) {
                continue;
            }
            if p_trim.contains(',') {
                let sub_parts: Vec<&str> = p_trim.split(',').collect();
                let sub_cleaned: Vec<&str> = sub_parts
                    .into_iter()
                    .map(|sp| sp.trim())
                    .filter(|sp| !sp.is_empty() && !is_stat_token(sp))
                    .collect();
                if !sub_cleaned.is_empty() {
                    cleaned_parts.push(sub_cleaned.join(", "));
                }
            } else if !p_trim.is_empty() {
                cleaned_parts.push(p_trim.to_string());
            }
        }
        if !cleaned_parts.is_empty() {
            return cleaned_parts.join(" • ");
        }
    } else if raw.contains(',') {
        let parts: Vec<&str> = raw.split(',').collect();
        let cleaned: Vec<&str> = parts
            .into_iter()
            .map(|p| p.trim())
            .filter(|p| !p.is_empty() && !is_stat_token(p))
            .collect();
        if !cleaned.is_empty() {
            return cleaned.join(", ");
        }
    } else if is_stat_token(raw.trim()) {
        return raw.trim().to_string();
    }

    raw.trim().to_string()
}

// =========================================================================
// YTM 03 • PLAYER PAGE (Full-Screen Now Playing View)
// =========================================================================
pub struct PlayerPage {
    app_state: Entity<AppState>,
    #[allow(dead_code)]
    active_tab: String,
    #[allow(dead_code)]
    is_video_mode: bool,
    #[allow(dead_code)]
    autoplay_enabled: bool,
    selected_chip: String,
}

impl PlayerPage {
    pub fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        cx.observe(&app_state, |_this, _state, cx| {
            cx.notify();
        }).detach();
        Self {
            app_state,
            active_tab: "UP NEXT".to_string(),
            is_video_mode: false,
            autoplay_enabled: true,
            selected_chip: "All".to_string(),
        }
    }
}

impl Render for PlayerPage {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_mobile = window.viewport_size().width < px(768.0);
        let state = self.app_state.read(cx);

        let title = state.current_track.as_ref().map(|t| t.title.clone()).unwrap_or_else(|| "No Track Playing".into());
        let artist = state.current_track.as_ref().map(|t| {
            let raw = if !t.album.is_empty() && t.album != "YouTube Music" && !t.album.starts_with("YouTube") {
                format!("{} • {}", t.artists.join(", "), t.album)
            } else {
                t.artists.join(", ")
            };
            clean_artist_subtitle(&raw)
        }).unwrap_or_else(|| "Select a track to play".into());

        let current_pos_secs = if state.playback_state == PlaybackState::Playing {
            state.timeline.position_at(std::time::Instant::now())
        } else {
            state.current_position
        };

        let cur_track_key = state.current_track.as_ref().map(|t| t.id.clone()).unwrap_or_default();
        let is_liked = !cur_track_key.is_empty() && state.liked_tracks.contains(&cur_track_key);

        let current_art = if let Some(ref cur) = state.current_track {
            if let Some(ui_t) = state.tracks.iter().find(|t| t.track_key == cur.id) {
                ui_t.artwork_path().map(|p| p.to_string_lossy().to_string().replace('\\', "/"))
            } else {
                let art_url = cur.artwork_url.as_ref().filter(|u| !u.is_empty()).cloned().or_else(|| {
                    if cur.id.len() == 11 && !cur.id.contains('/') && !cur.id.contains('\\') {
                        Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", cur.id))
                    } else {
                        None
                    }
                });
                state.thumbnail_manager.resolve_or_queue(&art_url, None)
            }
        } else {
            None
        };

        // Video Mode: 16:9 widescreen player (560x315 desktop, 320x180 mobile); Song Mode: 1:1 square (480x480 desktop, 280x280 mobile)
        let is_video = state.is_video_mode;
        let left_cover = if is_video {
            let video_w = if is_mobile { px(320.0) } else { px(560.0) };
            let video_h = if is_mobile { px(180.0) } else { px(315.0) };
            div()
                .relative()
                .w(video_w)
                .h(video_h)
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .relative()
                        .w_full()
                        .h_full()
                        .rounded(Radii::base())
                        .overflow_hidden()
                        .bg(rgb(0x0A0A0A))
                        .border_1()
                        .border_color(rgba(0xFFFFFF15))
                        .shadow(vec![
                            gpui::BoxShadow {
                                color: rgba(0x000000AA).into(),
                                offset: gpui::point(px(0.0), px(12.0)),
                                blur_radius: px(32.0),
                                spread_radius: px(4.0),
                                inset: false,
                            }
                        ])
                        .child(
                            if let Some(ref art_path) = current_art {
                                Artwork::new(video_w)
                                    .src(SharedString::from(art_path.clone()))
                                    .rounded(Radii::base())
                                    .into_any_element()
                            } else {
                                div()
                                    .size_full()
                                    .bg(rgb(0x181818))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(div().text_sm().text_color(rgb(0x717171)).child("🎬 Official Music Video"))
                                    .into_any_element()
                            }
                        )
                        .child(
                            div()
                                .absolute()
                                .bottom(px(12.0))
                                .left(px(12.0))
                                .px(px(8.0))
                                .py(px(4.0))
                                .rounded(Radii::small())
                                .bg(rgba(0x000000AA))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xFFFFFF))
                                        .child("▶ Video Mode")
                                )
                        )
                )
        } else {
            let cover_size = if is_mobile { px(280.0) } else { px(480.0) };
            div()
                .relative()
                .size(cover_size)
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .relative()
                        .size(cover_size)
                        .rounded(Radii::base())
                        .overflow_hidden()
                        .border_1()
                        .border_color(rgba(0xFFFFFF15))
                        .shadow(vec![
                            gpui::BoxShadow {
                                color: rgba(0x00000088).into(),
                                offset: gpui::point(px(0.0), px(12.0)),
                                blur_radius: px(32.0),
                                spread_radius: px(4.0),
                                inset: false,
                            }
                        ])
                        .child(
                            if let Some(ref art_path) = current_art {
                                Artwork::new(cover_size)
                                    .src(SharedString::from(art_path.clone()))
                                    .rounded(Radii::base())
                                    .into_any_element()
                            } else {
                                div()
                                    .size_full()
                                    .bg(rgb(0x181818))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_3xl()
                                            .text_color(rgb(0x555555))
                                            .child("🎵")
                                    )
                                    .into_any_element()
                            }
                        )
                )
        };

        // Authentic YouTube Music Song / Video Switcher Pill (Positioned above album cover / video surface)
        let state_for_toggle_song = self.app_state.clone();
        let state_for_toggle_video = self.app_state.clone();

        let song_video_toggle = div()
            .id("player_song_video_toggle")
            .h(px(32.0))
            .rounded(Radii::full())
            .bg(rgba(0xFFFFFF18))
            .p(px(2.0))
            .flex()
            .items_center()
            .child(
                div()
                    .id("player_toggle_song")
                    .px(px(14.0))
                    .h_full()
                    .rounded(Radii::full())
                    .bg(if !is_video { rgb(0xFFFFFF) } else { rgba(0x00000000) })
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| if is_video { s.bg(rgba(0xFFFFFF15)) } else { s })
                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                        state_for_toggle_song.update(cx, |s, cx| {
                            s.toggle_video_mode(false, cx);
                        });
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(if !is_video { rgb(0x000000) } else { rgb(0xAAAAAA) })
                            .child("Song")
                    )
            )
            .child(
                div()
                    .id("player_toggle_video")
                    .px(px(14.0))
                    .h_full()
                    .rounded(Radii::full())
                    .bg(if is_video { rgb(0xFFFFFF) } else { rgba(0x00000000) })
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| if !is_video { s.bg(rgba(0xFFFFFF15)) } else { s })
                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                        state_for_toggle_video.update(cx, |s, cx| {
                            s.toggle_video_mode(true, cx);
                        });
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(if is_video { rgb(0x000000) } else { rgb(0xAAAAAA) })
                            .child("Video")
                    )
            );

        let left_column = div()
            .id("player_left_column")
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.0))
            .child(song_video_toggle)
            .child(left_cover);

        // Right Column: Queue & Tabs
        let tabs = [
            (PlayerTab::UpNext, "UP NEXT"),
            (PlayerTab::Lyrics, "LYRICS"),
            (PlayerTab::Comments, "COMMENTS"),
            (PlayerTab::Related, "RELATED"),
        ];
        let app_state_for_tabs = self.app_state.clone();
        let active_tab = state.active_player_tab;

        let tabs_row = div()
            .flex()
            .items_center()
            .gap(px(24.0))
            .border_b_1()
            .border_color(rgb(0x212121))
            .pb(px(8.0))
            .children(tabs.into_iter().map(|(tab_enum, tab_label)| {
                let is_active = active_tab == tab_enum;
                let state_clone = app_state_for_tabs.clone();
                div()
                    .id(SharedString::from(format!("queue_tab_{}", tab_label)))
                    .cursor_pointer()
                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                        state_clone.update(cx, |s, cx| {
                            s.set_player_tab(tab_enum, cx);
                        });
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(if is_active { FontWeight::BOLD } else { FontWeight::SEMIBOLD })
                                    .text_color(if is_active { rgb(0xFFFFFF) } else { rgb(0xAAAAAA) })
                                    .child(tab_label)
                            )
                            .child(
                                div()
                                    .h(px(2.0))
                                    .w_full()
                                    .bg(if is_active { rgb(0xFFFFFF) } else { rgba(0x00000000) })
                            )
                    )
            }));

        // Queue Subheader: Playing from ... + Save button
        let state_for_save = self.app_state.clone();
        let key_for_save = cur_track_key.clone();
        let playing_from = state.source_playlist_title.clone()
            .or_else(|| match &state.queue_snapshot {
                Loadable::Ready(s) => s.source_playlist_title.clone(),
                _ => None,
            })
            .unwrap_or_else(|| "YouTube Music".to_string());

        let subheader = div()
            .flex()
            .items_center()
            .justify_between()
            .pt(px(12.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xAAAAAA))
                            .child("Playing from")
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xFFFFFF))
                            .child(playing_from)
                    )
            )
            .child(
                div()
                    .id("player_save_btn")
                    .px(px(14.0))
                    .py(px(6.0))
                    .rounded(Radii::full())
                    .bg(if is_liked { rgba(0xFFFFFF33) } else { rgb(0x212121) })
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x333333)))
                    .on_click(cx.listener({
                        let k = key_for_save.clone();
                        move |_this, _ev, _win, cx| {
                            state_for_save.update(cx, |s, cx| {
                                s.toggle_like(k.clone(), cx);
                            });
                        }
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if is_liked { rgb(0x38BDF8) } else { rgb(0xFFFFFF) })
                            .child(if is_liked { "✓ Saved" } else { "+ Save" })
                    )
            );

        // Autoplay Toggle Row
        let state_for_autoplay = self.app_state.clone();
        let is_autoplay = state.is_autoplay_enabled;
        let autoplay_row = div()
            .flex()
            .items_center()
            .justify_between()
            .pt(px(14.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(rgb(0xFFFFFF))
                            .child("Autoplay")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xAAAAAA))
                            .child("Add similar content to the end of the queue")
                    )
            )
            .child(
                div()
                    .id("player_autoplay_toggle")
                    .w(px(38.0))
                    .h(px(20.0))
                    .rounded(Radii::full())
                    .bg(if is_autoplay { rgb(0x3B82F6) } else { rgb(0x444444) })
                    .p(px(2.0))
                    .flex()
                    .items_center()
                    .justify_start()
                    .cursor_pointer()
                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                        state_for_autoplay.update(cx, |s, cx| {
                            s.toggle_autoplay(cx);
                        });
                    }))
                    .child(
                        div()
                            .size(px(16.0))
                            .rounded(Radii::full())
                            .bg(rgb(0xFFFFFF))
                            .ml(if is_autoplay { px(18.0) } else { px(0.0) })
                    )
            );


        // Autoplay Filter Chips Row
        let cur_chip = self.selected_chip.clone();
        let chips = vec!["All", "Familiar", "Romance", "Chill", "Upbeat", "Discover"];
        let mut chips_row_opt = if is_autoplay {
            Some(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .pt(px(12.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xAAAAAA))
                            .child("Autoplay is on")
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .children(chips.into_iter().map(|chip| {
                                let is_sel = cur_chip == chip;
                                let chip_str = chip.to_string();
                                div()
                                    .id(SharedString::from(format!("chip_{}", chip)))
                                    .px(px(12.0))
                                    .py(px(5.0))
                                    .rounded(Radii::small())
                                    .bg(if is_sel { rgb(0xFFFFFF) } else { rgb(0x212121) })
                                    .cursor_pointer()
                                    .hover(|s| if !is_sel { s.bg(rgb(0x333333)) } else { s })
                                    .on_click(cx.listener({
                                        let c = chip_str.clone();
                                        move |this, _ev, _win, cx| {
                                            this.selected_chip = c.clone();
                                            cx.notify();
                                        }
                                    }))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(if is_sel { FontWeight::BOLD } else { FontWeight::MEDIUM })
                                            .text_color(if is_sel { rgb(0x030303) } else { rgb(0xFFFFFF) })
                                            .child(chip)
                                    )
                            }))
                    )
                    .into_any_element()
            )
        } else {
            None
        };

        // Upcoming tracks elements
        let mut queue_elements: Vec<gpui::AnyElement> = Vec::new();
        let app_state_for_queue = self.app_state.clone();

        let q_snap = state.queue_mgr.snapshot();
        let has_multilane_items = !q_snap.actual_queue.is_empty()
            || !q_snap.user_queue.is_empty()
            || !q_snap.radio_upcoming.is_empty();

        if has_multilane_items {
            // Previous Tracks (History: 0..cursor)
            let history_count = q_snap.cursor.min(q_snap.actual_queue.len());
            if history_count > 0 {
                queue_elements.push(
                    div()
                        .pt(px(10.0))
                        .pb(px(4.0))
                        .px(px(4.0))
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x888888))
                        .child(format!("PREVIOUS TRACKS • {}", history_count))
                        .into_any_element()
                );

                for (idx, item) in q_snap.actual_queue[..history_count].iter().enumerate() {
                    let art_url = item.artwork_url.as_ref().filter(|u| !u.is_empty()).cloned().or_else(|| {
                        if item.id.len() == 11 && !item.id.contains('/') && !item.id.contains('\\') {
                            Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", item.id))
                        } else {
                            None
                        }
                    });
                    let art = state.thumbnail_manager.resolve_or_queue(&art_url, None);
                    let app_state_item = app_state_for_queue.clone();
                    let dur_str = format!("{}:{:02}", item.duration_secs / 60, item.duration_secs % 60);
                    let raw_sub = if !item.album.is_empty() && item.album != "YouTube Music" && !item.album.starts_with("YouTube") {
                        format!("{} • {}", item.artists.join(", "), item.album)
                    } else {
                        item.artists.join(", ")
                    };
                    let subtitle = clean_artist_subtitle(&raw_sub);

                    queue_elements.push(
                        div()
                            .id(SharedString::from(format!("hist_track_{}", idx)))
                            .flex()
                            .items_center()
                            .justify_between()
                            .px(px(8.0))
                            .py(px(6.0))
                            .rounded(Radii::small())
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xFFFFFF10)))
                            .on_click(cx.listener({
                                move |_this, _ev, _win, cx| {
                                    app_state_item.update(cx, |s, cx| {
                                        s.jump_to_queue_index(idx, cx);
                                    });
                                }
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .min_w_0()
                                    .child(
                                        if let Some(path) = art {
                                            Artwork::new(px(36.0))
                                                .src(SharedString::from(path))
                                                .rounded(Radii::small())
                                                .into_any_element()
                                        } else {
                                            div()
                                                .size(px(36.0))
                                                .rounded(Radii::small())
                                                .bg(rgb(0x282828))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(div().text_xs().text_color(rgb(0x717171)).child("🎵"))
                                                .into_any_element()
                                        }
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0xAAAAAA))
                                                    .child(item.title.clone())
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x666666))
                                                    .child(subtitle)
                                            )
                                    )
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x666666))
                                    .child(dur_str)
                            )
                            .into_any_element()
                    );
                }
            }

            // Currently Playing Track (Inline Highlighted at cursor)
            let cur_item = q_snap.actual_queue.get(q_snap.cursor).cloned().or_else(|| q_snap.current_track.clone());
            if let Some(cur) = cur_item {
                let cur_art_url = cur.artwork_url.as_ref().filter(|u| !u.is_empty()).cloned().or_else(|| {
                    if cur.id.len() == 11 && !cur.id.contains('/') && !cur.id.contains('\\') {
                        Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", cur.id))
                    } else {
                        None
                    }
                });
                let art = current_art.clone().or_else(|| state.thumbnail_manager.resolve_or_queue(&cur_art_url, None));
                let cur_title = if !cur.title.is_empty() { cur.title.clone() } else { title.clone() };
                let raw_sub = if !cur.album.is_empty() && cur.album != "YouTube Music" && !cur.album.starts_with("YouTube") {
                    format!("{} • {}", cur.artists.join(", "), cur.album)
                } else if !cur.artists.is_empty() {
                    cur.artists.join(", ")
                } else {
                    artist.clone()
                };
                let cur_sub = clean_artist_subtitle(&raw_sub);
                let now_dur = if cur.duration_secs > 0 {
                    cur.duration_secs
                } else if state.track_duration > 0.0 {
                    state.track_duration as u32
                } else {
                    180
                };
                let now_dur_str = format!("{}:{:02}", now_dur / 60, now_dur % 60);

                queue_elements.push(
                    div()
                        .pt(px(10.0))
                        .pb(px(4.0))
                        .px(px(4.0))
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x3EA6FF))
                        .child("NOW PLAYING")
                        .into_any_element()
                );

                queue_elements.push(
                    div()
                        .id("inline_now_playing_row")
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(10.0))
                        .py(px(8.0))
                        .rounded(Radii::small())
                        .bg(rgba(0x3EA6FF16))
                        .border_1()
                        .border_color(rgba(0x3EA6FF4D))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(10.0))
                                .min_w_0()
                                .child(
                                    div()
                                        .w(px(3.0))
                                        .h(px(28.0))
                                        .rounded_full()
                                        .bg(rgb(0x3EA6FF))
                                )
                                .child(
                                    if let Some(path) = art {
                                        Artwork::new(px(40.0))
                                            .src(SharedString::from(path))
                                            .rounded(Radii::small())
                                            .into_any_element()
                                    } else {
                                        div()
                                            .size(px(40.0))
                                            .rounded(Radii::small())
                                            .bg(rgb(0x282828))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(div().text_xs().text_color(rgb(0x717171)).child("🎵"))
                                            .into_any_element()
                                    }
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(2.0))
                                        .min_w_0()
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(rgb(0xFFFFFF))
                                                .child(cur_title)
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0xCCCCCC))
                                                .child(cur_sub)
                                        )
                                )
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.0))
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0x3EA6FF))
                                .child(
                                    div().text_xs().child("🔊")
                                )
                                .child(now_dur_str)
                        )
                        .into_any_element()
                );
            }
            // Lane 1: User Priority Queue
            if !q_snap.user_queue.is_empty() {
                let state_clone = app_state_for_queue.clone();
                queue_elements.push(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pt(px(12.0))
                        .pb(px(4.0))
                        .px(px(4.0))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xAAAAAA))
                                .child(format!("USER QUEUE • {}", q_snap.user_queue.len()))
                        )
                        .child(
                            div()
                                .id("clear_user_queue_btn")
                                .cursor_pointer()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(0xEF4444))
                                .hover(|s| s.underline())
                                .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                    state_clone.update(cx, |s, cx| {
                                        s.clear_user_queue(cx);
                                    });
                                }))
                                .child("Clear")
                        )
                        .into_any_element()
                );

                for (idx, item) in q_snap.user_queue.iter().enumerate() {
                    let art_url = item.artwork_url.as_ref().filter(|u| !u.is_empty()).cloned().or_else(|| {
                        if item.id.len() == 11 && !item.id.contains('/') && !item.id.contains('\\') {
                            Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", item.id))
                        } else {
                            None
                        }
                    });
                    let art = state.thumbnail_manager.resolve_or_queue(&art_url, None);
                    let app_state_item = app_state_for_queue.clone();
                    let item_clone = item.clone();
                    let dur_str = format!("{}:{:02}", item.duration_secs / 60, item.duration_secs % 60);
                    let raw_sub = if !item.album.is_empty() && item.album != "YouTube Music" && !item.album.starts_with("YouTube") {
                        format!("{} • {}", item.artists.join(", "), item.album)
                    } else {
                        item.artists.join(", ")
                    };
                    let subtitle = clean_artist_subtitle(&raw_sub);

                    queue_elements.push(
                        div()
                            .id(SharedString::from(format!("user_queue_track_{}", idx)))
                            .flex()
                            .items_center()
                            .justify_between()
                            .px(px(8.0))
                            .py(px(6.0))
                            .rounded(Radii::small())
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xFFFFFF15)))
                            .child(
                                div()
                                    .id(SharedString::from(format!("user_queue_track_info_{}", idx)))
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .min_w_0()
                                    .on_click(cx.listener({
                                        let tr_id = item_clone.id.clone();
                                        move |_this, _ev, _win, cx| {
                                            app_state_item.update(cx, |s, cx| {
                                                s.jump_to_track_id(&tr_id, cx);
                                            });
                                        }
                                    }))
                                    .child(
                                        if let Some(path) = art {
                                            Artwork::new(px(36.0))
                                                .src(SharedString::from(path))
                                                .rounded(Radii::small())
                                                .into_any_element()
                                        } else {
                                            div()
                                                .size(px(36.0))
                                                .rounded(Radii::small())
                                                .bg(rgb(0x282828))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(div().text_xs().text_color(rgb(0x717171)).child("🎵"))
                                                .into_any_element()
                                        }
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child(item.title.clone())
                                             )
                                             .child(
                                                 div()
                                                     .text_xs()
                                                     .text_color(rgb(0xAAAAAA))
                                                     .child(subtitle)
                                             )
                                     )
                             )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0xAAAAAA))
                                            .child(dur_str)
                                    )
                                    .child(
                                        div()
                                            .id(SharedString::from(format!("del_user_track_{}", idx)))
                                            .cursor_pointer()
                                            .p(px(4.0))
                                            .text_xs()
                                            .text_color(rgb(0x888888))
                                            .hover(|s| s.text_color(rgb(0xEF4444)))
                                            .on_click(cx.listener({
                                                let state_rem = app_state_for_queue.clone();
                                                move |_this, _ev, _win, cx| {
                                                    state_rem.update(cx, |s, cx| {
                                                        s.remove_user_queue_item(idx, cx);
                                                    });
                                                }
                                            }))
                                            .child("✕")
                                    )
                            )
                            .into_any_element()
                    );
                }
            }

            // Lane 2: Context Upcoming
            if !q_snap.context_upcoming.is_empty() {
                let ctx_label = q_snap.context_title.as_deref().unwrap_or("QUEUE");
                queue_elements.push(
                    div()
                        .pt(px(12.0))
                        .pb(px(4.0))
                        .px(px(4.0))
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xAAAAAA))
                        .child(format!("NEXT IN {} • {}", ctx_label.to_uppercase(), q_snap.context_upcoming.len()))
                        .into_any_element()
                );

                for (idx, item) in q_snap.context_upcoming.iter().enumerate() {
                    let art_url = item.artwork_url.as_ref().filter(|u| !u.is_empty()).cloned().or_else(|| {
                        if item.id.len() == 11 && !item.id.contains('/') && !item.id.contains('\\') {
                            Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", item.id))
                        } else {
                            None
                        }
                    });
                    let art = state.thumbnail_manager.resolve_or_queue(&art_url, None);
                    let app_state_item = app_state_for_queue.clone();
                    let item_clone = item.clone();
                    let dur_str = format!("{}:{:02}", item.duration_secs / 60, item.duration_secs % 60);
                    let raw_sub = if !item.album.is_empty() && item.album != "YouTube Music" && !item.album.starts_with("YouTube") {
                        format!("{} • {}", item.artists.join(", "), item.album)
                    } else {
                        item.artists.join(", ")
                    };
                    let subtitle = clean_artist_subtitle(&raw_sub);

                    queue_elements.push(
                        div()
                            .id(SharedString::from(format!("context_track_{}", idx)))
                            .flex()
                            .items_center()
                            .justify_between()
                            .px(px(8.0))
                            .py(px(6.0))
                            .rounded(Radii::small())
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xFFFFFF15)))
                            .on_click(cx.listener({
                                let tr_id = item_clone.id.clone();
                                move |_this, _ev, _win, cx| {
                                    app_state_item.update(cx, |s, cx| {
                                        if let Some(pos) = s.queue_mgr.actual_queue[s.queue_mgr.cursor + 1..].iter().position(|t| t.id == tr_id) {
                                            s.jump_to_queue_index(s.queue_mgr.cursor + 1 + pos, cx);
                                        } else {
                                            s.jump_to_track_id(&tr_id, cx);
                                        }
                                    });
                                }
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .min_w_0()
                                    .child(
                                        if let Some(path) = art {
                                            Artwork::new(px(36.0))
                                                .src(SharedString::from(path))
                                                .rounded(Radii::small())
                                                .into_any_element()
                                        } else {
                                            div()
                                                .size(px(36.0))
                                                .rounded(Radii::small())
                                                .bg(rgb(0x282828))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(div().text_xs().text_color(rgb(0x717171)).child("🎵"))
                                                .into_any_element()
                                        }
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child(item.title.clone())
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0xAAAAAA))
                                                    .child(subtitle)
                                            )
                                    )
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xAAAAAA))
                                    .child(dur_str)
                            )
                            .into_any_element()
                    );
                }
            }

            // Lane 3: Radio / Suggestions
            if is_autoplay && !q_snap.radio_upcoming.is_empty() {
                if let Some(cr) = chips_row_opt.take() {
                    queue_elements.push(cr);
                }
                queue_elements.push(
                    div()
                        .pt(px(12.0))
                        .pb(px(4.0))
                        .px(px(4.0))
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0xAAAAAA))
                        .child(format!("AUTOPLAY & SIMILAR • {}", q_snap.radio_upcoming.len()))
                        .into_any_element()
                );

                for (idx, item) in q_snap.radio_upcoming.iter().enumerate() {
                    let art_url = item.artwork_url.as_ref().filter(|u| !u.is_empty()).cloned().or_else(|| {
                        if item.id.len() == 11 && !item.id.contains('/') && !item.id.contains('\\') {
                            Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", item.id))
                        } else {
                            None
                        }
                    });
                    let art = state.thumbnail_manager.resolve_or_queue(&art_url, None);
                    let app_state_item = app_state_for_queue.clone();
                    let dur_str = format!("{}:{:02}", item.duration_secs / 60, item.duration_secs % 60);
                    let raw_sub = if !item.album.is_empty() && item.album != "YouTube Music" && !item.album.starts_with("YouTube") {
                        format!("{} • {}", item.artists.join(", "), item.album)
                    } else {
                        item.artists.join(", ")
                    };
                    let subtitle = clean_artist_subtitle(&raw_sub);

                    queue_elements.push(
                        div()
                            .id(SharedString::from(format!("radio_track_{}", idx)))
                            .flex()
                            .items_center()
                            .justify_between()
                            .px(px(8.0))
                            .py(px(6.0))
                            .rounded(Radii::small())
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xFFFFFF15)))
                            .child(
                                div()
                                    .id(SharedString::from(format!("radio_track_info_{}", idx)))
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .min_w_0()
                                    .on_click(cx.listener({
                                        move |_this, _ev, _win, cx| {
                                            app_state_item.update(cx, |s, cx| {
                                                s.play_radio_item(idx, cx);
                                            });
                                        }
                                    }))
                                    .child(
                                        if let Some(path) = art {
                                            Artwork::new(px(36.0))
                                                .src(SharedString::from(path))
                                                .rounded(Radii::small())
                                                .into_any_element()
                                        } else {
                                            div()
                                                .size(px(36.0))
                                                .rounded(Radii::small())
                                                .bg(rgb(0x282828))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(div().text_xs().text_color(rgb(0x717171)).child("🎵"))
                                                .into_any_element()
                                        }
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child(item.title.clone())
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0xAAAAAA))
                                                    .child(subtitle)
                                            )
                                    )
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0xAAAAAA))
                                            .child(dur_str)
                                    )
                                    .child(
                                        div()
                                            .id(SharedString::from(format!("promote_radio_{}", idx)))
                                            .cursor_pointer()
                                            .p(px(4.0))
                                            .text_xs()
                                            .text_color(rgb(0xAAAAAA))
                                            .hover(|s| s.text_color(rgb(0xFFFFFF)))
                                            .on_click(cx.listener({
                                                let state_prom = app_state_for_queue.clone();
                                                move |_this, _ev, _win, cx| {
                                                    state_prom.update(cx, |s, cx| {
                                                        s.promote_radio_item(idx, cx);
                                                    });
                                                }
                                            }))
                                            .child("＋ Queue")
                                    )
                            )
                            .into_any_element()
                    );
                }
            }
        } else {
            if let Some(cr) = chips_row_opt.take() {
                queue_elements.push(cr);
            }
            match &state.queue_snapshot {
                Loadable::Loading => {
                    for idx in 0..6 {
                        queue_elements.push(
                            div()
                                .id(SharedString::from(format!("queue_skeleton_{}", idx)))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px(px(8.0))
                                .py(px(6.0))
                                .rounded(Radii::small())
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(12.0))
                                        .child(
                                            div()
                                                .size(px(36.0))
                                                .rounded(Radii::small())
                                                .bg(rgba(0xFFFFFF15))
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap(px(6.0))
                                                .child(
                                                    div().w(px(140.0)).h(px(12.0)).rounded(px(2.0)).bg(rgba(0xFFFFFF20))
                                                )
                                                .child(
                                                    div().w(px(80.0)).h(px(10.0)).rounded(px(2.0)).bg(rgba(0xFFFFFF10))
                                                )
                                        )
                                )
                                .child(
                                    div().w(px(32.0)).h(px(10.0)).rounded(px(2.0)).bg(rgba(0xFFFFFF10))
                                )
                                .into_any_element()
                        );
                    }
                }
                Loadable::Ready(snap) => {
                    if snap.upcoming.is_empty() {
                        let state_clone = app_state_for_queue.clone();
                        queue_elements.push(
                            div()
                                .id("queue_empty_banner")
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .p(px(24.0))
                                .gap(px(12.0))
                                .rounded(Radii::base())
                                .bg(rgba(0xFFFFFF08))
                                .border_1()
                                .border_color(rgba(0xFFFFFF15))
                                .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("Queue is empty"))
                                .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Play any track from the home feed or start a YouTube radio station."))
                                .child(
                                    div()
                                        .id("queue_start_radio_btn")
                                        .px(px(16.0))
                                        .py(px(6.0))
                                        .rounded(Radii::full())
                                        .bg(rgb(0xFFFFFF))
                                        .cursor_pointer()
                                        .hover(|s| s.bg(rgb(0xDDDDDD)))
                                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                            state_clone.update(cx, |s, cx| {
                                                s.load_queue(cx);
                                            });
                                        }))
                                        .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0x030303)).child("Start YouTube Radio"))
                                )
                                .into_any_element()
                        );
                    } else {
                        let app_state_for_queue = self.app_state.clone();
                        for (idx, item) in snap.upcoming.iter().enumerate() {
                            let art = state.thumbnail_manager.resolve_or_queue(&item.thumbnail_url, None);
                            let app_state_item = app_state_for_queue.clone();

                            queue_elements.push(
                                div()
                                    .id(SharedString::from(format!("queue_track_{}", idx + 1)))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px(px(8.0))
                                    .py(px(6.0))
                                    .rounded(Radii::small())
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgba(0xFFFFFF15)))
                                    .on_click(cx.listener({
                                        move |_this, _ev, _win, cx| {
                                            app_state_item.update(cx, |s, cx| {
                                                s.play_queue_track_at(idx, cx);
                                            });
                                        }
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(12.0))
                                            .min_w_0()
                                            .child(
                                                if let Some(path) = art {
                                                    Artwork::new(px(36.0))
                                                        .src(SharedString::from(path))
                                                        .rounded(Radii::small())
                                                        .into_any_element()
                                                } else {
                                                    div()
                                                        .size(px(36.0))
                                                        .rounded(Radii::small())
                                                        .bg(rgb(0x282828))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .child(div().text_xs().text_color(rgb(0x717171)).child("🎵"))
                                                        .into_any_element()
                                                }
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(2.0))
                                                    .min_w_0()
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_weight(FontWeight::MEDIUM)
                                                            .text_color(rgb(0xFFFFFF))
                                                            .child(item.title.clone())
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(0xAAAAAA))
                                                            .child(item.artist.clone())
                                                    )
                                            )
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0xAAAAAA))
                                            .child(item.duration_str.clone())
                                    )
                                    .into_any_element()
                            );
                        }
                    }
                }
                Loadable::Error(err) => {
                    let state_clone = app_state_for_queue.clone();
                    let err_msg = err.clone();
                    queue_elements.push(
                        div()
                            .id("queue_error_banner")
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .p(px(20.0))
                            .gap(px(10.0))
                            .rounded(Radii::base())
                            .bg(rgba(0xEF444415))
                            .border_1()
                            .border_color(rgba(0xEF444433))
                            .child(div().text_xs().text_color(rgb(0xFCA5A5)).child(format!("Unable to load queue: {}", err_msg)))
                            .child(
                                div()
                                    .id("queue_retry_btn")
                                    .px(px(14.0))
                                    .py(px(5.0))
                                    .rounded(Radii::full())
                                    .bg(rgba(0xFFFFFF20))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgba(0xFFFFFF33)))
                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                        state_clone.update(cx, |s, cx| {
                                            s.load_queue(cx);
                                        });
                                    }))
                                    .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xFFFFFF)).child("Retry Queue"))
                            )
                            .into_any_element()
                    );
                }
                Loadable::Idle => {
                    let state_clone = app_state_for_queue.clone();
                    queue_elements.push(
                        div()
                            .id("queue_idle_banner")
                            .flex()
                            .flex_col()
                            .items_center()
                            .p(px(20.0))
                            .gap(px(8.0))
                            .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("No tracks in queue."))
                            .child(
                                div()
                                    .id("queue_load_btn")
                                    .px(px(14.0))
                                    .py(px(5.0))
                                    .rounded(Radii::full())
                                    .bg(rgb(0x212121))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x333333)))
                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                        state_clone.update(cx, |s, cx| {
                                            s.load_queue(cx);
                                        });
                                    }))
                                    .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xFFFFFF)).child("Load YouTube Radio Queue"))
                            )
                            .into_any_element()
                    );
                }
            }
        }

        let tab_body = match active_tab {
            PlayerTab::Lyrics => {
                match &state.lyrics_doc {
                    Loadable::Loading => {
                        let mut col = div()
                            .id("lyrics_loading_list")
                            .flex()
                            .flex_col()
                            .gap(px(16.0))
                            .pt(px(16.0));
                        for idx in 0..8 {
                            col = col.child(
                                div()
                                    .w(if idx % 2 == 0 { px(240.0) } else { px(180.0) })
                                    .h(px(20.0))
                                    .rounded(px(4.0))
                                    .bg(rgba(0xFFFFFF15))
                            );
                        }
                        col.into_any_element()
                    }
                    Loadable::Ready(doc) => {
                        let mut col = div()
                            .id("player_lyrics_list")
                            .flex()
                            .flex_col()
                            .gap(px(18.0))
                            .pt(px(16.0));

                        for (idx, line) in doc.lines.iter().enumerate() {
                            let is_current_line = if let Some(ms) = line.start_time_ms {
                                let cur_ms = (current_pos_secs * 1000.0) as u64;
                                cur_ms >= ms && (idx == doc.lines.len() - 1 || cur_ms < doc.lines[idx + 1].start_time_ms.unwrap_or(u64::MAX))
                            } else {
                                false
                            };

                            col = col.child(
                                div()
                                    .text_xl()
                                    .font_weight(if is_current_line { FontWeight::BOLD } else { FontWeight::SEMIBOLD })
                                    .text_color(if is_current_line { rgb(0xFFFFFF) } else { rgb(0x888888) })
                                    .hover(|s| s.text_color(rgb(0xFFFFFF)))
                                    .child(line.text.clone())
                            );
                        }

                        if let Some(ref src) = doc.source {
                            col = col.child(
                                div()
                                    .pt(px(24.0))
                                    .text_xs()
                                    .text_color(rgb(0x666666))
                                    .child(format!("Source: {}", src))
                            );
                        }
                        col.into_any_element()
                    }
                    Loadable::Error(err) => {
                        let state_clone = self.app_state.clone();
                        div()
                            .id("lyrics_error_pane")
                            .flex()
                            .flex_col()
                            .items_start()
                            .gap(px(12.0))
                            .pt(px(24.0))
                            .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xFFFFFF)).child("Lyrics unavailable"))
                            .child(div().text_xs().text_color(rgb(0x888888)).child(format!("Could not load lyrics: {}", err)))
                            .child(
                                div()
                                    .id("lyrics_retry_btn")
                                    .px(px(14.0))
                                    .py(px(5.0))
                                    .rounded(Radii::full())
                                    .bg(rgb(0x212121))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x333333)))
                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                        state_clone.update(cx, |s, cx| {
                                            s.load_lyrics(cx);
                                        });
                                    }))
                                    .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xFFFFFF)).child("Retry Lyrics"))
                            )
                            .into_any_element()
                    }
                    Loadable::Idle => {
                        let state_clone = self.app_state.clone();
                        div()
                            .id("lyrics_idle_pane")
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .pt(px(24.0))
                            .child(div().text_sm().text_color(rgb(0xAAAAAA)).child("No lyrics loaded."))
                            .child(
                                div()
                                    .id("lyrics_load_btn")
                                    .px(px(14.0))
                                    .py(px(5.0))
                                    .rounded(Radii::full())
                                    .bg(rgb(0x212121))
                                    .cursor_pointer()
                                    .hover(|s| s.bg(rgb(0x333333)))
                                    .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                        state_clone.update(cx, |s, cx| {
                                            s.load_lyrics(cx);
                                        });
                                    }))
                                    .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xFFFFFF)).child("Fetch Synchronized Lyrics"))
                            )
                            .into_any_element()
                    }
                }
            }
            PlayerTab::Comments => {
                div()
                    .id("player_comments_pane")
                    .flex()
                    .flex_col()
                    .gap(px(14.0))
                    .pt(px(12.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xFFFFFF))
                            .child("Top Comments (1,248)")
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .p(px(10.0))
                                    .rounded(Radii::small())
                                    .bg(rgba(0xFFFFFF08))
                                    .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0xCCCCCC)).child("@AaravMusic • 2 days ago"))
                                    .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("The composition here is beyond ethereal! That bassline gives goosebumps every single time."))
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .p(px(10.0))
                                    .rounded(Radii::small())
                                    .bg(rgba(0xFFFFFF08))
                                    .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0xCCCCCC)).child("@DevRust • 1 week ago"))
                                    .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("100% native Rust player with zero Electron memory leaks. Pure poetry in motion!"))
                            )
                    )
                    .into_any_element()
            }
            PlayerTab::Related => {
                let related_albums_view = if let Loadable::Ready(ref items) = state.library_albums {
                    items.iter().take(5).map(|item| {
                        let a_id = item.id.clone();
                        let a_title = item.title.clone();
                        let a_sub = item.subtitle.clone();
                        let a_thumb = item.thumbnail_url.clone();
                        let art = a_thumb.as_ref().and_then(|u| state.thumbnail_manager.resolve_or_queue(&Some(u.clone()), None));
                        let app_state_for_rel = self.app_state.clone();
                        div()
                            .id(SharedString::from(format!("related_album_{}", a_id)))
                            .flex()
                            .items_center()
                            .gap(px(12.0))
                            .p(px(6.0))
                            .rounded(Radii::small())
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xFFFFFF10)))
                            .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                let aid = a_id.clone();
                                let atitle = a_title.clone();
                                let athumb = a_thumb.clone();
                                app_state_for_rel.update(cx, |s, cx| {
                                    s.open_album_detail(aid, atitle, athumb, cx);
                                });
                            }))
                            .child(
                                if let Some(p) = art {
                                    Artwork::new(px(42.0)).src(SharedString::from(p)).rounded(Radii::small()).into_any_element()
                                } else {
                                    div().size(px(42.0)).rounded(Radii::small()).bg(rgb(0x282828)).flex().items_center().justify_center().child(div().text_xs().text_color(rgb(0x888888)).child("💽")).into_any_element()
                                }
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(div().text_xs().font_weight(FontWeight::SEMIBOLD).text_color(rgb(0xFFFFFF)).child(item.title.clone()))
                                    .child(div().text_xs().text_color(rgb(0xAAAAAA)).child(a_sub))
                            )
                    }).collect::<Vec<_>>()
                } else {
                    Vec::new()
                };

                div()
                    .id("player_related_pane")
                    .flex()
                    .flex_col()
                    .gap(px(14.0))
                    .pt(px(12.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xFFFFFF))
                            .child("You Might Also Like")
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .children(related_albums_view)
                    )
                    .into_any_element()
            }
            PlayerTab::UpNext => {
                // UP NEXT
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .pb(px(24.0))
                    .child(subheader)
                    .child(autoplay_row)
                    .child(
                        div()
                            .id("player_queue_list")
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .pt(px(4.0))
                            .children(queue_elements)
                    )
                    .into_any_element()
            }
        };

        let mut right_column = div()
            .flex()
            .flex_col()
            .w(if is_mobile { px(340.0) } else { px(500.0) })
            .min_w(if is_mobile { px(300.0) } else { px(420.0) })
            .gap(px(12.0));

        if is_mobile {
            right_column = right_column
                .child(tabs_row)
                .child(tab_body);
        } else {
            right_column = right_column
                .h_full()
                .min_h_0()
                .child(
                    tabs_row
                        .w_full()
                        .flex_none()
                )
                .child(
                    div()
                        .id("player_tab_body_scroller")
                        .flex_1()
                        .min_h_0()
                        .w_full()
                        .overflow_y_scroll()
                        .child(tab_body)
                );
        }

        let mut container = div()
            .id("player_page_container")
            .size_full()
            .min_h_0()
            .bg(rgba(0x00000000));

        if is_mobile {
            container = container
                .flex()
                .flex_col()
                .items_center()
                .overflow_y_scroll()
                .p(px(16.0))
                .pb(px(80.0))
                .gap(px(24.0));
        } else {
            container = container
                .flex()
                .flex_row()
                .items_center()
                .justify_center()
                .overflow_hidden()
                .p(px(32.0))
                .gap(px(48.0));
        }

        container
            .child(left_column)
            .child(right_column)
    }
}

// =========================================================================
// YTM 04 • PLAYLIST DETAIL PAGE (Private Playlist / Slow Chill)
// =========================================================================
pub struct PlaylistDetailPage {
    app_state: Entity<AppState>,
    #[allow(dead_code)]
    sort_mode: String,
}

impl PlaylistDetailPage {
    pub fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        cx.observe(&app_state, |_this, _state, cx| {
            cx.notify();
        }).detach();
        Self {
            app_state,
            sort_mode: "Custom order".to_string(),
        }
    }
}

impl Render for PlaylistDetailPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state.read(cx);

        if let Loadable::Ready(ref details) = state.active_playlist {
            let title = details.title.clone();
            let subtitle = details.subtitle.clone();
            let description = details.description.clone();
            let thumb_path = details.thumbnail_url.as_ref().and_then(|u| state.thumbnail_manager.resolve_or_queue(&Some(u.clone()), None));
            let current_track_key = state.current_track.as_ref().map(|t| t.id.clone());
            let is_playing = state.playback_state == PlaybackState::Playing;

            let app_state_play = self.app_state.clone();
            let app_state_shuffle = self.app_state.clone();
            let details_clone = details.clone();
            let details_for_shuffle = details.clone();
            let app_state_dl = self.app_state.clone();
            let details_for_dl = details.clone();

            let left_hero = div()
                .flex()
                .flex_col()
                .items_center()
                .w(px(280.0))
                .min_w(px(260.0))
                .gap(px(16.0))
                .child(
                    div()
                        .size(px(240.0))
                        .rounded(px(10.0))
                        .overflow_hidden()
                        .bg(rgb(0x1F1F1F))
                        .shadow(vec![
                            gpui::BoxShadow {
                                color: rgba(0x00000099).into(),
                                offset: gpui::point(px(0.0), px(8.0)),
                                blur_radius: px(24.0),
                                spread_radius: px(4.0),
                                inset: false,
                            }
                        ])
                        .child(
                            if let Some(ref p) = thumb_path {
                                Artwork::new(px(240.0))
                                    .src(SharedString::from(p.clone()))
                                    .rounded(px(10.0))
                                    .into_any_element()
                            } else {
                                div()
                                    .size_full()
                                    .bg(rgb(0x282828))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(div().text_2xl().text_color(rgb(0x666666)).child("🎵"))
                                    .into_any_element()
                            }
                        )
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(4.0))
                        .child(
                            div()
                                .text_2xl()
                                .font_weight(FontWeight::EXTRA_BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .text_center()
                                .child(title)
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(rgb(0xAAAAAA))
                                .child(subtitle)
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(0x717171))
                                .child(format!("{} tracks", details.tracks.len()))
                        )
                        .when_some(description, |this, desc| {
                            this.child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x888888))
                                    .max_h(px(60.0))
                                    .overflow_hidden()
                                    .text_center()
                                    .child(desc)
                            )
                        })
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.0))
                        .pt(px(4.0))
                        .child(
                            div()
                                .id("playlist_btn_play")
                                .px(px(20.0))
                                .py(px(8.0))
                                .rounded(Radii::full())
                                .bg(rgb(0xFFFFFF))
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgb(0xEEEEEE)))
                                .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                    let d = details_clone.clone();
                                    app_state_play.update(cx, |s, cx| {
                                        s.play_collection(&d, false, cx);
                                    });
                                }))
                                .child(div().text_sm().text_color(rgb(0x000000)).child("▶"))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0x000000))
                                        .child("Play")
                                )
                        )
                        .child(
                            div()
                                .id("playlist_btn_shuffle")
                                .px(px(16.0))
                                .py(px(8.0))
                                .rounded(Radii::full())
                                .bg(rgba(0xFFFFFF14))
                                .border_1()
                                .border_color(rgba(0xFFFFFF1A))
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .cursor_pointer()
                                .hover(|s| s.bg(rgba(0xFFFFFF26)))
                                .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                    let d = details_for_shuffle.clone();
                                    app_state_shuffle.update(cx, |s, cx| {
                                        s.play_collection(&d, true, cx);
                                    });
                                }))
                                .child(div().text_sm().text_color(rgb(0xFFFFFF)).child("🔀"))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(rgb(0xFFFFFF))
                                        .child("Shuffle")
                                )
                        )
                        .child(
                            div()
                                .id("playlist_btn_download")
                                .size(px(36.0))
                                .rounded(Radii::full())
                                .bg(rgba(0xFFFFFF14))
                                .border_1()
                                .border_color(rgba(0xFFFFFF1A))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .hover(|s| s.bg(rgba(0xFFFFFF26)))
                                .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                    let d = details_for_dl.clone();
                                    app_state_dl.update(cx, |s, cx| {
                                        s.trigger_download_collection(d, cx);
                                    });
                                }))
                                .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("↓"))
                        )
                        .child(
                            div()
                                .id("playlist_btn_bookmark")
                                .size(px(36.0))
                                .rounded(Radii::full())
                                .bg(rgba(0xFFFFFF14))
                                .border_1()
                                .border_color(rgba(0xFFFFFF1A))
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .hover(|s| s.bg(rgba(0xFFFFFF26)))
                                .child(div().text_xs().text_color(rgb(0xFFFFFF)).child("🔖"))
                        )
                );

            let mut track_rows = Vec::new();
            for (idx, track) in details.tracks.iter().enumerate() {
                let is_cur = current_track_key.as_ref() == Some(&track.id);
                let dur_secs = track.duration_secs;
                let dur_str = format!("{}:{:02}", dur_secs / 60, dur_secs % 60);
                let artists_str = track.artists.join(", ");
                let app_state_track = self.app_state.clone();
                let details_track = details.clone();

                track_rows.push(
                    div()
                        .id(SharedString::from(format!("pl_yt_track_{}", idx)))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px(px(16.0))
                        .py(px(8.0))
                        .rounded(px(6.0))
                        .cursor_pointer()
                        .bg(if is_cur { rgba(0xFFFFFF1A) } else { rgba(0x00000000) })
                        .hover(|s| s.bg(rgba(0xFFFFFF14)))
                        .on_click(cx.listener({
                            let d = details_track.clone();
                            move |_this, _ev, _win, cx| {
                                app_state_track.update(cx, |s, cx| {
                                    s.play_collection_track(&d, idx, cx);
                                });
                            }
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(16.0))
                                .min_w_0()
                                .child(
                                    div()
                                        .w(px(24.0))
                                        .flex()
                                        .justify_center()
                                        .text_xs()
                                        .text_color(if is_cur { rgb(0xFFFFFF) } else { rgb(0x888888) })
                                        .child(SharedString::from(if is_cur && is_playing { "▶".to_string() } else { format!("{}", idx + 1) }))
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(2.0))
                                        .min_w_0()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(if is_cur { rgb(0xFFFFFF) } else { rgb(0xEEEEEE) })
                                                .child(track.title.clone())
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x888888))
                                                .child(artists_str)
                                        )
                                )
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(16.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x888888))
                                        .child(track.album.clone())
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(if is_cur { rgb(0xFFFFFF) } else { rgb(0xAAAAAA) })
                                        .child(dur_str)
                                )
                        )
                );
            }

            return div()
                .id("playlist_page_container")
                .size_full()
                .min_h_0()
                .bg(rgba(0x00000000))
                .flex()
                .flex_row()
                .p(px(32.0))
                .pb(px(88.0))
                .gap(px(48.0))
                .overflow_y_scroll()
                .child(left_hero)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .children(track_rows)
                )
                .into_any_element();
        } else if let Loadable::Loading = state.active_playlist {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .p(px(32.0))
                .child(
                    div()
                        .text_base()
                        .text_color(rgb(0xAAAAAA))
                        .child("Loading playlist from YouTube Music...")
                )
                .into_any_element();
        } else if let Loadable::Error(ref err) = state.active_playlist {
            let app_state_retry = self.app_state.clone();
            return div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(16.0))
                .p(px(32.0))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0xEF4444))
                        .child(format!("Failed to load playlist: {}", err))
                )
                .child(
                    Button::new("Retry")
                        .variant(ButtonVariant::Primary)
                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                            app_state_retry.update(cx, |s, cx| s.load_library(cx));
                        }))
                )
                .into_any_element();
        }

        // 2x2 Collage Artwork with Edit Pencil Badge (Exact Figma 7:696)
        let collage_art = div()
            .relative()
            .size(px(240.0))
            .rounded(px(10.0))
            .overflow_hidden()
            .shadow(vec![
                gpui::BoxShadow {
                    color: rgba(0x00000099).into(),
                    offset: gpui::point(px(0.0), px(8.0)),
                    blur_radius: px(24.0),
                    spread_radius: px(4.0),
                    inset: false,
                }
            ])
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .size_full()
                    .child(Artwork::new(px(120.0)).src(SharedString::from("assets/artwork/velvet_midnight.jpg")))
                    .child(Artwork::new(px(120.0)).src(SharedString::from("assets/artwork/tokyo_1984.jpg")))
                    .child(Artwork::new(px(120.0)).src(SharedString::from("assets/artwork/slow_chill.jpg")))
                    .child(Artwork::new(px(120.0)).src(SharedString::from("assets/artwork/golden_hour.jpg")))
            )
            .child(
                // Edit Pencil Badge in Bottom Right
                div()
                    .absolute()
                    .bottom(px(8.0))
                    .right(px(8.0))
                    .size(px(36.0))
                    .rounded(Radii::full())
                    .bg(rgb(0xFFFFFF))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0xEEEEEE)))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x030303))
                            .child("✏")
                    )
            );

        // Left Column: Playlist Hero
        let engine_for_play = state.engine.clone();
        let first_tk = state.tracks.first().map(|t| t.track_key.clone());

        let left_hero = div()
            .flex()
            .flex_col()
            .items_center()
            .w(px(272.0))
            .min_w(px(260.0))
            .gap(px(16.0))
            .child(collage_art)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.0))
                    // Title: Slow Chill
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xFFFFFF))
                            .child("Slow Chill")
                    )
                    // Creator Ketan row
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .size(px(20.0))
                                    .rounded(Radii::full())
                                    .bg(rgb(0x0284C7))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(rgb(0xFFFFFF))
                                            .child("K")
                                    )
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("Ketan")
                            )
                    )
                    // Metadata lines
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xAAAAAA))
                            .child("Playlist • Private • 2026")
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xAAAAAA))
                            .child("229 tracks • 6+ hours")
                    )
            )
            // Action Buttons Row: Download, Edit, Big White Play, Share, 3-dots
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .pt(px(4.0))
                    // Download
                    .child(
                        div()
                            .id("playlist_btn_download")
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgb(0x181818))
                            .border_1()
                            .border_color(rgb(0x333333))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x282828)))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xFFFFFF))
                                    .child("↓")
                            )
                    )
                    // Edit Pencil
                    .child(
                        div()
                            .id("playlist_btn_edit")
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgb(0x181818))
                            .border_1()
                            .border_color(rgb(0x333333))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x282828)))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xFFFFFF))
                                    .child("✏")
                            )
                    )
                    // Big White Circular Play Button (52px)
                    .child(
                        div()
                            .id("playlist_btn_big_play")
                            .size(px(52.0))
                            .rounded(Radii::full())
                            .bg(rgb(0xFFFFFF))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0xEEEEEE)))
                            .on_click(cx.listener(move |_this, _ev, _win, _cx| {
                                if let Some(ref tk) = first_tk {
                                    engine_for_play.send_command(Command {
                                        origin: None,
                                        action: CommandAction::PlayTrack(tk.clone()),
                                    });
                                } else {
                                    engine_for_play.send_command(Command {
                                        origin: None,
                                        action: CommandAction::TogglePlay,
                                    });
                                }
                            }))
                            .child(
                                div()
                                    .text_base()
                                    .text_color(rgb(0x000000))
                                    .child("▶")
                            )
                    )
                    // Share
                    .child(
                        div()
                            .id("playlist_btn_share")
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgb(0x181818))
                            .border_1()
                            .border_color(rgb(0x333333))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x282828)))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xFFFFFF))
                                    .child("↗")
                            )
                    )
                    // 3-Dots
                    .child(
                        div()
                            .id("playlist_btn_more")
                            .size(px(36.0))
                            .rounded(Radii::full())
                            .bg(rgb(0x181818))
                            .border_1()
                            .border_color(rgb(0x333333))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x282828)))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xFFFFFF))
                                    .child("•••")
                            )
                    )
            );

        // Right Column: Sort & 9 Curated Tracks
        struct PlaylistTrackDef {
            idx: usize,
            title: &'static str,
            artist_album: &'static str,
            duration: &'static str,
            art_src: &'static str,
            is_liked: bool,
        }

        let playlist_tracks = [
            PlaylistTrackDef { idx: 1, title: "Numb", artist_album: "Men I Trust • Numb", duration: "3:40", art_src: "assets/artwork/velvet_midnight.jpg", is_liked: true },
            PlaylistTrackDef { idx: 2, title: "BIRDS OF A FEATHER", artist_album: "Billie Eilish • HIT ME HARD AND SOFT", duration: "3:31", art_src: "assets/artwork/rainy_afternoon.jpg", is_liked: true },
            PlaylistTrackDef { idx: 3, title: "Sunsetz", artist_album: "Cigarettes After Sex • Cigarettes After Sex", duration: "3:35", art_src: "assets/artwork/slow_chill.jpg", is_liked: false },
            PlaylistTrackDef { idx: 4, title: "Mind Over Matter (Reprise)", artist_album: "Young the Giant • Mirrorball / Mind Over Matter (Reprise)", duration: "3:51", art_src: "assets/artwork/tokyo_1984.jpg", is_liked: true },
            PlaylistTrackDef { idx: 5, title: "If You Were Mine", artist_album: "MINOVA • If You Were Mine", duration: "3:39", art_src: "assets/artwork/golden_hour.jpg", is_liked: false },
            PlaylistTrackDef { idx: 6, title: "Beanie", artist_album: "Chezile • Beanie", duration: "2:13", art_src: "assets/artwork/neon_horizon.jpg", is_liked: false },
            PlaylistTrackDef { idx: 7, title: "Don't Want A Love Song", artist_album: "Bryant Barnes • Don't Want A Love Song", duration: "3:04", art_src: "assets/artwork/folklore.jpg", is_liked: false },
            PlaylistTrackDef { idx: 8, title: "THE SHADE", artist_album: "Rex Orange County • WHO CARES?", duration: "3:02", art_src: "assets/artwork/rehbara.jpg", is_liked: false },
            PlaylistTrackDef { idx: 9, title: "The Less I Know The Better", artist_album: "Tame Impala • Currents", duration: "3:36", art_src: "assets/artwork/late_night_lofi.jpg", is_liked: true },
        ];

        let mut track_rows = Vec::new();
        for def in &playlist_tracks {
            let matched_track = state.tracks.iter().find(|t| t.title.eq_ignore_ascii_case(def.title))
                .or_else(|| state.tracks.get(def.idx));
            let tk = matched_track.map(|t| t.track_key.clone()).unwrap_or_else(|| format!("playlist_track_{}", def.idx));
            let eng = state.engine.clone();

            track_rows.push(
                div()
                    .id(SharedString::from(format!("playlist_track_{}", def.idx)))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(16.0))
                    .py(px(8.0))
                    .rounded(px(6.0))
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x181818)))
                    .on_click(cx.listener({
                        let tk = tk.clone();
                        move |_this, _ev, _win, _cx| {
                            eng.send_command(Command {
                                origin: None,
                                action: CommandAction::PlayTrack(tk.clone()),
                            });
                        }
                    }))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(16.0))
                            .min_w_0()
                            // 44x44 Artwork Thumbnail
                            .child(
                                div()
                                    .size(px(44.0))
                                    .rounded(px(3.0))
                                    .overflow_hidden()
                                    .child(
                                        Artwork::new(px(44.0))
                                            .src(SharedString::from(def.art_src))
                                            .rounded(px(3.0))
                                    )
                            )
                            // Title & Artist/Album
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .min_w_0()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(rgb(0xFFFFFF))
                                            .child(def.title)
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0xAAAAAA))
                                            .child(def.artist_album)
                                    )
                            )
                    )
                    // Right: Thumbs up icon + duration
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(20.0))
                            .children(if def.is_liked {
                                Some(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xFFFFFF))
                                        .child("👍")
                                )
                            } else {
                                None
                            })
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xAAAAAA))
                                    .child(def.duration)
                            )
                    )
            );
        }

        div()
            .id("playlist_detail_container")
            .size_full()
            .min_h_0()
            .bg(rgba(0x00000000))
            .flex()
            .flex_row()
            .p(px(32.0))
            .pb(px(88.0))
            .gap(px(48.0))
            .overflow_y_scroll()
            .child(left_hero)
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    // Sort header row: ≡ Sort
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .pb(px(8.0))
                            .border_b_1()
                            .border_color(rgb(0x212121))
                            .cursor_pointer()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child("≡ Sort")
                            )
                    )
                    .children(track_rows)
            )
            .into_any_element()
    }
}

// =========================================================================
// SEARCH PAGE
// =========================================================================
pub struct SearchPage {
    app_state: Entity<AppState>,
}

impl SearchPage {
    pub fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        cx.observe(&app_state, |_this, _state, cx| {
            cx.notify();
        }).detach();
        Self {
            app_state,
        }
    }
}

impl Render for SearchPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state.read(cx);
        let current_query = state.search_query.clone();

        let popular_chips = [
            "Coldplay", "Hans Zimmer", "The Weeknd", "Dua Lipa",
            "Lofi Beats", "Radiohead", "Billie Eilish", "Daft Punk",
            "Synthwave", "Taylor Swift", "Kendrick Lamar",
        ];

        let state_for_chips = self.app_state.clone();
        let chips_row = div()
            .flex()
            .items_center()
            .flex_wrap()
            .gap(px(8.0))
            .children(popular_chips.into_iter().map(|chip| {
                let state_clone = state_for_chips.clone();
                let chip_str = chip.to_string();
                let is_active = current_query.eq_ignore_ascii_case(chip);
                div()
                    .id(SharedString::from(format!("search_chip_{}", chip)))
                    .px(px(14.0))
                    .py(px(6.0))
                    .rounded(Radii::full())
                    .bg(if is_active { rgb(0xFFFFFF) } else { rgb(0x212121) })
                    .border_1()
                    .border_color(if is_active { rgb(0xFFFFFF) } else { rgba(0xFFFFFF15) })
                    .cursor_pointer()
                    .hover(|s| s.bg(rgb(0x383838)))
                    .on_click(cx.listener({
                        let c = chip_str.clone();
                        move |_this, _ev, _win, cx| {
                            state_clone.update(cx, |s, cx| {
                                s.perform_search(c.clone(), cx);
                            });
                        }
                    }))
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if is_active { rgb(0x030303) } else { rgb(0xFFFFFF) })
                            .child(chip_str)
                    )
            }));

        let content = match &state.search_results {
            Loadable::Loading => {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .py(px(60.0))
                    .gap(px(12.0))
                    .child(div().text_2xl().child("🔍"))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xFFFFFF))
                            .child(format!("Searching YouTube Music for \"{}\"...", current_query))
                    )
                    .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Streaming directly from YouTube Music InnerTube backend"))
                    .into_any_element()
            }
            Loadable::Ready(tracks) if tracks.is_empty() => {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .py(px(60.0))
                    .gap(px(8.0))
                    .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("No results found"))
                    .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Try a different search term or select a category above."))
                    .into_any_element()
            }
            Loadable::Ready(tracks) => {
                let state_for_play = self.app_state.clone();
                let mut rows = Vec::new();
                for (idx, track) in tracks.iter().enumerate() {
                    let track_id = track.id.clone();
                    let title = track.title.clone();
                    let artists_str = track.artists.join(", ");
                    let thumb_url = track.artwork_url.clone();
                    let art_path = state.thumbnail_manager.resolve_or_queue(&track.artwork_url, None);
                    let dur_secs = track.duration_secs;
                    let dur_str = format!("{}:{:02}", dur_secs / 60, dur_secs % 60);

                    let state_item = state_for_play.clone();
                    rows.push(
                        div()
                            .id(SharedString::from(format!("search_res_track_{}", idx + 1)))
                            .flex()
                            .items_center()
                            .justify_between()
                            .px(px(12.0))
                            .py(px(8.0))
                            .rounded(Radii::small())
                            .cursor_pointer()
                            .hover(|s| s.bg(rgba(0xFFFFFF15)))
                            .on_click(cx.listener({
                                let tid = track_id.clone();
                                let tt = title.clone();
                                let ta = artists_str.clone();
                                let th = thumb_url.clone();
                                move |_this, _ev, _win, cx| {
                                    state_item.update(cx, |s, cx| {
                                        s.play_youtube_track(
                                            tid.clone(),
                                            tt.clone(),
                                            ta.clone(),
                                            th.clone(),
                                            Some("Search Results".into()),
                                            cx,
                                        );
                                    });
                                }
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(16.0))
                                    .min_w_0()
                                    .child(
                                        div()
                                            .w(px(24.0))
                                            .text_xs()
                                            .text_color(rgb(0x888888))
                                            .child(format!("{}", idx + 1))
                                    )
                                    .child(
                                        if let Some(path) = art_path {
                                            Artwork::new(px(44.0))
                                                .src(SharedString::from(path))
                                                .rounded(Radii::small())
                                                .into_any_element()
                                        } else {
                                            div()
                                                .size(px(44.0))
                                                .rounded(Radii::small())
                                                .bg(rgb(0x282828))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(div().text_xs().text_color(rgb(0x717171)).child("🎵"))
                                                .into_any_element()
                                        }
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child(title)
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0xAAAAAA))
                                                    .child(artists_str)
                                            )
                                    )
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x888888))
                                    .child(dur_str)
                            )
                    );
                }

                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xFFFFFF))
                            .pb(px(8.0))
                            .child(format!("Top Results for \"{}\"", current_query))
                    )
                    .children(rows)
                    .into_any_element()
            }
            Loadable::Error(err) => {
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .py(px(40.0))
                    .gap(px(8.0))
                    .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(rgb(0xEF4444)).child("Search failed"))
                    .child(div().text_xs().text_color(rgb(0xAAAAAA)).child(err.clone()))
                    .into_any_element()
            }
            Loadable::Idle => {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .child(
                        div()
                            .p(px(20.0))
                            .rounded(Radii::large())
                            .bg(rgb(0x181818))
                            .border_1()
                            .border_color(rgba(0xFFFFFF12))
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(div().text_base().font_weight(FontWeight::BOLD).text_color(rgb(0xFFFFFF)).child("Explore YouTube Music"))
                            .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Type in the search bar above or select any artist/genre chip to search millions of songs instantly."))
                    )
                    .into_any_element()
            }
        };

        div()
            .id("search_page_container")
            .size_full()
            .min_h_0()
            .bg(rgba(0x00000000))
            .flex()
            .flex_col()
            .p(px(32.0))
            .pb(px(88.0))
            .gap(px(24.0))
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xFFFFFF))
                            .child(if current_query.is_empty() {
                                "Explore & Search".to_string()
                            } else {
                                format!("Results for \"{}\"", current_query)
                            })
                    )
            )
            .child(chips_row)
            .child(content)
    }
}
