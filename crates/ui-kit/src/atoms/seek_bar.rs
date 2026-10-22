use gpui::prelude::*;
use gpui::*;
use crate::atoms::{PlaybackTimeline, Slider};
use crate::tokens::{Radii, Theme};

#[derive(IntoElement)]
pub struct SeekBar {
    progress: f32, // 0.0 to 1.0
    timeline: Option<PlaybackTimeline>,
    peaks: Option<Vec<f32>>,
    on_seek: Option<Box<dyn Fn(f32, &mut Window, &mut App) + 'static>>,
}

impl SeekBar {
    pub fn new(progress: f32) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            timeline: None,
            peaks: None,
            on_seek: None,
        }
    }

    pub fn timeline(mut self, timeline: PlaybackTimeline) -> Self {
        self.timeline = Some(timeline);
        self
    }

    pub fn peaks(mut self, peaks: Vec<f32>) -> Self {
        self.peaks = Some(peaks);
        self
    }

    pub fn on_seek(mut self, handler: impl Fn(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_seek = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SeekBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        if let Some(peaks) = self.peaks {
            // Waveform seek bar
            let mut container = div()
                .flex()
                .items_end()
                .gap(px(1.0))
                .w_full()
                .h(px(24.0));
            
            let num_peaks = peaks.len();
            for (i, &peak) in peaks.iter().enumerate() {
                let peak_progress = i as f32 / num_peaks as f32;
                let color = if peak_progress <= self.progress {
                    palette.primary
                } else {
                    palette.surface_hover
                };

                let bar_height = px(24.0 * peak.clamp(0.1, 1.0));
                
                container = container.child(
                    div()
                        .w(px(3.0))
                        .h(bar_height)
                        .bg(color)
                        .rounded(Radii::full())
                );
            }
            container
        } else {
            // Flat seek bar fallback with analytical timeline interpolation
            let mut slider = if let Some(tl) = self.timeline {
                Slider::timeline(tl)
            } else {
                Slider::new(self.progress)
            }
            .id("seek_bar_slider")
            .w_full()
            .h(px(18.0));

            if let Some(h) = self.on_seek {
                slider = slider.on_change(h);
            }
            div()
                .w_full()
                .flex()
                .items_center()
                .child(slider)
        }
    }
}
