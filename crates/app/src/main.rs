#![recursion_limit = "256"]

mod debug;
pub mod download_manager;
pub mod artist_page;
mod engine_bridge;
mod pages;
pub mod palette_cache;
pub mod thumbnail_cache;

use artist_page::ArtistPage;
use engine_bridge::{AppState, Route};
use gpui::*;
use pages::{AlbumPage, HomePage, LibraryPage, PlayerPage, PlaylistDetailPage, PlaylistsPage, SearchPage, SettingsPage, clean_artist_subtitle};
use std::sync::Arc;
use rustic_core::api::{Command, CommandAction, PlaybackState};
use ui_kit::organisms::now_playing_bar::NowPlayingBar;
use ui_kit::organisms::sidebar_organism::SidebarOrganism;
use ui_kit::organisms::title_bar::TitleBar;
use ui_kit::molecules::sidebar::BottomNav;
use ui_kit::tokens::{Theme, ThemeConfig, Radii};

fn key_to_char(key: &str, shift: bool) -> Option<char> {
    match key {
        "space" => Some(' '),
        "minus" => Some(if shift { '_' } else { '-' }),
        "equal" => Some(if shift { '+' } else { '=' }),
        "bracketleft" => Some(if shift { '{' } else { '[' }),
        "bracketright" => Some(if shift { '}' } else { ']' }),
        "semicolon" => Some(if shift { ':' } else { ';' }),
        "quote" => Some(if shift { '"' } else { '\'' }),
        "comma" => Some(if shift { '<' } else { ',' }),
        "period" => Some(if shift { '>' } else { '.' }),
        "slash" => Some(if shift { '?' } else { '/' }),
        "backslash" => Some(if shift { '|' } else { '\\' }),
        "backquote" => Some(if shift { '~' } else { '`' }),
        s if s.len() == 1 => {
            let ch = s.chars().next()?;
            if shift {
                Some(match ch {
                    '1' => '!',
                    '2' => '@',
                    '3' => '#',
                    '4' => '$',
                    '5' => '%',
                    '6' => '^',
                    '7' => '&',
                    '8' => '*',
                    '9' => '(',
                    '0' => ')',
                    c => c.to_ascii_uppercase(),
                })
            } else {
                Some(ch)
            }
        }
        _ => None,
    }
}

struct RootView {
    app_state: Entity<AppState>,
    home_page: Entity<HomePage>,
    album_page: Entity<AlbumPage>,
    artist_page: Entity<ArtistPage>,
    player_page: Entity<PlayerPage>,
    playlist_page: Entity<PlaylistDetailPage>,
    library_page: Entity<LibraryPage>,
    playlists_page: Entity<PlaylistsPage>,
    settings_page: Entity<SettingsPage>,
    search_page: Entity<SearchPage>,
    search_input_buffer: String,
    search_focus_handle: FocusHandle,
    sign_in_cookie_buffer: String,
    sign_in_focus_handle: FocusHandle,
    last_applied_blur: Option<bool>,
}

impl RootView {
    fn new(cx: &mut Context<Self>, app_state: Entity<AppState>) -> Self {
        cx.observe(&app_state, |_this, _state, cx| {
            cx.notify();
        }).detach();

        let home_page = cx.new(|cx| HomePage::new(cx, app_state.clone()));
        let album_page = cx.new(|cx| AlbumPage::new(cx, app_state.clone()));
        let artist_page = cx.new(|cx| ArtistPage::new(cx, app_state.clone()));
        let player_page = cx.new(|cx| PlayerPage::new(cx, app_state.clone()));
        let playlist_page = cx.new(|cx| PlaylistDetailPage::new(cx, app_state.clone()));
        let library_page = cx.new(|cx| LibraryPage::new(cx, app_state.clone()));
        let playlists_page = cx.new(|cx| PlaylistsPage::new(cx, app_state.clone()));
        let settings_page = cx.new(|cx| SettingsPage::new(cx, app_state.clone()));
        let search_page = cx.new(|cx| SearchPage::new(cx, app_state.clone()));
        let search_focus_handle = cx.focus_handle();
        let sign_in_focus_handle = cx.focus_handle();
        
        Self {
            app_state,
            home_page,
            album_page,
            artist_page,
            player_page,
            playlist_page,
            library_page,
            playlists_page,
            settings_page,
            search_page,
            search_input_buffer: String::new(),
            search_focus_handle,
            sign_in_cookie_buffer: String::new(),
            sign_in_focus_handle,
            last_applied_blur: None,
        }
    }
}

impl Render for RootView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.app_state.read(cx);
        let title = state.current_track.as_ref().map(|t| t.title.clone()).unwrap_or_else(|| "No track playing".into());
        let artist = state.current_track.as_ref().map(|t| clean_artist_subtitle(&t.artists.join(", "))).unwrap_or_else(|| "RusticPlayer".into());
        let is_playing = state.playback_state == PlaybackState::Playing;

        let current_pos_secs = if is_playing {
            state.timeline.position_at(std::time::Instant::now())
        } else {
            state.current_position
        };
        let progress = if state.track_duration > 0.0 { (current_pos_secs / state.track_duration) as f32 } else { 0.0 };
        let timeline = state.timeline;

        let engine = state.engine.clone();
        let engine_for_seek = state.engine.clone();
        let track_duration = state.track_duration;

        let cur_track_key = state.current_track.as_ref().map(|t| t.id.clone());
        let is_liked = cur_track_key.as_ref().map(|k| state.liked_tracks.contains(k)).unwrap_or(false);
        let is_disliked = cur_track_key.as_ref().map(|k| state.disliked_tracks.contains(k)).unwrap_or(false);
        let is_captions_enabled = state.is_captions_enabled;
        let is_queue_open = state.is_queue_open;

        let state_for_like = self.app_state.clone();
        let state_for_dislike = self.app_state.clone();
        let cur_key_for_like = cur_track_key.clone();
        let cur_key_for_dislike = cur_track_key.clone();
        let state_for_more = self.app_state.clone();
        let state_for_captions = self.app_state.clone();
        let state_for_queue = self.app_state.clone();
        let state_for_mute = self.app_state.clone();
        let state_for_seek = self.app_state.clone();
        let state_for_glass = self.app_state.clone();
        let state_for_expand = self.app_state.clone();
        let state_for_next = self.app_state.clone();
        let state_for_prev = self.app_state.clone();
        let state_for_vol = self.app_state.clone();

        let cur_secs = current_pos_secs as u64;
        let cur_str = format!("{}:{:02}", cur_secs / 60, cur_secs % 60);
        let dur_secs = state.track_duration as u64;
        let dur_str = format!("{}:{:02}", dur_secs / 60, dur_secs % 60);

        let is_player_route = state.current_route == Route::Player;

        let state_for_repeat = self.app_state.clone();
        let state_for_shuffle = self.app_state.clone();
        let repeat_mode_val = match state.repeat_mode {
            engine_bridge::RepeatMode::Off => 0,
            engine_bridge::RepeatMode::All => 1,
            engine_bridge::RepeatMode::One => 2,
        };

        let subtitle_str = if let Some(ref cur) = state.current_track {
            if let Some(ui_t) = state.tracks.iter().find(|t| t.track_key == cur.id) {
                if !ui_t.album.is_empty() && ui_t.album != ui_t.title {
                    clean_artist_subtitle(&format!("{} • {}", ui_t.artist, ui_t.album))
                } else {
                    clean_artist_subtitle(&ui_t.artist)
                }
            } else {
                clean_artist_subtitle(&cur.artists.join(", "))
            }
        } else {
            artist.to_string()
        };

        let mut now_playing = NowPlayingBar::new(title.clone(), artist.clone())
            .subtitle(subtitle_str)
            .is_playing(is_playing)
            .progress(progress)
            .timeline(timeline)
            .times(cur_str, dur_str)
            .volume(state.volume)
            .is_liked(is_liked)
            .is_disliked(is_disliked)
            .is_captions_enabled(is_captions_enabled)
            .is_buffering(state.playback_state == PlaybackState::Buffering)
            .is_queue_open(is_queue_open)
            .is_glass(state.enable_blur)
            .is_expanded(is_player_route)
            .repeat_mode(repeat_mode_val)
            .is_shuffled(state.is_shuffled)
            .on_repeat(move |_ev, _win, cx| {
                state_for_repeat.update(cx, |s, cx| {
                    s.cycle_repeat(cx);
                });
            })
            .on_shuffle(move |_ev, _win, cx| {
                state_for_shuffle.update(cx, |s, cx| {
                    s.toggle_shuffle(cx);
                });
            })
            .on_toggle_glass(move |_ev, _win, cx| {
                state_for_glass.update(cx, |s, cx| {
                    s.toggle_blur(cx);
                });
            })
            .on_play_pause(move |_ev, _win, _app| {
                engine.send_command(Command {
                    origin: None,
                    action: CommandAction::TogglePlay,
                });
            })
            .on_next(move |_ev, _win, cx| {
                state_for_next.update(cx, |s, cx| {
                    s.play_next_track(cx);
                });
            })
            .on_prev(move |_ev, _win, cx| {
                state_for_prev.update(cx, |s, cx| {
                    s.play_previous_track(cx);
                });
            })
            .on_seek(move |frac, _win, cx| {
                let target_secs = (frac as f64) * track_duration;
                state_for_seek.update(cx, |s, cx| {
                    s.seek_to(target_secs, cx);
                });
                engine_for_seek.send_command(Command {
                    origin: None,
                    action: CommandAction::Seek(std::time::Duration::from_secs_f64(target_secs)),
                });
            })
            .on_volume(move |vol, _win, cx| {
                state_for_vol.update(cx, |s, cx| {
                    s.set_volume(vol, cx);
                });
            })
            .on_like_toggle(move |_ev, _win, cx| {
                if let Some(ref k) = cur_key_for_like {
                    let k = k.clone();
                    state_for_like.update(cx, |s, cx| {
                        s.toggle_like(k, cx);
                    });
                }
            })
            .on_dislike_toggle(move |_ev, _win, cx| {
                if let Some(ref k) = cur_key_for_dislike {
                    let k = k.clone();
                    state_for_dislike.update(cx, |s, cx| {
                        s.toggle_dislike(k, cx);
                    });
                }
            })
            .on_more_menu(move |_ev, _win, cx| {
                state_for_more.update(cx, |s, cx| {
                    s.toggle_queue(cx);
                });
            })
            .on_captions_toggle(move |_ev, _win, cx| {
                state_for_captions.update(cx, |s, cx| {
                    s.toggle_captions(cx);
                });
            })
            .on_queue_toggle(move |_ev, _win, cx| {
                state_for_queue.update(cx, |s, cx| {
                    s.toggle_queue(cx);
                });
            })
            .on_volume_mute(move |_ev, _win, cx| {
                state_for_mute.update(cx, |s, cx| {
                    s.toggle_mute(cx);
                });
            })
            .on_expand(move |_ev, _win, cx| {
                state_for_expand.update(cx, |s, cx| {
                    if s.current_route == Route::Player {
                        s.go_back(cx);
                    } else {
                        s.open_player(cx);
                    }
                });
            });

        // If current track has artwork, attach it
        if let Some(ref cur) = state.current_track {
            if let Some(ui_t) = state.tracks.iter().find(|t| t.track_key == cur.id) {
                if let Some(art) = ui_t.artwork_path() {
                    now_playing = now_playing.artwork(SharedString::from(art.to_string_lossy().to_string().replace('\\', "/")));
                }
            } else if let Some(ref url) = cur.artwork_url {
                if let Some(path) = state.thumbnail_manager.resolve_or_queue(&Some(url.clone()), None) {
                    now_playing = now_playing.artwork(SharedString::from(path));
                }
            }
        }
        let palette = cx.global::<Theme>().palette;
        let is_blur = state.enable_blur;

        // Ensure the platform window's background appearance matches enable_blur dynamically without thrashing
        if self.last_applied_blur != Some(is_blur) {
            let target_appearance = if is_blur {
                gpui::WindowBackgroundAppearance::Blurred
            } else {
                gpui::WindowBackgroundAppearance::Opaque
            };
            window.set_background_appearance(target_appearance);
            self.last_applied_blur = Some(is_blur);
        }

        let active_tab = match &state.current_route {
            Route::Home => "Home",
            Route::Album(_) => "Album",
            Route::Artist(_) => "Artist",
            Route::Player => "Now Playing",
            Route::Playlist(_) => "Playlist",
            Route::Library => "Library",
            Route::Playlists => "Playlists",
            Route::Settings => "Settings",
            Route::Search => "Search",
        };

        // Frameless Title Bar
        let state_for_home = self.app_state.clone();
        let state_for_sidebar = self.app_state.clone();
        let state_for_theme = self.app_state.clone();
        let state_for_blur = self.app_state.clone();
        let state_for_debug = self.app_state.clone();
        let state_for_back = self.app_state.clone();
        let state_for_search = self.app_state.clone();
        let can_go_back = state.can_go_back();

        let is_search_active = state.is_search_overlay_open || state.current_route == Route::Search;
        let display_search_query = if !self.search_input_buffer.is_empty() {
            self.search_input_buffer.clone()
        } else {
            state.search_query.clone()
        };

        let state_for_clear_search = self.app_state.clone();
        let state_for_sign_in = self.app_state.clone();
        let is_mobile = window.viewport_size().width < px(768.0);
        let avatar_path = state.auth_state.account_avatar_url.as_ref().and_then(|url| {
            state.thumbnail_manager.resolve_or_queue(&Some(url.clone()), None)
        });

        let title_bar = TitleBar::new("RusticPlayer")
            .subtitle(active_tab)
            .is_mobile(is_mobile)
            .can_go_back(can_go_back)
            .search_query(display_search_query)
            .is_search_active(is_search_active)
            .on_search_clear(move |_ev, _win, cx| {
                state_for_clear_search.update(cx, |s, cx| {
                    s.clear_search(cx);
                });
            })
            .is_logged_in(state.auth_state.logged_in)
            .account_name(state.auth_state.account_name.clone().unwrap_or_else(|| "YouTube User".into()))
            .account_avatar(avatar_path.clone())
            .account_handle(state.auth_state.account_handle.clone())
            .on_sign_in(move |_ev, _win, cx| {
                state_for_sign_in.update(cx, |s, cx| {
                    s.toggle_sign_in_modal(cx);
                });
            })
            .on_home(move |_ev, _win, cx| {
                state_for_home.update(cx, |s, cx| {
                    s.navigate_to(Route::Home, cx);
                });
            })
            .on_back(move |_ev, _win, cx| {
                state_for_back.update(cx, |s, cx| {
                    s.go_back(cx);
                });
            })
            .on_search_click(move |_ev, _win, cx| {
                state_for_search.update(cx, |s, cx| {
                    s.open_search(cx);
                });
            })
            .on_toggle_sidebar(move |_ev, _win, cx| {
                state_for_sidebar.update(cx, |s, cx| {
                    s.toggle_sidebar(cx);
                });
            })
            .on_toggle_theme(move |_ev, _win, cx| {
                state_for_theme.update(cx, |s, cx| {
                    s.cycle_theme(cx);
                });
            })
            .on_toggle_blur(move |_ev, _win, cx| {
                state_for_blur.update(cx, |s, cx| {
                    s.toggle_blur(cx);
                });
            })
            .on_toggle_debug(move |_ev, _win, cx| {
                state_for_debug.update(cx, |s, cx| {
                    s.toggle_debug_overlay();
                    cx.notify();
                });
            });

        let app_state_for_nav = self.app_state.clone();
        let sidebar_selected = match &state.current_route {
            Route::Library => "Library",
            Route::Playlists => "Playlists",
            Route::Settings => "Settings",
            _ => "Home",
        };
        let sidebar = SidebarOrganism::new(sidebar_selected)
            .collapsed(state.sidebar_collapsed)
            .on_select(move |tab, _win, cx| {
                let route = match tab.as_ref() {
                    "Library" => Route::Library,
                    "Playlists" => Route::Playlists,
                    "Settings" => Route::Settings,
                    _ => Route::Home,
                };
                app_state_for_nav.update(cx, |s, cx| {
                    s.navigate_to(route, cx);
                });
            });

        let content_element = match &state.current_route {
            Route::Home => self.home_page.clone().into_any_element(),
            Route::Album(_) => self.album_page.clone().into_any_element(),
            Route::Artist(_) => self.artist_page.clone().into_any_element(),
            Route::Player => self.player_page.clone().into_any_element(),
            Route::Playlist(_) => self.playlist_page.clone().into_any_element(),
            Route::Library => self.library_page.clone().into_any_element(),
            Route::Playlists => self.playlists_page.clone().into_any_element(),
            Route::Settings => self.settings_page.clone().into_any_element(),
            Route::Search => self.search_page.clone().into_any_element(),
        };

        let debug_overlay = if state.show_debug_overlay {
            let diag = state.audio_diagnostics.clone().unwrap_or_else(|| audio::AudioDiagnostics {
                device_name: "Detecting audio device...".into(),
                sample_rate: 48000,
                channels: 2,
                sample_format: "F32".into(),
                is_running: false,
                has_error: false,
                last_error: None,
                underruns: 0,
                dac_frames: 0,
                dac_time_ns: 0,
            });

            let logs = state.log_store.get_items(state.debug_filter_errors);

            let state_for_reconnect = self.app_state.clone();
            let state_for_clear = self.app_state.clone();
            let state_for_filter = self.app_state.clone();
            let state_for_close = self.app_state.clone();
            let log_store_for_copy = state.log_store.clone();
            let diag_for_copy = diag.clone();
            let title_for_copy = title.clone();
            let artist_for_copy = artist.clone();
            let state_str_for_copy = format!("{:?}", state.playback_state);

            Some(
                ui_kit::organisms::debug_overlay::DebugOverlay::new()
                    .audio_device(diag.device_name)
                    .audio_sample_rate(diag.sample_rate)
                    .audio_channels(diag.channels)
                    .audio_format(diag.sample_format)
                    .audio_is_running(diag.is_running)
                    .audio_has_error(diag.has_error)
                    .audio_last_error(diag.last_error)
                    .audio_underruns(diag.underruns)
                    .audio_dac_frames(diag.dac_frames)
                    .current_position_secs(current_pos_secs)
                    .track_duration_secs(track_duration)
                    .track_title(title.clone())
                    .track_artist(artist.clone())
                    .playback_state(format!("{:?}", state.playback_state))
                    .volume_percent((state.volume * 100.0) as u32)
                    .queue_len(state.queue.len())
                    .logs(logs)
                    .errors_only(state.debug_filter_errors)
                    .on_reconnect_audio(move |_ev, _win, cx| {
                        state_for_reconnect.update(cx, |s, cx| {
                            s.reconnect_audio();
                            cx.notify();
                        });
                    })
                    .on_copy_diagnostics(move |_ev, _win, _cx| {
                        let report = log_store_for_copy.generate_report(
                            Some(&diag_for_copy),
                            &title_for_copy,
                            &artist_for_copy,
                            current_pos_secs,
                            track_duration,
                            &state_str_for_copy,
                        );
                        let _ = std::fs::write("rustic_diagnostics.txt", &report);
                        tracing::info!(target: "app::debug", "Diagnostics report saved to rustic_diagnostics.txt");
                    })
                    .on_clear_logs(move |_ev, _win, cx| {
                        state_for_clear.update(cx, |s, cx| {
                            s.clear_logs();
                            cx.notify();
                        });
                    })
                    .on_toggle_filter(move |_ev, _win, cx| {
                        state_for_filter.update(cx, |s, cx| {
                            s.toggle_log_filter();
                            cx.notify();
                        });
                    })
                    .on_close(move |_ev, _win, cx| {
                        state_for_close.update(cx, |s, cx| {
                            s.toggle_debug_overlay();
                            cx.notify();
                        });
                    })
            )
        } else {
            None
        };

        let state_for_key = self.app_state.clone();
        let middle_content = if let Some(overlay) = debug_overlay {
            div()
                .size_full()
                .overflow_hidden()
                .child(overlay)
                .into_any_element()
        } else if is_mobile {
            div()
                .size_full()
                .overflow_hidden()
                .child(content_element)
                .into_any_element()
        } else if is_player_route {
            if !state.sidebar_collapsed {
                div()
                    .flex()
                    .flex_row()
                    .size_full()
                    .min_h_0()
                    .overflow_hidden()
                    .child(sidebar)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .min_h_0()
                            .overflow_hidden()
                            .child(content_element)
                    )
                    .into_any_element()
            } else {
                div()
                    .size_full()
                    .overflow_hidden()
                    .child(content_element)
                    .into_any_element()
            }
        } else {
            div()
                .flex()
                .flex_row()
                .size_full()
                .min_h_0()
                .overflow_hidden()
                .child(sidebar)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .min_h_0()
                        .overflow_hidden()
                        .child(content_element)
                )
                .into_any_element()
        };

        // Full-bleed true Gaussian ambient blur backdrop
        let ambient_img_path = state
            .active_ambient_image
            .as_deref()
            .unwrap_or("assets/artwork/ambient_blur.png");

        let ambient_backdrop = if state.enable_blur {
            div()
                .id("ambient_backdrop")
                .absolute()
                .inset_0()
                .overflow_hidden()
                .bg(rgb(0x030303))
                .child(
                    img(SharedString::from(ambient_img_path.to_string()))
                        .size_full()
                        .object_fit(ObjectFit::Cover)
                )
                // Dark atmospheric contrast scrim
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(rgba(0x00000033))
                )
                .into_any_element()
        } else {
            div()
                .id("ambient_backdrop_solid")
                .absolute()
                .inset_0()
                .bg(rgb(0x030303))
                .into_any_element()
        };

        let has_track = state.current_track.is_some();
        let content_pb = if is_mobile {
            if is_player_route || !has_track { px(0.0) } else { px(104.0) }
        } else {
            if has_track { px(72.0) } else { px(0.0) }
        };

        let state_for_mobile_expand = self.app_state.clone();
        let app_state_for_bottom_nav = self.app_state.clone();
        let eng_mobile = state.engine.clone();
        let mini_art = if let Some(ref cur) = state.current_track {
            if let Some(ui_t) = state.tracks.iter().find(|t| t.track_key == cur.id) {
                ui_t.artwork_path().map(|p| p.to_string_lossy().to_string().replace('\\', "/"))
            } else {
                None
            }
        } else {
            None
        };

        let state_for_mobile_next = self.app_state.clone();
        let mobile_mini_player = div()
            .id("mobile_mini_player_dock")
            .w_full()
            .h(px(48.0))
            .bg(rgba(0x212121EE))
            .border_b_1()
            .border_color(rgba(0xFFFFFF15))
            .flex()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .cursor_pointer()
            .on_click(move |_ev, _win, cx| {
                state_for_mobile_expand.update(cx, |s, cx| {
                    s.open_player(cx);
                });
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.0))
                    .min_w_0()
                    .flex_1()
                    .child(
                        if let Some(art) = mini_art {
                            div()
                                .size(px(32.0))
                                .rounded(Radii::small())
                                .overflow_hidden()
                                .child(
                                    img(SharedString::from(art))
                                        .size_full()
                                        .object_fit(ObjectFit::Cover)
                                )
                                .into_any_element()
                        } else {
                            div()
                                .size(px(32.0))
                                .rounded(Radii::small())
                                .bg(rgb(0x333333))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(div().text_xs().text_color(rgb(0x888888)).child("🎵"))
                                .into_any_element()
                        }
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xFFFFFF))
                                    .truncate()
                                    .child(title.clone())
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xAAAAAA))
                                    .truncate()
                                    .child(artist.clone())
                            )
                    )
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.0))
                    .child(
                        div()
                            .id("mobile_mini_play_btn")
                            .size(px(32.0))
                            .rounded(Radii::full())
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_click(move |_ev, _win, _app| {
                                eng_mobile.send_command(Command {
                                    origin: None,
                                    action: CommandAction::TogglePlay,
                                });
                            })
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xFFFFFF))
                                    .child(if is_playing { "⏸" } else { "▶" })
                            )
                    )
                    .child(
                        div()
                            .id("mobile_mini_next_btn")
                            .size(px(32.0))
                            .rounded(Radii::full())
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .on_click({
                                let state_next = state_for_mobile_next.clone();
                                move |_ev, _win, cx| {
                                    state_next.update(cx, |s, cx| {
                                        s.play_next_track(cx);
                                    });
                                }
                            })
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xFFFFFF))
                                    .child("⏭")
                            )
                    )
            );

        let mobile_bottom_nav = BottomNav::new(active_tab)
            .on_select(move |tab, _win, cx| {
                let route = match tab.as_ref() {
                    "Explore" => Route::Playlists,
                    "Library" => Route::Library,
                    _ => Route::Home,
                };
                app_state_for_bottom_nav.update(cx, |s, cx| {
                    s.navigate_to(route, cx);
                });
            });

        // Search suggestions dropdown overlay
        if state.is_search_overlay_open && self.search_input_buffer.is_empty() && !state.search_query.is_empty() {
            self.search_input_buffer = state.search_query.clone();
        }

        let search_overlay = if state.is_search_overlay_open {
            let state_close = self.app_state.clone();
            let state_submit = self.app_state.clone();
            let suggestions = state.search_suggestions.clone();
            let query_buf = self.search_input_buffer.clone();

            Some(
                div()
                    .id("search_overlay_backdrop")
                    .absolute()
                    .inset_0()
                    .top(px(46.0))
                    .bg(rgba(0x00000066))
                    .on_click(move |_ev, _win, cx| {
                        state_close.update(cx, |s, cx| {
                            s.set_search_overlay_open(false, cx);
                        });
                    })
                    .child(
                        div()
                            .id("search_dropdown_card")
                            .occlude()
                            .mx_auto()
                            .mt(px(4.0))
                            .w(px(520.0))
                            .max_w(px(520.0))
                            .rounded_b(Radii::base())
                            .rounded_t(Radii::small())
                            .bg(rgb(0x212121))
                            .border_1()
                            .border_color(rgba(0xFFFFFF20))
                            .shadow_lg()
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .track_focus(&self.search_focus_handle)
                            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _win, cx| {
                                match ev.keystroke.key.as_str() {
                                    "enter" => {
                                        let q = this.search_input_buffer.trim().to_string();
                                        if !q.is_empty() {
                                            this.app_state.update(cx, |s, cx| {
                                                s.perform_search(q, cx);
                                            });
                                        }
                                    }
                                    "backspace" => {
                                        this.search_input_buffer.pop();
                                        let q = this.search_input_buffer.clone();
                                        this.app_state.update(cx, |s, cx| {
                                            s.fetch_search_suggestions(q, cx);
                                        });
                                        cx.notify();
                                    }
                                    "escape" => {
                                        this.app_state.update(cx, |s, cx| {
                                            s.set_search_overlay_open(false, cx);
                                        });
                                    }
                                    k => {
                                        if !ev.keystroke.modifiers.control && !ev.keystroke.modifiers.alt {
                                            if let Some(ch) = key_to_char(k, ev.keystroke.modifiers.shift) {
                                                this.search_input_buffer.push(ch);
                                                let q = this.search_input_buffer.clone();
                                                this.app_state.update(cx, |s, cx| {
                                                    s.fetch_search_suggestions(q, cx);
                                                });
                                                cx.notify();
                                            }
                                        }
                                    }
                                }
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(10.0))
                                    .px(px(16.0))
                                    .py(px(10.0))
                                    .border_b_1()
                                    .border_color(rgba(0xFFFFFF15))
                                    .child(
                                        ui_kit::icon::Icon::new("icons/search.svg")
                                            .size(px(16.0))
                                            .color(rgb(0xFF0000))
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_sm()
                                            .text_color(if query_buf.is_empty() { rgb(0x888888) } else { rgb(0xFFFFFF) })
                                            .child(if query_buf.is_empty() {
                                                "Type to search songs, albums, artists...".to_string()
                                            } else {
                                                format!("{}|", query_buf)
                                            })
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .px(px(8.0))
                                            .py(px(2.0))
                                            .rounded(Radii::small())
                                            .bg(rgba(0xFFFFFF15))
                                            .text_color(rgb(0xAAAAAA))
                                            .child("ESC to close")
                                    )
                            )
                            .child(
                                if !suggestions.is_empty() {
                                    let mut sugg_items = Vec::new();
                                    for (idx, sugg) in suggestions.iter().take(8).enumerate() {
                                        let sugg_text = sugg.clone();
                                        let state_click = state_submit.clone();
                                        sugg_items.push(
                                            div()
                                                .id(SharedString::from(format!("search_sugg_{}", idx)))
                                                .flex()
                                                .items_center()
                                                .gap(px(12.0))
                                                .px(px(16.0))
                                                .py(px(10.0))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgba(0xFFFFFF15)))
                                                .on_click(move |_ev, _win, cx| {
                                                    let st = sugg_text.clone();
                                                    state_click.update(cx, |s, cx| {
                                                        s.perform_search(st, cx);
                                                    });
                                                })
                                                .child(
                                                    ui_kit::icon::Icon::new("icons/search.svg")
                                                        .size(px(14.0))
                                                        .color(rgb(0x888888))
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(FontWeight::MEDIUM)
                                                        .text_color(rgb(0xEEEEEE))
                                                        .child(sugg.clone())
                                                )
                                        );
                                    }
                                    div().flex().flex_col().children(sugg_items).into_any_element()
                                } else {
                                    div()
                                        .px(px(16.0))
                                        .py(px(12.0))
                                        .text_xs()
                                        .text_color(rgb(0x888888))
                                        .child("Press Enter to search, or start typing for suggestions...")
                                        .into_any_element()
                                }
                            )
                    )
                    .into_any_element()
            )
        } else {
            None
        };

        // Sign-in modal overlay
        let sign_in_modal = if state.is_sign_in_modal_open {
            let state_close = self.app_state.clone();
            let is_signing_in = state.is_signing_in;
            let err_msg = state.sign_in_error.clone();
            let cookie_buf = self.sign_in_cookie_buffer.clone();

            Some(
                div()
                    .id("sign_in_modal_backdrop")
                    .absolute()
                    .inset_0()
                    .bg(rgba(0x00000088))
                    .flex()
                    .items_center()
                    .justify_center()
                    .on_click(move |_ev, _win, cx| {
                        state_close.update(cx, |s, cx| {
                            s.toggle_sign_in_modal(cx);
                        });
                    })
                    .child(
                        div()
                            .id("sign_in_modal_card")
                            .occlude()
                            .w(px(520.0))
                            .p(px(24.0))
                            .rounded(Radii::large())
                            .bg(rgb(0x212121))
                            .border_1()
                            .border_color(rgba(0xFFFFFF20))
                            .shadow_lg()
                            .flex()
                            .flex_col()
                            .gap(px(16.0))
                            .track_focus(&self.sign_in_focus_handle)
                            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _win, cx| {
                                match ev.keystroke.key.as_str() {
                                    "enter" if ev.keystroke.modifiers.control => {
                                        let c = this.sign_in_cookie_buffer.trim().to_string();
                                        this.app_state.update(cx, |s, cx| {
                                            s.submit_cookies(c, cx);
                                        });
                                    }
                                    "escape" => {
                                        this.app_state.update(cx, |s, cx| {
                                            s.toggle_sign_in_modal(cx);
                                        });
                                    }
                                    "backspace" => {
                                        this.sign_in_cookie_buffer.pop();
                                        cx.notify();
                                    }
                                    "v" | "V" if ev.keystroke.modifiers.control => {
                                        if let Some(item) = cx.read_from_clipboard() {
                                            if let Some(text) = item.text() {
                                                this.sign_in_cookie_buffer.push_str(&text);
                                                cx.notify();
                                            }
                                        }
                                    }
                                    k => {
                                        if !ev.keystroke.modifiers.control && !ev.keystroke.modifiers.alt {
                                            if let Some(ch) = key_to_char(k, ev.keystroke.modifiers.shift) {
                                                this.sign_in_cookie_buffer.push(ch);
                                                cx.notify();
                                            }
                                        }
                                    }
                                }
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(10.0))
                                            .child(ui_kit::icon::Icon::new("icons/ytmusic_logo.svg").size(px(24.0)))
                                            .child(
                                                div()
                                                    .text_lg()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0xFFFFFF))
                                                    .child(if state.auth_state.logged_in { "YouTube Music Account" } else { "Sign in to YouTube Music" })
                                            )
                                    )
                                    .child(
                                        div()
                                            .id("btn_close_sign_in")
                                            .size(px(24.0))
                                            .rounded(Radii::full())
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .cursor_pointer()
                                            .hover(|s| s.bg(rgba(0xFFFFFF15)))
                                            .on_click(cx.listener(|this, _ev, _win, cx| {
                                                this.app_state.update(cx, |s, cx| {
                                                    s.toggle_sign_in_modal(cx);
                                                });
                                            }))
                                            .child(
                                                ui_kit::icon::Icon::new("icons/x.svg")
                                                    .size(px(12.0))
                                                    .color(rgb(0xAAAAAA))
                                            )
                                    )
                            )
                            .children(if state.auth_state.logged_in {
                                let name = state.auth_state.account_name.clone().unwrap_or_else(|| "Ketan".into());
                                let handle = state.auth_state.account_handle.clone().unwrap_or_else(|| "@ketan-saini".into());
                                let initial = name.chars().next().unwrap_or('U').to_uppercase().to_string();
                                Some(
                                    div()
                                        .p(px(12.0))
                                        .rounded(Radii::base())
                                        .bg(rgb(0x181818))
                                        .border_1()
                                        .border_color(rgba(0xFFFFFF15))
                                        .flex()
                                        .items_center()
                                        .gap(px(12.0))
                                        .child(
                                            div()
                                                .size(px(46.0))
                                                .rounded(Radii::full())
                                                .overflow_hidden()
                                                .bg(rgb(0x065FD4))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .child(
                                                    if let Some(ref path) = avatar_path {
                                                        img(SharedString::from(path.clone()))
                                                            .size_full()
                                                            .object_fit(ObjectFit::Cover)
                                                            .into_any_element()
                                                    } else {
                                                        div()
                                                            .text_lg()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_color(rgb(0xFFFFFF))
                                                            .child(initial)
                                                            .into_any_element()
                                                    }
                                                )
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap(px(2.0))
                                                .child(
                                                    div()
                                                        .text_base()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(rgb(0xFFFFFF))
                                                        .child(name)
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xAAAAAA))
                                                        .child(handle)
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap(px(6.0))
                                                        .mt(px(2.0))
                                                        .child(div().size(px(6.0)).rounded(Radii::full()).bg(rgb(0x22C55E)))
                                                        .child(div().text_xs().text_color(rgb(0x22C55E)).child("Connected & Synced"))
                                                )
                                        )
                                )
                            } else {
                                None
                            })
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0xAAAAAA))
                                    .child(if state.auth_state.logged_in {
                                        "Paste new cookies below or click Import to update session cookies or switch accounts."
                                    } else {
                                        "Authenticate with your YouTube Music cookies to sync liked songs, private playlists, subscriptions, and personalized recommendations."
                                    })
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .id("btn_import_cookies_file")
                                            .px(px(12.0))
                                            .py(px(6.0))
                                            .rounded(Radii::small())
                                            .bg(rgba(0xFFFFFF15))
                                            .hover(|s| s.bg(rgba(0xFFFFFF25)))
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _ev, _win, cx| {
                                                if let Ok(content) = std::fs::read_to_string("cookies.json") {
                                                    this.sign_in_cookie_buffer = content;
                                                    cx.notify();
                                                } else {
                                                    this.app_state.update(cx, |s, cx| {
                                                        s.sign_in_error = Some("cookies.json not found in working directory".into());
                                                        cx.notify();
                                                    });
                                                }
                                            }))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0x3EA6FF))
                                                    .child("📁 Import cookies.json")
                                            )
                                    )
                                    .child(
                                        div()
                                            .id("btn_paste_clipboard_cookies")
                                            .px(px(12.0))
                                            .py(px(6.0))
                                            .rounded(Radii::small())
                                            .bg(rgba(0xFFFFFF15))
                                            .hover(|s| s.bg(rgba(0xFFFFFF25)))
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _ev, _win, cx| {
                                                if let Some(item) = cx.read_from_clipboard() {
                                                    if let Some(text) = item.text() {
                                                        this.sign_in_cookie_buffer = text;
                                                        cx.notify();
                                                    }
                                                }
                                            }))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(rgb(0x3EA6FF))
                                                    .child("📋 Paste from Clipboard")
                                            )
                                    )
                            )
                            .child(
                                div()
                                    .h(px(110.0))
                                    .p(px(10.0))
                                    .rounded(Radii::small())
                                    .bg(rgb(0x181818))
                                    .border_1()
                                    .border_color(rgba(0xFFFFFF15))
                                    .overflow_hidden()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(if cookie_buf.is_empty() { rgb(0x666666) } else { rgb(0xCCCCCC) })
                                            .child(if cookie_buf.is_empty() {
                                                "Paste cookies JSON or raw Cookie header here (or click 'Import cookies.json')...".to_string()
                                            } else {
                                                if cookie_buf.len() > 300 {
                                                    format!("{}... ({} chars)", &cookie_buf[..300], cookie_buf.len())
                                                } else {
                                                    cookie_buf.clone()
                                                }
                                            })
                                    )
                            )
                            .children(err_msg.map(|msg| {
                                div()
                                    .px(px(10.0))
                                    .py(px(6.0))
                                    .rounded(Radii::small())
                                    .bg(rgba(0xFF000022))
                                    .border_1()
                                    .border_color(rgba(0xFF000044))
                                    .text_xs()
                                    .text_color(rgb(0xFF6B6B))
                                    .child(msg)
                            }))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_end()
                                    .gap(px(10.0))
                                    .child(
                                        div()
                                            .id("btn_cancel_sign_in_footer")
                                            .px(px(16.0))
                                            .py(px(8.0))
                                            .rounded(Radii::full())
                                            .cursor_pointer()
                                            .hover(|s| s.bg(rgba(0xFFFFFF10)))
                                            .on_click(cx.listener(|this, _ev, _win, cx| {
                                                this.app_state.update(cx, |s, cx| {
                                                    s.toggle_sign_in_modal(cx);
                                                });
                                            }))
                                            .child(div().text_xs().text_color(rgb(0xAAAAAA)).child("Cancel"))
                                    )
                                    .child(
                                        div()
                                            .id("btn_confirm_sign_in")
                                            .px(px(20.0))
                                            .py(px(8.0))
                                            .rounded(Radii::full())
                                            .bg(if is_signing_in { rgb(0x555555) } else { rgb(0x3EA6FF) })
                                            .hover(|s| s.opacity(0.85))
                                            .cursor_pointer()
                                            .on_click(cx.listener(|this, _ev, _win, cx| {
                                                let c = this.sign_in_cookie_buffer.trim().to_string();
                                                this.app_state.update(cx, |s, cx| {
                                                    s.submit_cookies(c, cx);
                                                });
                                            }))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(rgb(0x000000))
                                                    .child(if is_signing_in { "Signing in..." } else { "Sign in" })
                                            )
                                    )
                            )
                    )
                    .into_any_element()
            )
        } else {
            None
        };

        div()
            .id("root_container")
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(palette.background)
            .on_key_down(move |ev: &KeyDownEvent, _win, cx| {
                if ev.keystroke.key == "f12" || (ev.keystroke.modifiers.control && ev.keystroke.key == "d") {
                    state_for_key.update(cx, |s, cx| {
                        s.toggle_debug_overlay();
                        cx.notify();
                    });
                }
            })
            .child(ambient_backdrop)
            .child(title_bar)
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .pb(content_pb)
                    .overflow_hidden()
                    .child(middle_content)
            )
            .child(
                if is_mobile {
                    if !is_player_route {
                        let mut mob_dock = div()
                            .absolute()
                            .bottom_0()
                            .left_0()
                            .right_0();
                        if has_track {
                            mob_dock = mob_dock.child(mobile_mini_player);
                        }
                        mob_dock.child(mobile_bottom_nav).into_any_element()
                    } else {
                        div().into_any_element()
                    }
                } else if has_track {
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .h(px(72.0))
                        .child(now_playing)
                        .into_any_element()
                } else {
                    div().into_any_element()
                }
            )
            .children({
                if state.is_queue_open && !is_player_route {
                    let state_for_q = self.app_state.clone();
                    let state_for_close = self.app_state.clone();
                    let state_for_up = self.app_state.clone();
                    let state_for_dn = self.app_state.clone();
                    let state_for_rem = self.app_state.clone();

                    let mut q_items = Vec::new();
                    if let Some(ref cur) = state.current_track {
                        let cur_art_url = cur.artwork_url.as_ref().filter(|u| !u.is_empty()).cloned().or_else(|| {
                            if cur.id.len() == 11 && !cur.id.contains('/') && !cur.id.contains('\\') {
                                Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", cur.id))
                            } else {
                                None
                            }
                        });
                        let thumb_path = state.thumbnail_manager.resolve_or_queue(&cur_art_url, None);
                        q_items.push(ui_kit::organisms::queue_panel::QueueItemViewData {
                            id: cur.id.clone().into(),
                            title: cur.title.clone().into(),
                            artist: clean_artist_subtitle(&cur.artists.join(", ")).into(),
                            duration: Some(format!("{}:{:02}", cur.duration_secs / 60, cur.duration_secs % 60).into()),
                            thumbnail_path: thumb_path.map(|p| p.into()),
                            is_active: true,
                            lane: "Playing Now".into(),
                            group: ui_kit::organisms::queue_panel::QueueGroup::Current,
                        });
                    }

                    for (lane, t) in state.queue_mgr.upcoming_tracks() {
                        let t_art_url = t.artwork_url.as_ref().filter(|u| !u.is_empty()).cloned().or_else(|| {
                            if t.id.len() == 11 && !t.id.contains('/') && !t.id.contains('\\') {
                                Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", t.id))
                            } else {
                                None
                            }
                        });
                        let thumb_path = state.thumbnail_manager.resolve_or_queue(&t_art_url, None);
                        let lane_name = match lane {
                            rustic_core::queue::QueueLane::User => "Up Next",
                            rustic_core::queue::QueueLane::Context => "Playlist",
                            rustic_core::queue::QueueLane::Radio => "Autoplay",
                            rustic_core::queue::QueueLane::History => "History",
                            rustic_core::queue::QueueLane::Current => "Playing Now",
                        };
                        let grp = match lane {
                            rustic_core::queue::QueueLane::User | rustic_core::queue::QueueLane::Context | rustic_core::queue::QueueLane::History => ui_kit::organisms::queue_panel::QueueGroup::UpNext,
                            rustic_core::queue::QueueLane::Radio => ui_kit::organisms::queue_panel::QueueGroup::Suggestions,
                            rustic_core::queue::QueueLane::Current => ui_kit::organisms::queue_panel::QueueGroup::Current,
                        };
                        q_items.push(ui_kit::organisms::queue_panel::QueueItemViewData {
                            id: t.id.clone().into(),
                            title: t.title.into(),
                            artist: clean_artist_subtitle(&t.artists.join(", ")).into(),
                            duration: Some(format!("{}:{:02}", t.duration_secs / 60, t.duration_secs % 60).into()),
                            thumbnail_path: thumb_path.map(|p| p.into()),
                            is_active: false,
                            lane: lane_name.into(),
                            group: grp,
                        });
                    }

                    Some(
                        div()
                            .absolute()
                            .top(px(48.0))
                            .right_0()
                            .bottom(px(72.0))
                            .w(px(360.0))
                            .shadow(vec![
                                BoxShadow {
                                    color: rgba(0x00000080).into(),
                                    offset: point(px(-4.0), px(0.0)),
                                    blur_radius: px(16.0),
                                    spread_radius: px(0.0),
                                    inset: false,
                                }
                            ])
                            .child(
                                ui_kit::organisms::queue_panel::QueuePanel::new()
                                    .items(q_items)
                                    .on_select(move |idx, _win, cx| {
                                        state_for_q.update(cx, |s, cx| {
                                            s.play_queue_index(idx, cx);
                                        });
                                    })
                                    .on_move_up(move |idx, _win, cx| {
                                        state_for_up.update(cx, |s, cx| {
                                            s.move_queue_item_up(idx, cx);
                                        });
                                    })
                                    .on_move_down(move |idx, _win, cx| {
                                        state_for_dn.update(cx, |s, cx| {
                                            s.move_queue_item_down(idx, cx);
                                        });
                                    })
                                    .on_remove(move |idx, _win, cx| {
                                        state_for_rem.update(cx, |s, cx| {
                                            s.remove_queue_item(idx, cx);
                                        });
                                    })
                                    .on_close(move |_ev, _win, cx| {
                                        state_for_close.update(cx, |s, cx| {
                                            s.toggle_queue(cx);
                                        });
                                    })
                            )
                    )
                } else {
                    None
                }
            })
            .children(search_overlay)
            .children(sign_in_modal)
    }
}


struct LocalAssetSource {
    base_path: std::path::PathBuf,
}

impl gpui::AssetSource for LocalAssetSource {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        let clean = path.trim_start_matches('/').trim_start_matches('\\');

        // Check if path is a remote HTTP(S) URL
        if path.starts_with("http://") || path.starts_with("https://") {
            let hash = thumbnail_cache::ThumbnailManager::hash_url(path);
            let cached_path = self.base_path.join("cache").join("thumbnails").join(format!("{}.jpg", hash));
            if let Ok(data) = std::fs::read(&cached_path) {
                return Ok(Some(std::borrow::Cow::Owned(data)));
            }
        }

        // 1. Check direct from base_path (e.g. cache/artwork/... or assets/...)
        let p1 = self.base_path.join(clean);
        if let Ok(data) = std::fs::read(&p1) {
            return Ok(Some(std::borrow::Cow::Owned(data)));
        }

        // 2. Check inside assets/ (e.g. "icons/play.svg" -> "assets/icons/play.svg")
        let p2 = self.base_path.join("assets").join(clean);
        if let Ok(data) = std::fs::read(&p2) {
            return Ok(Some(std::borrow::Cow::Owned(data)));
        }

        // 3. Check direct path as provided
        if let Ok(data) = std::fs::read(path) {
            return Ok(Some(std::borrow::Cow::Owned(data)));
        }
        Ok(None)
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        let clean_path = path.trim_start_matches('/');
        let mut results = Vec::new();
        let full_path = self.base_path.join(clean_path);
        if let Ok(entries) = std::fs::read_dir(&full_path) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    results.push(name.into());
                }
            }
        }
        Ok(results)
    }
}

#[cfg(target_os = "windows")]
#[link(name = "winmm")]
unsafe extern "system" {
    fn timeBeginPeriod(uperiod: u32) -> u32;
}

fn main() {
    debug::init_panic_hook();

    #[cfg(target_os = "windows")]
    unsafe {
        timeBeginPeriod(1);
    }

    let log_store = Arc::new(debug::LogStore::new(250));
    let memory_layer = debug::MemoryLogLayer::new(log_store.clone());

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("app_debug.log");
    let file_layer = log_file.ok().map(|f| tracing_subscriber::fmt::layer().with_ansi(false).with_writer(Arc::new(f)));

    use tracing_subscriber::layer::SubscriberExt;
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer)
        .with(memory_layer);
    let _ = tracing::subscriber::set_global_default(subscriber);

    let rt = tokio::runtime::Runtime::new().expect("Failed to start Tokio runtime");
    let rt = Box::leak(Box::new(rt));

    let current_dir = std::env::current_dir().unwrap_or_default();
    let asset_source = LocalAssetSource {
        base_path: current_dir,
    };

    let application = gpui::Application::with_platform(gpui_platform::current_platform(false))
        .with_assets(asset_source);

    let log_store_for_app = log_store.clone();

    application.run(move |cx: &mut App| {
        let guard = rt.enter();
        std::mem::forget(guard);
        
        let theme = Theme::from_config(ThemeConfig::default());
        cx.set_global(theme);

        // Start engine
        let engine = engine::Engine::new().expect("Failed to start engine");
        let engine_arc = Arc::new(engine);
        
        let model = cx.new(|cx| AppState::new(engine_arc.clone(), log_store_for_app, cx));

        let (win_w, win_h) = if let (Ok(w_str), Ok(h_str)) = (std::env::var("RUSTIC_WINDOW_WIDTH"), std::env::var("RUSTIC_WINDOW_HEIGHT")) {
            (w_str.parse::<f32>().unwrap_or(1200.0), h_str.parse::<f32>().unwrap_or(750.0))
        } else {
            (1200.0, 750.0)
        };

        let options = gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds {
                origin: gpui::Point::new(100.0.into(), 100.0.into()),
                size: gpui::Size {
                    width: win_w.into(),
                    height: win_h.into(),
                },
            })),
            window_background: gpui::WindowBackgroundAppearance::Blurred,
            window_decorations: Some(gpui::WindowDecorations::Client),
            window_min_size: Some(gpui::size(px(360.0), px(480.0))),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("RusticPlayer".into()),
                appears_transparent: true,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        eprintln!("[RusticPlayer] Calling cx.open_window...");
        match cx.open_window(options, |_window, cx| {
            cx.new(|cx| RootView::new(cx, model.clone()))
        }) {
            Ok(handle) => {
                eprintln!("[RusticPlayer] Successfully opened window: {:?}", handle);
            }
            Err(err) => {
                eprintln!("[RusticPlayer] FAILED to open window with custom options: {:?}", err);
                let fallback = gpui::WindowOptions {
                    window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds {
                        origin: gpui::Point::new(100.0.into(), 100.0.into()),
                        size: gpui::Size {
                            width: win_w.into(),
                            height: win_h.into(),
                        },
                    })),
                    ..Default::default()
                };
                match cx.open_window(fallback, |_window, cx| {
                    cx.new(|cx| RootView::new(cx, model.clone()))
                }) {
                    Ok(handle) => eprintln!("[RusticPlayer] Successfully opened fallback window: {:?}", handle),
                    Err(e2) => eprintln!("[RusticPlayer] FAILED to open fallback window: {:?}", e2),
                }
            }
        }
    });
}
