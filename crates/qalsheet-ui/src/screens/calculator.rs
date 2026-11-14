//! Calculator screen, including the watch-sized keypad and history drawer.

use eframe::egui::{
    self, Align2, Color32, Id, Margin, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use qalsheet_core::calculator::{CalculatorState, FunctionKey};
use qalsheet_core::expr;
use qalsheet_core::format;

use crate::app::Feedback;
use crate::icons::{self, Icon};
use crate::screens::common;
use crate::theme::{self, Metrics, Palette, Weight};
use crate::widgets::{self, KeyKind};

#[derive(Default)]
pub struct CalculatorUi {
    pub show_history: bool,
    pub note_target: Option<u64>,
    pub note_head: Option<u64>,
    pub note_sheet: bool,
    pub note_draft: String,
    note_focus: bool,
}

pub fn screen(
    ui: &mut Ui,
    state: &mut CalculatorState,
    screen_ui: &mut CalculatorUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let area = ui.max_rect();
    sync_note_target(state, screen_ui);
    if metrics.watch {
        watch_calculator(ui, area, state, screen_ui, metrics, feedback);
    } else {
        full_calculator(ui, area, state, screen_ui, metrics, feedback);
    }
}

fn sync_note_target(state: &CalculatorState, screen_ui: &mut CalculatorUi) {
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

fn full_calculator(
    ui: &mut Ui,
    area: Rect,
    state: &mut CalculatorState,
    screen_ui: &mut CalculatorUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let wide = !metrics.compact && area.width() >= 760.0;
    let side_width = if wide {
        metrics.fs(300.0).min(area.width() * 0.36)
    } else {
        0.0
    };
    let gap = metrics.fs(12.0);
    let left = Rect::from_min_max(
        area.min,
        Pos2::new(
            area.right() - side_width - if wide { gap } else { 0.0 },
            area.bottom(),
        ),
    );

    let display_height = if wide {
        metrics.fs(190.0)
    } else {
        (area.height() * 0.30).clamp(metrics.fs(136.0), metrics.fs(188.0))
    };
    let display = Rect::from_min_size(left.min, Vec2::new(left.width(), display_height));
    display_card(ui, display, state, metrics, feedback, screen_ui);

    let body_top = display.bottom() + metrics.fs(10.0);
    let controls = metrics.controls();
    let notes_height = if !state.show_scientific {
        controls.fs(58.0)
    } else {
        0.0
    };
    let notes_rect = Rect::from_min_size(
        Pos2::new(left.left(), body_top),
        Vec2::new(left.width(), notes_height),
    );
    let keypad_top = if notes_height > 0.0 {
        notes_rect.bottom() + metrics.fs(8.0)
    } else {
        body_top
    };
    let memory_height = if metrics.watch {
        0.0
    } else if metrics.square_keys {
        controls.fs(27.0)
    } else {
        metrics.fs(27.0)
    };
    let keypad_bottom = left.bottom()
        - memory_height
        - if memory_height > 0.0 {
            metrics.fs(8.0)
        } else {
            0.0
        };
    let mut keypad_area = Rect::from_min_max(
        Pos2::new(left.left(), keypad_top),
        Pos2::new(left.right(), keypad_bottom),
    );
    if notes_height > 0.0 {
        notes_bar(ui, notes_rect, state, screen_ui, metrics);
    }
    if wide {
        // Keep huge desktop windows from producing absurdly tall keys.
        let max_height = metrics.fs(500.0);
        if keypad_area.height() > max_height {
            keypad_area =
                Rect::from_min_size(keypad_area.min, Vec2::new(keypad_area.width(), max_height));
        }
    }

    if state.show_scientific && !wide {
        science_keypad(ui, keypad_area, state, metrics);
    } else {
        main_keypad(ui, keypad_area, state, metrics);
    }

    if memory_height > 0.0 {
        let memory_rect = Rect::from_min_size(
            Pos2::new(left.left(), keypad_bottom + metrics.fs(8.0)),
            Vec2::new(left.width(), memory_height),
        );
        memory_row(ui, memory_rect, state, metrics, feedback);
    }

    if wide {
        let right = Rect::from_min_max(Pos2::new(area.right() - side_width, area.top()), area.max);
        let functions_height = (right.height() * 0.47).clamp(metrics.fs(300.0), metrics.fs(380.0));
        let functions = Rect::from_min_size(right.min, Vec2::new(right.width(), functions_height));
        functions_panel(ui, functions, state, metrics);
        let history =
            Rect::from_min_max(Pos2::new(right.left(), functions.bottom() + gap), right.max);
        history_panel(ui, history, state, screen_ui, metrics, feedback);
    } else if !state.show_scientific && !metrics.compact {
        // A little keyboard hint fills the otherwise empty bottom edge.
        let hint = Rect::from_min_size(
            Pos2::new(left.left(), left.bottom() - metrics.fs(16.0)),
            Vec2::new(left.width(), metrics.fs(12.0)),
        );
        if hint.top() > keypad_area.bottom() + metrics.fs(2.0) {
            theme::text(
                ui.painter(),
                hint.center(),
                Align2::CENTER_CENTER,
                "Keyboard: digits · + − * / · Enter = · Esc clear",
                metrics.fs(8.0),
                Weight::Medium,
                Palette::FAINT,
            );
        }
    }

    if screen_ui.show_history {
        history_modal(ui, state, screen_ui, metrics, feedback);
    }
    if screen_ui.note_sheet {
        notes_sheet(ui, state, screen_ui, metrics);
    }
}

fn display_card(
    ui: &mut Ui,
    rect: Rect,
    state: &mut CalculatorState,
    metrics: &Metrics,
    feedback: &mut Feedback,
    screen_ui: &mut CalculatorUi,
) {
    let painter = ui.painter().clone();
    theme::card(
        &painter,
        rect,
        metrics.fs(18.0),
        Palette::CARD_LIFT,
        Color32::from_white_alpha(38),
    );
    // Violet top edge like the watch's display panel.
    painter.line_segment(
        [
            Pos2::new(rect.left() + metrics.fs(16.0), rect.top() + 1.0),
            Pos2::new(rect.right() - metrics.fs(16.0), rect.top() + 1.0),
        ],
        Stroke::new(1.0, Palette::VIOLET.gamma_multiply(0.55)),
    );
    let pad = metrics.fs(12.0);
    let controls = metrics.controls();
    let header_height = controls.fs(24.0);
    let header = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(7.0)),
        Vec2::new(rect.width() - pad * 2.0, header_height),
    );

    let switch_width = controls.fs(72.0);
    let switch_rect = Rect::from_min_size(header.min, Vec2::new(switch_width, header_height));
    let selected = if state.show_scientific { 1 } else { 0 };
    if let Some(index) = widgets::segmented(ui, switch_rect, &["123", "fx"], selected, &controls) {
        state.show_scientific_page(index == 1);
    }

    let right_icon = controls.fs(26.0);
    let icon_gap = controls.fs(4.0);
    let mut x = header.right();
    let history_rect = Rect::from_min_size(
        Pos2::new(x - right_icon, header.center().y - right_icon / 2.0),
        Vec2::splat(right_icon),
    );
    if widgets::icon_button(ui, history_rect, Icon::History, "History", &controls).clicked() {
        screen_ui.show_history = true;
    }
    x -= right_icon + icon_gap;

    let copy_rect = Rect::from_min_size(
        Pos2::new(x - right_icon, header.center().y - right_icon / 2.0),
        Vec2::splat(right_icon),
    );
    if widgets::icon_button(ui, copy_rect, Icon::Copy, "Copy", &controls).clicked() {
        ui.ctx().copy_text(state.copy_text());
        feedback.show(ui.ctx(), "Result copied");
    }
    x -= right_icon + icon_gap;

    if let Some(memory) = state.memory() {
        let width = controls.fs(60.0);
        let badge_height = controls.fs(20.0);
        let badge = Rect::from_min_size(
            Pos2::new(x - width, header.center().y - badge_height / 2.0),
            Vec2::new(width, badge_height),
        );
        painter.rect_filled(
            badge,
            badge.height() * 0.5,
            Palette::VIOLET.gamma_multiply(0.16),
        );
        theme::fitted_text(
            &painter,
            badge,
            Align2::CENTER_CENTER,
            &format!("M {}", format::plain_number(memory)),
            controls.fs(9.0),
            controls.fs(6.0),
            Weight::SemiBold,
            Palette::VIOLET_SOFT,
        );
        x -= width + icon_gap;
    }

    let tag_width = controls.fs(46.0);
    let tag_height = controls.fs(22.0);
    let tag = Rect::from_min_size(
        Pos2::new(x - tag_width, header.center().y - tag_height / 2.0),
        Vec2::new(tag_width, tag_height),
    );
    let tag_response = ui.interact(tag, ui.scope_id().with("angle-tag"), Sense::click());
    let active = state.angle_mode == qalsheet_core::AngleMode::Radians;
    painter.rect_filled(
        tag,
        tag.height() * 0.5,
        if active {
            Palette::TEAL.gamma_multiply(0.16)
        } else {
            Palette::CARD_LIFT
        },
    );
    painter.rect_stroke(
        tag,
        tag.height() * 0.5,
        Stroke::new(
            1.0,
            if active {
                Palette::TEAL.gamma_multiply(0.35)
            } else {
                Color32::from_white_alpha(16)
            },
        ),
        StrokeKind::Inside,
    );
    theme::text(
        &painter,
        tag.center(),
        Align2::CENTER_CENTER,
        state.angle_mode.label(),
        controls.fs(9.0),
        Weight::Bold,
        if active {
            Palette::TEAL_SOFT
        } else {
            Palette::VIOLET_SOFT
        },
    );
    if tag_response.clicked() {
        state.toggle_angle_mode();
        feedback.show(ui.ctx(), format!("Angle mode {}", state.angle_mode.label()));
    }

    let top = state.top_text();
    let mut load_history = None;
    let result_height = controls.fs(48.0);
    let result_rect = Rect::from_min_max(
        Pos2::new(
            rect.left() + pad,
            rect.bottom() - result_height - metrics.fs(4.0),
        ),
        Pos2::new(rect.right() - pad, rect.bottom() - metrics.fs(6.0)),
    );
    let tape = Rect::from_min_max(
        Pos2::new(rect.left() + pad, header.bottom() + metrics.fs(2.0)),
        Pos2::new(rect.right() - pad, result_rect.top() - metrics.fs(2.0)),
    );
    draw_display_tape(
        ui,
        &painter,
        tape,
        state,
        &controls,
        &top,
        &mut load_history,
    );

    let display = state.display_text();
    let color = if state.is_error() {
        Palette::ERROR
    } else {
        Palette::TEXT
    };
    theme::fitted_text(
        &painter,
        result_rect,
        Align2::RIGHT_CENTER,
        &display,
        metrics.fs(38.0),
        metrics.fs(16.0),
        Weight::ExtraBold,
        color,
    );

    if let Some(index) = load_history {
        if let Some(entry) = state.history.get(index).cloned() {
            state.load_history(&entry);
            screen_ui.note_target = Some(entry.id);
            screen_ui.note_draft = entry.note.clone();
        }
    }
}

fn draw_display_tape(
    ui: &mut Ui,
    painter: &egui::Painter,
    rect: Rect,
    state: &CalculatorState,
    metrics: &Metrics,
    top: &str,
    load_history: &mut Option<usize>,
) {
    if top == "qalsheet" && state.history.is_empty() {
        theme::text(
            painter,
            rect.center(),
            Align2::CENTER_CENTER,
            "QALSHEET",
            metrics.fs(8.5),
            Weight::Bold,
            Palette::VIOLET_SOFT,
        );
        theme::text(
            painter,
            rect.center() + Vec2::new(0.0, metrics.fs(13.0)),
            Align2::CENTER_CENTER,
            "Recent calculations appear here",
            metrics.fs(7.5),
            Weight::Medium,
            Palette::FAINT,
        );
        return;
    }

    theme::text(
        painter,
        Pos2::new(rect.left(), rect.top() + metrics.fs(3.0)),
        Align2::LEFT_TOP,
        "RECENT",
        metrics.fs(7.0),
        Weight::Bold,
        Palette::FAINT,
    );

    let row_height = metrics.fs(17.0);
    let mut y = rect.bottom();
    if top != "qalsheet" {
        y -= row_height;
        let row = Rect::from_min_max(
            Pos2::new(rect.left(), y),
            Pos2::new(rect.right(), y + row_height),
        );
        let color = if state.top_is_preview() {
            Palette::VIOLET_SOFT
        } else {
            Palette::MUTED
        };
        theme::fitted_text(
            painter,
            row,
            Align2::RIGHT_CENTER,
            top,
            metrics.fs(10.5),
            metrics.fs(7.0),
            Weight::Medium,
            color,
        );
    }

    let mut entries: Vec<usize> = state
        .history
        .iter()
        .enumerate()
        .rev()
        .map(|(index, _)| index)
        .collect();
    if top != "qalsheet" {
        if let Some(index) = entries.first().copied() {
            if state.history[index].expression == top {
                entries.remove(0);
            }
        }
    }

    for index in entries {
        if y - row_height < rect.top() + metrics.fs(12.0) {
            break;
        }
        y -= row_height;
        let row = Rect::from_min_max(
            Pos2::new(rect.left(), y),
            Pos2::new(rect.right(), y + row_height),
        );
        let response = ui.interact(
            row,
            ui.scope_id().with(("display-tape", index)),
            Sense::click(),
        );
        if response.hovered() {
            painter.rect_filled(row, metrics.fs(4.0), Color32::from_white_alpha(6));
        }
        let entry = &state.history[index];
        theme::fitted_text(
            painter,
            row,
            Align2::RIGHT_CENTER,
            &format!("{} = {}", entry.expression, entry.result),
            metrics.fs(9.5),
            metrics.fs(6.5),
            Weight::Medium,
            if response.hovered() {
                Palette::TEXT
            } else {
                Palette::MUTED
            },
        );
        if response.clicked() {
            *load_history = Some(index);
        }
    }
}

fn notes_bar(
    ui: &mut Ui,
    rect: Rect,
    state: &CalculatorState,
    screen_ui: &mut CalculatorUi,
    metrics: &Metrics,
) {
    let controls = metrics.controls();
    let response = ui.interact(rect, ui.scope_id().with("calculator-notes"), Sense::click());
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
        let summary = format!("{} = {}", entry.expression, entry.result);
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
                Pos2::new(rect.right() - pad, rect.bottom() - controls.fs(5.0)),
            ),
            Align2::LEFT_CENTER,
            if preview.is_empty() {
                "Tap to add a note"
            } else {
                preview
            },
            controls.fs(10.5),
            controls.fs(7.0),
            Weight::Medium,
            if preview.is_empty() {
                Palette::FAINT
            } else {
                Palette::TEXT
            },
        );
        if response.clicked() {
            screen_ui.note_draft = entry.note.clone();
            screen_ui.note_sheet = true;
            screen_ui.note_focus = true;
        }
    } else {
        theme::text(
            painter,
            Pos2::new(rect.left() + pad, rect.center().y + controls.fs(6.0)),
            Align2::LEFT_CENTER,
            "Run a calculation to attach a note",
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
    state: &mut CalculatorState,
    screen_ui: &mut CalculatorUi,
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
    let mut calculate = false;
    let mut clear = false;
    let mut clear_history = false;
    let mut load = None;
    let mut delete = None;
    let response = egui::Modal::new(Id::unique("calculator-notes"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(150))
        .show(ui.ctx(), |ui| {
            ui.set_width(width);
            ui.set_max_height(height);
            if widgets::modal_header(
                ui,
                "Notes",
                &format!("{} = {}", entry.expression, entry.result),
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
            let field_response = widgets::field_multiline(
                ui,
                field_rect,
                &mut screen_ui.note_draft,
                "Write a note… type 2+2= to append the result",
                &controls,
            );
            if screen_ui.note_focus {
                field_response.request_focus();
                screen_ui.note_focus = false;
            }
            if field_response.changed() {
                autocomplete_arithmetic(&mut screen_ui.note_draft, state.angle_mode);
            }
            state.set_history_note(target_id, screen_ui.note_draft.clone());
            ui.add_space(controls.fs(6.0));
            let actions = Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(ui.available_width(), controls.fs(28.0)),
            );
            ui.allocate_rect(actions, Sense::hover());
            let calculate_rect =
                Rect::from_min_size(actions.min, Vec2::new(controls.fs(100.0), actions.height()));
            if widgets::pill(ui, calculate_rect, "Calculate", false, true, &controls).clicked() {
                calculate = true;
            }
            let clear_rect = Rect::from_min_size(
                Pos2::new(calculate_rect.right() + controls.fs(6.0), actions.top()),
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
                            Vec2::new(ui.available_width(), history_row_height(metrics)),
                            Sense::click(),
                        );
                        let delete_width = controls.fs(34.0);
                        let content = Rect::from_min_max(
                            row.min,
                            Pos2::new(row.right() - delete_width, row.bottom()),
                        );
                        history_row(
                            ui,
                            content,
                            item.expression.clone(),
                            item.result.clone(),
                            item.angle_mode.map(|mode| mode.label()),
                            &item.note,
                            metrics,
                        );
                        let delete_rect = Rect::from_center_size(
                            Pos2::new(row.right() - delete_width / 2.0, row.center().y),
                            Vec2::splat(controls.fs(26.0)),
                        );
                        let delete_response = ui.interact(
                            delete_rect,
                            ui.scope_id().with(("calc-notes-delete", item.id)),
                            Sense::click(),
                        );
                        if delete_response.hovered() {
                            ui.painter().circle_filled(
                                delete_rect.center(),
                                controls.fs(13.0),
                                Color32::from_white_alpha(8),
                            );
                        }
                        icons::icon(
                            ui.painter(),
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
    if calculate {
        autocomplete_arithmetic(&mut screen_ui.note_draft, state.angle_mode);
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

fn autocomplete_arithmetic(text: &mut String, angle: qalsheet_core::AngleMode) {
    let start = text.rfind('\n').map(|index| index + 1).unwrap_or(0);
    let line = text[start..].trim_end();
    if !line.ends_with('=') {
        return;
    }
    let expression = line[..line.len() - 1].trim_end();
    if expression.is_empty() {
        return;
    }
    if let Ok(value) = expr::evaluate(expression, angle, &[]) {
        let result = format::plain_number(value);
        let replacement = format!("{expression} = {result}");
        text.replace_range(start.., &replacement);
    }
}

fn main_keypad(ui: &mut Ui, area: Rect, state: &mut CalculatorState, metrics: &Metrics) {
    let gap = metrics.fs(5.0);
    let cells = if metrics.square_keys {
        common::square_grid(area, 5, 5, gap)
    } else {
        common::grid(area, 5, 5, gap)
    };
    let specs: [(&str, KeyKind); 25] = [
        ("C", KeyKind::Clear),
        ("\u{232B}", KeyKind::Soft),
        ("(", KeyKind::Operator),
        (")", KeyKind::Operator),
        ("%", KeyKind::Utility),
        ("7", KeyKind::Digit),
        ("8", KeyKind::Digit),
        ("9", KeyKind::Digit),
        ("\u{00F7}", KeyKind::Operator),
        ("^", KeyKind::Operator),
        ("4", KeyKind::Digit),
        ("5", KeyKind::Digit),
        ("6", KeyKind::Digit),
        ("\u{00D7}", KeyKind::Operator),
        ("\u{221A}", KeyKind::Utility),
        ("1", KeyKind::Digit),
        ("2", KeyKind::Digit),
        ("3", KeyKind::Digit),
        ("\u{2212}", KeyKind::Operator),
        ("\u{03C0}", KeyKind::Constant),
        ("\u{00B1}", KeyKind::Utility),
        ("0", KeyKind::Digit),
        (".", KeyKind::Digit),
        ("+", KeyKind::Operator),
        ("=", KeyKind::Equals),
    ];
    for (index, (label, kind)) in specs.iter().enumerate() {
        let rect = cells[index];
        let icon = (*label == "\u{232B}").then_some(Icon::Backspace);
        let response = widgets::key_with_icon(ui, rect, label, *kind, metrics, icon);
        if response.clicked() {
            press_main(state, label);
        }
    }
}

fn watch_calculator(
    ui: &mut Ui,
    area: Rect,
    state: &mut CalculatorState,
    screen_ui: &mut CalculatorUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let header = top_header(ui, area, state, metrics, screen_ui);
    let display_height = (area.height() * 0.30).clamp(metrics.fs(40.0), metrics.fs(58.0));
    let display = Rect::from_min_size(
        Pos2::new(area.left(), header.bottom() + metrics.fs(2.0)),
        Vec2::new(area.width(), display_height),
    );
    watch_display(ui, display, state, metrics);

    let keypad = Rect::from_min_max(
        Pos2::new(area.left(), display.bottom() + metrics.fs(2.0)),
        area.max,
    );
    if state.show_scientific {
        watch_science_keypad(ui, keypad, state, metrics);
    } else {
        watch_main_keypad(ui, keypad, state, metrics);
    }
    if screen_ui.show_history {
        history_modal(ui, state, screen_ui, metrics, feedback);
    }
}

fn top_header(
    ui: &mut Ui,
    area: Rect,
    state: &mut CalculatorState,
    metrics: &Metrics,
    screen_ui: &mut CalculatorUi,
) -> Rect {
    let header = Rect::from_min_size(
        Pos2::new(area.left() + metrics.fs(14.0), area.top()),
        Vec2::new(area.width() - metrics.fs(28.0), metrics.fs(18.0)),
    );
    let switch = Rect::from_min_size(header.min, Vec2::new(metrics.fs(50.0), metrics.fs(18.0)));
    // A single generous hit target is easier on a round watch than two tiny
    // segmented cells. The two labels are still drawn for clarity.
    let page_response = ui.interact(
        switch,
        ui.scope_id().with("watch-page-toggle"),
        Sense::click(),
    );
    if page_response.clicked() {
        state.show_scientific_page(!state.show_scientific);
    }
    let painter = ui.painter();
    painter.rect_filled(switch, switch.height() * 0.5, Palette::CARD_LIFT);
    painter.rect_stroke(
        switch,
        switch.height() * 0.5,
        Stroke::new(1.0, Color32::from_white_alpha(20)),
        StrokeKind::Inside,
    );
    let half = switch.width() / 2.0;
    let active = if state.show_scientific { 1 } else { 0 };
    painter.rect_filled(
        Rect::from_min_size(
            Pos2::new(switch.left() + active as f32 * half, switch.top()),
            Vec2::new(half, switch.height()),
        ),
        switch.height() * 0.5,
        Palette::VIOLET,
    );
    theme::text(
        painter,
        Pos2::new(switch.left() + half * 0.5, switch.center().y),
        Align2::CENTER_CENTER,
        "123",
        metrics.fs(7.0),
        Weight::Bold,
        if active == 0 {
            Palette::INK
        } else {
            Palette::MUTED
        },
    );
    theme::text(
        painter,
        Pos2::new(switch.left() + half * 1.5, switch.center().y),
        Align2::CENTER_CENTER,
        "fx",
        metrics.fs(7.0),
        Weight::Bold,
        if active == 1 {
            Palette::INK
        } else {
            Palette::MUTED
        },
    );
    let tag = Rect::from_min_size(
        Pos2::new(header.right() - metrics.fs(30.0), header.top()),
        Vec2::new(metrics.fs(30.0), metrics.fs(18.0)),
    );
    let response = ui.interact(tag, ui.scope_id().with("watch-angle"), Sense::click());
    ui.painter().rect_filled(
        tag,
        tag.height() * 0.5,
        if state.angle_mode == qalsheet_core::AngleMode::Radians {
            Palette::TEAL.gamma_multiply(0.16)
        } else {
            Palette::CARD_LIFT
        },
    );
    ui.painter().rect_stroke(
        tag,
        tag.height() * 0.5,
        Stroke::new(1.0, Color32::from_white_alpha(20)),
        StrokeKind::Inside,
    );
    theme::text(
        ui.painter(),
        tag.center(),
        Align2::CENTER_CENTER,
        state.angle_mode.label(),
        metrics.fs(7.5),
        Weight::Bold,
        if state.angle_mode == qalsheet_core::AngleMode::Radians {
            Palette::TEAL_SOFT
        } else {
            Palette::VIOLET_SOFT
        },
    );
    if response.clicked() {
        state.toggle_angle_mode();
    }
    if state.history.is_empty() {
        return header;
    }
    let history = Rect::from_min_size(
        Pos2::new(tag.left() - metrics.fs(20.0), tag.top()),
        Vec2::splat(metrics.fs(18.0)),
    );
    if widgets::icon_button(ui, history, Icon::History, "History", metrics).clicked() {
        screen_ui.show_history = true;
    }
    header
}

fn watch_display(ui: &mut Ui, rect: Rect, state: &CalculatorState, metrics: &Metrics) {
    let painter = ui.painter();
    let top = state.top_text();
    let top_rect = Rect::from_min_max(
        Pos2::new(
            rect.left() + metrics.fs(8.0),
            rect.bottom() - metrics.fs(15.0),
        ),
        Pos2::new(
            rect.right() - metrics.fs(8.0),
            rect.bottom() - metrics.fs(2.0),
        ),
    );
    if top != "qalsheet" {
        let color = if state.top_is_preview() {
            Palette::VIOLET_SOFT
        } else {
            Palette::MUTED
        };
        theme::fitted_text(
            painter,
            top_rect,
            Align2::RIGHT_CENTER,
            &top,
            metrics.fs(7.0),
            metrics.fs(5.5),
            Weight::Medium,
            color,
        );
    }
    let result_rect = Rect::from_min_max(
        Pos2::new(rect.left() + metrics.fs(6.0), rect.top() + metrics.fs(1.0)),
        Pos2::new(
            rect.right() - metrics.fs(6.0),
            rect.bottom() - metrics.fs(16.0),
        ),
    );
    let display = state.display_text();
    theme::fitted_text(
        painter,
        result_rect,
        Align2::RIGHT_CENTER,
        &display,
        metrics.fs(27.0),
        metrics.fs(12.0),
        Weight::ExtraBold,
        if state.is_error() {
            Palette::ERROR
        } else {
            Palette::TEXT
        },
    );
}

fn watch_main_keypad(ui: &mut Ui, area: Rect, state: &mut CalculatorState, metrics: &Metrics) {
    // The original Wear layout: a full 5 x 4 grid with the empty corners left
    // deliberately blank. This keeps every key large enough for a fingertip.
    let cells = common::grid(area, 4, 5, metrics.fs(3.0));
    let rows: [[(&str, KeyKind); 5]; 4] = [
        [
            ("7", KeyKind::Digit),
            ("8", KeyKind::Digit),
            ("9", KeyKind::Digit),
            ("\u{00F7}", KeyKind::Operator),
            ("C", KeyKind::Clear),
        ],
        [
            ("4", KeyKind::Digit),
            ("5", KeyKind::Digit),
            ("6", KeyKind::Digit),
            ("\u{00D7}", KeyKind::Operator),
            ("\u{232B}", KeyKind::Soft),
        ],
        [
            ("1", KeyKind::Digit),
            ("2", KeyKind::Digit),
            ("3", KeyKind::Digit),
            ("\u{2212}", KeyKind::Operator),
            ("=", KeyKind::Equals),
        ],
        [
            ("", KeyKind::Utility),
            ("0", KeyKind::Digit),
            (".", KeyKind::Digit),
            ("+", KeyKind::Operator),
            ("", KeyKind::Utility),
        ],
    ];
    for row in 0..4 {
        for col in 0..5 {
            let (label, kind) = rows[row][col];
            if label.is_empty() {
                continue;
            }
            let icon = (label == "\u{232B}").then_some(Icon::Backspace);
            if widgets::key_with_icon(ui, cells[row * 5 + col], label, kind, metrics, icon)
                .clicked()
            {
                press_main(state, label);
            }
        }
    }
}

fn watch_science_keypad(ui: &mut Ui, area: Rect, state: &mut CalculatorState, metrics: &Metrics) {
    let gap = metrics.fs(3.0);
    let cells = common::grid(area, 4, 5, gap);
    let rows: [[Option<FunctionKey>; 5]; 3] = [
        [
            Some(FunctionKey::Sin),
            Some(FunctionKey::Cos),
            Some(FunctionKey::Tan),
            Some(FunctionKey::Asin),
            Some(FunctionKey::Acos),
        ],
        [
            Some(FunctionKey::Atan),
            Some(FunctionKey::Ln),
            Some(FunctionKey::Log),
            Some(FunctionKey::Sqrt),
            Some(FunctionKey::Square),
        ],
        [
            Some(FunctionKey::Power),
            Some(FunctionKey::Pi),
            Some(FunctionKey::E),
            Some(FunctionKey::OpenParen),
            Some(FunctionKey::CloseParen),
        ],
    ];
    for row in 0..3 {
        for col in 0..5 {
            let Some(function) = rows[row][col] else {
                continue;
            };
            if widgets::key(
                ui,
                cells[row * 5 + col],
                function.label(),
                function_kind(function),
                metrics,
            )
            .clicked()
            {
                state.apply_function(function);
            }
        }
    }
    let bottom: [FunctionKey; 4] = [
        FunctionKey::Factorial,
        FunctionKey::Percent,
        FunctionKey::Sign,
        FunctionKey::Reciprocal,
    ];
    let key_width = (area.width() - gap * 4.0) / 5.0;
    let start = area.left() + (key_width + gap) * 0.5;
    let y = cells[15].top();
    for (index, function) in bottom.iter().enumerate() {
        let rect = Rect::from_min_size(
            Pos2::new(start + index as f32 * (key_width + gap), y),
            Vec2::new(key_width, cells[15].height()),
        );
        if widgets::key(
            ui,
            rect,
            function.label(),
            function_kind(*function),
            metrics,
        )
        .clicked()
        {
            state.apply_function(*function);
        }
    }
}

fn press_main(state: &mut CalculatorState, label: &str) {
    match label {
        "\u{221A}" => state.apply_function(FunctionKey::Sqrt),
        "\u{03C0}" => state.apply_function(FunctionKey::Pi),
        "\u{00B1}" => state.apply_function(FunctionKey::Sign),
        "%" => state.apply_function(FunctionKey::Percent),
        "\u{232B}" => state.press("backspace"),
        _ => state.press(label),
    }
}

fn science_keypad(ui: &mut Ui, area: Rect, state: &mut CalculatorState, metrics: &Metrics) {
    let gap = metrics.fs(4.0);
    let cells = if metrics.square_keys {
        common::square_grid(area, 6, 5, gap)
    } else {
        common::grid(area, 6, 5, gap)
    };
    let keys = [
        FunctionKey::Sin,
        FunctionKey::Cos,
        FunctionKey::Tan,
        FunctionKey::Asin,
        FunctionKey::Acos,
        FunctionKey::Atan,
        FunctionKey::Sinh,
        FunctionKey::Cosh,
        FunctionKey::Tanh,
        FunctionKey::Ln,
        FunctionKey::Log,
        FunctionKey::Log2,
        FunctionKey::Exp,
        FunctionKey::Sqrt,
        FunctionKey::Cbrt,
        FunctionKey::Square,
        FunctionKey::Power,
        FunctionKey::Reciprocal,
        FunctionKey::Abs,
        FunctionKey::Factorial,
        FunctionKey::Floor,
        FunctionKey::Ceil,
        FunctionKey::Round,
        FunctionKey::OpenParen,
        FunctionKey::CloseParen,
        FunctionKey::Pi,
        FunctionKey::E,
        FunctionKey::Tau,
        FunctionKey::Phi,
        FunctionKey::Ans,
    ];
    for (index, function) in keys.iter().enumerate() {
        let response = widgets::key(
            ui,
            cells[index],
            function.label(),
            function_kind(*function),
            metrics,
        );
        if response.clicked() {
            state.apply_function(*function);
            // Returning to the numeric page is the watch behaviour and makes
            // entering the argument immediate.
            state.show_scientific_page(false);
            break;
        }
    }
}

fn function_kind(function: FunctionKey) -> KeyKind {
    match function {
        FunctionKey::Pi
        | FunctionKey::E
        | FunctionKey::Tau
        | FunctionKey::Phi
        | FunctionKey::OpenParen
        | FunctionKey::CloseParen => KeyKind::Constant,
        FunctionKey::Sign
        | FunctionKey::Percent
        | FunctionKey::Factorial
        | FunctionKey::Reciprocal
        | FunctionKey::Floor
        | FunctionKey::Ceil
        | FunctionKey::Round => KeyKind::Utility,
        _ => KeyKind::Operator,
    }
}

fn memory_row(
    ui: &mut Ui,
    rect: Rect,
    state: &mut CalculatorState,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let controls = metrics.controls();
    let pill_metrics = &controls;
    let labels = ["MC", "MR", "M+", "M\u{2212}"];
    let width = pill_metrics.fs(42.0);
    let gap = pill_metrics.fs(6.0);
    for (index, label) in labels.iter().enumerate() {
        let r = Rect::from_min_size(
            Pos2::new(rect.left() + index as f32 * (width + gap), rect.top()),
            Vec2::new(width, rect.height()),
        );
        if widgets::pill(ui, r, label, false, false, pill_metrics).clicked() {
            match *label {
                "MC" => {
                    state.memory_clear();
                    feedback.show(ui.ctx(), "Memory cleared");
                }
                "MR" => {
                    if state.memory().is_some() {
                        state.memory_recall();
                    } else {
                        feedback.show(ui.ctx(), "Memory is empty");
                    }
                }
                "M+" => {
                    state.memory_add();
                    feedback.show(ui.ctx(), "Added to memory");
                }
                _ => {
                    state.memory_subtract();
                    feedback.show(ui.ctx(), "Subtracted from memory");
                }
            }
        }
    }
    let text_x = rect.left() + 4.0 * (width + gap) + pill_metrics.fs(8.0);
    if text_x < rect.right() {
        theme::text(
            ui.painter(),
            Pos2::new(text_x, rect.center().y),
            Align2::LEFT_CENTER,
            if let Some(memory) = state.memory() {
                format!("M = {}", format::plain_number(memory))
            } else {
                "Memory empty".to_string()
            },
            pill_metrics.fs(9.5),
            Weight::Medium,
            if state.memory().is_some() {
                Palette::VIOLET_SOFT
            } else {
                Palette::FAINT
            },
        );
    }
}

fn functions_panel(ui: &mut Ui, rect: Rect, state: &mut CalculatorState, metrics: &Metrics) {
    theme::card(
        ui.painter(),
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(14),
    );
    let pad = metrics.fs(12.0);
    let header = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(8.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(18.0)),
    );
    theme::text(
        ui.painter(),
        header.left_center(),
        Align2::LEFT_CENTER,
        "SCIENTIFIC",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    theme::text(
        ui.painter(),
        header.right_center(),
        Align2::RIGHT_CENTER,
        "DEG / RAD",
        metrics.fs(8.0),
        Weight::SemiBold,
        Palette::MUTED,
    );
    let grid = Rect::from_min_max(
        Pos2::new(rect.left() + pad, header.bottom() + metrics.fs(8.0)),
        Pos2::new(rect.right() - pad, rect.bottom() - pad),
    );
    science_keypad(ui, grid, state, metrics);
}

fn history_panel(
    ui: &mut Ui,
    rect: Rect,
    state: &mut CalculatorState,
    screen_ui: &mut CalculatorUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    theme::card(
        ui.painter(),
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(14),
    );
    let pad = metrics.fs(12.0);
    let header = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(8.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(20.0)),
    );
    theme::text(
        ui.painter(),
        header.left_center(),
        Align2::LEFT_CENTER,
        "HISTORY",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let clear_width = metrics.fs(50.0);
    let clear = Rect::from_min_size(
        Pos2::new(
            header.right() - clear_width,
            header.center().y - metrics.fs(10.0),
        ),
        Vec2::new(clear_width, metrics.fs(20.0)),
    );
    if widgets::pill(ui, clear, "Clear", false, true, metrics).clicked() {
        state.clear_history();
        feedback.show(ui.ctx(), "History cleared");
    }
    let list = Rect::from_min_max(
        Pos2::new(
            rect.left() + metrics.fs(8.0),
            header.bottom() + metrics.fs(4.0),
        ),
        Pos2::new(
            rect.right() - metrics.fs(8.0),
            rect.bottom() - metrics.fs(8.0),
        ),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(list)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(list);
    if state.history.is_empty() {
        crate::widgets::empty_state(&mut child, list, "No calculations yet.", metrics);
        return;
    }
    let mut load = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            for (index, entry) in state.history.iter().enumerate() {
                let (row, response) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), history_row_height(metrics)),
                    Sense::click(),
                );
                history_row(
                    ui,
                    row,
                    entry.expression.clone(),
                    entry.result.clone(),
                    entry.angle_mode.map(|mode| mode.label()),
                    &entry.note,
                    metrics,
                );
                if response.clicked() {
                    load = Some(index);
                }
                ui.add_space(metrics.fs(4.0));
            }
        });
    if let Some(index) = load {
        if let Some(entry) = state.history.get(index).cloned() {
            state.load_history(&entry);
            screen_ui.note_target = Some(entry.id);
            screen_ui.note_draft = entry.note.clone();
            screen_ui.show_history = false;
            feedback.show(ui.ctx(), "Loaded from history");
        }
    }
}

fn history_row_height(metrics: &Metrics) -> f32 {
    metrics.controls().fs(46.0)
}

fn history_row(
    ui: &mut Ui,
    rect: Rect,
    expression: String,
    result: String,
    mode: Option<&'static str>,
    note: &str,
    metrics: &Metrics,
) {
    let text_metrics = metrics.controls();
    let painter = ui.painter();
    painter.rect_filled(rect, text_metrics.fs(10.0), Palette::CARD_LIFT);
    painter.rect_stroke(
        rect,
        text_metrics.fs(10.0),
        Stroke::new(1.0, Color32::from_white_alpha(12)),
        StrokeKind::Inside,
    );
    let pad = text_metrics.fs(9.0);
    let has_note = !note.trim().is_empty();
    let (expression_bottom, result_bottom, note_top) = if has_note {
        (
            rect.top() + rect.height() * 0.33,
            rect.top() + rect.height() * 0.64,
            rect.top() + rect.height() * 0.64,
        )
    } else {
        (
            rect.top() + rect.height() * 0.50,
            rect.bottom() - text_metrics.fs(4.0),
            rect.bottom(),
        )
    };
    theme::fitted_text(
        painter,
        Rect::from_min_max(
            Pos2::new(rect.left() + pad, rect.top() + text_metrics.fs(2.0)),
            Pos2::new(rect.right() - pad, expression_bottom),
        ),
        Align2::LEFT_CENTER,
        &expression,
        text_metrics.fs(8.0),
        text_metrics.fs(6.0),
        Weight::Medium,
        Palette::MUTED,
    );
    theme::fitted_text(
        painter,
        Rect::from_min_max(
            Pos2::new(rect.left() + pad, expression_bottom),
            Pos2::new(
                rect.right()
                    - pad
                    - if mode.is_some() {
                        text_metrics.fs(28.0)
                    } else {
                        0.0
                    },
                result_bottom,
            ),
        ),
        Align2::LEFT_CENTER,
        &result,
        text_metrics.fs(13.0),
        text_metrics.fs(8.0),
        Weight::Bold,
        Palette::TEXT,
    );
    if has_note {
        theme::fitted_text(
            painter,
            Rect::from_min_max(
                Pos2::new(rect.left() + pad, note_top + text_metrics.fs(1.0)),
                Pos2::new(rect.right() - pad, rect.bottom() - text_metrics.fs(2.0)),
            ),
            Align2::LEFT_CENTER,
            &format!("note · {}", note.lines().next().unwrap_or("").trim()),
            text_metrics.fs(7.3),
            text_metrics.fs(5.5),
            Weight::Medium,
            Palette::VIOLET_SOFT,
        );
    }
    if let Some(mode) = mode {
        common::badge(
            painter,
            Rect::from_min_size(
                Pos2::new(
                    rect.right() - pad - text_metrics.fs(24.0),
                    rect.center().y - text_metrics.fs(7.0),
                ),
                Vec2::new(text_metrics.fs(24.0), text_metrics.fs(14.0)),
            ),
            mode,
            Palette::VIOLET_SOFT,
            &text_metrics,
        );
    }
}

fn history_modal(
    ui: &mut Ui,
    state: &mut CalculatorState,
    screen_ui: &mut CalculatorUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let width = common::modal_width(ui, metrics.fs(500.0));
    let height = common::modal_max_height(ui);
    let frame = egui::Frame::NONE
        .fill(Palette::CARD)
        .stroke(Stroke::new(1.0, Palette::BORDER))
        .corner_radius(metrics.fs(20.0))
        .inner_margin(Margin::same(16));
    let mut close = false;
    let mut clear = false;
    let mut load = None;
    let response = egui::Modal::new(Id::unique("qal-history"))
        .frame(frame)
        .backdrop_color(Color32::from_black_alpha(150))
        .show(ui.ctx(), |ui| {
            ui.set_width(width);
            ui.set_max_height(height);
            if widgets::modal_header(
                ui,
                "History",
                &format!("{} calculations", state.history.len()),
                metrics,
                true,
            ) {
                close = true;
            }
            ui.add_space(metrics.fs(4.0));
            let bar = Rect::from_min_size(
                ui.cursor().min,
                Vec2::new(ui.available_width(), metrics.fs(22.0)),
            );
            ui.allocate_rect(bar, Sense::hover());
            let clear_rect = Rect::from_min_size(
                Pos2::new(bar.right() - metrics.fs(54.0), bar.top()),
                Vec2::new(metrics.fs(54.0), metrics.fs(20.0)),
            );
            if widgets::pill(ui, clear_rect, "Clear", false, true, metrics).clicked() {
                clear = true;
            }
            ui.add_space(metrics.fs(6.0));
            if state.history.is_empty() {
                let empty = Rect::from_min_size(
                    ui.cursor().min,
                    Vec2::new(ui.available_width(), metrics.fs(80.0)),
                );
                ui.allocate_rect(empty, Sense::hover());
                crate::widgets::empty_state(
                    ui,
                    empty,
                    "No calculations yet.\nResults will appear here.",
                    metrics,
                );
            } else {
                egui::ScrollArea::vertical()
                    .max_height(height - metrics.fs(92.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (index, entry) in state.history.iter().enumerate() {
                            let (row, response) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), history_row_height(metrics)),
                                Sense::click(),
                            );
                            history_row(
                                ui,
                                row,
                                entry.expression.clone(),
                                entry.result.clone(),
                                entry.angle_mode.map(|mode| mode.label()),
                                &entry.note,
                                metrics,
                            );
                            if response.clicked() {
                                load = Some(index);
                            }
                            ui.add_space(metrics.fs(4.0));
                        }
                    });
            }
        });
    if response.should_close() {
        close = true;
    }
    if clear {
        state.clear_history();
        feedback.show(ui.ctx(), "History cleared");
    }
    if let Some(index) = load {
        if let Some(entry) = state.history.get(index).cloned() {
            state.load_history(&entry);
            screen_ui.note_target = Some(entry.id);
            screen_ui.note_draft = entry.note.clone();
            feedback.show(ui.ctx(), "Loaded from history");
            close = true;
        }
    }
    if close {
        screen_ui.show_history = false;
    }
}
