use gpui::prelude::*;
use gpui::*;
use crate::atoms::{Text, TextVariant};
use crate::tokens::{Radii, Spacing, Theme};

#[derive(IntoElement, Clone)]
pub struct TrackRow {
    title: SharedString,
    artist: SharedString,
    album: SharedString,
    duration: SharedString,
    is_active: bool,
    on_click: Option<std::sync::Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync + 'static>>,
}

impl TrackRow {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            artist: "Unknown Artist".into(),
            album: "Unknown Album".into(),
            duration: "0:00".into(),
            is_active: false,
            on_click: None,
        }
    }

    pub fn artist(mut self, artist: impl Into<SharedString>) -> Self {
        self.artist = artist.into();
        self
    }

    pub fn album(mut self, album: impl Into<SharedString>) -> Self {
        self.album = album.into();
        self
    }

    pub fn duration(mut self, duration: impl Into<SharedString>) -> Self {
        self.duration = duration.into();
        self
    }

    pub fn active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync + 'static) -> Self {
        self.on_click = Some(std::sync::Arc::new(handler));
        self
    }
}

impl RenderOnce for TrackRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;
        
        let bg_color = if self.is_active {
            palette.surface_hover
        } else {
            rgba(0x00000000)
        };

        let title_color = if self.is_active { palette.primary } else { palette.text };

        let mut row = div()
            .id(SharedString::from(format!("track_{}_{}", self.title, self.artist)))
            .flex()
            .items_center()
            .w_full()
            .p(Spacing::s3())
            .bg(bg_color)
            .hover(|s| s.bg(palette.surface_hover))
            .rounded(Radii::base())
            .cursor_pointer()
            .child(
                div()
                    .w(Length::Definite(DefiniteLength::Fraction(0.4)))
                    .child(
                        Text::new(self.title)
                            .variant(TextVariant::Body)
                            .color(title_color)
                    )
            )
            .child(
                div()
                    .w(Length::Definite(DefiniteLength::Fraction(0.3)))
                    .child(
                        Text::new(self.artist)
                            .variant(TextVariant::Body)
                            .muted()
                    )
            )
            .child(
                div()
                    .w(Length::Definite(DefiniteLength::Fraction(0.2)))
                    .child(
                        Text::new(self.album)
                            .variant(TextVariant::Body)
                            .muted()
                    )
            )
            .child(
                div()
                    .w(Length::Definite(DefiniteLength::Fraction(0.1)))
                    .flex()
                    .justify_end()
                    .child(
                        Text::new(self.duration)
                            .variant(TextVariant::Body)
                            .muted()
                    )
            );

        if let Some(on_click) = self.on_click {
            row = row.on_click(move |ev, win, app| (*on_click)(ev, win, app));
        }

        row
    }
}
