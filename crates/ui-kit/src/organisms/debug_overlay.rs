use gpui::prelude::*;
use gpui::*;
use crate::tokens::{Spacing, Theme};
use crate::atoms::{Text, TextVariant};

#[derive(Clone, Debug)]
pub struct DebugLogItem {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

#[derive(IntoElement)]
pub struct DebugOverlay {
    pub audio_device: String,
    pub audio_sample_rate: u32,
    pub audio_channels: usize,
    pub audio_format: String,
    pub audio_is_running: bool,
    pub audio_has_error: bool,
    pub audio_last_error: Option<String>,
    pub audio_underruns: u64,
    pub audio_dac_frames: u64,
    pub current_position_secs: f64,
    pub track_duration_secs: f64,
    pub track_title: String,
    pub track_artist: String,
    pub playback_state: String,
    pub volume_percent: u32,
    pub queue_len: usize,
    pub logs: Vec<DebugLogItem>,
    pub errors_only: bool,
    pub on_reconnect_audio: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    pub on_copy_diagnostics: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    pub on_clear_logs: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    pub on_toggle_filter: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    pub on_close: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            audio_device: "Default Audio Device".into(),
            audio_sample_rate: 48000,
            audio_channels: 2,
            audio_format: "F32".into(),
            audio_is_running: false,
            audio_has_error: false,
            audio_last_error: None,
            audio_underruns: 0,
            audio_dac_frames: 0,
            current_position_secs: 0.0,
            track_duration_secs: 0.0,
            track_title: "No Track".into(),
            track_artist: "RusticPlayer".into(),
            playback_state: "Stopped".into(),
            volume_percent: 100,
            queue_len: 0,
            logs: Vec::new(),
            errors_only: false,
            on_reconnect_audio: None,
            on_copy_diagnostics: None,
            on_clear_logs: None,
            on_toggle_filter: None,
            on_close: None,
        }
    }

    pub fn audio_device(mut self, device: impl Into<String>) -> Self {
        self.audio_device = device.into();
        self
    }

    pub fn audio_sample_rate(mut self, sr: u32) -> Self {
        self.audio_sample_rate = sr;
        self
    }

    pub fn audio_channels(mut self, ch: usize) -> Self {
        self.audio_channels = ch;
        self
    }

    pub fn audio_format(mut self, fmt: impl Into<String>) -> Self {
        self.audio_format = fmt.into();
        self
    }

    pub fn audio_is_running(mut self, running: bool) -> Self {
        self.audio_is_running = running;
        self
    }

    pub fn audio_has_error(mut self, err: bool) -> Self {
        self.audio_has_error = err;
        self
    }

    pub fn audio_last_error(mut self, err: Option<String>) -> Self {
        self.audio_last_error = err;
        self
    }

    pub fn audio_underruns(mut self, underruns: u64) -> Self {
        self.audio_underruns = underruns;
        self
    }

    pub fn audio_dac_frames(mut self, frames: u64) -> Self {
        self.audio_dac_frames = frames;
        self
    }

    pub fn current_position_secs(mut self, secs: f64) -> Self {
        self.current_position_secs = secs;
        self
    }

    pub fn track_duration_secs(mut self, secs: f64) -> Self {
        self.track_duration_secs = secs;
        self
    }

    pub fn track_title(mut self, title: impl Into<String>) -> Self {
        self.track_title = title.into();
        self
    }

    pub fn track_artist(mut self, artist: impl Into<String>) -> Self {
        self.track_artist = artist.into();
        self
    }

    pub fn playback_state(mut self, state: impl Into<String>) -> Self {
        self.playback_state = state.into();
        self
    }

    pub fn volume_percent(mut self, vol: u32) -> Self {
        self.volume_percent = vol;
        self
    }

    pub fn queue_len(mut self, len: usize) -> Self {
        self.queue_len = len;
        self
    }

    pub fn logs(mut self, logs: Vec<DebugLogItem>) -> Self {
        self.logs = logs;
        self
    }

    pub fn errors_only(mut self, errors: bool) -> Self {
        self.errors_only = errors;
        self
    }

    pub fn on_reconnect_audio(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_reconnect_audio = Some(Box::new(handler));
        self
    }

    pub fn on_copy_diagnostics(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_copy_diagnostics = Some(Box::new(handler));
        self
    }

    pub fn on_clear_logs(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_clear_logs = Some(Box::new(handler));
        self
    }

    pub fn on_toggle_filter(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_filter = Some(Box::new(handler));
        self
    }

    pub fn on_close(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_close = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for DebugOverlay {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        let status_color = if self.audio_has_error {
            rgba(0xFF453AFF)
        } else if self.audio_is_running {
            rgba(0x30D158FF)
        } else {
            rgba(0xFF9F0AFF)
        };

        let status_label = if self.audio_has_error {
            "FAULTED / UNPLUGGED"
        } else if self.audio_is_running {
            "STREAM HEALTHY"
        } else {
            "IDLE / PAUSED"
        };

        let cur_secs = self.current_position_secs as u64;
        let cur_ms = ((self.current_position_secs - cur_secs as f64) * 1000.0) as u64;
        let dur_secs = self.track_duration_secs as u64;

        div()
            .id("debug_overlay_backdrop")
            .w_full()
            .h_full()
            .bg(rgba(0x0a0c14F6))
            .flex()
            .flex_col()
            .p(Spacing::s4())
            .gap(Spacing::s3())
            // Header Bar
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .pb(Spacing::s2())
                    .border_b_1()
                    .border_color(rgba(0xFFFFFF18))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(Spacing::s3())
                            .child(
                                Text::new("🛠️ RUSTICPLAYER DEBUG & DIAGNOSTIC HUD")
                                    .variant(TextVariant::H2)
                                    .color(palette.text)
                            )
                            .child(
                                div()
                                    .px(px(8.0))
                                    .py(px(2.0))
                                    .rounded(px(10.0))
                                    .bg(status_color.opacity(0.18))
                                    .border_1()
                                    .border_color(status_color)
                                    .child(
                                        Text::new(format!("● {}", status_label))
                                            .variant(TextVariant::Caption)
                                            .color(status_color)
                                    )
                            )
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(Spacing::s2())
                            .when_some(self.on_reconnect_audio, |el, handler| {
                                el.child(
                                    div()
                                        .id("btn_reconnect_audio")
                                        .flex()
                                        .items_center()
                                        .gap(px(4.0))
                                        .px(px(10.0))
                                        .py(px(4.0))
                                        .rounded(px(6.0))
                                        .bg(palette.primary)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(palette.primary_hover))
                                        .on_click(handler)
                                        .child(
                                            Text::new("🔄 Reconnect Audio")
                                                .variant(TextVariant::Caption)
                                                .color(palette.on_primary)
                                        )
                                )
                            })
                            .when_some(self.on_copy_diagnostics, |el, handler| {
                                el.child(
                                    div()
                                        .id("btn_copy_diagnostics")
                                        .flex()
                                        .items_center()
                                        .gap(px(4.0))
                                        .px(px(10.0))
                                        .py(px(4.0))
                                        .rounded(px(6.0))
                                        .bg(palette.surface_elevated)
                                        .border_1()
                                        .border_color(palette.border)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(palette.surface_hover))
                                        .on_click(handler)
                                        .child(
                                            Text::new("📋 Copy Report")
                                                .variant(TextVariant::Caption)
                                                .color(palette.text)
                                        )
                                )
                            })
                            .when_some(self.on_close, |el, handler| {
                                el.child(
                                    div()
                                        .id("btn_close_debug")
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(px(28.0))
                                        .rounded(px(6.0))
                                        .bg(palette.surface)
                                        .border_1()
                                        .border_color(palette.border)
                                        .cursor_pointer()
                                        .hover(|s| s.bg(palette.surface_hover))
                                        .on_click(handler)
                                        .child(
                                            Text::new("✕")
                                                .variant(TextVariant::Body)
                                                .color(palette.text)
                                        )
                                )
                            })
                    )
            )
            // 3-Column Metrics Grid
            .child(
                div()
                    .w_full()
                    .flex()
                    .gap(Spacing::s3())
                    // Column 1: Audio Engine & DAC
                    .child(
                        div()
                            .flex_1()
                            .p(Spacing::s3())
                            .rounded(px(8.0))
                            .bg(rgba(0xFFFFFF08))
                            .border_1()
                            .border_color(rgba(0xFFFFFF15))
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                Text::new("🔊 AUDIO ENGINE & DAC")
                                    .variant(TextVariant::Caption)
                                    .color(rgba(0x64D2FFFF))
                            )
                            .child(
                                Text::new(format!("Endpoint: {}", self.audio_device))
                                    .variant(TextVariant::Caption)
                                    .color(palette.text)
                            )
                            .child(
                                Text::new(format!("Config: {}Hz · {}ch ({})", self.audio_sample_rate, self.audio_channels, self.audio_format))
                                    .variant(TextVariant::Caption)
                                    .color(palette.text_muted)
                            )
                            .child(
                                Text::new(format!("Underruns: {} | DAC Frames: {}", self.audio_underruns, self.audio_dac_frames))
                                    .variant(TextVariant::Caption)
                                    .color(if self.audio_underruns > 0 { rgba(0xFF9F0AFF) } else { palette.text_muted })
                            )
                            .when_some(self.audio_last_error, |el, err| {
                                el.child(
                                    Text::new(format!("Error: {}", err))
                                        .variant(TextVariant::Caption)
                                        .color(rgba(0xFF453AFF))
                                )
                            })
                    )
                    // Column 2: Playback Ground Truth
                    .child(
                        div()
                            .flex_1()
                            .p(Spacing::s3())
                            .rounded(px(8.0))
                            .bg(rgba(0xFFFFFF08))
                            .border_1()
                            .border_color(rgba(0xFFFFFF15))
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                Text::new("🎵 PLAYBACK STATE & CLOCK")
                                    .variant(TextVariant::Caption)
                                    .color(rgba(0xBF5AF2FF))
                            )
                            .child(
                                Text::new(format!("Track: {}", self.track_title))
                                    .variant(TextVariant::Caption)
                                    .color(palette.text)
                            )
                            .child(
                                Text::new(format!("Artist: {} | State: {}", self.track_artist, self.playback_state))
                                    .variant(TextVariant::Caption)
                                    .color(palette.text_muted)
                            )
                            .child(
                                Text::new(format!("Position: {}:{:02}.{:03} / {}:{:02} | Vol: {}%",
                                    cur_secs / 60, cur_secs % 60, cur_ms,
                                    dur_secs / 60, dur_secs % 60,
                                    self.volume_percent
                                ))
                                    .variant(TextVariant::Caption)
                                    .color(palette.text_muted)
                            )
                    )
                    // Column 3: Platform & Runtime
                    .child(
                        div()
                            .flex_1()
                            .p(Spacing::s3())
                            .rounded(px(8.0))
                            .bg(rgba(0xFFFFFF08))
                            .border_1()
                            .border_color(rgba(0xFFFFFF15))
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                Text::new("💻 PLATFORM & CRASH TRAP")
                                    .variant(TextVariant::Caption)
                                    .color(rgba(0x30D158FF))
                            )
                            .child(
                                Text::new("Backend: DirectX 11.1 · DirectWrite")
                                    .variant(TextVariant::Caption)
                                    .color(palette.text)
                            )
                            .child(
                                Text::new("Crash Trap: Active (rustic_panic.log)")
                                    .variant(TextVariant::Caption)
                                    .color(palette.text_muted)
                            )
                            .child(
                                Text::new(format!("Queue Size: {} | Log Entries: {}", self.queue_len, self.logs.len()))
                                    .variant(TextVariant::Caption)
                                    .color(palette.text_muted)
                            )
                    )
            )
            // Live Tracing Log Console
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .rounded(px(8.0))
                    .bg(rgba(0x04060bee))
                    .border_1()
                    .border_color(rgba(0xFFFFFF20))
                    .p(Spacing::s3())
                    .gap(Spacing::s2())
                    // Console Header & Filter controls
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .items_center()
                            .justify_between()
                            .pb(px(4.0))
                            .border_b_1()
                            .border_color(rgba(0xFFFFFF10))
                            .child(
                                Text::new("📜 LIVE TRACING EVENT LOG")
                                    .variant(TextVariant::Caption)
                                    .color(rgba(0xFFFFFF80))
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(Spacing::s2())
                                    .when_some(self.on_toggle_filter, |el, handler| {
                                        el.child(
                                            div()
                                                .id("btn_toggle_log_filter")
                                                .px(px(8.0))
                                                .py(px(2.0))
                                                .rounded(px(4.0))
                                                .bg(if self.errors_only { rgba(0xFF453A33) } else { rgba(0xFFFFFF15) })
                                                .border_1()
                                                .border_color(if self.errors_only { rgba(0xFF453AFF) } else { rgba(0xFFFFFF30) })
                                                .cursor_pointer()
                                                .on_click(handler)
                                                .child(
                                                    Text::new(if self.errors_only { "Filtered: Errors & Warnings" } else { "Filter: All Logs" })
                                                        .variant(TextVariant::Caption)
                                                        .color(if self.errors_only { rgba(0xFF453AFF) } else { palette.text })
                                                )
                                        )
                                    })
                                    .when_some(self.on_clear_logs, |el, handler| {
                                        el.child(
                                            div()
                                                .id("btn_clear_logs")
                                                .px(px(8.0))
                                                .py(px(2.0))
                                                .rounded(px(4.0))
                                                .bg(rgba(0xFFFFFF10))
                                                .cursor_pointer()
                                                .hover(|s| s.bg(rgba(0xFFFFFF20)))
                                                .on_click(handler)
                                                .child(
                                                    Text::new("Clear")
                                                        .variant(TextVariant::Caption)
                                                        .color(palette.text_muted)
                                                )
                                        )
                                    })
                            )
                    )
                    // Scrollable log lines
                    .child(
                        div()
                            .id("debug_log_scroll_container")
                            .flex_1()
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .when(self.logs.is_empty(), |el| {
                                el.child(
                                    div()
                                        .p(Spacing::s4())
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            Text::new("No log events recorded yet.")
                                                .variant(TextVariant::Caption)
                                                .color(palette.text_muted)
                                        )
                                )
                            })
                            .children(
                                self.logs.into_iter().rev().map(|item| {
                                    let level_color = match item.level.to_uppercase().as_str() {
                                        "ERROR" => rgba(0xFF453AFF),
                                        "WARN" => rgba(0xFFD60AFF),
                                        "INFO" => rgba(0x32D74BFF),
                                        _ => rgba(0x8E8E93FF),
                                    };

                                    div()
                                        .flex()
                                        .items_start()
                                        .gap(Spacing::s2())
                                        .py(px(1.0))
                                        .child(
                                            Text::new(item.timestamp)
                                                .variant(TextVariant::Caption)
                                                .color(rgba(0x8E8E9399))
                                        )
                                        .child(
                                            div()
                                                .px(px(4.0))
                                                .py(px(1.0))
                                                .rounded(px(3.0))
                                                .bg(level_color.opacity(0.15))
                                                .child(
                                                    Text::new(item.level)
                                                        .variant(TextVariant::Caption)
                                                        .color(level_color)
                                                )
                                        )
                                        .child(
                                            Text::new(format!("[{}]", item.target))
                                                .variant(TextVariant::Caption)
                                                .color(rgba(0x64D2FFBB))
                                        )
                                        .child(
                                            Text::new(item.message)
                                                .variant(TextVariant::Caption)
                                                .color(palette.text)
                                        )
                                })
                            )
                    )
            )
    }
}
