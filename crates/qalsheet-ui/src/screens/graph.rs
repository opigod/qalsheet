//! Graph screen: multiple functions, pan/zoom, trace, roots, intersections, table.

use eframe::egui::{
    self, Align2, Color32, Id, Margin, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Ui,
    Vec2,
};
use qalsheet_core::expr::{self, AngleMode, Expression};
use qalsheet_core::format;
use qalsheet_core::graph;

use crate::app::Feedback;
use crate::icons::Icon;
use crate::screens::common;
use crate::theme::{self, GRAPH_COLORS, Metrics, Palette, Weight};
use crate::widgets;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisTab {
    Roots,
    Intersections,
    Table,
}

impl AnalysisTab {
    fn label(self) -> &'static str {
        match self {
            Self::Roots => "Roots",
            Self::Intersections => "Intersect",
            Self::Table => "Table",
        }
    }
}

pub struct GraphFunction {
    pub expression: String,
    pub color: usize,
    pub visible: bool,
    pub compiled: Option<Expression>,
    pub error: Option<String>,
}

impl GraphFunction {
    fn new(expression: &str, color: usize) -> Self {
        let mut function = Self {
            expression: expression.to_string(),
            color,
            visible: true,
            compiled: None,
            error: None,
        };
        function.recompile();
        function
    }

    pub fn recompile(&mut self) {
        match expr::compile(&self.expression) {
            Ok(compiled) => {
                self.compiled = Some(compiled);
                self.error = None;
            }
            Err(error) => {
                self.compiled = None;
                self.error = Some(error.message);
            }
        }
    }

    pub fn color(&self) -> Color32 {
        GRAPH_COLORS[self.color % GRAPH_COLORS.len()]
    }
}

pub struct GraphState {
    pub functions: Vec<GraphFunction>,
    pub selected: usize,
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub trace_x: Option<f64>,
    pub analysis: AnalysisTab,
    pub roots: Vec<f64>,
    pub intersections: Vec<(f64, f64)>,
    pub table: Vec<(f64, Option<f64>)>,
    pub analysis_dirty: bool,
    pub show_functions: bool,
    pub show_analysis: bool,
    pub angle: AngleMode,
}

impl GraphState {
    pub fn new() -> Self {
        let mut state = Self {
            functions: vec![
                GraphFunction::new("sin(x)", 0),
                GraphFunction::new("0.25x^2 - 3", 1),
            ],
            selected: 0,
            x_min: -10.0,
            x_max: 10.0,
            y_min: -6.0,
            y_max: 6.0,
            trace_x: None,
            analysis: AnalysisTab::Roots,
            roots: Vec::new(),
            intersections: Vec::new(),
            table: Vec::new(),
            analysis_dirty: true,
            show_functions: false,
            show_analysis: false,
            angle: AngleMode::Radians,
        };
        state.fit_y();
        state
    }

    pub fn selected_function(&self) -> Option<&GraphFunction> {
        self.functions.get(self.selected)
    }

    pub fn add_function(&mut self) {
        if self.functions.len() >= 6 {
            return;
        }
        let color = (0..6)
            .find(|index| {
                self.functions
                    .iter()
                    .all(|function| function.color != *index)
            })
            .unwrap_or(self.functions.len());
        self.functions.push(GraphFunction::new("x", color));
        self.selected = self.functions.len() - 1;
        self.analysis_dirty = true;
    }

    pub fn remove_function(&mut self, index: usize) {
        if self.functions.len() <= 1 || index >= self.functions.len() {
            return;
        }
        self.functions.remove(index);
        if self.selected >= self.functions.len() {
            self.selected = self.functions.len() - 1;
        }
        self.analysis_dirty = true;
    }

    pub fn select(&mut self, index: usize) {
        if index < self.functions.len() {
            self.selected = index;
            self.analysis_dirty = true;
        }
    }

    pub fn fit_y(&mut self) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for function in &self.functions {
            if !function.visible {
                continue;
            }
            let Some(compiled) = &function.compiled else {
                continue;
            };
            for sample in 0..=360 {
                let x = self.x_min + (self.x_max - self.x_min) * sample as f64 / 360.0;
                if let Ok(y) = compiled.eval(self.angle, &[("x", x)]) {
                    if y.is_finite() && y.abs() < 10_000.0 {
                        min = min.min(y);
                        max = max.max(y);
                    }
                }
            }
        }
        if !min.is_finite() || !max.is_finite() {
            self.y_min = -6.0;
            self.y_max = 6.0;
            return;
        }
        if (max - min).abs() < 1e-6 {
            self.y_min = min - 1.0;
            self.y_max = max + 1.0;
            return;
        }
        let margin = (max - min) * 0.12;
        self.y_min = min - margin;
        self.y_max = max + margin;
    }

    pub fn reset(&mut self) {
        self.x_min = -10.0;
        self.x_max = 10.0;
        self.trace_x = None;
        self.analysis_dirty = true;
        self.fit_y();
    }

    pub fn zoom(&mut self, factor: f64, anchor_x: f64, anchor_y: f64) {
        let factor = factor.clamp(0.25, 4.0);
        let nx = anchor_x + (self.x_min - anchor_x) * factor;
        let xx = anchor_x + (self.x_max - anchor_x) * factor;
        let ny = anchor_y + (self.y_min - anchor_y) * factor;
        let yy = anchor_y + (self.y_max - anchor_y) * factor;
        if (xx - nx).abs() > 1e-7 && (xx - nx).abs() < 1e12 {
            self.x_min = nx;
            self.x_max = xx;
            self.y_min = ny;
            self.y_max = yy;
        }
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        let ux = (self.x_max - self.x_min) * dx;
        let uy = (self.y_max - self.y_min) * dy;
        self.x_min -= ux;
        self.x_max -= ux;
        self.y_min -= uy;
        self.y_max -= uy;
    }

    pub fn run_analysis(&mut self) {
        self.analysis_dirty = false;
        self.roots.clear();
        self.intersections.clear();
        self.table.clear();
        let Some(selected) = self.functions.get(self.selected) else {
            return;
        };
        let Some(compiled) = selected.compiled.clone() else {
            return;
        };
        match self.analysis {
            AnalysisTab::Roots => {
                self.roots = graph::find_roots(&compiled, self.angle, self.x_min, self.x_max, 1600);
            }
            AnalysisTab::Intersections => {
                let other = self
                    .functions
                    .iter()
                    .enumerate()
                    .find(|(index, function)| *index != self.selected && function.visible)
                    .and_then(|(_, function)| function.compiled.clone());
                if let Some(other) = other {
                    self.intersections = graph::find_intersections(
                        &compiled, &other, self.angle, self.x_min, self.x_max, 1600,
                    );
                }
            }
            AnalysisTab::Table => {
                let step = graph::nice_step(self.x_max - self.x_min, 10.0);
                self.table = graph::table_values(&compiled, self.angle, self.x_min, step, 14);
            }
        }
    }

    fn ensure_analysis(&mut self) {
        if self.analysis_dirty {
            self.run_analysis();
        }
    }
}

pub fn screen(ui: &mut Ui, state: &mut GraphState, metrics: &Metrics, feedback: &mut Feedback) {
    if metrics.watch {
        return;
    }
    let area = ui.max_rect();
    state.ensure_analysis();
    if !metrics.compact && area.width() >= 820.0 {
        wide_screen(ui, area, state, metrics, feedback);
    } else {
        compact_screen(ui, area, state, metrics, feedback);
    }
    if state.show_functions {
        functions_modal(ui, state, metrics, feedback);
    }
    if state.show_analysis {
        analysis_modal(ui, state, metrics, feedback);
    }
}

fn wide_screen(
    ui: &mut Ui,
    area: Rect,
    state: &mut GraphState,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let panel_width = metrics.fs(320.0).min(area.width() * 0.34);
    let gap = metrics.fs(14.0);
    let graph = Rect::from_min_max(
        area.min,
        Pos2::new(area.right() - panel_width - gap, area.bottom()),
    );
    let panel = Rect::from_min_max(Pos2::new(area.right() - panel_width, area.top()), area.max);
    graph_card(ui, graph, state, metrics, feedback);

    let functions_height = metrics.fs(292.0).min(panel.height() * 0.48);
    let functions = Rect::from_min_size(panel.min, Vec2::new(panel.width(), functions_height));
    functions_panel(ui, functions, state, metrics);
    let analysis = Rect::from_min_max(Pos2::new(panel.left(), functions.bottom() + gap), panel.max);
    analysis_panel(ui, analysis, state, metrics);
}

fn compact_screen(
    ui: &mut Ui,
    area: Rect,
    state: &mut GraphState,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let top_height = metrics.fs(104.0);
    let top = Rect::from_min_size(area.min, Vec2::new(area.width(), top_height));
    compact_functions(ui, top, state, metrics);
    let hint_height = metrics.fs(36.0);
    let graph = Rect::from_min_max(
        Pos2::new(area.left(), top.bottom() + metrics.fs(8.0)),
        Pos2::new(area.right(), area.bottom() - hint_height - metrics.fs(6.0)),
    );
    graph_card(ui, graph, state, metrics, feedback);
    let toolbar = Rect::from_min_size(
        Pos2::new(area.left(), area.bottom() - hint_height),
        Vec2::new(area.width(), hint_height),
    );
    compact_analysis_bar(ui, toolbar, state, metrics);
}

fn compact_functions(ui: &mut Ui, rect: Rect, state: &mut GraphState, metrics: &Metrics) {
    theme::card(
        ui.painter(),
        rect,
        metrics.fs(14.0),
        Palette::CARD,
        Color32::from_white_alpha(24),
    );
    let pad = metrics.fs(10.0);
    let header = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(3.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(22.0)),
    );
    theme::text(
        ui.painter(),
        header.left_center(),
        Align2::LEFT_CENTER,
        "FUNCTIONS",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let add = Rect::from_min_size(
        Pos2::new(
            header.right() - metrics.fs(52.0),
            header.center().y - metrics.fs(10.0),
        ),
        Vec2::new(metrics.fs(52.0), metrics.fs(20.0)),
    );
    if widgets::pill(ui, add, "+ Add", false, true, metrics).clicked() {
        state.add_function();
    }
    let chips = Rect::from_min_size(
        Pos2::new(rect.left() + pad, header.bottom() + metrics.fs(2.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(26.0)),
    );
    let mut x = chips.left();
    for index in 0..state.functions.len() {
        let function = &state.functions[index];
        let label = format!("y{}", index + 1);
        let width = metrics.fs(42.0);
        let chip = Rect::from_min_size(Pos2::new(x, chips.top()), Vec2::new(width, chips.height()));
        let id = ui.scope_id().with(("fn-chip", index));
        let response = ui.interact(chip, id, Sense::click());
        let active = index == state.selected;
        ui.painter().rect_filled(
            chip,
            chip.height() * 0.5,
            if active {
                function.color().gamma_multiply(0.22)
            } else {
                Palette::CARD_LIFT
            },
        );
        ui.painter().circle_filled(
            Pos2::new(chip.left() + metrics.fs(10.0), chip.center().y),
            metrics.fs(3.0),
            function.color(),
        );
        theme::text(
            ui.painter(),
            Pos2::new(chip.left() + metrics.fs(17.0), chip.center().y),
            Align2::LEFT_CENTER,
            &label,
            metrics.fs(9.0),
            Weight::Bold,
            if active {
                Palette::TEXT
            } else {
                Palette::MUTED
            },
        );
        if response.clicked() {
            state.select(index);
        }
        x += width + metrics.fs(5.0);
        if x + width > chips.right() {
            break;
        }
    }
    let field_rect = Rect::from_min_size(
        Pos2::new(rect.left() + pad, chips.bottom() + metrics.fs(2.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(30.0)),
    );
    let selected = state.selected.min(state.functions.len() - 1);
    let changed = widgets::field(
        ui,
        field_rect,
        &mut state.functions[selected].expression,
        "f(x) = sin(x), x^2/4 - 3",
        metrics,
    )
    .changed();
    if changed {
        state.functions[selected].recompile();
        state.analysis_dirty = true;
        state.fit_y();
    }
    if let Some(error) = &state.functions[selected].error {
        theme::text(
            ui.painter(),
            Pos2::new(
                field_rect.left() + metrics.fs(2.0),
                field_rect.bottom() + metrics.fs(7.0),
            ),
            Align2::LEFT_CENTER,
            error,
            metrics.fs(8.0),
            Weight::Medium,
            Palette::ERROR,
        );
    }
}

fn graph_card(
    ui: &mut Ui,
    rect: Rect,
    state: &mut GraphState,
    metrics: &Metrics,
    _feedback: &mut Feedback,
) {
    theme::card(
        ui.painter(),
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(26),
    );
    let pad = metrics.fs(10.0);
    let toolbar = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(6.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(26.0)),
    );
    theme::text(
        ui.painter(),
        toolbar.left_center(),
        Align2::LEFT_CENTER,
        "GRAPH",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let readout = if let Some(x) = state.trace_x {
        if let Some(function) = state.selected_function() {
            if let Some(compiled) = &function.compiled {
                if let Ok(y) = compiled.eval(state.angle, &[("x", x)]) {
                    format!("x {}   y {}", format::axis_label(x), format::axis_label(y))
                } else {
                    format!("x {}", format::axis_label(x))
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    if !readout.is_empty() {
        theme::text(
            ui.painter(),
            toolbar.center(),
            Align2::CENTER_CENTER,
            &readout,
            metrics.fs(9.5),
            Weight::SemiBold,
            Palette::TEAL_SOFT,
        );
    }
    let button = metrics.fs(24.0);
    let mut x = toolbar.right();
    let reset = Rect::from_min_size(
        Pos2::new(x - button, toolbar.center().y - button / 2.0),
        Vec2::splat(button),
    );
    if widgets::icon_button(ui, reset, Icon::Fit, "Reset view", metrics).clicked() {
        state.reset();
    }
    x -= button + metrics.fs(5.0);
    let zoom_out = Rect::from_min_size(
        Pos2::new(x - button, toolbar.center().y - button / 2.0),
        Vec2::splat(button),
    );
    if widgets::icon_button(ui, zoom_out, Icon::ZoomOut, "Zoom out", metrics).clicked() {
        let cx = (state.x_min + state.x_max) / 2.0;
        let cy = (state.y_min + state.y_max) / 2.0;
        state.zoom(1.25, cx, cy);
        state.analysis_dirty = true;
    }
    x -= button + metrics.fs(5.0);
    let zoom_in = Rect::from_min_size(
        Pos2::new(x - button, toolbar.center().y - button / 2.0),
        Vec2::splat(button),
    );
    if widgets::icon_button(ui, zoom_in, Icon::ZoomIn, "Zoom in", metrics).clicked() {
        let cx = (state.x_min + state.x_max) / 2.0;
        let cy = (state.y_min + state.y_max) / 2.0;
        state.zoom(0.8, cx, cy);
        state.analysis_dirty = true;
    }

    let canvas = Rect::from_min_max(
        Pos2::new(rect.left() + 1.0, toolbar.bottom() + metrics.fs(4.0)),
        Pos2::new(rect.right() - 1.0, rect.bottom() - 1.0),
    );
    // Interaction first so the frame draws the updated window.
    let response = ui.allocate_rect(canvas, Sense::click_and_drag());
    if response.dragged_by(PointerButton::Primary) {
        let delta = response.drag_delta();
        if delta != Vec2::ZERO {
            state.pan(
                delta.x as f64 / canvas.width().max(1.0) as f64,
                delta.y as f64 / canvas.height().max(1.0) as f64,
            );
            state.trace_x = None;
        }
    }
    if response.drag_stopped() {
        state.analysis_dirty = true;
    }
    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let x = state.x_min
                + (pos.x - canvas.left()) as f64 / canvas.width().max(1.0) as f64
                    * (state.x_max - state.x_min);
            state.trace_x = Some(x);
        }
    }
    if response.hovered() {
        let (scroll, zoom_delta) =
            ui.input(|input| (input.smooth_scroll_delta.y, input.zoom_delta()));
        if scroll.abs() > 0.01 {
            let anchor = response.hover_pos().unwrap_or(canvas.center());
            let ax = state.x_min
                + (anchor.x - canvas.left()) as f64 / canvas.width().max(1.0) as f64
                    * (state.x_max - state.x_min);
            let ay = state.y_max
                - (anchor.y - canvas.top()) as f64 / canvas.height().max(1.0) as f64
                    * (state.y_max - state.y_min);
            state.zoom((scroll as f64 * 0.0025).exp(), ax, ay);
        }
        if (zoom_delta - 1.0).abs() > 0.001 {
            let anchor = response.hover_pos().unwrap_or(canvas.center());
            let ax = state.x_min
                + (anchor.x - canvas.left()) as f64 / canvas.width().max(1.0) as f64
                    * (state.x_max - state.x_min);
            let ay = state.y_max
                - (anchor.y - canvas.top()) as f64 / canvas.height().max(1.0) as f64
                    * (state.y_max - state.y_min);
            state.zoom(1.0 / zoom_delta as f64, ax, ay);
        }
    }
    if response.double_clicked() {
        state.reset();
    }

    let painter = ui.painter().with_clip_rect(canvas);
    draw_grid(&painter, canvas, state, metrics);
    draw_functions(&painter, canvas, state);
    draw_trace(&painter, canvas, state, metrics);
    draw_analysis_markers(&painter, canvas, state);
    let hint = if metrics.compact {
        "Drag · pinch · tap to trace"
    } else {
        "Drag to pan · scroll to zoom · tap to trace"
    };
    let hint_size = theme::fit_size(
        &painter,
        hint,
        canvas.width() * 0.52,
        metrics.fs(7.5),
        metrics.fs(5.5),
        Weight::Medium,
    );
    let hint_rect = Rect::from_min_size(
        Pos2::new(
            canvas.left() + metrics.fs(6.0),
            canvas.top() + metrics.fs(5.0),
        ),
        Vec2::new(
            theme::text_width(&painter, hint, hint_size, Weight::Medium) + metrics.fs(10.0),
            metrics.fs(14.0),
        ),
    );
    painter.rect_filled(
        hint_rect,
        metrics.fs(7.0),
        Palette::INK.gamma_multiply(0.72),
    );
    painter.text(
        hint_rect.center(),
        Align2::CENTER_CENTER,
        hint,
        theme::font(hint_size, Weight::Medium),
        Palette::FAINT,
    );
}

fn draw_grid(painter: &egui::Painter, rect: Rect, state: &GraphState, metrics: &Metrics) {
    painter.rect_filled(rect, 0.0, Color32::from_rgb(0x0D, 0x0B, 0x12));
    let x_range = state.x_max - state.x_min;
    let y_range = state.y_max - state.y_min;
    if x_range <= 0.0 || y_range <= 0.0 {
        return;
    }
    let step_x = graph::nice_step(x_range, 10.0);
    let step_y = graph::nice_step(y_range, 8.0);
    let to_screen = |x: f64, y: f64| {
        Pos2::new(
            rect.left() + ((x - state.x_min) / x_range) as f32 * rect.width(),
            rect.bottom() - ((y - state.y_min) / y_range) as f32 * rect.height(),
        )
    };
    let alpha_minor = 12;
    let alpha_major = 26;
    let mut major_x = (state.x_min / step_x).ceil() * step_x;
    while major_x <= state.x_max {
        let sx = to_screen(major_x, 0.0).x;
        painter.line_segment(
            [Pos2::new(sx, rect.top()), Pos2::new(sx, rect.bottom())],
            Stroke::new(1.0, Color32::from_white_alpha(alpha_minor)),
        );
        let half = step_x / 2.0;
        let mut minor = major_x + half;
        while minor < major_x + step_x && minor <= state.x_max {
            let mx = to_screen(minor, 0.0).x;
            painter.line_segment(
                [Pos2::new(mx, rect.top()), Pos2::new(mx, rect.bottom())],
                Stroke::new(1.0, Color32::from_white_alpha(5)),
            );
            minor += half;
        }
        painter.text(
            Pos2::new(sx + metrics.fs(3.0), rect.bottom() - metrics.fs(10.0)),
            Align2::LEFT_CENTER,
            format::axis_label(major_x),
            theme::font(metrics.fs(7.5), Weight::Medium),
            Palette::FAINT,
        );
        major_x += step_x;
    }
    let mut major_y = (state.y_min / step_y).ceil() * step_y;
    while major_y <= state.y_max {
        let sy = to_screen(0.0, major_y).y;
        painter.line_segment(
            [Pos2::new(rect.left(), sy), Pos2::new(rect.right(), sy)],
            Stroke::new(1.0, Color32::from_white_alpha(alpha_minor)),
        );
        let half = step_y / 2.0;
        let mut minor = major_y + half;
        while minor < major_y + step_y && minor <= state.y_max {
            let my = to_screen(0.0, minor).y;
            painter.line_segment(
                [Pos2::new(rect.left(), my), Pos2::new(rect.right(), my)],
                Stroke::new(1.0, Color32::from_white_alpha(5)),
            );
            minor += half;
        }
        painter.text(
            Pos2::new(rect.left() + metrics.fs(6.0), sy - metrics.fs(6.0)),
            Align2::LEFT_CENTER,
            format::axis_label(major_y),
            theme::font(metrics.fs(7.5), Weight::Medium),
            Palette::FAINT,
        );
        major_y += step_y;
    }
    // Axes.
    if state.x_min <= 0.0 && state.x_max >= 0.0 {
        let sx = to_screen(0.0, 0.0).x;
        painter.line_segment(
            [Pos2::new(sx, rect.top()), Pos2::new(sx, rect.bottom())],
            Stroke::new(1.2, Color32::from_white_alpha(alpha_major + 35)),
        );
    }
    if state.y_min <= 0.0 && state.y_max >= 0.0 {
        let sy = to_screen(0.0, 0.0).y;
        painter.line_segment(
            [Pos2::new(rect.left(), sy), Pos2::new(rect.right(), sy)],
            Stroke::new(1.2, Color32::from_white_alpha(alpha_major + 35)),
        );
    }
    painter.rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, Color32::from_white_alpha(14)),
        StrokeKind::Inside,
    );
}

fn draw_functions(painter: &egui::Painter, rect: Rect, state: &GraphState) {
    let x_range = state.x_max - state.x_min;
    let y_range = state.y_max - state.y_min;
    if x_range <= 0.0 || y_range <= 0.0 {
        return;
    }
    let samples = rect.width().max(2.0) as usize;
    for function in &state.functions {
        if !function.visible {
            continue;
        }
        let Some(compiled) = &function.compiled else {
            continue;
        };
        let mut segments: Vec<Vec<Pos2>> = Vec::new();
        let mut current: Vec<Pos2> = Vec::new();
        let mut previous_screen: Option<Pos2> = None;
        for index in 0..=samples {
            let x = state.x_min + x_range * index as f64 / samples as f64;
            let y = compiled.eval(state.angle, &[("x", x)]).ok();
            let Some(y) = y.filter(|y| y.is_finite()) else {
                if current.len() > 1 {
                    segments.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
                previous_screen = None;
                continue;
            };
            let sx = rect.left() + (index as f32 / samples as f32) * rect.width();
            let sy = rect.bottom() - ((y - state.y_min) / y_range) as f32 * rect.height();
            let point = Pos2::new(sx, sy);
            if let Some(previous) = previous_screen {
                if (point.y - previous.y).abs() > rect.height() * 0.7 {
                    if current.len() > 1 {
                        segments.push(std::mem::take(&mut current));
                    } else {
                        current.clear();
                    }
                }
            }
            if sy > rect.top() - rect.height() * 1.5 && sy < rect.bottom() + rect.height() * 1.5 {
                current.push(point);
            } else if current.len() > 1 {
                segments.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
            previous_screen = Some(point);
        }
        if current.len() > 1 {
            segments.push(current);
        }
        let color = function.color();
        for segment in segments {
            painter.add(egui::Shape::line(segment, Stroke::new(2.0, color)));
        }
    }
}

fn draw_trace(painter: &egui::Painter, rect: Rect, state: &GraphState, metrics: &Metrics) {
    let Some(x) = state.trace_x else {
        return;
    };
    let x_range = state.x_max - state.x_min;
    let y_range = state.y_max - state.y_min;
    if x < state.x_min || x > state.x_max || x_range <= 0.0 || y_range <= 0.0 {
        return;
    }
    let sx = rect.left() + ((x - state.x_min) / x_range) as f32 * rect.width();
    crate::icons::dashed_line(
        painter,
        Pos2::new(sx, rect.top()),
        Pos2::new(sx, rect.bottom()),
        Stroke::new(1.0, Palette::TEAL_SOFT.gamma_multiply(0.55)),
    );
    let painter = painter;
    let mut label_rows: Vec<(String, Color32, f32)> = Vec::new();
    let mut first = true;
    for (index, function) in state.functions.iter().enumerate() {
        if !function.visible {
            continue;
        }
        let Some(compiled) = &function.compiled else {
            continue;
        };
        let Ok(y) = compiled.eval(state.angle, &[("x", x)]) else {
            continue;
        };
        if !y.is_finite() || y < state.y_min || y > state.y_max {
            continue;
        }
        let sy = rect.bottom() - ((y - state.y_min) / y_range) as f32 * rect.height();
        painter.circle_filled(Pos2::new(sx, sy), 4.0, function.color());
        painter.circle_stroke(
            Pos2::new(sx, sy),
            6.0,
            Stroke::new(1.0, Color32::WHITE.gamma_multiply(0.5)),
        );
        label_rows.push((
            format!("y{} = {}", index + 1, format::axis_label(y)),
            function.color(),
            sy,
        ));
        first = false;
    }
    let _ = first;
    if label_rows.is_empty() {
        return;
    }
    let width = metrics.fs(86.0);
    let row_height = metrics.fs(14.0);
    let height = row_height * label_rows.len() as f32 + metrics.fs(10.0);
    let left = if sx + width + metrics.fs(12.0) < rect.right() {
        sx + metrics.fs(10.0)
    } else {
        sx - width - metrics.fs(10.0)
    };
    let card = Rect::from_min_size(
        Pos2::new(left, (rect.top() + metrics.fs(8.0)).max(rect.top() + 4.0)),
        Vec2::new(width, height),
    );
    painter.rect_filled(card, metrics.fs(8.0), Palette::INK.gamma_multiply(0.86));
    painter.rect_stroke(
        card,
        metrics.fs(8.0),
        Stroke::new(1.0, Color32::from_white_alpha(22)),
        StrokeKind::Inside,
    );
    for (index, (text, color, _)) in label_rows.iter().enumerate() {
        painter.circle_filled(
            Pos2::new(
                card.left() + metrics.fs(8.0),
                card.top() + metrics.fs(9.0) + row_height * index as f32,
            ),
            metrics.fs(2.5),
            *color,
        );
        painter.text(
            Pos2::new(
                card.left() + metrics.fs(14.0),
                card.top() + metrics.fs(9.0) + row_height * index as f32,
            ),
            Align2::LEFT_CENTER,
            text,
            theme::font(metrics.fs(8.0), Weight::SemiBold),
            color_soften(*color),
        );
    }
    let _ = label_rows[0].2;
}

fn color_soften(color: Color32) -> Color32 {
    theme::blend(color, Color32::WHITE, 0.35)
}

fn draw_analysis_markers(painter: &egui::Painter, rect: Rect, state: &GraphState) {
    let x_range = state.x_max - state.x_min;
    if x_range <= 0.0 {
        return;
    }
    for root in &state.roots {
        if *root < state.x_min || *root > state.x_max {
            continue;
        }
        if state.y_min > 0.0 || state.y_max < 0.0 {
            continue;
        }
        let sx = rect.left() + ((*root - state.x_min) / x_range) as f32 * rect.width();
        let sy = rect.bottom()
            - ((0.0 - state.y_min) / (state.y_max - state.y_min)) as f32 * rect.height();
        painter.circle_stroke(
            Pos2::new(sx, sy),
            4.0,
            Stroke::new(1.2, Palette::VIOLET_SOFT),
        );
    }
    for (x, y) in &state.intersections {
        if *x < state.x_min || *x > state.x_max || *y < state.y_min || *y > state.y_max {
            continue;
        }
        let sx = rect.left() + ((*x - state.x_min) / x_range) as f32 * rect.width();
        let sy = rect.bottom()
            - ((*y - state.y_min) / (state.y_max - state.y_min)) as f32 * rect.height();
        painter.circle_stroke(Pos2::new(sx, sy), 5.0, Stroke::new(1.5, Palette::AMBER));
    }
}

fn functions_panel(ui: &mut Ui, rect: Rect, state: &mut GraphState, metrics: &Metrics) {
    theme::card(
        ui.painter(),
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(26),
    );
    let pad = metrics.fs(12.0);
    let header = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(6.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(24.0)),
    );
    theme::text(
        ui.painter(),
        header.left_center(),
        Align2::LEFT_CENTER,
        "FUNCTIONS",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let add = Rect::from_min_size(
        Pos2::new(
            header.right() - metrics.fs(58.0),
            header.center().y - metrics.fs(11.0),
        ),
        Vec2::new(metrics.fs(58.0), metrics.fs(22.0)),
    );
    if widgets::pill(ui, add, "+ Add", false, true, metrics).clicked() {
        state.add_function();
    }
    let row_height = metrics.fs(36.0);
    let mut y = header.bottom() + metrics.fs(2.0);
    let mut remove = None;
    let mut changed = None;
    for index in 0..state.functions.len() {
        if y + row_height > rect.bottom() - metrics.fs(4.0) {
            break;
        }
        let row = Rect::from_min_size(
            Pos2::new(rect.left() + metrics.fs(7.0), y),
            Vec2::new(rect.width() - metrics.fs(14.0), row_height),
        );
        let active = index == state.selected;
        if active {
            ui.painter()
                .rect_filled(row, metrics.fs(9.0), Palette::VIOLET.gamma_multiply(0.10));
        }
        let color = state.functions[index].color();
        ui.painter().circle_filled(
            Pos2::new(row.left() + metrics.fs(9.0), row.center().y),
            metrics.fs(4.0),
            color,
        );
        let field_rect = Rect::from_min_max(
            Pos2::new(row.left() + metrics.fs(18.0), row.top() + metrics.fs(3.0)),
            Pos2::new(
                row.right() - metrics.fs(53.0),
                row.bottom() - metrics.fs(3.0),
            ),
        );
        let response = widgets::field(
            ui,
            field_rect,
            &mut state.functions[index].expression,
            "f(x)",
            metrics,
        );
        if response.changed() {
            changed = Some(index);
        }
        if response.clicked() {
            state.selected = index;
            state.analysis_dirty = true;
        }
        let eye = Rect::from_min_size(
            Pos2::new(
                row.right() - metrics.fs(50.0),
                row.center().y - metrics.fs(10.0),
            ),
            Vec2::splat(metrics.fs(20.0)),
        );
        if widgets::icon_toggle(
            ui,
            eye,
            if state.functions[index].visible {
                Icon::Eye
            } else {
                Icon::EyeOff
            },
            state.functions[index].visible,
            "visible",
        )
        .clicked()
        {
            state.functions[index].visible = !state.functions[index].visible;
            state.analysis_dirty = true;
            state.fit_y();
        }
        let trash = Rect::from_min_size(
            Pos2::new(
                row.right() - metrics.fs(26.0),
                row.center().y - metrics.fs(10.0),
            ),
            Vec2::splat(metrics.fs(20.0)),
        );
        if widgets::icon_button(ui, trash, Icon::Trash, "Delete function", metrics).clicked() {
            remove = Some(index);
        }
        if let Some(error) = &state.functions[index].error {
            theme::text(
                ui.painter(),
                Pos2::new(field_rect.left(), row.bottom() - metrics.fs(1.0)),
                Align2::LEFT_CENTER,
                error,
                metrics.fs(6.0),
                Weight::Medium,
                Palette::ERROR,
            );
        }
        y += row_height + metrics.fs(2.0);
    }
    if let Some(index) = changed {
        state.functions[index].recompile();
        state.analysis_dirty = true;
        state.fit_y();
    }
    if let Some(index) = remove {
        state.remove_function(index);
    }
    let hint = if state
        .functions
        .iter()
        .any(|function| function.error.is_some())
    {
        "Check the red function"
    } else {
        "Tap a function to trace it"
    };
    theme::text(
        ui.painter(),
        Pos2::new(rect.left() + pad, rect.bottom() - metrics.fs(8.0)),
        Align2::LEFT_CENTER,
        hint,
        metrics.fs(7.5),
        Weight::Medium,
        Palette::FAINT,
    );
    let _ = y;
}

fn analysis_panel(ui: &mut Ui, rect: Rect, state: &mut GraphState, metrics: &Metrics) {
    theme::card(
        ui.painter(),
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(26),
    );
    let pad = metrics.fs(12.0);
    let header = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(6.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(24.0)),
    );
    theme::text(
        ui.painter(),
        header.left_center(),
        Align2::LEFT_CENTER,
        "ANALYSIS",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let segmented = Rect::from_min_size(
        Pos2::new(rect.left() + pad, header.bottom() + metrics.fs(2.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(26.0)),
    );
    let labels = [
        AnalysisTab::Roots.label(),
        AnalysisTab::Intersections.label(),
        AnalysisTab::Table.label(),
    ];
    if let Some(index) = widgets::segmented(
        ui,
        segmented,
        &labels,
        match state.analysis {
            AnalysisTab::Roots => 0,
            AnalysisTab::Intersections => 1,
            AnalysisTab::Table => 2,
        },
        metrics,
    ) {
        state.analysis = match index {
            0 => AnalysisTab::Roots,
            1 => AnalysisTab::Intersections,
            _ => AnalysisTab::Table,
        };
        state.analysis_dirty = true;
    }
    let run_rect = Rect::from_min_size(
        Pos2::new(rect.left() + pad, segmented.bottom() + metrics.fs(4.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(24.0)),
    );
    if widgets::text_button(ui, run_rect, "Find", false, metrics).clicked() {
        state.run_analysis();
    }
    let list = Rect::from_min_max(
        Pos2::new(rect.left() + pad, run_rect.bottom() + metrics.fs(6.0)),
        Pos2::new(rect.right() - pad, rect.bottom() - pad),
    );
    draw_analysis_list(ui, list, state, metrics);
}

fn draw_analysis_list(ui: &mut Ui, rect: Rect, state: &mut GraphState, metrics: &Metrics) {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect);
    if state.analysis == AnalysisTab::Table {
        if state.table.is_empty() {
            widgets::empty_state(&mut child, rect, "Tap Find for a table", metrics);
            return;
        }
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(&mut child, |ui| {
                for (x, y) in &state.table {
                    let (row, _) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), metrics.fs(20.0)),
                        Sense::hover(),
                    );
                    ui.painter().line_segment(
                        [row.left_center(), row.right_center()],
                        Stroke::new(1.0, Color32::from_white_alpha(8)),
                    );
                    ui.painter().text(
                        row.left_center() + Vec2::new(metrics.fs(2.0), 0.0),
                        Align2::LEFT_CENTER,
                        format::axis_label(*x),
                        theme::font(metrics.fs(8.5), Weight::Medium),
                        Palette::MUTED,
                    );
                    ui.painter().text(
                        row.right_center() - Vec2::new(metrics.fs(2.0), 0.0),
                        Align2::RIGHT_CENTER,
                        y.map(format::axis_label).unwrap_or_else(|| "—".to_string()),
                        theme::font(metrics.fs(8.5), Weight::SemiBold),
                        Palette::TEXT,
                    );
                }
            });
        return;
    }
    let items: Vec<(f64, Option<f64>)> = match state.analysis {
        AnalysisTab::Roots => state.roots.iter().map(|x| (*x, Some(0.0))).collect(),
        AnalysisTab::Intersections => state
            .intersections
            .iter()
            .map(|(x, y)| (*x, Some(*y)))
            .collect(),
        AnalysisTab::Table => Vec::new(),
    };
    if items.is_empty() {
        widgets::empty_state(&mut child, rect, "No results in this window", metrics);
        return;
    }
    let mut trace = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            for (index, (x, y)) in items.iter().enumerate() {
                let (row, response) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), metrics.fs(26.0)),
                    Sense::click(),
                );
                if index % 2 == 0 {
                    ui.painter()
                        .rect_filled(row, metrics.fs(6.0), Color32::from_white_alpha(5));
                }
                ui.painter().text(
                    row.left_center() + Vec2::new(metrics.fs(6.0), 0.0),
                    Align2::LEFT_CENTER,
                    format!("x = {}", format::axis_label(*x)),
                    theme::font(metrics.fs(9.0), Weight::SemiBold),
                    Palette::TEXT,
                );
                if let Some(y) = y {
                    ui.painter().text(
                        row.right_center() - Vec2::new(metrics.fs(6.0), 0.0),
                        Align2::RIGHT_CENTER,
                        format!("y = {}", format::axis_label(*y)),
                        theme::font(metrics.fs(9.0), Weight::Medium),
                        Palette::TEAL_SOFT,
                    );
                }
                if response.clicked() {
                    trace = Some(*x);
                }
            }
        });
    if let Some(x) = trace {
        state.trace_x = Some(x);
    }
}

fn compact_analysis_bar(ui: &mut Ui, rect: Rect, state: &mut GraphState, metrics: &Metrics) {
    let gap = metrics.fs(6.0);
    let count = 4;
    let width = (rect.width() - gap * (count as f32 - 1.0)) / count as f32;
    for index in 0..3 {
        let button = Rect::from_min_size(
            Pos2::new(rect.left() + index as f32 * (width + gap), rect.top()),
            Vec2::new(width, rect.height()),
        );
        let label = match index {
            0 => "Roots",
            1 => "Intersect",
            _ => "Table",
        };
        if widgets::pill(
            ui,
            button,
            label,
            false,
            state.analysis as usize == index,
            metrics,
        )
        .clicked()
        {
            state.analysis = match index {
                0 => AnalysisTab::Roots,
                1 => AnalysisTab::Intersections,
                _ => AnalysisTab::Table,
            };
            state.analysis_dirty = true;
            state.run_analysis();
            state.show_analysis = true;
        }
    }
    let reset = Rect::from_min_size(
        Pos2::new(rect.left() + 3.0 * (width + gap), rect.top()),
        Vec2::new(width, rect.height()),
    );
    if widgets::pill(ui, reset, "Reset", false, false, metrics).clicked() {
        state.reset();
    }
}

fn functions_modal(
    ui: &mut Ui,
    state: &mut GraphState,
    metrics: &Metrics,
    _feedback: &mut Feedback,
) {
    let width = common::modal_width(ui, metrics.fs(440.0));
    let frame = egui::Frame::NONE
        .fill(Palette::CARD)
        .stroke(Stroke::new(1.0, Palette::BORDER))
        .corner_radius(metrics.fs(20.0))
        .inner_margin(Margin::same(16));
    let mut close = false;
    let response = egui::Modal::new(Id::unique("graph-functions"))
        .frame(frame)
        .show(ui.ctx(), |ui| {
            ui.set_width(width);
            if widgets::modal_header(ui, "Functions", "Up to six curves", metrics, true) {
                close = true;
            }
            ui.add_space(metrics.fs(8.0));
            if widgets::text_button(
                ui,
                Rect::from_min_size(
                    ui.cursor().min,
                    Vec2::new(ui.available_width(), metrics.fs(28.0)),
                ),
                "+ Add function",
                true,
                metrics,
            )
            .clicked()
            {
                state.add_function();
            }
            ui.add_space(metrics.fs(8.0));
            for index in 0..state.functions.len() {
                let row = Rect::from_min_size(
                    ui.cursor().min,
                    Vec2::new(ui.available_width(), metrics.fs(30.0)),
                );
                ui.allocate_rect(row, Sense::hover());
                ui.painter().circle_filled(
                    Pos2::new(row.left() + metrics.fs(6.0), row.center().y),
                    metrics.fs(4.0),
                    state.functions[index].color(),
                );
                let field_rect = Rect::from_min_max(
                    Pos2::new(row.left() + metrics.fs(16.0), row.top()),
                    Pos2::new(row.right() - metrics.fs(30.0), row.bottom()),
                );
                let changed = widgets::field(
                    ui,
                    field_rect,
                    &mut state.functions[index].expression,
                    "f(x)",
                    metrics,
                )
                .changed();
                if changed {
                    state.functions[index].recompile();
                    state.analysis_dirty = true;
                    state.fit_y();
                }
                let remove = Rect::from_min_size(
                    Pos2::new(row.right() - metrics.fs(26.0), row.top() + metrics.fs(4.0)),
                    Vec2::splat(metrics.fs(22.0)),
                );
                if widgets::icon_button(ui, remove, Icon::Trash, "Delete", metrics).clicked() {
                    state.remove_function(index);
                    break;
                }
                ui.add_space(metrics.fs(4.0));
            }
        });
    if response.should_close() {
        close = true;
    }
    if close {
        state.show_functions = false;
    }
}

fn analysis_modal(
    ui: &mut Ui,
    state: &mut GraphState,
    metrics: &Metrics,
    _feedback: &mut Feedback,
) {
    let width = common::modal_width(ui, metrics.fs(430.0));
    let height = common::modal_max_height(ui) * 0.72;
    let frame = egui::Frame::NONE
        .fill(Palette::CARD)
        .stroke(Stroke::new(1.0, Palette::BORDER))
        .corner_radius(metrics.fs(20.0))
        .inner_margin(Margin::same(16));
    let mut close = false;
    let response = egui::Modal::new(Id::unique("graph-analysis"))
        .frame(frame)
        .show(ui.ctx(), |ui| {
            ui.set_width(width);
            ui.set_max_height(height);
            if widgets::modal_header(ui, state.analysis.label(), "Visible window", metrics, true) {
                close = true;
            }
            ui.add_space(metrics.fs(6.0));
            let list = Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(ui.available_width(), height - metrics.fs(60.0)),
            );
            draw_analysis_list(ui, list, state, metrics);
        });
    if response.should_close() {
        close = true;
    }
    if close {
        state.show_analysis = false;
    }
}
