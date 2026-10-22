use gpui::prelude::*;
use gpui::*;
use crate::molecules::TrackRow;
use crate::tokens::{Radii, Spacing, Theme};

#[derive(Clone)]
pub struct TrackItem {
    pub id: SharedString,
    pub title: SharedString,
    pub artist: SharedString,
    pub album: SharedString,
    pub duration: SharedString,
    pub is_active: bool,
}

#[derive(IntoElement)]
pub struct TrackList {
    tracks: Vec<TrackItem>,
    on_select: Option<std::sync::Arc<dyn Fn(SharedString, &mut Window, &mut App) + Send + Sync + 'static>>,
}

impl TrackList {
    pub fn new(tracks: Vec<TrackItem>) -> Self {
        Self {
            tracks,
            on_select: None,
        }
    }

    pub fn on_select(mut self, handler: impl Fn(SharedString, &mut Window, &mut App) + Send + Sync + 'static) -> Self {
        self.on_select = Some(std::sync::Arc::new(handler));
        self
    }
}

impl RenderOnce for TrackList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;

        let mut list = div()
            .id("track-list-scroll")
            .flex()
            .flex_col()
            .w_full()
            .flex_1()
            .overflow_y_scroll();

        let handler = self.on_select.clone();

        for item in self.tracks {
            let item_id = item.id.clone();
            let mut row = TrackRow::new(item.title)
                .artist(item.artist)
                .album(item.album)
                .duration(item.duration)
                .active(item.is_active);

            if let Some(h) = handler.clone() {
                row = row.on_click(move |_ev, win, app| {
                    h(item_id.clone(), win, app);
                });
            }

            list = list.child(row);
        }

        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .overflow_hidden()
            .child(
                // Sticky Table Header
                div()
                    .flex()
                    .items_center()
                    .w_full()
                    .px(Spacing::s4())
                    .py(Spacing::s2())
                    .border_b_1()
                    .border_color(palette.border_subtle)
                    .bg(palette.surface)
                    .rounded_t(Radii::base())
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(div().w(Length::Definite(DefiniteLength::Fraction(0.4))).child("TITLE").text_color(palette.text_muted))
                    .child(div().w(Length::Definite(DefiniteLength::Fraction(0.3))).child("ARTIST").text_color(palette.text_muted))
                    .child(div().w(Length::Definite(DefiniteLength::Fraction(0.2))).child("ALBUM").text_color(palette.text_muted))
                    .child(div().w(Length::Definite(DefiniteLength::Fraction(0.1))).flex().justify_end().child("TIME").text_color(palette.text_muted))
            )
            .child(list)
    }
}
