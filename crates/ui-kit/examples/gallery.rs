use gpui::prelude::*;
use gpui::*;
use ui_kit::tokens::{Palette, Radii, Spacing, Theme};
use ui_kit::atoms::{Button, ButtonVariant, IconButton, IconButtonVariant, Text, TextVariant, Slider, SeekBar};
use ui_kit::molecules::{AlbumCard, ArtistCard, TrackRow};
use ui_kit::organisms::now_playing_bar::NowPlayingBar;

struct GalleryView {
    is_dark_mode: bool,
    theme: Theme,
    slider_val: f32,
}

impl GalleryView {
    fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            is_dark_mode: true,
            theme: Theme::with_palette(Palette::dark()),
            slider_val: 0.45,
        }
    }

    fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.is_dark_mode = !self.is_dark_mode;
        self.theme.palette = if self.is_dark_mode {
            Palette::dark()
        } else {
            Palette::light()
        };
        cx.notify();
    }
}

impl Render for GalleryView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = &self.theme.palette;

        div()
            .id("gallery_scroll")
            .flex()
            .flex_col()
            .size_full()
            .bg(palette.background)
            .p(Spacing::s6())
            .gap(Spacing::s5())
            .overflow_y_scroll()
            // Header
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(Text::new("Rustic UI-Kit Gallery").variant(TextVariant::H1))
                            .child(Text::new("Showcase of design tokens, atoms, molecules, and organisms").variant(TextVariant::Caption).muted())
                    )
                    .child(
                        Button::new(if self.is_dark_mode { "Switch to Light Mode" } else { "Switch to Dark Mode" })
                            .variant(ButtonVariant::Primary)
                            .on_click(cx.listener(|this, _ev, _window, cx| this.toggle_theme(cx)))
                    )
            )
            // Showcase: Floating Dynamic Island Dock (Glass Mode & Solid Mode)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(Spacing::s3())
                    .p(Spacing::s4())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border)
                    .child(Text::new("Organisms: Floating Dynamic Glass Island Dock (Live Active Playback)").variant(TextVariant::H2))
                    .child(
                        NowPlayingBar::new("Barsaat", "Banjaare & Roni")
                            .subtitle("Banjaare & Roni • 51M views • 599K likes")
                            .is_playing(true)
                            .progress(0.48)
                            .times("1:48", "3:06")
                            .volume(0.85)
                            .is_liked(true)
                            .is_captions_enabled(true)
                            .repeat_mode(1)
                            .is_shuffled(true)
                            .is_glass(true)
                    )
                    .child(
                        NowPlayingBar::new("Cybernetic Drift", "Tokyo Rose")
                            .subtitle("Tokyo Rose • 12M views • 142K likes")
                            .is_playing(false)
                            .progress(0.15)
                            .times("0:32", "3:30")
                            .volume(0.60)
                            .is_disliked(true)
                            .is_glass(false)
                    )
            )
            // Color Palette Tokens
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(Spacing::s3())
                    .p(Spacing::s4())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border)
                    .child(Text::new("Design Tokens: Palette").variant(TextVariant::H2))
                    .child(
                        div()
                            .flex()
                            .gap(Spacing::s3())
                            .child(div().size(px(40.0)).rounded(Radii::base()).bg(palette.primary).border_1().border_color(palette.border))
                            .child(div().size(px(40.0)).rounded(Radii::base()).bg(palette.surface).border_1().border_color(palette.border))
                            .child(div().size(px(40.0)).rounded(Radii::base()).bg(palette.surface_hover).border_1().border_color(palette.border))
                            .child(div().size(px(40.0)).rounded(Radii::base()).bg(palette.background).border_1().border_color(palette.border))
                            .child(div().size(px(40.0)).rounded(Radii::base()).bg(palette.text).border_1().border_color(palette.border))
                    )
            )
            // Atoms: Buttons & Sliders
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(Spacing::s3())
                    .p(Spacing::s4())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border)
                    .child(Text::new("Atoms: Buttons & Sliders").variant(TextVariant::H2))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(Spacing::s3())
                            .child(Button::new("Primary Action").variant(ButtonVariant::Primary))
                            .child(Button::new("Secondary").variant(ButtonVariant::Secondary))
                            .child(Button::new("Ghost").variant(ButtonVariant::Ghost))
                            .child(IconButton::new("icons/play.svg").variant(IconButtonVariant::Primary))
                            .child(IconButton::new("icons/heart.svg").variant(IconButtonVariant::Ghost))
                    )
                    .child(
                        div()
                            .w(px(320.0))
                            .mt(Spacing::s2())
                            .child(Slider::new(self.slider_val))
                    )
                    .child(
                        div()
                            .w(px(320.0))
                            .child(SeekBar::new(self.slider_val).peaks(vec![0.3, 0.6, 0.9, 0.5, 0.8, 0.4, 0.7, 0.2, 0.5, 0.9, 0.7, 0.4]))
                    )
            )
            // Molecules: Cards & Rows
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(Spacing::s3())
                    .p(Spacing::s4())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border)
                    .child(Text::new("Molecules: Cards & Rows").variant(TextVariant::H2))
                    .child(
                        div()
                            .flex()
                            .gap(Spacing::s4())
                            .child(AlbumCard::new("Neon Horizon", "Vektor Prime"))
                            .child(ArtistCard::new("Komorebi"))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(Spacing::s2())
                                    .child(TrackRow::new("Midnight Circuit").artist("Vektor Prime").album("Neon Horizon").duration("3:42").active(true))
                                    .child(TrackRow::new("Window Fog").artist("Komorebi").album("Rainy Afternoon").duration("2:15"))
                                    .child(TrackRow::new("Bourbon & Brass").artist("The Silk Quartet").album("Velvet Midnight").duration("4:05"))
                            )
                    )
            )
            // Organism: Now Playing Bar
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(Spacing::s3())
                    .p(Spacing::s4())
                    .rounded(Radii::large())
                    .bg(palette.surface)
                    .border_1()
                    .border_color(palette.border)
                    .child(Text::new("Organisms: Now Playing Bar").variant(TextVariant::H2))
                    .child(
                        NowPlayingBar::new("Midnight Circuit", "Vektor Prime")
                            .is_playing(true)
                            .progress(0.45)
                            .volume(0.8)
                    )
            )
    }
}

fn main() {
    let platform = gpui_platform::current_platform(false);
    Application::with_platform(platform).run(move |cx: &mut App| {
        cx.set_global(Theme::with_palette(Palette::dark()));

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point::new(0.0.into(), 0.0.into()),
                size: Size {
                    width: 1000.0.into(),
                    height: 750.0.into(),
                },
            })),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_window, cx| {
            cx.new(|cx| GalleryView::new(cx))
        });
    });
}
