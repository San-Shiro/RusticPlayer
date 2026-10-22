use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use gpui::prelude::*;
use gpui::*;
use crate::tokens::Theme;

type SliderHandler = dyn FnMut(f32, &mut Window, &mut App);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaybackTimeline {
    pub anchor_position: f64,
    pub anchor_instant: Instant,
    pub playback_rate: f64,
    pub duration: f64,
    pub playing: bool,
    pub revision: u64,
}

impl PlaybackTimeline {
    pub fn new(anchor_position: f64, duration: f64, playing: bool) -> Self {
        Self {
            anchor_position,
            anchor_instant: Instant::now(),
            playback_rate: 1.0,
            duration,
            playing,
            revision: 0,
        }
    }

    #[inline]
    pub fn position_at(&self, now: Instant) -> f64 {
        let pos = if self.playing {
            self.anchor_position + now.saturating_duration_since(self.anchor_instant).as_secs_f64() * self.playback_rate
        } else {
            self.anchor_position
        };
        pos.clamp(0.0, self.duration.max(0.0))
    }

    #[inline]
    pub fn fraction_at(&self, now: Instant) -> f32 {
        if self.duration <= 0.0 {
            0.0
        } else {
            (self.position_at(now) / self.duration).clamp(0.0, 1.0) as f32
        }
    }
}

impl Default for PlaybackTimeline {
    fn default() -> Self {
        Self::new(0.0, 0.0, false)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SliderValue {
    Static(f32),
    Timeline(PlaybackTimeline),
}

pub struct Slider {
    id: Option<ElementId>,
    style: StyleRefinement,
    value: SliderValue,
    on_change: Option<Rc<RefCell<SliderHandler>>>,
    change_interval: Option<Duration>,
    active_color: Option<Rgba>,
}

impl Slider {
    pub fn new(value: f32) -> Self {
        Self {
            id: Some(ElementId::Name("slider".into())),
            style: StyleRefinement::default(),
            value: SliderValue::Static(value.clamp(0.0, 1.0)),
            on_change: None,
            change_interval: Some(Duration::from_millis(16)),
            active_color: None,
        }
    }

    pub fn timeline(timeline: PlaybackTimeline) -> Self {
        Self {
            id: Some(ElementId::Name("slider".into())),
            style: StyleRefinement::default(),
            value: SliderValue::Timeline(timeline),
            on_change: None,
            change_interval: Some(Duration::from_millis(16)),
            active_color: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn active_color(mut self, color: impl Into<Rgba>) -> Self {
        self.active_color = Some(color.into());
        self
    }

    pub fn value(mut self, value: f32) -> Self {
        self.value = SliderValue::Static(value.clamp(0.0, 1.0));
        self
    }

    pub fn with_timeline(mut self, timeline: PlaybackTimeline) -> Self {
        self.value = SliderValue::Timeline(timeline);
        self
    }

    pub fn on_change(mut self, func: impl FnMut(f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(RefCell::new(func)));
        self
    }

    pub fn change_interval(mut self, interval: Duration) -> Self {
        self.change_interval = Some(interval);
        self
    }
}

impl Styled for Slider {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl IntoElement for Slider {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[derive(Clone, Debug)]
struct SliderState {
    is_dragging: bool,
    last_change_time: Instant,
    drag_value: f32,
    applied_revision: u64,
    correction_secs: f64,
    last_paint_time: Option<Instant>,
    last_display_fraction: f32,
    last_display_secs: f64,
    reset_animation: Option<(f32, Instant)>,
}

impl Default for SliderState {
    fn default() -> Self {
        Self {
            is_dragging: false,
            last_change_time: Instant::now(),
            drag_value: 0.0,
            applied_revision: 0,
            correction_secs: 0.0,
            last_paint_time: None,
            last_display_fraction: 0.0,
            last_display_secs: 0.0,
            reset_animation: None,
        }
    }
}

impl Element for Slider {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        self.id.clone()
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.refine(&self.style);
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        _: &mut App,
    ) -> Self::PrepaintState {
        // Extended hitbox so user has a comfortable 20px vertical hit target
        let hitbox_bounds = bounds.extend(Edges {
            top: px(8.0),
            bottom: px(8.0),
            left: px(2.0),
            right: px(2.0),
        });

        window.insert_hitbox(hitbox_bounds, HitboxBehavior::Normal)
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let theme = cx.global::<Theme>();
        let palette = &theme.palette;
        let is_hovered = hitbox.is_hovered(window);

        window.with_optional_element_state(
            id,
            |v: Option<Option<Rc<RefCell<SliderState>>>>, window| {
                let state_rc = v
                    .flatten()
                    .unwrap_or_else(|| Rc::new(RefCell::new(SliderState::default())));

                let mut state = state_rc.borrow_mut();
                let now = Instant::now();
                let dt = state
                    .last_paint_time
                    .map(|last| now.saturating_duration_since(last).as_secs_f64())
                    .unwrap_or(0.0)
                    .min(0.1);
                state.last_paint_time = Some(now);

                const CORRECTION_TAU: f64 = 0.100;
                const CORRECTION_EPSILON: f64 = 0.0005;

                let effective_value: f32 = if state.is_dragging {
                    state.reset_animation = None;
                    state.drag_value
                } else {
                    match self.value {
                        SliderValue::Static(val) => val,
                        SliderValue::Timeline(timeline) => {
                            if timeline.revision != state.applied_revision {
                                let new_raw = timeline.position_at(now);
                                // If the position jumped down from > 5% to < 1s, trigger smooth glide-to-zero
                                if state.last_display_fraction > 0.05 && new_raw <= 1.0 {
                                    state.reset_animation = Some((state.last_display_fraction, now));
                                    state.correction_secs = 0.0;
                                } else {
                                    let prev_display = if state.applied_revision > 0 {
                                        state.last_display_secs
                                    } else {
                                        timeline.position_at(now)
                                    };
                                    let diff = prev_display - new_raw;
                                    if diff.abs() <= 0.050 {
                                        state.correction_secs += diff;
                                    } else {
                                        state.correction_secs = 0.0;
                                    }
                                }
                                state.applied_revision = timeline.revision;
                            }

                            state.correction_secs *= (-dt / CORRECTION_TAU).exp();
                            if state.correction_secs.abs() < CORRECTION_EPSILON {
                                state.correction_secs = 0.0;
                            }

                            const RESET_DURATION: f32 = 0.280; // 280ms smooth glide on track transition
                            let val = if let Some((start_frac, start_time)) = state.reset_animation {
                                let t = (now.saturating_duration_since(start_time).as_secs_f32() / RESET_DURATION).clamp(0.0, 1.0);
                                let ease = 1.0 - (1.0 - t).powi(3);
                                let target = if timeline.duration > 0.0 {
                                    (timeline.position_at(now) / timeline.duration).clamp(0.0, 1.0) as f32
                                } else {
                                    0.0
                                };
                                let animated = start_frac * (1.0 - ease) + target * ease;
                                state.last_display_secs = (animated as f64) * timeline.duration;
                                if t >= 1.0 {
                                    state.reset_animation = None;
                                }
                                animated
                            } else {
                                let displayed_pos = timeline.position_at(now) + state.correction_secs;
                                state.last_display_secs = displayed_pos;
                                if timeline.duration > 0.0 {
                                    (displayed_pos / timeline.duration).clamp(0.0, 1.0) as f32
                                } else {
                                    0.0
                                }
                            };

                            state.last_display_fraction = val;
                            val
                        }
                    }
                };

                let should_animate = state.is_dragging
                    || state.reset_animation.is_some()
                    || match self.value {
                        SliderValue::Static(_) => false,
                        SliderValue::Timeline(t) => t.playing || state.correction_secs != 0.0,
                    };

                if should_animate {
                    window.request_animation_frame();
                }

                drop(state);

                let track_h = if is_hovered { px(6.0) } else { px(4.0) };
                let track_y = bounds.origin.y + (bounds.size.height - track_h) / 2.0;

                let track_bounds = Bounds {
                    origin: Point { x: bounds.origin.x, y: track_y },
                    size: Size { width: bounds.size.width, height: track_h },
                };

                let mut inner_bounds = track_bounds;
                inner_bounds.size.width = track_bounds.size.width * effective_value;

                let corners = Corners::all(track_h / 2.0);

                window.set_cursor_style(CursorStyle::PointingHand, hitbox);

                // 1. Draw background track (subtle translucent surface highlight)
                let bg_color = rgba(0xFFFFFF28);
                window.paint_quad(quad(
                    track_bounds,
                    corners,
                    bg_color,
                    Edges::all(px(0.0)),
                    rgb(0x000000),
                    BorderStyle::Solid,
                ));

                let active_fill = self.active_color.unwrap_or(palette.primary);

                // 2. Draw filled progress track (active/primary color)
                if effective_value > 0.001 {
                    window.paint_quad(quad(
                        inner_bounds,
                        corners,
                        active_fill,
                        Edges::all(px(0.0)),
                        rgb(0x000000),
                        BorderStyle::Solid,
                    ));
                }

                // 3. Draw thumb knob (white circle with subtle shadow when hovered or active)
                if is_hovered || effective_value > 0.0 {
                    let thumb_radius = if is_hovered { px(6.0) } else { px(4.0) };
                    let thumb_size = thumb_radius * 2.0;
                    let thumb_x = (inner_bounds.origin.x + inner_bounds.size.width - thumb_radius)
                        .clamp(bounds.origin.x, bounds.origin.x + bounds.size.width - thumb_size);
                    let thumb_y = bounds.origin.y + (bounds.size.height - thumb_size) / 2.0;

                    let thumb_bounds = Bounds {
                        origin: Point { x: thumb_x, y: thumb_y },
                        size: Size { width: thumb_size, height: thumb_size },
                    };

                    window.paint_quad(quad(
                        thumb_bounds,
                        Corners::all(thumb_radius),
                        rgba(0xFFFFFFFF),
                        Edges::all(px(1.0)),
                        active_fill,
                        BorderStyle::Solid,
                    ));
                }

                // 4. Mouse event handling for click & drag scrubbing
                if let Some(func) = self.on_change.as_ref() {
                    let change_interval = self.change_interval;
                    let min_interval = change_interval.unwrap_or(Duration::from_millis(1));
                    let func = func.clone();
                    let func_move = func.clone();
                    let func_release = func.clone();

                    let state_1 = state_rc.clone();
                    let hitbox = hitbox.clone();

                    window.on_mouse_event(move |ev: &MouseDownEvent, phase, window, cx| {
                        if phase != DispatchPhase::Bubble || !hitbox.is_hovered(window) {
                            return;
                        }

                        window.prevent_default();
                        cx.stop_propagation();

                        let relative = ev.position - bounds.origin;
                        let relative_x: f32 = relative.x.into();
                        let width: f32 = bounds.size.width.into();
                        let value = if width > 0.0 { (relative_x / width).clamp(0.0, 1.0) } else { 0.0 };

                        (func.borrow_mut())(value, window, cx);
                        let mut st = state_1.borrow_mut();
                        st.is_dragging = true;
                        st.last_change_time = Instant::now();
                        st.drag_value = value;
                        window.refresh();
                    });

                    let state_2 = state_rc.clone();

                    window.on_mouse_event(move |ev: &MouseMoveEvent, phase, window, cx| {
                        if phase != DispatchPhase::Bubble {
                            return;
                        }

                        let mut st = state_2.borrow_mut();
                        if !st.is_dragging {
                            return;
                        }

                        let relative = ev.position - bounds.origin;
                        let relative_x: f32 = relative.x.into();
                        let width: f32 = bounds.size.width.into();
                        let value = if width > 0.0 { (relative_x / width).clamp(0.0, 1.0) } else { 0.0 };

                        st.drag_value = value;

                        let now = Instant::now();
                        if now.duration_since(st.last_change_time) >= min_interval {
                            (func_move.borrow_mut())(value, window, cx);
                            st.last_change_time = now;
                        }
                        window.refresh();
                    });

                    let state_3 = state_rc.clone();
                    let flush_on_release = change_interval.is_some();

                    window.on_mouse_event(move |_ev: &MouseUpEvent, phase, window, cx| {
                        if phase != DispatchPhase::Bubble {
                            return;
                        }

                        let mut st = state_3.borrow_mut();
                        if st.is_dragging && flush_on_release {
                            (func_release.borrow_mut())(st.drag_value, window, cx);
                        }
                        st.is_dragging = false;
                        window.refresh();
                    });
                }

                ((), if id.is_some() { Some(state_rc) } else { None })
            }
        );
    }
}

