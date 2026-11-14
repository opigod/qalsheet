//! Tools: data/statistics, programmer mode, numeric calculus and constants.

use eframe::egui::{self, Align2, Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use qalsheet_core::calculator::CalculatorState;
use qalsheet_core::calculus;
use qalsheet_core::constants::{self};
use qalsheet_core::expr;
use qalsheet_core::format;
use qalsheet_core::programmer::{self, BaseRow, BitOp, ProgrammerState};
use qalsheet_core::stats::{self, Regression, Stats};

use crate::app::{Feedback, Tab};
use crate::icons::{self, Icon};
use crate::screens::common;
use crate::theme::{self, Metrics, Palette, Weight};
use crate::widgets::{self};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolsTab {
    Data,
    Programmer,
    Calculus,
    Constants,
}

impl ToolsTab {
    fn label(self) -> &'static str {
        match self {
            Self::Data => "Data",
            Self::Programmer => "Programmer",
            Self::Calculus => "Calculus",
            Self::Constants => "Constants",
        }
    }

    const ALL: [Self; 4] = [
        Self::Data,
        Self::Programmer,
        Self::Calculus,
        Self::Constants,
    ];
}

pub struct ToolsState {
    pub tab: ToolsTab,
    pub data_input: String,
    pub stats: Option<Stats>,
    pub pairs: bool,
    pub pair_input: String,
    pub regression: Option<Regression>,
    pub programmer: ProgrammerState,
    pub calculus_expr: String,
    pub calculus_x: String,
    pub calculus_a: String,
    pub calculus_b: String,
    pub derivative_result: Option<Result<f64, String>>,
    pub integral_result: Option<Result<f64, String>>,
    pub constant_query: String,
}

impl ToolsState {
    pub fn new() -> Self {
        let data_input = "2, 4, 4, 4, 5, 5, 7, 9".to_string();
        let stats = Stats::compute(&stats::parse_numbers(&data_input));
        Self {
            tab: ToolsTab::Data,
            data_input,
            stats,
            pairs: false,
            pair_input: "1 2\n2 4\n3 5\n4 4\n5 6".to_string(),
            regression: None,
            programmer: ProgrammerState::new(),
            calculus_expr: "sin(x) + x^2 / 8".to_string(),
            calculus_x: "1.5".to_string(),
            calculus_a: "0".to_string(),
            calculus_b: "3.14".to_string(),
            derivative_result: None,
            integral_result: None,
            constant_query: String::new(),
        }
    }

    fn recompute_stats(&mut self) {
        self.stats = Stats::compute(&stats::parse_numbers(&self.data_input));
    }

    fn recompute_regression(&mut self) {
        self.regression = stats::parse_pairs(&self.pair_input)
            .and_then(|(xs, ys)| stats::linear_regression(&xs, &ys));
    }
}

pub fn screen(
    ui: &mut Ui,
    state: &mut ToolsState,
    calculator: &mut CalculatorState,
    metrics: &Metrics,
    feedback: &mut Feedback,
    tab: &mut Tab,
) {
    if metrics.watch {
        return;
    }
    let area = ui.max_rect();
    if metrics.compact {
        let controls = metrics.controls();
        let tabs = Rect::from_min_size(area.min, Vec2::new(area.width(), controls.fs(26.0)));
        let labels = ["Data", "Program", "Calculus", "Symbols"];
        if let Some(index) = widgets::segmented(
            ui,
            tabs,
            &labels,
            match state.tab {
                ToolsTab::Data => 0,
                ToolsTab::Programmer => 1,
                ToolsTab::Calculus => 2,
                ToolsTab::Constants => 3,
            },
            &controls,
        ) {
            state.tab = ToolsTab::ALL[index];
        }
        let content = Rect::from_min_max(
            Pos2::new(area.left(), tabs.bottom() + metrics.fs(6.0)),
            area.max,
        );
        draw_tool(ui, content, state, calculator, metrics, feedback, tab, true);
    } else {
        let rail_width = metrics.fs(170.0);
        let rail = Rect::from_min_size(area.min, Vec2::new(rail_width, area.height()));
        tools_rail(ui, rail, state, metrics);
        let content = Rect::from_min_max(
            Pos2::new(rail.right() + metrics.fs(14.0), area.top()),
            area.max,
        );
        draw_tool(
            ui, content, state, calculator, metrics, feedback, tab, false,
        );
    }
}

fn draw_tool(
    ui: &mut Ui,
    rect: Rect,
    state: &mut ToolsState,
    calculator: &mut CalculatorState,
    metrics: &Metrics,
    feedback: &mut Feedback,
    tab: &mut Tab,
    compact: bool,
) {
    match state.tab {
        ToolsTab::Data => data_tool(ui, rect, state, metrics, feedback, compact),
        ToolsTab::Programmer => programmer_tool(ui, rect, state, metrics, feedback, compact),
        ToolsTab::Calculus => {
            calculus_tool(ui, rect, state, calculator, metrics, feedback, compact)
        }
        ToolsTab::Constants => {
            constants_tool(ui, rect, state, calculator, metrics, feedback, tab, compact)
        }
    }
}

fn tools_rail(ui: &mut Ui, rect: Rect, state: &mut ToolsState, metrics: &Metrics) {
    theme::card(
        ui.painter(),
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(18),
    );
    theme::text(
        ui.painter(),
        Pos2::new(
            rect.left() + metrics.fs(14.0),
            rect.top() + metrics.fs(18.0),
        ),
        Align2::LEFT_CENTER,
        "TOOLS",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let mut y = rect.top() + metrics.fs(32.0);
    for tool in ToolsTab::ALL {
        let row = Rect::from_min_size(
            Pos2::new(rect.left() + metrics.fs(8.0), y),
            Vec2::new(rect.width() - metrics.fs(16.0), metrics.fs(34.0)),
        );
        let id = ui.scope_id().with(("tools-tab", tool.label()));
        let response = ui.interact(row, id, Sense::click());
        let active = state.tab == tool;
        if active {
            ui.painter()
                .rect_filled(row, metrics.fs(9.0), Palette::VIOLET.gamma_multiply(0.15));
            ui.painter().rect_filled(
                Rect::from_min_size(
                    Pos2::new(row.left(), row.center().y - metrics.fs(8.0)),
                    Vec2::new(metrics.fs(2.0), metrics.fs(16.0)),
                ),
                1.0,
                Palette::VIOLET,
            );
        }
        theme::text(
            ui.painter(),
            Pos2::new(row.left() + metrics.fs(14.0), row.center().y),
            Align2::LEFT_CENTER,
            tool.label(),
            metrics.fs(10.5),
            if active { Weight::Bold } else { Weight::Medium },
            if active {
                Palette::TEXT
            } else {
                Palette::MUTED
            },
        );
        if response.clicked() {
            state.tab = tool;
        }
        y += metrics.fs(38.0);
    }
    theme::text(
        ui.painter(),
        Pos2::new(
            rect.left() + metrics.fs(14.0),
            rect.bottom() - metrics.fs(10.0),
        ),
        Align2::LEFT_CENTER,
        "No network · no runtime",
        metrics.fs(7.0),
        Weight::Medium,
        Palette::FAINT,
    );
}

fn scroll_page(ui: &mut Ui, rect: Rect, add: impl FnOnce(&mut Ui)) {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(rect);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            ui.set_width((rect.width() - 8.0).max(120.0));
            add(ui);
        });
}

fn card_height(metrics: &Metrics, base: f32) -> f32 {
    metrics.fs(base)
}

fn data_tool(
    ui: &mut Ui,
    rect: Rect,
    state: &mut ToolsState,
    metrics: &Metrics,
    feedback: &mut Feedback,
    compact: bool,
) {
    scroll_page(ui, rect, |ui| {
        let width = ui.available_width();
        let input_height = if compact {
            metrics.fs(104.0)
        } else {
            metrics.fs(112.0)
        };
        let (card, _) = ui.allocate_exact_size(Vec2::new(width, input_height), Sense::hover());
        theme::card(
            ui.painter(),
            card,
            metrics.fs(16.0),
            Palette::CARD,
            Color32::from_white_alpha(18),
        );
        theme::text(
            ui.painter(),
            Pos2::new(
                card.left() + metrics.fs(12.0),
                card.top() + metrics.fs(14.0),
            ),
            Align2::LEFT_CENTER,
            "DATA",
            metrics.fs(8.0),
            Weight::Bold,
            Palette::FAINT,
        );
        theme::text(
            ui.painter(),
            Pos2::new(
                card.right() - metrics.fs(12.0),
                card.top() + metrics.fs(14.0),
            ),
            Align2::RIGHT_CENTER,
            "comma, spaces or new lines",
            metrics.fs(7.5),
            Weight::Medium,
            Palette::MUTED,
        );
        let field = Rect::from_min_size(
            Pos2::new(
                card.left() + metrics.fs(10.0),
                card.top() + metrics.fs(24.0),
            ),
            Vec2::new(
                card.width() - metrics.fs(20.0),
                card.height() - metrics.fs(30.0),
            ),
        );
        let changed = widgets::field_multiline(
            ui,
            field,
            &mut state.data_input,
            "2, 4, 4, 5, 7, 9",
            metrics,
        )
        .changed();
        if changed {
            state.recompute_stats();
        }
        ui.add_space(metrics.fs(8.0));
        let actions = Rect::from_min_size(ui.cursor().min, Vec2::new(width, metrics.fs(26.0)));
        ui.allocate_rect(actions, Sense::hover());
        let sample =
            Rect::from_min_size(actions.min, Vec2::new(metrics.fs(82.0), actions.height()));
        if widgets::pill(ui, sample, "Sample", false, false, metrics).clicked() {
            state.data_input = "2, 4, 4, 4, 5, 5, 7, 9".to_string();
            state.recompute_stats();
        }
        let copy = Rect::from_min_size(
            Pos2::new(sample.right() + metrics.fs(6.0), actions.top()),
            Vec2::new(metrics.fs(82.0), actions.height()),
        );
        if widgets::pill(ui, copy, "Copy", false, true, metrics).clicked() {
            if let Some(stats) = &state.stats {
                ui.ctx().copy_text(stats::summary(stats));
                feedback.show(ui.ctx(), "Summary copied");
            }
        }
        ui.add_space(metrics.fs(8.0));

        if let Some(stats) = state.stats.clone() {
            let sum_of_squares: f64 = stats::parse_numbers(&state.data_input)
                .iter()
                .map(|value| value * value)
                .sum();
            let standard_error = if stats.count > 0 {
                stats.sample_stddev / (stats.count as f64).sqrt()
            } else {
                0.0
            };
            let coefficient_variation = if stats.mean != 0.0 {
                stats.sample_stddev / stats.mean.abs() * 100.0
            } else {
                0.0
            };
            let mode = stats
                .mode
                .map(format::plain_number)
                .unwrap_or_else(|| "—".to_string());
            let values = [
                ("Count", format::plain_number(stats.count as f64)),
                ("Sum", format::plain_number(stats.sum)),
                ("Mean", format::plain_number(stats.mean)),
                ("Median", format::plain_number(stats.median)),
                ("Min", format::plain_number(stats.min)),
                ("Max", format::plain_number(stats.max)),
                ("Range", format::plain_number(stats.range)),
                ("Mode", mode),
                ("Q1", format::plain_number(stats.q1)),
                ("Q3", format::plain_number(stats.q3)),
                ("IQR", format::plain_number(stats.iqr)),
                ("Variance", format::plain_number(stats.sample_variance)),
                (
                    "Population var",
                    format::plain_number(stats.population_variance),
                ),
                ("Std. error", format::plain_number(standard_error)),
                ("CV %", format::plain_number(coefficient_variation)),
                ("Sum of squares", format::plain_number(sum_of_squares)),
                ("Sample SD", format::plain_number(stats.sample_stddev)),
                (
                    "Population SD",
                    format::plain_number(stats.population_stddev),
                ),
            ];
            let tile_metrics = metrics.controls();
            let columns = if compact { 2 } else { 4 };
            let rows = values.len().div_ceil(columns);
            let gap = metrics.fs(6.0);
            let tile_height = tile_metrics.fs(52.0);
            let grid_height = rows as f32 * tile_height + (rows as f32 - 1.0) * gap;
            let (grid, _) = ui.allocate_exact_size(Vec2::new(width, grid_height), Sense::hover());
            let tile_width = (grid.width() - gap * (columns as f32 - 1.0)) / columns as f32;
            for (index, (label, value)) in values.iter().enumerate() {
                let row = index / columns;
                let col = index % columns;
                let tile = Rect::from_min_size(
                    Pos2::new(
                        grid.left() + col as f32 * (tile_width + gap),
                        grid.top() + row as f32 * (tile_height + gap),
                    ),
                    Vec2::new(tile_width, tile_height),
                );
                let accent = if index < 5 {
                    Palette::VIOLET_SOFT
                } else if index < 11 {
                    Palette::TEAL_SOFT
                } else {
                    Palette::PINK
                };
                common::stat_tile(ui, tile, label, value, accent, &tile_metrics);
            }
            ui.add_space(metrics.fs(8.0));
            distribution_card(
                ui,
                width,
                stats::parse_numbers(&state.data_input).as_slice(),
                metrics,
            );
            ui.add_space(metrics.fs(8.0));
        } else {
            let (empty, _) =
                ui.allocate_exact_size(Vec2::new(width, metrics.fs(50.0)), Sense::hover());
            widgets::empty_state(ui, empty, "Type some numbers to see statistics", metrics);
        }

        // Regression.
        let (pair_header, _) =
            ui.allocate_exact_size(Vec2::new(width, metrics.fs(26.0)), Sense::hover());
        theme::text(
            ui.painter(),
            pair_header.left_center(),
            Align2::LEFT_CENTER,
            "PAIRED DATA · LINEAR REGRESSION",
            metrics.fs(8.0),
            Weight::Bold,
            Palette::FAINT,
        );
        let toggle = Rect::from_min_size(
            Pos2::new(pair_header.right() - metrics.fs(72.0), pair_header.top()),
            Vec2::new(metrics.fs(72.0), pair_header.height()),
        );
        if widgets::pill(
            ui,
            toggle,
            if state.pairs { "Hide" } else { "Show" },
            false,
            state.pairs,
            metrics,
        )
        .clicked()
        {
            state.pairs = !state.pairs;
            if state.pairs && state.regression.is_none() {
                state.recompute_regression();
            }
        }
        if state.pairs {
            let (pair_card, _) = ui.allocate_exact_size(
                Vec2::new(width, card_height(metrics, 106.0)),
                Sense::hover(),
            );
            theme::card(
                ui.painter(),
                pair_card,
                metrics.fs(14.0),
                Palette::CARD,
                Color32::from_white_alpha(16),
            );
            let field = Rect::from_min_size(
                Pos2::new(
                    pair_card.left() + metrics.fs(10.0),
                    pair_card.top() + metrics.fs(8.0),
                ),
                Vec2::new(
                    pair_card.width() - metrics.fs(20.0),
                    pair_card.height() - metrics.fs(16.0),
                ),
            );
            let changed = widgets::field_multiline(
                ui,
                field,
                &mut state.pair_input,
                "1 2\n2 4\n3 6",
                metrics,
            )
            .changed();
            if changed {
                state.recompute_regression();
            }
            ui.add_space(metrics.fs(6.0));
            if let Some(regression) = &state.regression {
                let text = format!(
                    "y = {}x {:+}    r² = {}    n = {}",
                    format::plain_number(regression.slope),
                    regression.intercept,
                    format::plain_number(regression.r_squared),
                    regression.count
                );
                theme::text(
                    ui.painter(),
                    Pos2::new(
                        ui.cursor().min.x + metrics.fs(2.0),
                        ui.cursor().min.y + metrics.fs(8.0),
                    ),
                    Align2::LEFT_CENTER,
                    text,
                    metrics.fs(9.0),
                    Weight::SemiBold,
                    Palette::TEAL_SOFT,
                );
                ui.add_space(metrics.fs(18.0));
            } else {
                theme::text(
                    ui.painter(),
                    Pos2::new(
                        ui.cursor().min.x + metrics.fs(2.0),
                        ui.cursor().min.y + metrics.fs(8.0),
                    ),
                    Align2::LEFT_CENTER,
                    "Add at least two pairs, one per line",
                    metrics.fs(8.5),
                    Weight::Medium,
                    Palette::MUTED,
                );
                ui.add_space(metrics.fs(18.0));
            }
        }
    });
}

fn distribution_card(ui: &mut Ui, width: f32, values: &[f64], metrics: &Metrics) {
    if values.len() < 2 {
        return;
    }
    let controls = if metrics.square_keys {
        metrics.controls()
    } else {
        *metrics
    };
    let height = controls.fs(122.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    let painter = ui.painter();
    theme::card(
        painter,
        rect,
        metrics.fs(14.0),
        Palette::CARD,
        Color32::from_white_alpha(16),
    );
    let pad = metrics.fs(12.0);
    theme::text(
        painter,
        Pos2::new(rect.left() + pad, rect.top() + controls.fs(14.0)),
        Align2::LEFT_CENTER,
        "DISTRIBUTION",
        controls.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    theme::text(
        painter,
        Pos2::new(rect.right() - pad, rect.top() + controls.fs(14.0)),
        Align2::RIGHT_CENTER,
        format!(
            "mean {} · median {}",
            format::plain_number(mean(values)),
            format::plain_number(median(values))
        ),
        controls.fs(8.0),
        Weight::Medium,
        Palette::MUTED,
    );

    let plot = Rect::from_min_max(
        Pos2::new(rect.left() + pad, rect.top() + controls.fs(28.0)),
        Pos2::new(rect.right() - pad, rect.bottom() - controls.fs(22.0)),
    );
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    let bins = if span <= f64::EPSILON { 1 } else { 9 };
    let mut counts = vec![0usize; bins];
    for value in values {
        let position = if span <= f64::EPSILON {
            0.0
        } else {
            ((value - min) / span * bins as f64).floor()
        };
        let index = (position as usize).min(bins - 1);
        counts[index] += 1;
    }
    let max_count = counts.iter().copied().max().unwrap_or(1).max(1);
    let gap = controls.fs(2.0);
    let bar_width = (plot.width() - gap * (bins as f32 - 1.0)) / bins as f32;
    for (index, count) in counts.iter().enumerate() {
        if *count == 0 {
            continue;
        }
        let bar_height = plot.height() * *count as f32 / max_count as f32;
        let bar = Rect::from_min_size(
            Pos2::new(
                plot.left() + index as f32 * (bar_width + gap),
                plot.bottom() - bar_height,
            ),
            Vec2::new(bar_width, bar_height),
        );
        painter.rect_filled(
            bar,
            controls.fs(2.5),
            theme::GRAPH_COLORS[index % theme::GRAPH_COLORS.len()].gamma_multiply(0.78),
        );
    }
    painter.line_segment(
        [plot.left_bottom(), plot.right_bottom()],
        Stroke::new(1.0, Color32::from_white_alpha(20)),
    );
    for (value, color) in [
        (mean(values), Palette::TEAL),
        (median(values), Palette::VIOLET),
    ] {
        if span > f64::EPSILON {
            let x = plot.left() + ((value - min) / span).clamp(0.0, 1.0) as f32 * plot.width();
            painter.line_segment(
                [Pos2::new(x, plot.top()), Pos2::new(x, plot.bottom())],
                Stroke::new(1.0, color.gamma_multiply(0.8)),
            );
        }
    }
    theme::text(
        painter,
        Pos2::new(plot.left(), rect.bottom() - controls.fs(10.0)),
        Align2::LEFT_CENTER,
        format::plain_number(min),
        controls.fs(7.5),
        Weight::Medium,
        Palette::FAINT,
    );
    theme::text(
        painter,
        Pos2::new(plot.right(), rect.bottom() - controls.fs(10.0)),
        Align2::RIGHT_CENTER,
        format!("max {}", format::plain_number(max)),
        controls.fs(7.5),
        Weight::Medium,
        Palette::FAINT,
    );
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

fn programmer_tool(
    ui: &mut Ui,
    rect: Rect,
    state: &mut ToolsState,
    metrics: &Metrics,
    feedback: &mut Feedback,
    _compact: bool,
) {
    scroll_page(ui, rect, |ui| {
        let width = ui.available_width();
        let header = Rect::from_min_size(ui.cursor().min, Vec2::new(width, metrics.fs(26.0)));
        ui.allocate_rect(header, Sense::hover());
        theme::text(
            ui.painter(),
            header.left_center(),
            Align2::LEFT_CENTER,
            "PROGRAMMER",
            metrics.fs(8.0),
            Weight::Bold,
            Palette::FAINT,
        );
        let toggle = Rect::from_min_size(
            Pos2::new(header.right() - metrics.fs(94.0), header.top()),
            Vec2::new(metrics.fs(94.0), header.height()),
        );
        if let Some(index) = widgets::segmented(
            ui,
            toggle,
            &["32 bit", "64 bit"],
            if state.programmer.width == 32 { 0 } else { 1 },
            metrics,
        ) {
            state.programmer.width = if index == 0 { 32 } else { 64 };
        }
        ui.add_space(metrics.fs(6.0));

        let field_height = metrics.fs(38.0);
        let (a, _) = ui.allocate_exact_size(Vec2::new(width, field_height), Sense::hover());
        theme::text(
            ui.painter(),
            Pos2::new(a.left() + metrics.fs(2.0), a.top() - metrics.fs(4.0)),
            Align2::LEFT_CENTER,
            "VALUE",
            metrics.fs(7.0),
            Weight::Bold,
            Palette::MUTED,
        );
        widgets::field(
            ui,
            a,
            &mut state.programmer.input,
            "0xFF, 0b1010, 255",
            metrics,
        );
        ui.add_space(metrics.fs(6.0));
        let (b, _) = ui.allocate_exact_size(Vec2::new(width, field_height), Sense::hover());
        widgets::field(
            ui,
            b,
            &mut state.programmer.operand,
            "second operand",
            metrics,
        );
        ui.add_space(metrics.fs(8.0));

        let (ops, _) = ui.allocate_exact_size(Vec2::new(width, metrics.fs(58.0)), Sense::hover());
        let op_gap = metrics.fs(5.0);
        let op_width = (ops.width() - op_gap * 2.0) / 3.0;
        for (index, op) in BitOp::ALL.iter().enumerate() {
            let row = index / 3;
            let col = index % 3;
            let cell = Rect::from_min_size(
                Pos2::new(
                    ops.left() + col as f32 * (op_width + op_gap),
                    ops.top() + row as f32 * (metrics.fs(26.0) + op_gap),
                ),
                Vec2::new(op_width, metrics.fs(26.0)),
            );
            if widgets::pill(
                ui,
                cell,
                op.label(),
                state.programmer.operation == *op,
                false,
                metrics,
            )
            .clicked()
            {
                state.programmer.operation = *op;
            }
        }
        ui.add_space(metrics.fs(8.0));

        if let Some(result) = state.programmer.result() {
            let (banner, _) =
                ui.allocate_exact_size(Vec2::new(width, metrics.fs(52.0)), Sense::hover());
            theme::card(
                ui.painter(),
                banner,
                metrics.fs(14.0),
                Palette::VIOLET.gamma_multiply(0.12),
                Palette::VIOLET.gamma_multiply(0.32),
            );
            theme::text(
                ui.painter(),
                Pos2::new(
                    banner.left() + metrics.fs(12.0),
                    banner.top() + metrics.fs(15.0),
                ),
                Align2::LEFT_CENTER,
                "RESULT",
                metrics.fs(7.5),
                Weight::Bold,
                Palette::VIOLET_SOFT,
            );
            theme::fitted_text(
                ui.painter(),
                Rect::from_min_max(
                    Pos2::new(
                        banner.left() + metrics.fs(12.0),
                        banner.top() + metrics.fs(22.0),
                    ),
                    Pos2::new(
                        banner.right() - metrics.fs(12.0),
                        banner.bottom() - metrics.fs(6.0),
                    ),
                ),
                Align2::LEFT_CENTER,
                &format!("{} ({})", result, state.programmer.signed(result)),
                metrics.fs(22.0),
                metrics.fs(10.0),
                Weight::ExtraBold,
                Palette::TEXT,
            );
            ui.add_space(metrics.fs(8.0));
            let rows = programmer::base_rows(result, state.programmer.width);
            for row in rows {
                let (card, _) =
                    ui.allocate_exact_size(Vec2::new(width, metrics.fs(34.0)), Sense::click());
                let response = ui.interact(
                    card,
                    ui.scope_id().with(("base", row.label)),
                    Sense::click(),
                );
                theme::card(
                    ui.painter(),
                    card,
                    metrics.fs(10.0),
                    Palette::CARD,
                    Color32::from_white_alpha(16),
                );
                theme::text(
                    ui.painter(),
                    Pos2::new(card.left() + metrics.fs(10.0), card.center().y),
                    Align2::LEFT_CENTER,
                    row.label,
                    metrics.fs(8.0),
                    Weight::Bold,
                    Palette::FAINT,
                );
                theme::fitted_text(
                    ui.painter(),
                    Rect::from_min_max(
                        Pos2::new(card.left() + metrics.fs(52.0), card.top() + metrics.fs(6.0)),
                        Pos2::new(
                            card.right() - metrics.fs(28.0),
                            card.bottom() - metrics.fs(6.0),
                        ),
                    ),
                    Align2::RIGHT_CENTER,
                    &row.value,
                    metrics.fs(11.0),
                    metrics.fs(7.0),
                    Weight::SemiBold,
                    Palette::TEXT,
                );
                icons::icon(
                    ui.painter(),
                    Pos2::new(card.right() - metrics.fs(13.0), card.center().y),
                    metrics.fs(11.0),
                    Icon::Copy,
                    Palette::FAINT,
                );
                if response.clicked() {
                    ui.ctx().copy_text(row.value.clone());
                    feedback.show(ui.ctx(), format!("{} copied", row.label));
                }
                ui.add_space(metrics.fs(4.0));
            }
            ui.add_space(metrics.fs(8.0));
            let bits = Rect::from_min_size(ui.cursor().min, Vec2::new(width, metrics.fs(38.0)));
            ui.allocate_rect(bits, Sense::hover());
            theme::text(
                ui.painter(),
                bits.left_top() + Vec2::new(metrics.fs(2.0), metrics.fs(6.0)),
                Align2::LEFT_CENTER,
                "BITS",
                metrics.fs(7.0),
                Weight::Bold,
                Palette::FAINT,
            );
            theme::fitted_text(
                ui.painter(),
                Rect::from_min_max(
                    Pos2::new(bits.left(), bits.top() + metrics.fs(12.0)),
                    bits.right_bottom(),
                ),
                Align2::LEFT_CENTER,
                &programmer::grouped_binary(result, state.programmer.width),
                metrics.fs(8.0),
                metrics.fs(5.5),
                Weight::Medium,
                Palette::MUTED,
            );
            ui.add_space(metrics.fs(6.0));
        } else {
            let (empty, _) =
                ui.allocate_exact_size(Vec2::new(width, metrics.fs(40.0)), Sense::hover());
            widgets::empty_state(
                ui,
                empty,
                "Enter a number in decimal, hex, binary or octal",
                metrics,
            );
        }
    });
}

fn calculus_tool(
    ui: &mut Ui,
    rect: Rect,
    state: &mut ToolsState,
    calculator: &mut CalculatorState,
    metrics: &Metrics,
    feedback: &mut Feedback,
    compact: bool,
) {
    let angle = calculator.angle_mode;
    scroll_page(ui, rect, |ui| {
        let width = ui.available_width();
        let header = Rect::from_min_size(ui.cursor().min, Vec2::new(width, metrics.fs(26.0)));
        ui.allocate_rect(header, Sense::hover());
        theme::text(
            ui.painter(),
            header.left_center(),
            Align2::LEFT_CENTER,
            "CALCULUS · NUMERIC",
            metrics.fs(8.0),
            Weight::Bold,
            Palette::FAINT,
        );
        theme::text(
            ui.painter(),
            header.right_center(),
            Align2::RIGHT_CENTER,
            format!("{} mode", angle.label()),
            metrics.fs(8.0),
            Weight::SemiBold,
            Palette::MUTED,
        );
        ui.add_space(metrics.fs(6.0));
        let expr_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(width, metrics.fs(40.0)));
        ui.allocate_rect(expr_rect, Sense::hover());
        theme::text(
            ui.painter(),
            Pos2::new(
                expr_rect.left() + metrics.fs(2.0),
                expr_rect.top() - metrics.fs(4.0),
            ),
            Align2::LEFT_CENTER,
            "f(x)",
            metrics.fs(7.0),
            Weight::Bold,
            Palette::MUTED,
        );
        widgets::field(
            ui,
            expr_rect,
            &mut state.calculus_expr,
            "sin(x) + x^2 / 8",
            metrics,
        );
        ui.add_space(metrics.fs(10.0));

        let columns = if compact { 1 } else { 3 };
        let gap = metrics.fs(6.0);
        let field_width = if columns == 1 {
            width
        } else {
            (width - gap * (columns as f32 - 1.0)) / columns as f32
        };
        let row = Rect::from_min_size(ui.cursor().min, Vec2::new(width, metrics.fs(38.0)));
        ui.allocate_rect(row, Sense::hover());
        let labels = [
            ("f'(x) at x", &mut state.calculus_x),
            ("a", &mut state.calculus_a),
            ("b", &mut state.calculus_b),
        ];
        let mut x = row.left();
        for (label, value) in labels {
            let field = Rect::from_min_size(
                Pos2::new(x, row.top()),
                Vec2::new(field_width, row.height()),
            );
            ui.painter().text(
                Pos2::new(
                    field.left() + metrics.fs(2.0),
                    field.top() - metrics.fs(2.0),
                ),
                Align2::LEFT_CENTER,
                label,
                theme::font(metrics.fs(6.5), Weight::Bold),
                Palette::MUTED,
            );
            widgets::field(ui, field, value, "0", metrics);
            x += field_width + gap;
        }
        ui.add_space(metrics.fs(8.0));
        let button_width = if compact {
            width
        } else {
            (width - metrics.fs(8.0)) / 2.0
        };
        let der = Rect::from_min_size(ui.cursor().min, Vec2::new(button_width, metrics.fs(28.0)));
        ui.allocate_rect(der, Sense::hover());
        if widgets::text_button(ui, der, "Derivative", false, metrics).clicked() {
            let x = state
                .calculus_x
                .replace('\u{2212}', "-")
                .parse::<f64>()
                .ok();
            let result = x.ok_or_else(|| "Enter a point".to_string()).and_then(|x| {
                let compiled =
                    expr::compile(&state.calculus_expr).map_err(|error| error.message)?;
                calculus::derivative(&compiled, x, angle).map_err(|error| error.message)
            });
            state.derivative_result = Some(result);
        }
        let integ = Rect::from_min_size(
            Pos2::new(der.right() + metrics.fs(8.0), der.top()),
            Vec2::new(button_width, der.height()),
        );
        if !compact {
            ui.allocate_rect(integ, Sense::hover());
            if widgets::text_button(ui, integ, "Integrate", true, metrics).clicked() {
                compute_integral(state, angle);
            }
        } else {
            ui.add_space(metrics.fs(6.0));
            let second = Rect::from_min_size(ui.cursor().min, Vec2::new(width, metrics.fs(28.0)));
            ui.allocate_rect(second, Sense::hover());
            if widgets::text_button(ui, second, "Integrate", true, metrics).clicked() {
                compute_integral(state, angle);
            }
            ui.add_space(metrics.fs(8.0));
        }
        if !compact {
            ui.add_space(metrics.fs(8.0));
        }
        if let Some(result) = &state.derivative_result {
            result_banner(ui, width, metrics, "DERIVATIVE", result);
        }
        if let Some(result) = &state.integral_result {
            result_banner(ui, width, metrics, "INTEGRAL", result);
        }
        ui.add_space(metrics.fs(8.0));
        theme::text(
            ui.painter(),
            Pos2::new(
                ui.cursor().min.x + metrics.fs(2.0),
                ui.cursor().min.y + metrics.fs(8.0),
            ),
            Align2::LEFT_CENTER,
            "Derivative uses a 5-point stencil. Integrals use adaptive Simpson.",
            metrics.fs(7.5),
            Weight::Medium,
            Palette::FAINT,
        );
        ui.add_space(metrics.fs(18.0));
        let _ = feedback;
    });
}

fn compute_integral(state: &mut ToolsState, angle: qalsheet_core::AngleMode) {
    let parse = |text: &str| text.replace('\u{2212}', "-").parse::<f64>().ok();
    let lower = parse(&state.calculus_a);
    let upper = parse(&state.calculus_b);
    let result = match (lower, upper) {
        (Some(a), Some(b)) => expr::compile(&state.calculus_expr)
            .map_err(|error| error.message)
            .and_then(|compiled| {
                calculus::integrate(&compiled, a, b, angle).map_err(|error| error.message)
            }),
        _ => Err("Enter both limits".to_string()),
    };
    state.integral_result = Some(result);
}

fn result_banner(
    ui: &mut Ui,
    width: f32,
    metrics: &Metrics,
    label: &str,
    result: &Result<f64, String>,
) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, metrics.fs(44.0)), Sense::hover());
    let (fill, border, color, text) = match result {
        Ok(value) => (
            Palette::TEAL.gamma_multiply(0.10),
            Palette::TEAL.gamma_multiply(0.30),
            Palette::TEAL_SOFT,
            format::plain_number(*value),
        ),
        Err(error) => (
            Palette::ERROR.gamma_multiply(0.08),
            Palette::ERROR.gamma_multiply(0.28),
            Palette::ERROR,
            error.clone(),
        ),
    };
    theme::card(ui.painter(), rect, metrics.fs(12.0), fill, border);
    theme::text(
        ui.painter(),
        Pos2::new(
            rect.left() + metrics.fs(10.0),
            rect.top() + metrics.fs(11.0),
        ),
        Align2::LEFT_CENTER,
        label,
        metrics.fs(7.0),
        Weight::Bold,
        Palette::MUTED,
    );
    theme::fitted_text(
        ui.painter(),
        Rect::from_min_max(
            Pos2::new(
                rect.left() + metrics.fs(10.0),
                rect.top() + metrics.fs(18.0),
            ),
            Pos2::new(
                rect.right() - metrics.fs(10.0),
                rect.bottom() - metrics.fs(4.0),
            ),
        ),
        Align2::LEFT_CENTER,
        &text,
        metrics.fs(16.0),
        metrics.fs(8.0),
        Weight::Bold,
        color,
    );
}

fn constants_tool(
    ui: &mut Ui,
    rect: Rect,
    state: &mut ToolsState,
    calculator: &mut CalculatorState,
    metrics: &Metrics,
    feedback: &mut Feedback,
    tab: &mut Tab,
    _compact: bool,
) {
    let search = Rect::from_min_size(rect.min, Vec2::new(rect.width(), metrics.fs(34.0)));
    let _ = widgets::field(
        ui,
        search,
        &mut state.constant_query,
        "Search constants",
        metrics,
    );
    let list = Rect::from_min_max(
        Pos2::new(rect.left(), search.bottom() + metrics.fs(8.0)),
        rect.max,
    );
    let results = constants::search(&state.constant_query);
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(list)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(list);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            let width = ui.available_width();
            let empty = results.is_empty();
            for constant in results.iter().copied() {
                let (row, response) =
                    ui.allocate_exact_size(Vec2::new(width, metrics.fs(46.0)), Sense::click());
                if response.hovered() {
                    ui.painter()
                        .rect_filled(row, metrics.fs(10.0), Color32::from_white_alpha(6));
                }
                theme::text(
                    ui.painter(),
                    Pos2::new(
                        row.left() + metrics.fs(8.0),
                        row.center().y - metrics.fs(7.0),
                    ),
                    Align2::LEFT_CENTER,
                    constant.name,
                    metrics.fs(9.5),
                    Weight::SemiBold,
                    Palette::TEXT,
                );
                theme::text(
                    ui.painter(),
                    Pos2::new(
                        row.left() + metrics.fs(8.0),
                        row.center().y + metrics.fs(7.0),
                    ),
                    Align2::LEFT_CENTER,
                    format!("{} · {}", constant.symbol, constant.note),
                    metrics.fs(7.0),
                    Weight::Medium,
                    Palette::MUTED,
                );
                theme::text(
                    ui.painter(),
                    Pos2::new(row.right() - metrics.fs(46.0), row.center().y),
                    Align2::RIGHT_CENTER,
                    format!("{} {}", format::plain_number(constant.value), constant.unit),
                    metrics.fs(8.0),
                    Weight::SemiBold,
                    Palette::TEAL_SOFT,
                );
                if response.clicked() {
                    ui.ctx().copy_text(format::plain_number(constant.value));
                    feedback.show(ui.ctx(), "Constant copied");
                }
                let use_rect = Rect::from_min_size(
                    Pos2::new(
                        row.right() - metrics.fs(40.0),
                        row.center().y - metrics.fs(9.0),
                    ),
                    Vec2::new(metrics.fs(30.0), metrics.fs(18.0)),
                );
                if widgets::icon_button(ui, use_rect, Icon::Plus, "Insert constant", metrics)
                    .clicked()
                {
                    calculator.insert_number(constant.value);
                    *tab = Tab::Calculator;
                    feedback.show(ui.ctx(), "Inserted into calculator");
                }
                ui.painter().line_segment(
                    [row.left_bottom(), row.right_bottom()],
                    Stroke::new(1.0, Color32::from_white_alpha(8)),
                );
            }
            if empty {
                let empty =
                    Rect::from_min_size(ui.cursor().min, Vec2::new(width, metrics.fs(50.0)));
                widgets::empty_state(ui, empty, "No constants match", metrics);
            }
        });
}

// Keep the copy rows helper type available to callers that want to inspect it.
#[allow(dead_code)]
fn base_rows_to_copy(rows: &[BaseRow]) -> Vec<String> {
    rows.iter()
        .map(|row| format!("{} {}", row.label, row.value))
        .collect()
}
