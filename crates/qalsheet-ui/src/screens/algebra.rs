//! Algebra editor and polynomial solver screen.

use eframe::egui::{
    self, Align2, Color32, Id, Margin, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use qalsheet_core::algebra::{AlgebraResult, AlgebraState};
use qalsheet_core::expr;
use qalsheet_core::format;

use crate::app::Feedback;
use crate::icons::{self, Icon};
use crate::screens::common;
use crate::theme::{self, Metrics, Palette, Weight};
use crate::widgets::{self, KeyKind};

#[derive(Default)]
pub struct AlgebraUi {
    show_variable_picker: bool,
    note_target: Option<u64>,
    note_head: Option<u64>,
    pub note_sheet: bool,
    note_draft: String,
}

pub fn screen(
    ui: &mut Ui,
    state: &mut AlgebraState,
    screen_ui: &mut AlgebraUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    sync_note_target(state, screen_ui);
    if metrics.watch {
        watch_screen(ui, state, screen_ui, metrics, feedback);
    } else if metrics.compact {
        compact_screen(ui, state, screen_ui, metrics, feedback);
    } else {
        wide_screen(ui, state, screen_ui, metrics, feedback);
    }
    if screen_ui.show_variable_picker {
        variable_picker(ui, state, screen_ui, metrics);
    }
    if screen_ui.note_sheet {
        notes_sheet(ui, state, screen_ui, metrics);
    }
}

fn sync_note_target(state: &AlgebraState, screen_ui: &mut AlgebraUi) {
    let head = state.history.first().map(|entry| entry.id);
    if head != screen_ui.note_head {
        screen_ui.note_head = head;
        screen_ui.note_target = head;
    } else if screen_ui.note_target.is_none()
        || !state
            .history
            .iter()
            .any(|entry| Some(entry.id) == screen_ui.note_target)
    {
        screen_ui.note_target = head;
    }
}

fn wide_screen(
    ui: &mut Ui,
    state: &mut AlgebraState,
    screen_ui: &mut AlgebraUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let area = ui.max_rect();
    let right_width = metrics.fs(320.0).min(area.width() * 0.36);
    let gap = metrics.fs(14.0);
    let left = Rect::from_min_max(
        area.min,
        Pos2::new(area.right() - right_width - gap, area.bottom()),
    );
    let right = Rect::from_min_max(Pos2::new(area.right() - right_width, area.top()), area.max);
    let display = Rect::from_min_size(left.min, Vec2::new(left.width(), metrics.fs(180.0)));
    algebra_display(ui, display, state, screen_ui, metrics, feedback, false);
    let notes = Rect::from_min_size(
        Pos2::new(left.left(), display.bottom() + metrics.fs(8.0)),
        Vec2::new(left.width(), metrics.controls().fs(58.0)),
    );
    notes_bar(ui, notes, state, screen_ui, metrics);
    let keypad_top = notes.bottom() + metrics.fs(8.0);
    let keypad_height = (left.bottom() - keypad_top).min(metrics.fs(360.0));
    let keypad = Rect::from_min_size(
        Pos2::new(left.left(), keypad_top),
        Vec2::new(left.width(), keypad_height),
    );
    algebra_keypad(ui, keypad, state, metrics);
    let remaining = Rect::from_min_max(
        Pos2::new(left.left(), keypad.bottom() + metrics.fs(10.0)),
        left.max,
    );
    if remaining.height() > metrics.fs(54.0) {
        let painter = ui.painter().clone();
        theme::card(
            &painter,
            remaining,
            metrics.fs(14.0),
            Palette::CARD,
            Color32::from_white_alpha(16),
        );
        theme::text(
            &painter,
            Pos2::new(
                remaining.left() + metrics.fs(14.0),
                remaining.top() + metrics.fs(14.0),
            ),
            Align2::LEFT_CENTER,
            "SOLVER NOTES",
            metrics.fs(8.0),
            Weight::Bold,
            Palette::FAINT,
        );
        let notes = [
            "Single-letter polynomials up to degree 12",
            "Exact linear and quadratic roots",
            "Rational equations keep nonzero constraints",
        ];
        let mut y = remaining.top() + metrics.fs(28.0);
        for note in notes {
            if y > remaining.bottom() - metrics.fs(8.0) {
                break;
            }
            theme::text(
                &painter,
                Pos2::new(remaining.left() + metrics.fs(14.0), y),
                Align2::LEFT_CENTER,
                format!("· {note}"),
                metrics.fs(8.0),
                Weight::Medium,
                Palette::MUTED,
            );
            y += metrics.fs(13.0);
        }
    }

    let solution_height = metrics.fs(230.0);
    let solution = Rect::from_min_size(right.min, Vec2::new(right.width(), solution_height));
    solution_card(ui, solution, state, metrics);
    let examples = Rect::from_min_max(Pos2::new(right.left(), solution.bottom() + gap), right.max);
    examples_card(ui, examples, state, metrics);
}

fn compact_screen(
    ui: &mut Ui,
    state: &mut AlgebraState,
    screen_ui: &mut AlgebraUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let area = ui.max_rect();
    let display_height = (area.height() * 0.35).clamp(metrics.fs(170.0), metrics.fs(230.0));
    let display = Rect::from_min_size(area.min, Vec2::new(area.width(), display_height));
    algebra_display(ui, display, state, screen_ui, metrics, feedback, true);
    let controls = metrics.controls();
    let notes_height = if metrics.square_keys {
        controls.fs(112.0)
    } else {
        controls.fs(62.0)
    };
    let notes = Rect::from_min_size(
        Pos2::new(area.left(), display.bottom() + metrics.fs(8.0)),
        Vec2::new(area.width(), notes_height),
    );
    if notes_height > 0.0 {
        notes_bar(ui, notes, state, screen_ui, metrics);
    }
    let keypad = Rect::from_min_max(
        Pos2::new(
            area.left(),
            if notes_height > 0.0 {
                notes.bottom() + metrics.fs(8.0)
            } else {
                display.bottom() + metrics.fs(8.0)
            },
        ),
        area.max,
    );
    algebra_keypad(ui, keypad, state, metrics);
}

fn watch_screen(
    ui: &mut Ui,
    state: &mut AlgebraState,
    screen_ui: &mut AlgebraUi,
    metrics: &Metrics,
    _feedback: &mut Feedback,
) {
    let area = ui.max_rect();
    let display_height = area.height() * 0.36;
    let display = Rect::from_min_size(area.min, Vec2::new(area.width(), display_height));
    watch_display(ui, display, state, screen_ui, metrics);
    let keypad = Rect::from_min_max(
        Pos2::new(area.left(), display.bottom() + metrics.fs(2.0)),
        area.max,
    );
    watch_keypad(ui, keypad, state, metrics);
}

fn notes_bar(
    ui: &mut Ui,
    rect: Rect,
    state: &AlgebraState,
    screen_ui: &mut AlgebraUi,
    metrics: &Metrics,
) {
    let controls = metrics.controls();
    let response = ui.interact(rect, ui.scope_id().with("algebra-notes"), Sense::click());
    let target = screen_ui
        .note_target
        .and_then(|id| state.history.iter().find(|entry| entry.id == id));
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        controls.fs(13.0),
        if response.hovered() {
            Palette::CARD_LIFT
        } else {
            Palette::CARD
        },
    );
    painter.rect_stroke(
        rect,
        controls.fs(13.0),
        Stroke::new(
            1.0,
            if response.hovered() {
                Palette::VIOLET.gamma_multiply(0.45)
            } else {
                Color32::from_white_alpha(14)
            },
        ),
        StrokeKind::Inside,
    );
    let pad = controls.fs(12.0);
    theme::text(
        painter,
        Pos2::new(rect.left() + pad, rect.top() + controls.fs(11.0)),
        Align2::LEFT_CENTER,
        "NOTES",
        controls.fs(8.0),
        Weight::Bold,
        Palette::VIOLET_SOFT,
    );
    if let Some(entry) = target {
        let summary = format!(
            "{} -> {}",
            entry.equation,
            entry
                .result
                .as_ref()
                .map(AlgebraResult::text)
                .unwrap_or_else(|| "—".to_string())
        );
        theme::fitted_text(
            painter,
            Rect::from_min_max(
                Pos2::new(
                    rect.left() + controls.fs(74.0),
                    rect.top() + controls.fs(3.0),
                ),
                Pos2::new(
                    rect.right() - controls.fs(34.0),
                    rect.top() + controls.fs(20.0),
                ),
            ),
            Align2::RIGHT_CENTER,
            &summary,
            controls.fs(8.0),
            controls.fs(6.0),
            Weight::Medium,
            Palette::FAINT,
        );
        let preview = entry.note.lines().next().unwrap_or("").trim();
        theme::fitted_text(
            painter,
            Rect::from_min_max(
                Pos2::new(rect.left() + pad, rect.top() + controls.fs(24.0)),
                Pos2::new(rect.right() - pad, rect.top() + controls.fs(52.0)),
            ),
            Align2::LEFT_CENTER,
            if preview.is_empty() {
                "Tap to add a note"
            } else {
                preview
            },
            controls.fs(11.0),
            controls.fs(7.0),
            Weight::Medium,
            if preview.is_empty() {
                Palette::FAINT
            } else {
                Palette::TEXT
            },
        );
        if !preview.is_empty() {
            theme::text(
                painter,
                Pos2::new(rect.left() + pad, rect.bottom() - controls.fs(9.0)),
                Align2::LEFT_CENTER,
                "Tap to edit",
                controls.fs(8.0),
                Weight::Medium,
                Palette::FAINT,
            );
        }
        if response.clicked() {
            screen_ui.note_draft = entry.note.clone();
            screen_ui.note_sheet = true;
        }
    } else {
        theme::text(
            painter,
            Pos2::new(rect.left() + pad, rect.center().y + controls.fs(6.0)),
            Align2::LEFT_CENTER,
            "Solve an equation to attach a note",
            controls.fs(9.0),
            Weight::Medium,
            Palette::FAINT,
        );
    }
    icons::icon(
        painter,
        Pos2::new(rect.right() - controls.fs(16.0), rect.center().y),
        controls.fs(12.0),
        Icon::ArrowRight,
        Palette::MUTED,
    );
}

fn notes_sheet(
    ui: &mut Ui,
    state: &mut AlgebraState,
    screen_ui: &mut AlgebraUi,
    metrics: &Metrics,
) {
    let Some(target_id) = screen_ui.note_target else {
        screen_ui.note_sheet = false;
        return;
    };
    let Some(entry) = state
        .history
        .iter()
        .find(|entry| entry.id == target_id)
        .cloned()
    else {
        screen_ui.note_sheet = false;
        return;
    };
    let controls = metrics.controls();
    let width = common::modal_width(ui, metrics.fs(560.0));
    let height = common::modal_max_height(ui);
    let frame = egui::Frame::NONE
        .fill(Palette::CARD)
        .stroke(Stroke::new(1.0, Palette::BORDER))
        .corner_radius(metrics.fs(20.0))
        .inner_margin(Margin::same(16));
    let mut close = false;
    let mut solve = false;
    let mut clear = false;
    let mut clear_history = false;
    let mut load = None;
    let mut delete = None;
    let response = egui::Modal::new(Id::unique("algebra-notes"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(150))
        .show(ui.ctx(), |ui| {
            ui.set_width(width);
            ui.set_max_height(height);
            if widgets::modal_header(
                ui,
                "Notes",
                &format!(
                    "{} -> {}",
                    entry.equation,
                    entry
                        .result
                        .as_ref()
                        .map(AlgebraResult::text)
                        .unwrap_or_else(|| "—".to_string())
                ),
                &controls,
                true,
            ) {
                close = true;
            }
            ui.add_space(controls.fs(5.0));
            let field_height = (height * 0.24).clamp(controls.fs(92.0), controls.fs(170.0));
            let (field_rect, _) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), field_height),
                Sense::hover(),
            );
            let response = widgets::field_multiline(
                ui,
                field_rect,
                &mut screen_ui.note_draft,
                "Write a note… type 2+2= to append the result",
                &controls,
            );
            if response.changed() {
                autocomplete_arithmetic(&mut screen_ui.note_draft);
            }
            state.set_history_note(target_id, screen_ui.note_draft.clone());
            ui.add_space(controls.fs(6.0));
            let actions = Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(ui.available_width(), controls.fs(28.0)),
            );
            ui.allocate_rect(actions, Sense::hover());
            let solve_rect =
                Rect::from_min_size(actions.min, Vec2::new(controls.fs(92.0), actions.height()));
            if widgets::pill(ui, solve_rect, "Solve", false, true, &controls).clicked() {
                solve = true;
            }
            let clear_rect = Rect::from_min_size(
                Pos2::new(solve_rect.right() + controls.fs(6.0), actions.top()),
                Vec2::new(controls.fs(64.0), actions.height()),
            );
            if widgets::pill(ui, clear_rect, "Clear", false, false, &controls).clicked() {
                clear = true;
            }
            ui.add_space(controls.fs(6.0));
            let (history_header, _) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), controls.fs(22.0)),
                Sense::hover(),
            );
            theme::text(
                ui.painter(),
                history_header.left_center(),
                Align2::LEFT_CENTER,
                "HISTORY",
                controls.fs(8.0),
                Weight::Bold,
                Palette::FAINT,
            );
            let clear_history_rect = Rect::from_min_size(
                Pos2::new(
                    history_header.right() - controls.fs(100.0),
                    history_header.top(),
                ),
                Vec2::new(controls.fs(100.0), history_header.height()),
            );
            if widgets::pill(
                ui,
                clear_history_rect,
                "Clear history",
                false,
                true,
                &controls,
            )
            .clicked()
            {
                clear_history = true;
            }
            ui.add_space(controls.fs(2.0));
            let list_height = ui.available_height().max(controls.fs(70.0));
            egui::ScrollArea::vertical()
                .max_height(list_height)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for item in state.history.iter().take(8) {
                        let (row, row_response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), controls.fs(48.0)),
                            Sense::click(),
                        );
                        let delete_width = controls.fs(34.0);
                        let content_right = row.right() - delete_width;
                        let painter = ui.painter();
                        painter.rect_filled(
                            row,
                            controls.fs(9.0),
                            if Some(item.id) == screen_ui.note_target {
                                Palette::VIOLET.gamma_multiply(0.13)
                            } else {
                                Color32::from_white_alpha(4)
                            },
                        );
                        let result = item
                            .result
                            .as_ref()
                            .map(AlgebraResult::text)
                            .unwrap_or_else(|| "—".to_string());
                        theme::fitted_text(
                            painter,
                            Rect::from_min_max(
                                Pos2::new(
                                    row.left() + controls.fs(9.0),
                                    row.top() + controls.fs(2.0),
                                ),
                                Pos2::new(content_right - controls.fs(4.0), row.center().y),
                            ),
                            Align2::LEFT_CENTER,
                            &item.equation,
                            controls.fs(9.0),
                            controls.fs(6.0),
                            Weight::Medium,
                            Palette::MUTED,
                        );
                        theme::fitted_text(
                            painter,
                            Rect::from_min_max(
                                Pos2::new(row.left() + controls.fs(9.0), row.center().y),
                                Pos2::new(
                                    content_right - controls.fs(4.0),
                                    row.bottom() - controls.fs(2.0),
                                ),
                            ),
                            Align2::LEFT_CENTER,
                            &format!(
                                "{}{}",
                                result,
                                if item.note.trim().is_empty() {
                                    String::new()
                                } else {
                                    format!("  ·  note: {}", item.note.trim())
                                }
                            ),
                            controls.fs(8.2),
                            controls.fs(5.8),
                            Weight::SemiBold,
                            Palette::TEAL_SOFT,
                        );
                        let delete_rect = Rect::from_center_size(
                            Pos2::new(row.right() - delete_width / 2.0, row.center().y),
                            Vec2::splat(controls.fs(26.0)),
                        );
                        let delete_response = ui.interact(
                            delete_rect,
                            ui.scope_id().with(("alg-notes-delete", item.id)),
                            Sense::click(),
                        );
                        if delete_response.hovered() {
                            painter.circle_filled(
                                delete_rect.center(),
                                controls.fs(13.0),
                                Color32::from_white_alpha(8),
                            );
                        }
                        icons::icon(
                            painter,
                            delete_rect.center(),
                            controls.fs(12.0),
                            Icon::Trash,
                            if delete_response.hovered() {
                                Palette::ERROR
                            } else {
                                Palette::FAINT
                            },
                        );
                        if delete_response.clicked() {
                            delete = Some(item.id);
                        } else if row_response.clicked() {
                            load = Some(item.id);
                        }
                        ui.add_space(controls.fs(3.0));
                    }
                });
        });
    if response.should_close() {
        close = true;
    }
    if solve {
        append_solution(&mut screen_ui.note_draft, state.variable);
        state.set_history_note(target_id, screen_ui.note_draft.clone());
    }
    if clear {
        screen_ui.note_draft.clear();
        state.set_history_note(target_id, String::new());
    }
    if clear_history {
        state.clear_history();
        screen_ui.note_target = None;
        screen_ui.note_draft.clear();
        screen_ui.note_sheet = false;
    }
    if let Some(id) = delete {
        state.history.retain(|entry| entry.id != id);
        if screen_ui.note_target == Some(id) {
            screen_ui.note_target = state.history.first().map(|entry| entry.id);
            screen_ui.note_draft.clear();
            screen_ui.note_sheet = false;
        }
    }
    if let Some(id) = load {
        if let Some(entry) = state.history.iter().find(|entry| entry.id == id).cloned() {
            state.load_history(&entry);
            screen_ui.note_target = Some(entry.id);
            screen_ui.note_draft = entry.note.clone();
        }
    }
    if close {
        state.set_history_note(target_id, screen_ui.note_draft.clone());
        screen_ui.note_sheet = false;
    }
}

fn autocomplete_arithmetic(text: &mut String) {
    let start = text.rfind('\n').map(|index| index + 1).unwrap_or(0);
    let line = text[start..].trim_end();
    if !line.ends_with('=') {
        return;
    }
    let expression = line[..line.len() - 1].trim_end();
    if expression.is_empty() {
        return;
    }
    if let Ok(value) = expr::evaluate(expression, qalsheet_core::AngleMode::Degrees, &[]) {
        text.replace_range(
            start..,
            &format!("{expression} = {}", format::plain_number(value)),
        );
    }
}

fn append_solution(text: &mut String, variable: char) {
    let start = text.rfind('\n').map(|index| index + 1).unwrap_or(0);
    let line = text[start..].trim_end();
    if line.is_empty() || line.contains("->") {
        return;
    }
    if line.contains('=') {
        let right = line
            .split_once('=')
            .map(|(_, right)| right.trim())
            .unwrap_or("");
        if right.is_empty() {
            return;
        }
        let result = qalsheet_core::algebra::solve(line, variable);
        if result.is_solution() || result.is_warning() {
            text.push_str("  ->  ");
            text.push_str(&result.text());
        }
    } else if let Ok(value) = expr::evaluate(line, qalsheet_core::AngleMode::Degrees, &[]) {
        text.push_str(" = ");
        text.push_str(&format::plain_number(value));
    }
}

fn algebra_display(
    ui: &mut Ui,
    rect: Rect,
    state: &mut AlgebraState,
    screen_ui: &mut AlgebraUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
    compact: bool,
) {
    let painter = ui.painter().clone();
    theme::card(
        &painter,
        rect,
        metrics.fs(18.0),
        Palette::CARD,
        Color32::from_white_alpha(20),
    );
    let pad = metrics.fs(12.0);
    let header = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(6.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(26.0)),
    );
    let button = metrics.fs(24.0);
    let mut x = header.right();
    let solve_width = if compact {
        metrics.fs(66.0)
    } else {
        metrics.fs(78.0)
    };
    let solve = Rect::from_min_size(
        Pos2::new(x - solve_width, header.center().y - metrics.fs(12.0)),
        Vec2::new(solve_width, metrics.fs(24.0)),
    );
    x -= solve_width + metrics.fs(5.0);
    if widgets::text_button(ui, solve, "Solve", true, metrics).clicked() {
        state.solve();
        feedback.show(ui.ctx(), "Equation solved");
    }
    let power = Rect::from_min_size(
        Pos2::new(x - button, header.center().y - button / 2.0),
        Vec2::splat(button),
    );
    if widgets::key(ui, power, "x^", KeyKind::Utility, metrics).clicked() {
        state.insert_power();
    }
    x -= button + metrics.fs(5.0);
    let back = Rect::from_min_size(
        Pos2::new(x - button, header.center().y - button / 2.0),
        Vec2::splat(button),
    );
    if widgets::icon_button(ui, back, Icon::Backspace, "Backspace", metrics).clicked() {
        state.press("backspace");
    }
    x -= button + metrics.fs(5.0);
    let clear = Rect::from_min_size(
        Pos2::new(x - button, header.center().y - button / 2.0),
        Vec2::splat(button),
    );
    if widgets::key(ui, clear, "C", KeyKind::Clear, metrics).clicked() {
        state.press("C");
    }
    x -= button + metrics.fs(6.0);
    let variable_width = metrics.fs(48.0);
    let variable = Rect::from_min_size(
        Pos2::new(x - variable_width, header.center().y - metrics.fs(11.0)),
        Vec2::new(variable_width, metrics.fs(22.0)),
    );
    if widgets::pill(
        ui,
        variable,
        &state.variable.to_string(),
        false,
        true,
        metrics,
    )
    .clicked()
    {
        screen_ui.show_variable_picker = true;
    }
    icons::icon(
        ui.painter(),
        Pos2::new(variable.right() - metrics.fs(7.0), variable.center().y),
        metrics.fs(7.0),
        Icon::ChevronDown,
        Palette::VIOLET_SOFT,
    );

    theme::text(
        &painter,
        Pos2::new(header.left(), header.center().y),
        Align2::LEFT_CENTER,
        "ALGEBRA",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );

    let top = state.display_text();
    let is_result = state.result.is_some();
    let top_rect = Rect::from_min_max(
        Pos2::new(rect.left() + pad, header.bottom() + metrics.fs(8.0)),
        Pos2::new(rect.right() - pad, header.bottom() + metrics.fs(28.0)),
    );
    if !state.equation.is_empty() {
        theme::fitted_text(
            &painter,
            top_rect,
            Align2::RIGHT_CENTER,
            &state.equation,
            metrics.fs(11.0),
            metrics.fs(7.0),
            Weight::Medium,
            Palette::MUTED,
        );
    }
    let result_rect = Rect::from_min_max(
        Pos2::new(rect.left() + pad, top_rect.bottom()),
        Pos2::new(rect.right() - pad, rect.bottom() - metrics.fs(26.0)),
    );
    let color = if state.is_error() {
        Palette::ERROR
    } else if state.is_solution() {
        Palette::TEAL_SOFT
    } else if state.is_warning() {
        Palette::AMBER
    } else {
        Palette::TEXT
    };
    theme::fitted_text(
        &painter,
        result_rect,
        Align2::RIGHT_CENTER,
        &top,
        metrics.fs(32.0),
        metrics.fs(13.0),
        Weight::ExtraBold,
        color,
    );
    let hint_rect = Rect::from_min_max(
        Pos2::new(rect.left() + pad, rect.bottom() - metrics.fs(26.0)),
        Pos2::new(rect.right() - pad, rect.bottom() - metrics.fs(6.0)),
    );
    theme::fitted_text(
        &painter,
        hint_rect,
        Align2::RIGHT_CENTER,
        &state.hint_text(),
        metrics.fs(8.5),
        metrics.fs(6.0),
        Weight::Medium,
        if state.is_error() {
            Palette::ERROR
        } else {
            Palette::MUTED
        },
    );
    let _ = is_result;
}

fn algebra_keypad(ui: &mut Ui, area: Rect, state: &mut AlgebraState, metrics: &Metrics) {
    let gap = metrics.fs(5.0);
    let rows = 4;
    let cells = if metrics.square_keys {
        common::square_grid(area, rows, 5, gap)
    } else {
        common::grid(area, rows, 5, gap)
    };
    let fixed: [(&str, KeyKind); 15] = [
        ("7", KeyKind::Digit),
        ("8", KeyKind::Digit),
        ("9", KeyKind::Digit),
        ("\u{00F7}", KeyKind::Operator),
        ("(", KeyKind::Operator),
        ("4", KeyKind::Digit),
        ("5", KeyKind::Digit),
        ("6", KeyKind::Digit),
        ("\u{00D7}", KeyKind::Operator),
        (")", KeyKind::Operator),
        ("1", KeyKind::Digit),
        ("2", KeyKind::Digit),
        ("3", KeyKind::Digit),
        ("\u{2212}", KeyKind::Operator),
        ("=", KeyKind::Equals),
    ];
    for (index, (label, kind)) in fixed.iter().enumerate() {
        if widgets::key(ui, cells[index], label, *kind, metrics).clicked() {
            state.press(label);
        }
    }
    let last = if metrics.square_keys {
        common::square_centered_row(area, 3, rows, 4, 5, gap)
    } else {
        common::centered_row(area, 3, rows, 4, 5, gap)
    };
    let labels: [(&str, KeyKind); 4] = [
        ("0", KeyKind::Digit),
        (".", KeyKind::Digit),
        (
            if state.variable == 'x' { "x" } else { "var" },
            KeyKind::Accent,
        ),
        ("+", KeyKind::Operator),
    ];
    for (index, (label, kind)) in labels.iter().enumerate() {
        if widgets::key(ui, last[index], label, *kind, metrics).clicked() {
            if index == 2 {
                state.insert_variable();
            } else {
                state.press(label);
            }
        }
    }
}

fn watch_display(
    ui: &mut Ui,
    rect: Rect,
    state: &mut AlgebraState,
    screen_ui: &mut AlgebraUi,
    metrics: &Metrics,
) {
    let pad = metrics.fs(8.0);
    let header = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top()),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(22.0)),
    );
    let button = metrics.fs(20.0);
    let mut x = header.right();
    let solve = Rect::from_min_size(
        Pos2::new(x - metrics.fs(46.0), header.center().y - metrics.fs(10.0)),
        Vec2::new(metrics.fs(46.0), metrics.fs(20.0)),
    );
    if widgets::pill(ui, solve, "Solve", true, false, metrics).clicked() {
        state.solve();
    }
    x -= metrics.fs(51.0);
    let power = Rect::from_min_size(
        Pos2::new(x - button, header.center().y - button / 2.0),
        Vec2::splat(button),
    );
    if widgets::key(ui, power, "x^", KeyKind::Utility, metrics).clicked() {
        state.insert_power();
    }
    x -= button + metrics.fs(3.0);
    let back = Rect::from_min_size(
        Pos2::new(x - button, header.center().y - button / 2.0),
        Vec2::splat(button),
    );
    if widgets::key_with_icon(
        ui,
        back,
        "\u{232B}",
        KeyKind::Soft,
        metrics,
        Some(Icon::Backspace),
    )
    .clicked()
    {
        state.press("backspace");
    }
    x -= button + metrics.fs(3.0);
    let clear = Rect::from_min_size(
        Pos2::new(x - button, header.center().y - button / 2.0),
        Vec2::splat(button),
    );
    if widgets::key(ui, clear, "C", KeyKind::Clear, metrics).clicked() {
        state.press("C");
    }
    let variable = Rect::from_min_size(
        Pos2::new(header.left(), header.center().y - metrics.fs(9.0)),
        Vec2::new(metrics.fs(40.0), metrics.fs(18.0)),
    );
    if widgets::pill(
        ui,
        variable,
        &state.variable.to_string(),
        false,
        true,
        metrics,
    )
    .clicked()
    {
        screen_ui.show_variable_picker = true;
    }
    icons::icon(
        ui.painter(),
        Pos2::new(variable.right() - metrics.fs(7.0), variable.center().y),
        metrics.fs(7.0),
        Icon::ChevronDown,
        Palette::VIOLET_SOFT,
    );

    let result_rect = Rect::from_min_max(
        Pos2::new(
            rect.left() + metrics.fs(8.0),
            header.bottom() + metrics.fs(2.0),
        ),
        Pos2::new(
            rect.right() - metrics.fs(8.0),
            rect.bottom() - metrics.fs(16.0),
        ),
    );
    theme::fitted_text(
        ui.painter(),
        result_rect,
        Align2::RIGHT_CENTER,
        &state.display_text(),
        metrics.fs(22.0),
        metrics.fs(10.0),
        Weight::ExtraBold,
        if state.is_error() {
            Palette::ERROR
        } else if state.is_solution() {
            Palette::TEAL_SOFT
        } else if state.is_warning() {
            Palette::AMBER
        } else {
            Palette::TEXT
        },
    );
    theme::fitted_text(
        ui.painter(),
        Rect::from_min_max(
            Pos2::new(
                rect.left() + metrics.fs(8.0),
                rect.bottom() - metrics.fs(18.0),
            ),
            Pos2::new(
                rect.right() - metrics.fs(8.0),
                rect.bottom() - metrics.fs(2.0),
            ),
        ),
        Align2::RIGHT_CENTER,
        &state.hint_text(),
        metrics.fs(7.0),
        metrics.fs(5.5),
        Weight::Medium,
        if state.is_error() {
            Palette::ERROR
        } else {
            Palette::MUTED
        },
    );
}

fn watch_keypad(ui: &mut Ui, area: Rect, state: &mut AlgebraState, metrics: &Metrics) {
    let gap = metrics.fs(3.0);
    let cells = common::grid(area, 4, 5, gap);
    let fixed: [[(&str, KeyKind); 5]; 3] = [
        [
            ("7", KeyKind::Digit),
            ("8", KeyKind::Digit),
            ("9", KeyKind::Digit),
            ("\u{00F7}", KeyKind::Operator),
            ("(", KeyKind::Operator),
        ],
        [
            ("4", KeyKind::Digit),
            ("5", KeyKind::Digit),
            ("6", KeyKind::Digit),
            ("\u{00D7}", KeyKind::Operator),
            (")", KeyKind::Operator),
        ],
        [
            ("1", KeyKind::Digit),
            ("2", KeyKind::Digit),
            ("3", KeyKind::Digit),
            ("\u{2212}", KeyKind::Operator),
            ("=", KeyKind::Equals),
        ],
    ];
    for row in 0..3 {
        for col in 0..5 {
            let (label, kind) = fixed[row][col];
            if widgets::key(ui, cells[row * 5 + col], label, kind, metrics).clicked() {
                state.press(label);
            }
        }
    }
    let key_width = (area.width() - gap * 4.0) / 5.0;
    let start = area.left() + (key_width + gap) * 0.5;
    let y = cells[15].top();
    let bottom: [(&str, KeyKind); 4] = [
        ("0", KeyKind::Digit),
        (".", KeyKind::Digit),
        (
            if state.variable == 'x' { "x" } else { "var" },
            KeyKind::Accent,
        ),
        ("+", KeyKind::Operator),
    ];
    for (index, (label, kind)) in bottom.iter().enumerate() {
        let rect = Rect::from_min_size(
            Pos2::new(start + index as f32 * (key_width + gap), y),
            Vec2::new(key_width, cells[15].height()),
        );
        if widgets::key(ui, rect, label, *kind, metrics).clicked() {
            if index == 2 {
                state.insert_variable();
            } else {
                state.press(label);
            }
        }
    }
}

fn solution_card(ui: &mut Ui, rect: Rect, state: &AlgebraState, metrics: &Metrics) {
    let painter = ui.painter().clone();
    theme::card(
        &painter,
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(20),
    );
    let pad = metrics.fs(14.0);
    theme::text(
        &painter,
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(17.0)),
        Align2::LEFT_CENTER,
        "SOLUTION",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let result_rect = Rect::from_min_max(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(32.0)),
        Pos2::new(rect.right() - pad, rect.top() + metrics.fs(88.0)),
    );
    theme::fitted_text(
        &painter,
        result_rect,
        Align2::LEFT_CENTER,
        &state.display_text(),
        metrics.fs(30.0),
        metrics.fs(12.0),
        Weight::ExtraBold,
        if state.is_error() {
            Palette::ERROR
        } else if state.is_solution() {
            Palette::TEAL_SOFT
        } else if state.is_warning() {
            Palette::AMBER
        } else {
            Palette::TEXT
        },
    );
    let hint_rect = Rect::from_min_max(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(92.0)),
        Pos2::new(rect.right() - pad, rect.bottom() - metrics.fs(12.0)),
    );
    let hint = state.hint_text();
    let mut y = hint_rect.top() + metrics.fs(8.0);
    let lines: Vec<&str> = if hint.contains('\n') {
        hint.lines().collect()
    } else {
        vec![hint.as_str()]
    };
    for line in lines {
        if y > hint_rect.bottom() {
            break;
        }
        theme::text(
            &painter,
            Pos2::new(hint_rect.left(), y),
            Align2::LEFT_CENTER,
            line,
            metrics.fs(9.0),
            Weight::Medium,
            if state.is_error() {
                Palette::ERROR
            } else {
                Palette::MUTED
            },
        );
        y += metrics.fs(14.0);
    }
    if let Some(result) = &state.result {
        let detail = match result {
            AlgebraResult::Solved { .. } => "One real solution",
            AlgebraResult::SolvedRoots { values, .. } => {
                if values.len() == 2 {
                    "Two real roots"
                } else {
                    "Multiple real roots"
                }
            }
            AlgebraResult::Expression { .. } => "Symbolic solution",
            AlgebraResult::NoSolution => "The equation has no solution",
            AlgebraResult::NoRealSolution => "The discriminant is negative",
            AlgebraResult::InfiniteSolutions => "Both sides are identical",
            AlgebraResult::Error(_) => "Check the equation",
        };
        let chip = Rect::from_min_size(
            Pos2::new(rect.left() + pad, hint_rect.bottom() - metrics.fs(20.0)),
            Vec2::new(rect.width() - pad * 2.0, metrics.fs(18.0)),
        );
        painter.rect_filled(chip, metrics.fs(9.0), Color32::from_white_alpha(8));
        theme::text(
            &painter,
            chip.center(),
            Align2::CENTER_CENTER,
            detail,
            metrics.fs(8.0),
            Weight::SemiBold,
            Palette::VIOLET_SOFT,
        );
    }
}

fn examples_card(ui: &mut Ui, rect: Rect, state: &mut AlgebraState, metrics: &Metrics) {
    let painter = ui.painter().clone();
    theme::card(
        &painter,
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(20),
    );
    let pad = metrics.fs(14.0);
    theme::text(
        &painter,
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(16.0)),
        Align2::LEFT_CENTER,
        "TRY AN EXAMPLE",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let examples = [
        "3x + 5 = 20",
        "x^2 = 4",
        "5/x = 2/3",
        "2x + 3y = 12",
        "x^3 - 6x^2 + 11x - 6 = 0",
        "4[y - 3(2y - 1)] = 2[y - 2(3y + 2)]",
    ];
    let mut y = rect.top() + metrics.fs(30.0);
    let row_height = metrics.fs(30.0);
    for example in examples {
        if y + row_height > rect.bottom() - metrics.fs(14.0) {
            break;
        }
        let row = Rect::from_min_size(
            Pos2::new(rect.left() + metrics.fs(8.0), y),
            Vec2::new(
                rect.width() - metrics.fs(16.0),
                row_height - metrics.fs(3.0),
            ),
        );
        let id = ui.scope_id().with(("example", example));
        let response = ui.interact(row, id, Sense::click());
        if response.hovered() {
            ui.painter()
                .rect_filled(row, metrics.fs(8.0), Color32::from_white_alpha(8));
        }
        theme::text(
            ui.painter(),
            Pos2::new(row.left() + metrics.fs(8.0), row.center().y),
            Align2::LEFT_CENTER,
            example,
            metrics.fs(9.0),
            Weight::Medium,
            Palette::MUTED,
        );
        icons::icon(
            ui.painter(),
            Pos2::new(row.right() - metrics.fs(10.0), row.center().y),
            metrics.fs(10.0),
            Icon::ArrowRight,
            Palette::FAINT,
        );
        if response.clicked() {
            state.set_equation(example);
            state.solve();
        }
        y += row_height;
    }
    theme::text(
        &painter,
        Pos2::new(rect.left() + pad, rect.bottom() - metrics.fs(12.0)),
        Align2::LEFT_CENTER,
        "Single-letter polynomial, any letter",
        metrics.fs(7.5),
        Weight::Medium,
        Palette::FAINT,
    );
    let _ = pad;
}

fn variable_picker(
    ui: &mut Ui,
    state: &mut AlgebraState,
    screen_ui: &mut AlgebraUi,
    metrics: &Metrics,
) {
    let width = common::modal_width(ui, metrics.fs(420.0));
    let frame = egui::Frame::NONE
        .fill(Palette::CARD)
        .stroke(Stroke::new(1.0, Palette::BORDER))
        .corner_radius(metrics.fs(20.0))
        .inner_margin(Margin::same(16));
    let mut close = false;
    let mut selected = None;
    let response = egui::Modal::new(Id::unique("algebra-variables"))
        .frame(frame)
        .show(ui.ctx(), |ui| {
            ui.set_width(width);
            ui.set_max_height(common::modal_max_height(ui));
            if widgets::modal_header(
                ui,
                "Variable",
                "Choose the letter to solve for",
                metrics,
                true,
            ) {
                close = true;
            }
            ui.add_space(metrics.fs(8.0));
            let letters: Vec<char> = ('a'..='z').collect();
            let gap = if metrics.watch {
                metrics.fs(3.0)
            } else {
                metrics.fs(5.0)
            };
            let size = if metrics.watch {
                metrics.fs(23.0)
            } else {
                metrics.fs(40.0)
            };
            let columns = 6;
            let rows = letters.len().div_ceil(columns);
            let grid = Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(ui.available_width(), rows as f32 * (size + gap)),
            );
            ui.allocate_rect(grid, Sense::hover());
            for (index, letter) in letters.iter().enumerate() {
                let row = index / columns;
                let col = index % columns;
                let cell = Rect::from_min_size(
                    Pos2::new(
                        grid.left()
                            + col as f32
                                * ((grid.width() - gap * (columns as f32 - 1.0)) / columns as f32
                                    + gap),
                        grid.top() + row as f32 * (size + gap),
                    ),
                    Vec2::new(
                        (grid.width() - gap * (columns as f32 - 1.0)) / columns as f32,
                        size,
                    ),
                );
                let active = *letter == state.variable;
                let response = widgets::key(
                    ui,
                    cell,
                    &letter.to_string(),
                    if active {
                        KeyKind::Accent
                    } else {
                        KeyKind::Soft
                    },
                    metrics,
                );
                if response.clicked() {
                    selected = Some(*letter);
                }
            }
        });
    if response.should_close() {
        close = true;
    }
    if let Some(letter) = selected {
        state.select_variable(letter);
        close = true;
    }
    if close {
        screen_ui.show_variable_picker = false;
    }
}
