use gpui::prelude::*;
use gpui::*;
use crate::atoms::{PlaybackTimeline, Slider, VideoFrameProvider, VideoSurface};
use crate::icon::Icon;
use crate::tokens::Radii;
use std::sync::Arc;
use std::time::Duration;

#[derive(IntoElement)]
pub struct NowPlayingBar {
    title: SharedString,
    artist: SharedString,
    subtitle: Option<SharedString>,
    artwork: Option<ImageSource>,
    video_provider: Option<Arc<dyn VideoFrameProvider>>,
    current_clock: Duration,
    is_playing: bool,
    is_buffering: bool,
    progress: f32,
    timeline: Option<PlaybackTimeline>,
    volume: f32,
    is_liked: bool,
    is_disliked: bool,
    is_queue_open: bool,
    is_glass: bool,
    current_time: Option<SharedString>,
    duration_time: Option<SharedString>,
    is_expanded: bool,
    repeat_mode: u8,
    is_shuffled: bool,
    is_captions_enabled: bool,
    on_play_pause: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_next: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_prev: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_like_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_dislike_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_more_menu: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_captions_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_queue_toggle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_volume_mute: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_toggle_glass: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_expand: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_repeat: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_shuffle: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_seek: Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>,
    on_volume: Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>,
}

impl NowPlayingBar {
    pub fn new(title: impl Into<SharedString>, artist: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            artist: artist.into(),
            subtitle: None,
            artwork: None,
            video_provider: None,
            current_clock: Duration::ZERO,
            is_playing: false,
            is_buffering: false,
            progress: 0.0,
            timeline: None,
            volume: 1.0,
            is_liked: false,
            is_disliked: false,
            is_queue_open: false,
            is_glass: true,
            is_expanded: false,
            repeat_mode: 0,
            is_shuffled: false,
            is_captions_enabled: false,
            current_time: None,
            duration_time: None,
            on_play_pause: None,
            on_next: None,
            on_prev: None,
            on_like_toggle: None,
            on_dislike_toggle: None,
            on_more_menu: None,
            on_captions_toggle: None,
            on_queue_toggle: None,
            on_volume_mute: None,
            on_toggle_glass: None,
            on_expand: None,
            on_repeat: None,
            on_shuffle: None,
            on_seek: None,
            on_volume: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn is_disliked(mut self, disliked: bool) -> Self {
        self.is_disliked = disliked;
        self
    }

    pub fn is_captions_enabled(mut self, enabled: bool) -> Self {
        self.is_captions_enabled = enabled;
        self
    }

    pub fn is_buffering(mut self, buffering: bool) -> Self {
        self.is_buffering = buffering;
        self
    }

    pub fn on_dislike_toggle(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_dislike_toggle = Some(Box::new(handler));
        self
    }

    pub fn on_more_menu(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_more_menu = Some(Box::new(handler));
        self
    }

    pub fn on_captions_toggle(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_captions_toggle = Some(Box::new(handler));
        self
    }

    pub fn repeat_mode(mut self, mode: u8) -> Self {
        self.repeat_mode = mode;
        self
    }

    pub fn is_shuffled(mut self, shuffled: bool) -> Self {
        self.is_shuffled = shuffled;
        self
    }

    pub fn on_repeat(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_repeat = Some(Box::new(handler));
        self
    }

    pub fn on_shuffle(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_shuffle = Some(Box::new(handler));
        self
    }

    pub fn is_expanded(mut self, expanded: bool) -> Self {
        self.is_expanded = expanded;
        self
    }

    pub fn on_expand(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_expand = Some(Box::new(handler));
        self
    }

    pub fn timeline(mut self, timeline: PlaybackTimeline) -> Self {
        self.timeline = Some(timeline);
        self
    }

    pub fn is_glass(mut self, glass: bool) -> Self {
        self.is_glass = glass;
        self
    }

    pub fn on_toggle_glass(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_glass = Some(Box::new(handler));
        self
    }

    pub fn video_provider(mut self, provider: Option<Arc<dyn VideoFrameProvider>>, clock: Duration) -> Self {
        self.video_provider = provider;
        self.current_clock = clock;
        self
    }

    pub fn times(mut self, current: impl Into<SharedString>, total: impl Into<SharedString>) -> Self {
        self.current_time = Some(current.into());
        self.duration_time = Some(total.into());
        self
    }

    pub fn on_seek(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_seek = Some(Box::new(handler));
        self
    }

    pub fn on_volume(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_volume = Some(Box::new(handler));
        self
    }

    pub fn on_volume_mute(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_volume_mute = Some(Box::new(handler));
        self
    }

    pub fn artwork(mut self, src: impl Into<ImageSource>) -> Self {
        self.artwork = Some(src.into());
        self
    }

    pub fn is_playing(mut self, playing: bool) -> Self {
        self.is_playing = playing;
        self
    }

    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }

    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn is_liked(mut self, liked: bool) -> Self {
        self.is_liked = liked;
        self
    }

    pub fn is_queue_open(mut self, open: bool) -> Self {
        self.is_queue_open = open;
        self
    }

    pub fn on_play_pause(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_play_pause = Some(Box::new(handler));
        self
    }

    pub fn on_next(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_next = Some(Box::new(handler));
        self
    }

    pub fn on_prev(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_prev = Some(Box::new(handler));
        self
    }

    pub fn on_like_toggle(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_like_toggle = Some(Box::new(handler));
        self
    }

    pub fn on_queue_toggle(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_queue_toggle = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for NowPlayingBar {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let cur_time = self.current_time.unwrap_or_else(|| "0:00".into());
        let tot_time = self.duration_time.unwrap_or_else(|| "0:00".into());
        let display_time = format!("{} / {}", cur_time, tot_time);

        // 1. Top Edge Scrubber Bar (Full Width, 3px height, Red Playhead)
        let seek_handler = self.on_seek.map(std::rc::Rc::new);
        let on_seek_click = seek_handler.clone();

        let live_progress = match self.timeline {
            Some(tl) => {
                if tl.playing {
                    window.request_animation_frame();
                }
                tl.fraction_at(std::time::Instant::now())
            }
            None => self.progress,
        };

        let top_scrubber = div()
            .id("ytm_top_scrubber")
            .relative()
            .w_full()
            .h(px(3.0))
            .cursor_pointer()
            .bg(rgb(0x383838))
            .hover(|s| s.h(px(5.0)))
            .on_click(move |ev: &ClickEvent, window, app| {
                if let Some(ref h) = on_seek_click {
                    let win_w: f32 = window.viewport_size().width.into();
                    let frac = if win_w > 0.0 {
                        let x: f32 = ev.position().x.into();
                        (x / win_w).clamp(0.0, 1.0)
                    } else {
                        0.5
                    };
                    h(frac, window, app);
                }
            })
            .child(
                div()
                    .relative()
                    .h_full()
                    .w(relative(live_progress.clamp(0.0, 1.0)))
                    .bg(rgb(0xFF0000))
                    .child(
                        div()
                            .absolute()
                            .top(px(-3.5))
                            .right(px(-5.0))
                            .size(px(10.0))
                            .rounded(Radii::full())
                            .bg(rgb(0xFF0000))
                            .shadow(vec![crate::tokens::Elevation::sm()])
                    )
            );

        // 2. Left Controls: Prev, Play/Pause, Next, Time
        let mut prev_btn = div()
            .id("ytm_btn_prev")
            .size(px(36.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .child(Icon::new("icons/skip-back.svg").size(px(18.0)).color(rgb(0xFFFFFF)));

        if let Some(h) = self.on_prev {
            prev_btn = prev_btn.on_click(h);
        }

        let mut play_btn = div()
            .id("ytm_btn_play_pause")
            .size(px(40.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF20)));

        if self.is_buffering {
            window.request_animation_frame();
            let elapsed = crate::anim_epoch().elapsed().as_secs_f32();
            let pulse_opacity = 0.40 + 0.55 * (elapsed * 5.5).sin().abs();
            play_btn = play_btn
                .opacity(pulse_opacity)
                .child(Icon::new("icons/loader.svg").size(px(22.0)).color(rgb(0xFFFFFF)));
        } else {
            let play_icon = if self.is_playing { "icons/pause.svg" } else { "icons/play.svg" };
            play_btn = play_btn.child(Icon::new(play_icon).size(px(22.0)).color(rgb(0xFFFFFF)));
        }

        if let Some(h) = self.on_play_pause {
            play_btn = play_btn.on_click(h);
        }

        let mut next_btn = div()
            .id("ytm_btn_next")
            .size(px(36.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .child(Icon::new("icons/skip-forward.svg").size(px(18.0)).color(rgb(0xFFFFFF)));

        if let Some(h) = self.on_next {
            next_btn = next_btn.on_click(h);
        }

        let left_controls = div()
            .flex()
            .items_center()
            .gap(px(4.0))
            .child(prev_btn)
            .child(play_btn)
            .child(next_btn)
            .child(
                div()
                    .ml(px(8.0))
                    .text_xs()
                    .text_color(rgb(0xAAAAAA))
                    .child(display_time)
            );

        // 3. Center Track Info Card: 56x34 (16:9) Video/Album Thumbnail + Title + Subtitle + Thumbs Up + Thumbs Down + More
        let expand_handler = self.on_expand.map(std::rc::Rc::new);
        let on_expand_click = expand_handler.clone();
        let on_expand_chevron = expand_handler.clone();

        let thumb_elem = if let Some(provider) = self.video_provider {
            let mut surface = VideoSurface::new(self.current_clock)
                .provider(provider)
                .width(px(56.0))
                .height(px(34.0))
                .rounded(px(4.0));
            if let Some(src) = self.artwork {
                surface = surface.fallback_artwork(src);
            }
            surface.into_any_element()
        } else if let Some(src) = self.artwork {
            div()
                .w(px(56.0))
                .h(px(34.0))
                .flex_shrink_0()
                .rounded(px(4.0))
                .overflow_hidden()
                .child(
                    img(src)
                        .w_full()
                        .h_full()
                        .object_fit(ObjectFit::Cover)
                )
                .into_any_element()
        } else {
            div()
                .w(px(56.0))
                .h(px(34.0))
                .rounded(px(4.0))
                .bg(rgb(0x282828))
                .flex()
                .items_center()
                .justify_center()
                .child(div().text_xs().text_color(rgb(0x717171)).child("🎵"))
                .into_any_element()
        };

        let sub_text = self.subtitle.unwrap_or(self.artist);

        let mut track_clickable = div()
            .id("ytm_center_track_click")
            .flex()
            .items_center()
            .gap(px(12.0))
            .cursor_pointer()
            .child(thumb_elem)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .justify_center()
                    .gap(px(1.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xFFFFFF))
                            .child(self.title)
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xAAAAAA))
                            .child(sub_text)
                    )
            );

        if let Some(ref h) = on_expand_click {
            let h = h.clone();
            track_clickable = track_clickable.on_click(move |ev, win, app| {
                h(ev, win, app);
            });
        }

        // Thumbs Up / Like Button
        let like_icon = if self.is_liked { "icons/thumbs-up-filled.svg" } else { "icons/thumbs-up.svg" };
        let like_color = if self.is_liked { rgb(0xFFFFFF) } else { rgb(0xAAAAAA) };
        let mut like_btn = div()
            .id("ytm_btn_like")
            .size(px(32.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .child(Icon::new(like_icon).size(px(18.0)).color(like_color));

        if let Some(h) = self.on_like_toggle {
            like_btn = like_btn.on_click(h);
        }

        // Thumbs Down / Dislike Button
        let dislike_icon = if self.is_disliked { "icons/thumbs-down-filled.svg" } else { "icons/thumbs-down.svg" };
        let dislike_color = if self.is_disliked { rgb(0xFFFFFF) } else { rgb(0xAAAAAA) };
        let mut dislike_btn = div()
            .id("ytm_btn_dislike")
            .size(px(32.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .child(Icon::new(dislike_icon).size(px(18.0)).color(dislike_color));

        if let Some(h) = self.on_dislike_toggle {
            dislike_btn = dislike_btn.on_click(h);
        }

        // More Options Button (⋮)
        let mut more_btn = div()
            .id("ytm_btn_more")
            .size(px(32.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .child(Icon::new("icons/dots-vertical.svg").size(px(18.0)).color(rgb(0xAAAAAA)));

        if let Some(h) = self.on_more_menu {
            more_btn = more_btn.on_click(h);
        }

        let center_card = div()
            .id("ytm_center_card")
            .flex()
            .items_center()
            .gap(px(12.0))
            .child(track_clickable)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(2.0))
                    .child(like_btn)
                    .child(dislike_btn)
                    .child(more_btn)
            );

        // 4. Right Controls: Volume with Slider, CC (Subtitles), Repeat, Shuffle, Down Caret
        let mut vol_btn = div()
            .id("ytm_btn_vol_mute")
            .size(px(32.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .child(
                Icon::new(if self.volume <= 0.01 { "icons/volume-x.svg" } else { "icons/volume-2.svg" })
                    .size(px(18.0))
                    .color(rgb(0xAAAAAA))
            );

        if let Some(h) = self.on_volume_mute {
            vol_btn = vol_btn.on_click(h);
        }

        let mut vol_slider = Slider::new(self.volume)
            .id("ytm_vol_slider")
            .w(px(70.0))
            .h(px(14.0))
            .active_color(rgb(0xFF0000));

        if let Some(on_vol) = self.on_volume {
            vol_slider = vol_slider.on_change(move |v, win, app| on_vol(v, win, app));
        }

        // Subtitles / Closed Captions [CC] Button
        let is_captions = self.is_captions_enabled;
        let mut captions_btn = div()
            .id("ytm_btn_captions")
            .occlude()
            .size(px(32.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .bg(if is_captions { rgba(0xFFFFFF25) } else { rgba(0x00000000) })
            .hover(move |s| {
                if is_captions {
                    s.bg(rgba(0xFFFFFF35))
                } else {
                    s.bg(rgba(0xFFFFFF15))
                }
            })
            .child(
                Icon::new("icons/closed-captions.svg")
                    .size(px(18.0))
                    .color(if is_captions { rgb(0xFFFFFF) } else { rgb(0xAAAAAA) })
            );

        if let Some(h) = self.on_captions_toggle {
            captions_btn = captions_btn.on_click(h);
        }

        let repeat_mode = self.repeat_mode;
        let repeat_active = repeat_mode > 0;
        let repeat_icon = if repeat_mode == 2 {
            "icons/repeat-once.svg"
        } else {
            "icons/repeat.svg"
        };
        let mut repeat_btn = div()
            .id("ytm_btn_repeat")
            .occlude()
            .size(px(32.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .bg(if repeat_mode == 2 {
                rgba(0xFFFFFF30)
            } else if repeat_mode == 1 {
                rgba(0xFFFFFF20)
            } else {
                rgba(0x00000000)
            })
            .hover(move |s| {
                if repeat_mode == 2 {
                    s.bg(rgba(0xFFFFFF40))
                } else if repeat_mode == 1 {
                    s.bg(rgba(0xFFFFFF30))
                } else {
                    s.bg(rgba(0xFFFFFF15))
                }
            })
            .child(
                Icon::new(repeat_icon)
                    .size(px(18.0))
                    .color(if repeat_active { rgb(0xFFFFFF) } else { rgb(0xAAAAAA) })
            );

        if let Some(h) = self.on_repeat {
            repeat_btn = repeat_btn.on_click(h);
        }

        let is_shuffled = self.is_shuffled;
        let mut shuffle_btn = div()
            .id("ytm_btn_shuffle")
            .occlude()
            .size(px(32.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .bg(if is_shuffled { rgba(0xFFFFFF25) } else { rgba(0x00000000) })
            .hover(move |s| {
                if is_shuffled {
                    s.bg(rgba(0xFFFFFF35))
                } else {
                    s.bg(rgba(0xFFFFFF15))
                }
            })
            .child(
                Icon::new("icons/shuffle.svg")
                    .size(px(18.0))
                    .color(if is_shuffled { rgb(0xFFFFFF) } else { rgb(0xAAAAAA) })
            );

        if let Some(h) = self.on_shuffle {
            shuffle_btn = shuffle_btn.on_click(h);
        }

        let mut caret_btn = div()
            .id("ytm_btn_expand_caret")
            .size(px(32.0))
            .rounded(Radii::full())
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .hover(|s| s.bg(rgba(0xFFFFFF15)))
            .child(
                Icon::new(if self.is_expanded { "icons/caret-down.svg" } else { "icons/caret-down.svg" })
                    .size(px(14.0))
                    .color(rgb(0xAAAAAA))
            );

        if let Some(ref h) = on_expand_chevron {
            let h = h.clone();
            caret_btn = caret_btn.on_click(move |ev, win, app| h(ev, win, app));
        }

        let right_controls = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(vol_btn)
            .child(vol_slider)
            .child(div().w(px(4.0)))
            .child(captions_btn)
            .child(repeat_btn)
            .child(shuffle_btn)
            .child(caret_btn);

        // Container: Fixed 72px dock at bottom edge
        let bg_color = if self.is_glass {
            rgba(0x181818EE)
        } else {
            rgb(0x212121)
        };

        div()
            .id("ytm_bottom_dock")
            .flex_shrink_0()
            .relative()
            .w_full()
            .h(px(72.0))
            .bg(bg_color)
            .border_t_1()
            .border_color(rgba(0xFFFFFF0D))
            .flex()
            .flex_col()
            .child(top_scrubber)
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(16.0))
                    .child(left_controls)
                    .child(center_card)
                    .child(right_controls)
            )
    }
}
