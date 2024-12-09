use floem::event::EventListener;
use floem::reactive::{create_rw_signal, RwSignal, SignalGet, SignalUpdate};
use floem::taffy::Position;
use floem::{
    self,
    context::{ComputeLayoutCx, EventCx, LayoutCx, PaintCx, StyleCx, UpdateCx},
    event::{Event, EventPropagation},
    kurbo::{self, Line, Point, Rect},
    peniko::{Brush, Color},
    style::Style,
    taffy::{Display, Layout, NodeId, TaffyTree},
    text::{Attrs, AttrsList, TextLayout},
    unit::UnitExt,
    views::{container, label, stack, Decorators},
    AppState, View, ViewId,
};
use floem_renderer::Renderer;

use std::time::Duration;

use stunts_engine::animations::{
    AnimationData, AnimationProperty, EasingType, KeyframeValue, Sequence, UIKeyframe,
};

/// State for the timeline component
#[derive(Debug, Clone)]
pub struct TimelineState {
    pub current_time: Duration,
    pub zoom_level: f64,
    pub scroll_offset: f64,
    pub dragging: Option<DragOperation>,
    pub hovered_keyframe: Option<(String, Duration)>,
    pub property_expansions: im::HashMap<String, bool>,
    pub selected_keyframes: RwSignal<Vec<UIKeyframe>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyframeId {
    pub property_path: String,
    pub time: Duration,
}

#[derive(Debug, Clone)]
pub enum DragState {
    Keyframe(KeyframeId),
    TimeSlider(f64),
    Scrolling { start_x: f64, start_offset: f64 },
}

/// Configuration for the timeline
#[derive(Debug, Clone)]
pub struct TimelineConfig {
    pub width: f64,
    pub height: f64,
    pub header_height: f64,
    pub property_width: f64,
    pub row_height: f64,
    // Add offset parameters
    pub offset_x: f64,
    pub offset_y: f64,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 400.0,
            header_height: 30.0,
            property_width: 150.0,
            row_height: 24.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

pub struct TimelineGridView {
    id: ViewId,
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: RwSignal<Option<AnimationData>>,
    style: Style,
}

impl TimelineGridView {
    pub fn new(
        state: TimelineState,
        config: TimelineConfig,
        animation_data: RwSignal<Option<AnimationData>>,
    ) -> Self {
        Self {
            id: ViewId::new(),
            state: create_rw_signal(state),
            config: config.clone(),
            animation_data,
            // style: Style::default(),
            style: Style::new()
                .margin_left(300.0)
                .margin_top(100.0)
                .position(Position::Absolute)
                .width(config.clone().width)
                .height(config.clone().height),
        }
        .on_event(EventListener::PointerMove, move |e| {
            println!("event recieved");
            EventPropagation::Continue
        })
    }

    pub fn draw_property_label(&self, cx: &mut PaintCx, property: &AnimationProperty, y: f64) {
        let mut text_layout = TextLayout::new();
        let mut attrs_list = AttrsList::new(Attrs::new().color(Color::BLACK).font_size(12.0));

        // Indent based on hierarchy level
        let indent = property.property_path.matches('/').count() as f64 * 15.0;

        // Add expand/collapse indicator if has children
        let prefix = if !property.children.is_empty() {
            if self
                .state
                .get()
                .property_expansions
                .get(&property.property_path)
                .copied()
                .unwrap_or(false)
            {
                "▼ "
            } else {
                "▶ "
            }
        } else {
            "  "
        };

        text_layout.set_text(&format!("{}{}", prefix, property.name), attrs_list);
        cx.draw_text(
            &text_layout,
            Point::new(10.0 + indent, y + self.config.row_height / 2.0 - 6.0),
        );
    }

    pub fn draw_time_grid(&self, cx: &mut PaintCx) {
        let duration = self
            .animation_data
            .get()
            .expect("Couldn't get animation data")
            .duration
            .as_secs_f64();
        let step = (0.5 / self.state.get().zoom_level).max(0.1);

        for time in (0..(duration / step) as i32).map(|i| i as f64 * step) {
            let x = self.config.offset_x
                + time_to_x(
                    self.state,
                    self.config.clone(),
                    Duration::from_secs_f64(time),
                );

            // Vertical grid line
            cx.stroke(
                &Line::new(
                    Point::new(x, self.config.offset_y),
                    Point::new(x, self.config.offset_y + self.config.height),
                ),
                &Color::GRAY,
                1.0,
            );

            // Time label
            let mut attrs_list = AttrsList::new(Attrs::new().color(Color::BLACK));
            let mut text_layout = TextLayout::new();
            text_layout.set_text(&format!("{:.1}s", time), attrs_list);

            cx.draw_text(&text_layout, Point::new(x, self.config.offset_y));
        }
    }

    pub fn draw_keyframe(&self, cx: &mut PaintCx, center: Point, selected: bool) {
        let size = 6.0;
        let color = if selected {
            Color::rgb8(66, 135, 245)
        } else {
            Color::rgb8(245, 166, 35)
        };

        // Draw diamond shape
        let path = kurbo::BezPath::from_vec(vec![
            kurbo::PathEl::MoveTo(Point::new(
                center.x + self.config.offset_x,
                center.y + self.config.offset_y - size,
            )),
            kurbo::PathEl::LineTo(Point::new(
                center.x + self.config.offset_x + size,
                center.y + self.config.offset_y,
            )),
            kurbo::PathEl::LineTo(Point::new(
                center.x + self.config.offset_x,
                center.y + self.config.offset_y + size,
            )),
            kurbo::PathEl::LineTo(Point::new(
                center.x + self.config.offset_x - size,
                center.y + self.config.offset_y,
            )),
            kurbo::PathEl::ClosePath,
        ]);

        cx.fill(&path, color, 1.0);
    }

    pub fn draw_keyframes(&self, cx: &mut PaintCx) {
        // Track visible vertical space used
        let mut current_y = self.config.header_height;

        // Draw keyframes for each property
        for property in &self
            .animation_data
            .get()
            .expect("Couldn't get animation data")
            .properties
        {
            if let Some(y) = self.draw_property_keyframes(cx, property, current_y) {
                current_y = y;
            }
        }
    }

    pub fn draw_property_keyframes(
        &self,
        cx: &mut PaintCx,
        property: &AnimationProperty,
        start_y: f64,
    ) -> Option<f64> {
        let mut current_y = start_y;

        // Check if property is visible (based on scroll position and view height)
        if current_y > self.config.height {
            return None;
        }

        // Draw property's own keyframes
        let selected_keyframes = self.state.get().selected_keyframes.get();

        for keyframe in &property.keyframes {
            let x = time_to_x(self.state, self.config.clone(), keyframe.time);

            // Skip if outside visible area
            if x < -10.0 || x > self.config.width + 10.0 {
                continue;
            }

            let selected = selected_keyframes.contains(&keyframe);

            // Draw the keyframe marker
            self.draw_keyframe(
                cx,
                Point::new(x, (current_y + self.config.row_height / 2.0)),
                selected,
            );

            // draw connecting lines between keyframes
            // (only if there's a previous keyframe in our visible range)
            if let Some(prev_keyframe) = property
                .keyframes
                .iter()
                .filter(|k| k.time < keyframe.time)
                .last()
            {
                let prev_x = time_to_x(self.state, self.config.clone(), prev_keyframe.time);
                if prev_x >= -10.0 {
                    cx.stroke(
                        &Line::new(
                            Point::new(
                                self.config.offset_x + prev_x,
                                self.config.offset_y + (current_y + self.config.row_height / 2.0),
                            ),
                            Point::new(
                                self.config.offset_x + x,
                                self.config.offset_y + (current_y + self.config.row_height / 2.0),
                            ),
                        ),
                        &Color::DARK_GRAY,
                        1.0,
                    );
                }
            }
        }

        self.draw_property_label(cx, property, current_y);

        current_y += self.config.row_height;

        // If the property is expanded, draw child properties
        if self
            .state
            .get()
            .property_expansions
            .get(&property.property_path)
            .copied()
            .unwrap_or(false)
        {
            for child in &property.children {
                self.draw_property_label(cx, child, current_y);
                if let Some(new_y) = self.draw_property_keyframes(cx, child, current_y) {
                    current_y = new_y;
                } else {
                    // Child property was outside visible area, we can stop here
                    break;
                }
            }
        }

        Some(current_y)
    }

    /// Calculate the Y position for a given property path
    pub fn get_property_y_position(&self, property_path: &str) -> f64 {
        let mut y_position = self.config.header_height;

        // Helper function to search through properties recursively
        fn find_property_y(
            properties: &[AnimationProperty],
            target_path: &str,
            current_y: &mut f64,
            row_height: f64,
            property_expansions: &im::HashMap<String, bool>,
        ) -> Option<f64> {
            for property in properties {
                // Check if this is the property we're looking for
                if property.property_path == target_path {
                    return Some(*current_y);
                }

                // Move to next row
                *current_y += row_height;

                // If this property is expanded and has children, search them
                if property_expansions
                    .get(&property.property_path)
                    .copied()
                    .unwrap_or(false)
                {
                    if let Some(y) = find_property_y(
                        &property.children,
                        target_path,
                        current_y,
                        row_height,
                        property_expansions,
                    ) {
                        return Some(y);
                    }
                }
            }
            None
        }

        // Search through properties and return the found Y position or a default
        let y = find_property_y(
            &self
                .animation_data
                .get()
                .expect("Couldn't get animation data")
                .properties,
            property_path,
            &mut y_position,
            self.config.row_height,
            &self.state.get().property_expansions,
        );

        // Add the offset_y to the final position
        self.config.offset_y + y.unwrap_or(y_position) + (self.config.row_height / 2.0)
    }

    pub fn request_repaint(&self) {
        self.id.request_paint();
    }
}

fn hit_test_keyframe(
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: AnimationData,
    point: Point,
) -> Option<(String, UIKeyframe)> {
    let mut current_y = config.header_height;
    let row_height = config.row_height.clone();
    let hit_radius = 6.0;

    for property in &animation_data.properties {
        // Check if point is within the property's vertical bounds
        let property_height = row_height;
        let y_center = current_y + property_height / 2.0;

        // Check keyframes
        for keyframe in &property.keyframes {
            let x = time_to_x(state, config.clone(), keyframe.time);
            let keyframe_point = Point::new(x, y_center);

            if point.distance(keyframe_point) <= hit_radius {
                return Some((property.property_path.clone(), keyframe.clone()));
            }
        }

        if property.children.len() > 0 {
            current_y += row_height; // for header expansion row

            for child in &property.children {
                let property_height = row_height;
                let y_center = current_y + property_height / 2.0;

                for keyframe in &child.keyframes {
                    let x = time_to_x(state, config.clone(), keyframe.time);
                    let keyframe_point = Point::new(x, y_center);

                    if point.distance(keyframe_point) <= hit_radius {
                        return Some((child.property_path.clone(), keyframe.clone()));
                    }
                }

                current_y += row_height;
            }
        } else {
            current_y += row_height;
        }
    }
    None
}

fn time_to_x(state: RwSignal<TimelineState>, config: TimelineConfig, time: Duration) -> f64 {
    let time_secs = time.as_secs_f64();
    let base_spacing = config.property_width; // pixels per second at zoom level 1.0
    (time_secs * base_spacing * state.get().zoom_level) - state.get().scroll_offset
}

fn x_to_time(state: RwSignal<TimelineState>, config: TimelineConfig, x: f64) -> Duration {
    let base_spacing = config.property_width;
    let time_secs = (x + state.get().scroll_offset) / (base_spacing * state.get().zoom_level);
    Duration::from_secs_f64(time_secs.max(0.0))
}

#[derive(Clone, Debug)]
pub enum DragOperation {
    Playhead(f64),
    Keyframe {
        property_path: String,
        original_time: Duration,
        start_x: f64,
    },
    None,
}

impl View for TimelineGridView {
    fn id(&self) -> ViewId {
        self.id
    }

    fn view_style(&self) -> Option<Style> {
        Some(self.style.clone())
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "TimelineGridView".into()
    }

    fn paint(&mut self, cx: &mut PaintCx) {
        // Draw background
        let background_rect = kurbo::Rect::new(
            self.config.offset_x,
            self.config.offset_y,
            self.config.offset_x + self.config.width,
            self.config.offset_y + self.config.height,
        );
        cx.fill(&background_rect, Color::WHITE, 1.0);

        // Draw grid
        self.draw_time_grid(cx);

        // Draw keyframes
        self.draw_keyframes(cx);

        // Draw playhead with offset
        let playhead_x = self.config.offset_x
            + time_to_x(
                self.state,
                self.config.clone(),
                self.state.get().current_time,
            );
        cx.stroke(
            &Line::new(
                Point::new(playhead_x, self.config.offset_y),
                Point::new(playhead_x, self.config.offset_y + self.config.height),
            ),
            &Color::rgb8(255, 0, 0),
            2.0,
        );

        // Add hover effects
        if let Some((property_path, time)) = &self.state.get().hovered_keyframe {
            let y = self.get_property_y_position(property_path);
            let x = time_to_x(self.state, self.config.clone(), *time);

            // Draw hover highlight
            let hover_size = 8.0;
            cx.stroke(
                &kurbo::Circle::new(Point::new(x, y), hover_size),
                &Color::rgba8(255, 165, 0, 128), // Semi-transparent orange
                2.0,
            );
        }
    }

    // Make sure compute_layout returns proper bounds
    fn compute_layout(&mut self, _cx: &mut ComputeLayoutCx) -> Option<Rect> {
        Some(Rect::new(
            self.config.offset_x,
            self.config.offset_y,
            self.config.offset_x + self.config.width,
            self.config.offset_y + self.config.height,
        ))
    }

    // Implement other required View trait methods with default behavior
    fn update(&mut self, _cx: &mut UpdateCx, _state: Box<dyn std::any::Any>) {
        println!("update");
        self.id.request_layout();
    }

    fn layout(&mut self, _cx: &mut LayoutCx) -> NodeId {
        let node = self.id().new_taffy_node();
        node
    }

    fn scroll_to(&mut self, _cx: &mut AppState, _target: ViewId, _rect: Option<Rect>) -> bool {
        false
    }
}

#[derive(Clone)]
struct TimelineHandle {
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: RwSignal<Option<AnimationData>>,
    view_id: ViewId,
}

pub fn create_timeline(
    state: TimelineState,
    config: TimelineConfig,
    animation_data: RwSignal<Option<AnimationData>>,
) -> impl View {
    let test = TimelineGridView::new(state, config, animation_data);

    let view_id = test.id;

    // Create a lightweight handle for events
    let handle = TimelineHandle {
        state: test.state.clone(),
        config: test.config.clone(),
        animation_data: test.animation_data,
        view_id,
    };

    let handle_move = handle.clone();
    let handle_up = handle.clone();
    let handle_wheel = handle.clone();

    container((test))
        .style(|s| {
            s.width(1200.0)
                .height(300.0)
                .margin_top(50.0)
                .margin_left(50.0)
                .background(Color::LIGHT_CORAL)
        })
        .on_event(EventListener::PointerDown, move |e| {
            println!("PointerDown");
            let scale_factor = 1.25; // hardcode test
            let position = Point::new(
                e.point().expect("Couldn't get point").x as f64,
                e.point().expect("Couldn't get point").y as f64,
            );

            handle_mouse_down(
                handle.state,
                handle.config.clone(),
                handle.animation_data,
                position,
            );
            handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerMove, move |e| {
            // println!("PointerMove");
            let scale_factor = 1.25;
            let position = Point::new(
                e.point().expect("Couldn't get point").x as f64,
                e.point().expect("Couldn't get point").y as f64,
            );

            handle_mouse_move(
                handle_move.state,
                handle_move.config.clone(),
                handle_move.animation_data,
                position,
            );
            handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerUp, move |e| {
            println!("PointerUp");
            let scale_factor = 1.25;
            let position = Point::new(
                e.point().expect("Couldn't get point").x as f64,
                e.point().expect("Couldn't get point").y as f64,
            );
            handle_mouse_up(handle_up.state, position);
            handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerWheel, move |e| {
            println!("PointerWheel");
            // Add wheel handling
            handle_scroll(handle_wheel.state, 0.1);
            handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
}

fn handle_mouse_down(
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: RwSignal<Option<AnimationData>>,
    pos: Point,
) -> EventPropagation {
    println!("handle_mouse_down");
    let state_data = state.get();
    // Check if clicking on a keyframe
    if let Some((property_path, ui_keyframe)) = hit_test_keyframe(
        state,
        config.clone(),
        animation_data.get().expect("Couldn't get animation data"),
        pos,
    ) {
        println!("start move keyframe {:?}", ui_keyframe.time);
        state.update(|s| {
            s.dragging = Some(DragOperation::Keyframe {
                property_path,
                original_time: ui_keyframe.time,
                start_x: pos.x,
            })
        });
        let mut new_selection = Vec::new();
        new_selection.push(ui_keyframe);
        state.get().selected_keyframes.set(new_selection);
        return EventPropagation::Stop;
    }

    // Check if clicking on timeline (for playhead)
    if pos.y <= config.header_height {
        let time = x_to_time(state, config, pos.x);
        println!("start move playhead {:?}", time);
        state.update(|s| s.current_time = time);
        state.update(|s| s.dragging = Some(DragOperation::Playhead(pos.x)));
        return EventPropagation::Stop;
    }

    EventPropagation::Continue
}

fn handle_mouse_move(
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: RwSignal<Option<AnimationData>>,
    pos: Point,
) -> EventPropagation {
    let state_data = state.get();
    if (state_data.dragging.is_some()) {
        let dragging = state_data.dragging.as_ref().expect("Couldn't get dragging");
        match dragging {
            DragOperation::Playhead(_) => {
                println!("moving playhead");
                let value = x_to_time(state, config.clone(), pos.x);
                state.update(|s| s.current_time = value);
                return EventPropagation::Stop;
            }
            DragOperation::Keyframe {
                property_path,
                original_time,
                start_x,
            } => {
                let delta_x = pos.x - start_x;
                let new_time = x_to_time(
                    state,
                    config.clone(),
                    time_to_x(state, config.clone(), *original_time) + delta_x,
                );

                println!("moving keyframe {:?}", new_time);

                // TODO: update_keyframe_time
                // self.update_keyframe_time(property_path, *original_time, new_time);

                return EventPropagation::Stop;
            }
            _ => {
                // Update hover state
                if let Some((property_path, ui_keyframe)) = hit_test_keyframe(
                    state,
                    config.clone(),
                    animation_data.get().expect("Couldn't get animation data"),
                    pos,
                ) {
                    state.update(|s| s.hovered_keyframe = Some((property_path, ui_keyframe.time)));
                } else {
                    state.update(|s| s.hovered_keyframe = None);
                }
                return EventPropagation::Continue;
            }
        }
    } else {
        return EventPropagation::Continue;
    }
}

fn handle_mouse_up(state: RwSignal<TimelineState>, _pos: Point) -> EventPropagation {
    state.update(|s| s.dragging = None);
    EventPropagation::Stop
}

fn handle_scroll(state: RwSignal<TimelineState>, delta: f64) -> EventPropagation {
    let state_data = state.get();

    println!("handle_scroll");
    if delta != 0.0 {
        // Adjust zoom level based on scroll
        let old_zoom = state_data.zoom_level;
        state.update(|s| {
            s.zoom_level = (state_data.zoom_level * (1.0 + delta * 0.001))
                .max(0.1)
                .min(10.0)
        });

        // Adjust scroll offset to keep the timeline position under the cursor
        // May want to use the cursor position for more precise zooming
        let zoom_ratio = state_data.zoom_level / old_zoom;

        state.update(|s| {
            s.scroll_offset *= zoom_ratio;
        });

        EventPropagation::Stop
    } else {
        EventPropagation::Continue
    }
}
