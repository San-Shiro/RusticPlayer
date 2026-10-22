// crates/app/src/artist_page.rs
//! Dedicated Artist Detail Page for YouTube Music artists.
//!
//! Renders:
//! - Hero header with circular avatar, name, subscriber pill badge, and bio snippet.
//! - Action bar with "Play" (top songs) and "Shuffle".
//! - Top Songs shelf with ranked track rows and inline playback.
//! - Albums & Singles carousels with navigation to album views.
//! - Fans Might Also Like section with navigation to related artists.

use gpui::prelude::*;
use gpui::*;
use rustic_core::api::PlaybackState;
use ui_kit::atoms::artwork::Artwork;
use ui_kit::tokens::Radii;
use crate::engine_bridge::{AppState, Loadable};

pub struct ArtistPage {
    app_state: Entity<AppState>,
}

impl ArtistPage {
    pub fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        cx.observe(&app_state, |_this, _state, cx| {
            cx.notify();
        }).detach();
        Self { app_state }
    }
}

impl Render for ArtistPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state.read(cx);

        match state.active_artist {
            Loadable::Idle => {
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(rgb(0x888888))
                    .child("Select an artist to view details")
                    .into_any_element()
            }
            Loadable::Loading => {
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(16.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xFFFFFF))
                            .child("Loading artist details...")
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x888888))
                            .child("Fetching top songs and discography from YouTube Music")
                    )
                    .into_any_element()
            }
            Loadable::Error(ref err) => {
                let err_msg = err.clone();
                let app_state = self.app_state.clone();
                let artist_id = state.active_artist_id.clone().unwrap_or_default();

                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(16.0))
                    .child(div().text_xl().text_color(rgb(0xFF5555)).child("Failed to load artist"))
                    .child(div().text_sm().text_color(rgb(0xAAAAAA)).child(err_msg))
                    .child(
                        div()
                            .id("artist_retry_btn")
                            .px(px(20.0))
                            .py(px(8.0))
                            .rounded(Radii::full())
                            .bg(rgb(0xFFFFFF))
                            .text_color(rgb(0x000000))
                            .font_weight(FontWeight::BOLD)
                            .cursor_pointer()
                            .child("Retry")
                            .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                let aid = artist_id.clone();
                                app_state.update(cx, |s, cx| {
                                    s.open_artist_detail(aid, "Artist".into(), None, cx);
                                });
                            }))
                    )
                    .into_any_element()
            }
            Loadable::Ready(ref details) => {
                let name = details.name.clone();
                let subscribers = details.subscribers.clone();
                let description = details.description.clone();
                let avatar_path = details.avatar_url.as_ref().and_then(|u| {
                    state.thumbnail_manager.resolve_or_queue(&Some(u.clone()), None)
                });
                let current_id = state.current_track.as_ref().map(|t| t.id.clone());
                let is_playing = state.playback_state == PlaybackState::Playing;

                let app_state_play = self.app_state.clone();
                let app_state_shuffle = self.app_state.clone();
                let details_clone = details.clone();
                let details_shuffle = details.clone();

                // 1. Hero header (Circular Avatar + Title + Subscribers + Play/Shuffle)
                let hero_section = div()
                    .flex()
                    .items_center()
                    .gap(px(28.0))
                    .pb(px(24.0))
                    .border_b_1()
                    .border_color(rgba(0xFFFFFF14))
                    .child(
                        // Circular Avatar
                        div()
                            .size(px(180.0))
                            .min_w(px(180.0))
                            .rounded(Radii::full())
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
                                if let Some(ref p) = avatar_path {
                                    Artwork::new(px(180.0))
                                        .src(SharedString::from(p.clone()))
                                        .rounded(Radii::full())
                                        .into_any_element()
                                } else {
                                    div()
                                        .size_full()
                                        .bg(rgb(0x282828))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(div().text_3xl().text_color(rgb(0x666666)).child("👤"))
                                        .into_any_element()
                                }
                            )
                    )
                    .child(
                        // Details: Name, Subscribers, Bio, Action Buttons
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(10.0))
                            .flex_1()
                            .child(
                                div()
                                    .text_3xl()
                                    .font_weight(FontWeight::EXTRA_BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .child(name)
                            )
                            .when_some(subscribers, |this, subs| {
                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .px(px(10.0))
                                                .py(px(3.0))
                                                .rounded(Radii::full())
                                                .bg(rgba(0xFFFFFF18))
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xCCCCCC))
                                                .child(subs)
                                        )
                                )
                            })
                            .when_some(description, |this, desc| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0x888888))
                                        .max_h(px(48.0))
                                        .overflow_hidden()
                                        .child(desc)
                                )
                            })
                            .child(
                                // Action bar: Play & Shuffle
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .pt(px(6.0))
                                    .child(
                                        div()
                                            .id("artist_btn_play")
                                            .px(px(22.0))
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
                                                    s.play_artist_track(&d, 0, cx);
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
                                            .id("artist_btn_shuffle")
                                            .px(px(18.0))
                                            .py(px(8.0))
                                            .rounded(Radii::full())
                                            .bg(rgba(0xFFFFFF14))
                                            .border_1()
                                            .border_color(rgba(0xFFFFFF1A))
                                            .flex()
                                            .items_center()
                                            .gap(px(6.0))
                                            .cursor_pointer()
                                            .hover(|s| s.bg(rgba(0xFFFFFF22)))
                                            .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                                let d = details_shuffle.clone();
                                                app_state_shuffle.update(cx, |s, cx| {
                                                    s.shuffle_artist_tracks(&d, cx);
                                                });
                                            }))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child("🔀 Shuffle")
                                            )
                                    )
                            )
                    );

                // 2. Top Songs Shelf
                let top_songs_section = if !details.top_songs.is_empty() {
                    let d_ref = details.clone();
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .child("Top Songs")
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .children(details.top_songs.iter().take(10).enumerate().map(|(idx, track)| {
                                    let app_state_track = self.app_state.clone();
                                    let d_for_click = d_ref.clone();
                                    let track_clone = track.clone();
                                    let is_active = current_id.as_ref() == Some(&track.id);
                                    let is_this_playing = is_active && is_playing;
                                    let dur_str = format!("{}:{:02}", track.duration_secs / 60, track.duration_secs % 60);
                                    let thumb_url = track.artwork_url.clone();
                                    let thumb_p = thumb_url.as_ref().and_then(|u| {
                                        state.thumbnail_manager.resolve_or_queue(&Some(u.clone()), None)
                                    });

                                    div()
                                        .id(SharedString::from(format!("artist_top_song_{}", idx)))
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .px(px(12.0))
                                        .py(px(8.0))
                                        .rounded(px(6.0))
                                        .bg(if is_active { rgba(0xFFFFFF18) } else { rgba(0x00000000) })
                                        .hover(|s| s.bg(rgba(0xFFFFFF14)))
                                        .cursor_pointer()
                                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                            let d = d_for_click.clone();
                                            app_state_track.update(cx, |s, cx| {
                                                s.play_artist_track(&d, idx, cx);
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
                                                        .w(px(24.0))
                                                        .text_sm()
                                                        .text_color(if is_active { rgb(0xFF4E4E) } else { rgb(0x888888) })
                                                        .child(if is_this_playing { "▶".to_string() } else { format!("{}", idx + 1) })
                                                )
                                                .child(
                                                    div()
                                                        .size(px(40.0))
                                                        .rounded(px(4.0))
                                                        .overflow_hidden()
                                                        .bg(rgb(0x1F1F1F))
                                                        .child(
                                                            if let Some(ref p) = thumb_p {
                                                                Artwork::new(px(40.0))
                                                                    .src(SharedString::from(p.clone()))
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
                                                                .font_weight(FontWeight::MEDIUM)
                                                                .text_color(if is_active { rgb(0xFF4E4E) } else { rgb(0xFFFFFF) })
                                                                .child(track.title.clone())
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(0x888888))
                                                                .child(track.album.clone())
                                                        )
                                                )
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(12.0))
                                                .child(
                                                    // Download icon / action
                                                    div()
                                                        .id(SharedString::from(format!("dl_btn_{}", idx)))
                                                        .text_xs()
                                                        .px(px(8.0))
                                                        .py(px(4.0))
                                                        .rounded(Radii::full())
                                                        .bg(rgba(0xFFFFFF0F))
                                                        .hover(|s| s.bg(rgba(0xFFFFFF22)))
                                                        .text_color(rgb(0xCCCCCC))
                                                        .child("⬇")
                                                        .on_click({
                                                            let app_state_dl = self.app_state.clone();
                                                            let t = track_clone.clone();
                                                            cx.listener(move |_this, _ev, _win, cx| {
                                                                let tr = t.clone();
                                                                app_state_dl.update(cx, |s, cx| {
                                                                    s.trigger_download_track(tr, cx);
                                                                });
                                                            })
                                                        })
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x717171))
                                                        .child(dur_str)
                                                )
                                        )
                                }))
                        )
                } else {
                    div()
                };

                // 3. Albums Carousel
                let albums_section = if !details.albums.is_empty() {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .child("Albums")
                        )
                        .child(
                            div()
                                .id("artist_albums_scroll")
                                .flex()
                                .gap(px(16.0))
                                .overflow_x_scroll()
                                .children(details.albums.iter().enumerate().map(|(idx, album)| {
                                    let app_state_album = self.app_state.clone();
                                    let aid = album.browse_id.clone();
                                    let atitle = album.title.clone();
                                    let athumb = album.thumbnail_url.clone();
                                    let thumb_p = athumb.as_ref().and_then(|u| {
                                        state.thumbnail_manager.resolve_or_queue(&Some(u.clone()), None)
                                    });

                                    div()
                                        .id(SharedString::from(format!("artist_alb_{}", idx)))
                                        .flex()
                                        .flex_col()
                                        .w(px(150.0))
                                        .min_w(px(150.0))
                                        .gap(px(8.0))
                                        .cursor_pointer()
                                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                            let id = aid.clone();
                                            let t = atitle.clone();
                                            let th = athumb.clone();
                                            app_state_album.update(cx, |s, cx| {
                                                s.open_album_detail(id, t, th, cx);
                                            });
                                        }))
                                        .child(
                                            div()
                                                .size(px(150.0))
                                                .rounded(px(6.0))
                                                .overflow_hidden()
                                                .bg(rgb(0x1F1F1F))
                                                .hover(|s| s.opacity(0.85))
                                                .child(
                                                    if let Some(ref p) = thumb_p {
                                                        Artwork::new(px(150.0))
                                                            .src(SharedString::from(p.clone()))
                                                            .into_any_element()
                                                    } else {
                                                        div().size_full().bg(rgb(0x282828)).into_any_element()
                                                    }
                                                )
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xFFFFFF))
                                                .max_w(px(150.0))
                                                .overflow_hidden()
                                                .child(album.title.clone())
                                        )
                                        .when_some(album.year.clone(), |this, yr| {
                                            this.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x888888))
                                                    .child(yr)
                                            )
                                        })
                                }))
                        )
                } else {
                    div()
                };

                // 4. Singles & EPs Carousel
                let singles_section = if !details.singles.is_empty() {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .child("Singles & EPs")
                        )
                        .child(
                            div()
                                .id("artist_singles_scroll")
                                .flex()
                                .gap(px(16.0))
                                .overflow_x_scroll()
                                .children(details.singles.iter().enumerate().map(|(idx, single)| {
                                    let app_state_single = self.app_state.clone();
                                    let sid = single.browse_id.clone();
                                    let stitle = single.title.clone();
                                    let sthumb = single.thumbnail_url.clone();
                                    let thumb_p = sthumb.as_ref().and_then(|u| {
                                        state.thumbnail_manager.resolve_or_queue(&Some(u.clone()), None)
                                    });

                                    div()
                                        .id(SharedString::from(format!("artist_sgl_{}", idx)))
                                        .flex()
                                        .flex_col()
                                        .w(px(150.0))
                                        .min_w(px(150.0))
                                        .gap(px(8.0))
                                        .cursor_pointer()
                                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                            let id = sid.clone();
                                            let t = stitle.clone();
                                            let th = sthumb.clone();
                                            app_state_single.update(cx, |s, cx| {
                                                s.open_album_detail(id, t, th, cx);
                                            });
                                        }))
                                        .child(
                                            div()
                                                .size(px(150.0))
                                                .rounded(px(6.0))
                                                .overflow_hidden()
                                                .bg(rgb(0x1F1F1F))
                                                .hover(|s| s.opacity(0.85))
                                                .child(
                                                    if let Some(ref p) = thumb_p {
                                                        Artwork::new(px(150.0))
                                                            .src(SharedString::from(p.clone()))
                                                            .into_any_element()
                                                    } else {
                                                        div().size_full().bg(rgb(0x282828)).into_any_element()
                                                    }
                                                )
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xFFFFFF))
                                                .max_w(px(150.0))
                                                .overflow_hidden()
                                                .child(single.title.clone())
                                        )
                                        .when_some(single.year.clone(), |this, yr| {
                                            this.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x888888))
                                                    .child(yr)
                                            )
                                        })
                                }))
                        )
                } else {
                    div()
                };

                // 5. Fans Might Also Like (Related Artists)
                let related_section = if !details.related_artists.is_empty() {
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(rgb(0xFFFFFF))
                                .child("Fans Might Also Like")
                        )
                        .child(
                            div()
                                .id("artist_related_scroll")
                                .flex()
                                .gap(px(20.0))
                                .overflow_x_scroll()
                                .children(details.related_artists.iter().enumerate().map(|(idx, rel)| {
                                    let app_state_rel = self.app_state.clone();
                                    let cid = rel.channel_id.clone();
                                    let rname = rel.name.clone();
                                    let rthumb = rel.thumbnail_url.clone();
                                    let thumb_p = rthumb.as_ref().and_then(|u| {
                                        state.thumbnail_manager.resolve_or_queue(&Some(u.clone()), None)
                                    });

                                    div()
                                        .id(SharedString::from(format!("artist_rel_{}", idx)))
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .w(px(130.0))
                                        .min_w(px(130.0))
                                        .gap(px(8.0))
                                        .cursor_pointer()
                                        .on_click(cx.listener(move |_this, _ev, _win, cx| {
                                            let id = cid.clone();
                                            let n = rname.clone();
                                            let th = rthumb.clone();
                                            app_state_rel.update(cx, |s, cx| {
                                                s.open_artist_detail(id, n, th, cx);
                                            });
                                        }))
                                        .child(
                                            div()
                                                .size(px(120.0))
                                                .rounded(Radii::full())
                                                .overflow_hidden()
                                                .bg(rgb(0x1F1F1F))
                                                .hover(|s| s.opacity(0.85))
                                                .child(
                                                    if let Some(ref p) = thumb_p {
                                                        Artwork::new(px(120.0))
                                                            .src(SharedString::from(p.clone()))
                                                            .rounded(Radii::full())
                                                            .into_any_element()
                                                    } else {
                                                        div()
                                                            .size_full()
                                                            .bg(rgb(0x282828))
                                                            .flex()
                                                            .items_center()
                                                            .justify_center()
                                                            .child(div().text_2xl().child("👤"))
                                                            .into_any_element()
                                                    }
                                                )
                                        )
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(rgb(0xFFFFFF))
                                                .text_center()
                                                .max_w(px(130.0))
                                                .overflow_hidden()
                                                .child(rel.name.clone())
                                        )
                                        .when_some(rel.subscribers.clone(), |this, subs| {
                                            this.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x888888))
                                                    .text_center()
                                                    .child(subs)
                                            )
                                        })
                                }))
                        )
                } else {
                    div()
                };

                // Root scrollable view
                div()
                    .id("artist_page_root_scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .p(px(32.0))
                    .flex()
                    .flex_col()
                    .gap(px(32.0))
                    .child(hero_section)
                    .child(top_songs_section)
                    .child(albums_section)
                    .child(singles_section)
                    .child(related_section)
                    .into_any_element()
            }
        }
    }
}
