//! Unit converter with the watch's card + keypad composition scaled up.

use eframe::egui::{
    self, Align2, Color32, Id, Margin, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2,
};
use qalsheet_core::units::{self, CATEGORIES, ConverterState, UnitTarget};

use crate::app::Feedback;
use crate::icons::{self, Icon};
use crate::screens::common;
use crate::theme::{self, Metrics, Palette, Weight};
use crate::widgets::{self, KeyKind};

#[derive(Default)]
pub struct ConverterUi {
    picker: Option<UnitTarget>,
    category_picker: bool,
    picker_search: String,
    favorites: Vec<(usize, usize)>,
}

pub fn screen(
    ui: &mut Ui,
    state: &mut ConverterState,
    screen_ui: &mut ConverterUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    if metrics.watch {
        watch_screen(ui, state, screen_ui, metrics, feedback);
    } else if metrics.compact {
        compact_screen(ui, state, screen_ui, metrics, feedback);
    } else {
        wide_screen(ui, state, screen_ui, metrics, feedback);
    }
    if let Some(target) = screen_ui.picker {
        unit_picker(ui, state, screen_ui, target, metrics);
    }
    if screen_ui.category_picker {
        category_picker(ui, state, screen_ui, metrics);
    }
}

fn wide_screen(
    ui: &mut Ui,
    state: &mut ConverterState,
    screen_ui: &mut ConverterUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let area = ui.max_rect();
    let rail_width = metrics.fs(194.0).min(area.width() * 0.23);
    let reference_width = metrics.fs(260.0).min(area.width() * 0.27);
    let gap = metrics.fs(14.0);
    let rail = Rect::from_min_size(area.min, Vec2::new(rail_width, area.height()));
    category_rail(ui, rail, state, metrics);
    let main_column = Rect::from_min_max(
        Pos2::new(rail.right() + gap, area.top()),
        Pos2::new(area.right() - reference_width - gap, area.bottom()),
    );
    let main_height = metrics.fs(356.0).min(main_column.height() * 0.52);
    let main = Rect::from_min_size(main_column.min, Vec2::new(main_column.width(), main_height));
    conversion_card(ui, main, state, screen_ui, metrics, feedback, true);
    let quick = Rect::from_min_max(Pos2::new(main.left(), main.bottom() + gap), main_column.max);
    quick_conversions_card(ui, quick, state, metrics, feedback);
    let reference = Rect::from_min_max(
        Pos2::new(area.right() - reference_width, area.top()),
        area.max,
    );
    reference_card(ui, reference, state, metrics, feedback);
}

fn compact_screen(
    ui: &mut Ui,
    state: &mut ConverterState,
    screen_ui: &mut ConverterUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let area = ui.max_rect();
    let controls = metrics.controls();
    let categories = Rect::from_min_size(
        area.min,
        Vec2::new(
            area.width(),
            if metrics.square_keys {
                controls.fs(28.0)
            } else {
                metrics.fs(30.0)
            },
        ),
    );
    category_chips(ui, categories, state, metrics);
    let card_height = (area.height() * 0.40).clamp(metrics.fs(150.0), metrics.fs(196.0));
    let card = Rect::from_min_size(
        Pos2::new(area.left(), categories.bottom() + metrics.fs(6.0)),
        Vec2::new(area.width(), card_height),
    );
    conversion_card(ui, card, state, screen_ui, metrics, feedback, false);
    let quick_height = if area.height() > metrics.fs(560.0) {
        controls.fs(112.0)
    } else {
        0.0
    };
    let quick = Rect::from_min_size(
        Pos2::new(area.left(), card.bottom() + metrics.fs(6.0)),
        Vec2::new(area.width(), quick_height),
    );
    if quick_height > 0.0 {
        quick_conversions_card(ui, quick, state, &controls, feedback);
    }
    let keypad = Rect::from_min_max(
        Pos2::new(
            area.left(),
            if quick_height > 0.0 {
                quick.bottom() + metrics.fs(6.0)
            } else {
                card.bottom() + metrics.fs(8.0)
            },
        ),
        area.max,
    );
    amount_keypad(ui, keypad, state, metrics);
}

fn watch_screen(
    ui: &mut Ui,
    state: &mut ConverterState,
    screen_ui: &mut ConverterUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let area = ui.max_rect();
    let width = area.width() * 0.84;
    let left = area.center().x - width / 2.0;
    let bar = Rect::from_min_size(
        Pos2::new(left, area.top()),
        Vec2::new(width, metrics.fs(22.0)),
    );
    watch_category_bar(ui, bar, state, screen_ui, metrics);
    let card = Rect::from_min_size(
        Pos2::new(left, bar.bottom() + metrics.fs(2.0)),
        Vec2::new(width, metrics.fs(86.0)),
    );
    watch_conversion_card(ui, card, state, screen_ui, metrics, feedback);
    let keypad = Rect::from_min_max(
        Pos2::new(left, card.bottom() + metrics.fs(2.0)),
        Pos2::new(left + width, area.bottom()),
    );
    amount_keypad(ui, keypad, state, metrics);
}

fn category_rail(ui: &mut Ui, rect: Rect, state: &mut ConverterState, metrics: &Metrics) {
    theme::card(
        ui.painter(),
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(18),
    );
    let pad = metrics.fs(10.0);
    theme::text(
        ui.painter(),
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(16.0)),
        Align2::LEFT_CENTER,
        "CATEGORY",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let mut y = rect.top() + metrics.fs(28.0);
    let row_height = metrics.fs(36.0);
    for (index, category) in CATEGORIES.iter().enumerate() {
        if y + row_height > rect.bottom() - metrics.fs(8.0) {
            break;
        }
        let row = Rect::from_min_size(
            Pos2::new(rect.left() + metrics.fs(6.0), y),
            Vec2::new(
                rect.width() - metrics.fs(12.0),
                row_height - metrics.fs(2.0),
            ),
        );
        let id = ui.scope_id().with(("conv-cat", index));
        let response = ui.interact(row, id, Sense::click());
        let active = state.category_index() == index;
        if active {
            ui.painter()
                .rect_filled(row, metrics.fs(9.0), Palette::VIOLET.gamma_multiply(0.16));
        } else if response.hovered() {
            ui.painter()
                .rect_filled(row, metrics.fs(9.0), Color32::from_white_alpha(6));
        }
        ui.painter().circle_filled(
            Pos2::new(row.left() + metrics.fs(13.0), row.center().y),
            metrics.fs(3.0),
            crate::theme::GRAPH_COLORS[index % crate::theme::GRAPH_COLORS.len()],
        );
        theme::text(
            ui.painter(),
            Pos2::new(row.left() + metrics.fs(24.0), row.center().y),
            Align2::LEFT_CENTER,
            category.label,
            metrics.fs(10.4),
            if active { Weight::Bold } else { Weight::Medium },
            if active {
                Palette::TEXT
            } else {
                Palette::MUTED
            },
        );
        if active {
            ui.painter().circle_filled(
                Pos2::new(row.right() - metrics.fs(8.0), row.center().y),
                metrics.fs(2.5),
                Palette::VIOLET,
            );
        }
        if response.clicked() {
            state.select_category(index);
        }
        y += row_height;
    }
}

fn category_chips(ui: &mut Ui, rect: Rect, state: &mut ConverterState, metrics: &Metrics) {
    let controls = metrics.controls();
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    child.set_clip_rect(rect);
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = controls.fs(5.0);
                for (index, category) in CATEGORIES.iter().enumerate() {
                    let active = state.category_index() == index;
                    let width = controls.fs(68.0);
                    let (cell, response) =
                        ui.allocate_exact_size(Vec2::new(width, controls.fs(24.0)), Sense::click());
                    let painter = ui.painter();
                    painter.rect_filled(
                        cell,
                        cell.height() * 0.5,
                        if active {
                            Palette::VIOLET
                        } else {
                            Palette::CARD_LIFT
                        },
                    );
                    if response.hovered() && !active {
                        painter.rect_filled(
                            cell,
                            cell.height() * 0.5,
                            Color32::from_white_alpha(12),
                        );
                    }
                    theme::fitted_text(
                        painter,
                        cell,
                        Align2::CENTER_CENTER,
                        category.label,
                        controls.fs(9.0),
                        controls.fs(6.0),
                        Weight::SemiBold,
                        if active { Palette::INK } else { Palette::MUTED },
                    );
                    if response.clicked() {
                        state.select_category(index);
                    }
                }
            });
        });
}

fn conversion_card(
    ui: &mut Ui,
    rect: Rect,
    state: &mut ConverterState,
    screen_ui: &mut ConverterUi,
    metrics: &Metrics,
    feedback: &mut Feedback,
    spacious: bool,
) {
    let painter = ui.painter().clone();
    theme::card(
        &painter,
        rect,
        metrics.fs(18.0),
        Palette::CARD,
        Color32::from_white_alpha(22),
    );
    let pad = metrics.fs(14.0);
    let header_y = rect.top() + metrics.fs(15.0);
    theme::text(
        &painter,
        Pos2::new(rect.left() + pad, header_y),
        Align2::LEFT_CENTER,
        state.category().label.to_uppercase(),
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    if spacious {
        let copy = Rect::from_min_size(
            Pos2::new(
                rect.right() - pad - metrics.fs(22.0),
                header_y - metrics.fs(11.0),
            ),
            Vec2::splat(metrics.fs(22.0)),
        );
        if widgets::icon_button(ui, copy, Icon::Copy, "Copy result", metrics).clicked() {
            ui.ctx().copy_text(state.result_text());
            feedback.show(ui.ctx(), "Result copied");
        }
        theme::text(
            &painter,
            Pos2::new(copy.left() - metrics.fs(8.0), header_y),
            Align2::RIGHT_CENTER,
            units::format_reference(state.from(), state.to()),
            metrics.fs(8.0),
            Weight::Medium,
            Palette::MUTED,
        );
    }

    let control_height = if spacious {
        metrics.fs(48.0)
    } else {
        metrics.fs(38.0)
    };
    let top = header_y + metrics.fs(if spacious { 16.0 } else { 12.0 });
    let left_width = (rect.width() - pad * 2.0 - metrics.fs(42.0)) / 2.0;
    let from_rect = Rect::from_min_size(
        Pos2::new(rect.left() + pad, top),
        Vec2::new(left_width, control_height),
    );
    let swap_rect = Rect::from_min_size(
        Pos2::new(
            from_rect.right() + metrics.fs(4.0),
            top + control_height / 2.0 - metrics.fs(17.0),
        ),
        Vec2::splat(metrics.fs(34.0)),
    );
    let to_rect = Rect::from_min_size(
        Pos2::new(swap_rect.right() + metrics.fs(4.0), top),
        Vec2::new(left_width, control_height),
    );
    if widgets::dropdown(
        ui,
        from_rect,
        state.from().symbol,
        state.from().name,
        metrics,
    )
    .clicked()
    {
        screen_ui.picker_search.clear();
        screen_ui.picker = Some(UnitTarget::From);
    }
    if widgets::dropdown(ui, to_rect, state.to().symbol, state.to().name, metrics).clicked() {
        screen_ui.picker_search.clear();
        screen_ui.picker = Some(UnitTarget::To);
    }
    if widgets::icon_button(ui, swap_rect, Icon::Swap, "Swap units", metrics).clicked() {
        state.swap();
    }

    let amount_top = from_rect.bottom() + metrics.fs(8.0);
    let amount_height = if spacious {
        metrics.fs(42.0)
    } else {
        metrics.fs(34.0)
    };
    let amount = Rect::from_min_size(
        Pos2::new(rect.left() + pad, amount_top),
        Vec2::new(rect.width() - pad * 2.0, amount_height),
    );
    field_amount(ui, amount, state, metrics);
    theme::text(
        &painter,
        Pos2::new(amount.left() + metrics.fs(10.0), amount.center().y),
        Align2::LEFT_CENTER,
        state.from().symbol,
        metrics.fs(10.0),
        Weight::Bold,
        Palette::TEAL_SOFT,
    );

    let result_top = amount.bottom() + metrics.fs(6.0);
    let result_rect = Rect::from_min_max(
        Pos2::new(rect.left() + pad, result_top),
        Pos2::new(
            rect.right() - pad,
            result_top
                + if spacious {
                    metrics.fs(72.0)
                } else {
                    metrics.fs(48.0)
                },
        ),
    );
    let text = state.result_text();
    let size = if spacious {
        metrics.fs(28.0)
    } else {
        metrics.fs(20.0)
    };
    theme::fitted_text(
        &painter,
        result_rect,
        Align2::RIGHT_CENTER,
        &format!("{} {}", text, state.to().symbol),
        size,
        metrics.fs(9.0),
        Weight::ExtraBold,
        Palette::TEAL_SOFT,
    );
    theme::text(
        &painter,
        Pos2::new(result_rect.left(), result_rect.center().y),
        Align2::LEFT_CENTER,
        "=",
        metrics.fs(12.0),
        Weight::Medium,
        Palette::FAINT,
    );
}

fn watch_category_bar(
    ui: &mut Ui,
    rect: Rect,
    state: &mut ConverterState,
    screen_ui: &mut ConverterUi,
    metrics: &Metrics,
) {
    let painter = ui.painter().clone();
    let button = Rect::from_min_size(
        rect.min,
        Vec2::new(rect.width() - metrics.fs(70.0), rect.height()),
    );
    painter.rect_filled(button, button.height() * 0.5, Palette::CARD_LIFT);
    theme::text(
        &painter,
        Pos2::new(button.left() + metrics.fs(8.0), button.center().y),
        Align2::LEFT_CENTER,
        state.category().label,
        metrics.fs(8.5),
        Weight::Bold,
        Palette::MUTED,
    );
    icons::icon(
        &painter,
        Pos2::new(button.right() - metrics.fs(8.0), button.center().y),
        metrics.fs(8.0),
        Icon::ChevronDown,
        Palette::MUTED,
    );
    if ui
        .interact(button, ui.scope_id().with("watch-category"), Sense::click())
        .clicked()
    {
        screen_ui.category_picker = true;
    }
    let sign = Rect::from_min_size(
        Pos2::new(button.right() + metrics.fs(4.0), rect.top()),
        Vec2::new(metrics.fs(20.0), rect.height()),
    );
    if widgets::pill(ui, sign, "\u{00B1}", false, false, metrics).clicked() {
        state.toggle_sign();
    }
    let swap = Rect::from_min_size(
        Pos2::new(sign.right() + metrics.fs(4.0), rect.top()),
        Vec2::new(metrics.fs(22.0), rect.height()),
    );
    if widgets::icon_button(ui, swap, Icon::Swap, "Swap", metrics).clicked() {
        state.swap();
    }
}

fn watch_unit_button(
    ui: &mut Ui,
    rect: Rect,
    symbol: &str,
    name: &str,
    metrics: &Metrics,
) -> egui::Response {
    let id = ui.scope_id().with((
        "watch-unit",
        symbol,
        rect.left().to_bits(),
        rect.top().to_bits(),
    ));
    let response = ui.interact(rect, id, Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.1);
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        metrics.fs(9.0),
        theme::blend(Palette::CARD_LIFT, Palette::KEY_LIFT, hover),
    );
    painter.rect_stroke(
        rect,
        metrics.fs(9.0),
        Stroke::new(1.0, Color32::from_white_alpha(18)),
        StrokeKind::Inside,
    );
    let left = rect.left() + metrics.fs(8.0);
    let symbol_size = metrics.fs(12.0);
    theme::text(
        painter,
        Pos2::new(left, rect.center().y),
        Align2::LEFT_CENTER,
        symbol,
        symbol_size,
        Weight::Bold,
        Palette::TEXT,
    );
    let symbol_width = theme::text_width(painter, symbol, symbol_size, Weight::Bold);
    let name_rect = Rect::from_min_max(
        Pos2::new(
            left + symbol_width + metrics.fs(5.0),
            rect.top() + metrics.fs(2.0),
        ),
        Pos2::new(
            rect.right() - metrics.fs(20.0),
            rect.bottom() - metrics.fs(2.0),
        ),
    );
    theme::fitted_text(
        painter,
        name_rect,
        Align2::LEFT_CENTER,
        name,
        metrics.fs(8.0),
        metrics.fs(5.0),
        Weight::Medium,
        Palette::MUTED,
    );
    icons::icon(
        painter,
        Pos2::new(rect.right() - metrics.fs(9.0), rect.center().y),
        metrics.fs(9.0),
        Icon::ChevronDown,
        Palette::MUTED,
    );
    response
}

fn watch_conversion_card(
    ui: &mut Ui,
    rect: Rect,
    state: &mut ConverterState,
    screen_ui: &mut ConverterUi,
    metrics: &Metrics,
    _feedback: &mut Feedback,
) {
    let painter = ui.painter().clone();
    theme::card(
        &painter,
        rect,
        metrics.fs(12.0),
        Palette::CARD,
        Color32::from_white_alpha(18),
    );
    let pad = metrics.fs(6.0);
    let gap = metrics.fs(7.0);
    let width = (rect.width() - pad * 2.0 - gap) / 2.0;
    let from = Rect::from_min_size(
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(4.0)),
        Vec2::new(width, metrics.fs(24.0)),
    );
    let to = Rect::from_min_size(
        Pos2::new(from.right() + gap, from.top()),
        Vec2::new(width, from.height()),
    );
    if watch_unit_button(ui, from, state.from().symbol, state.from().name, metrics).clicked() {
        screen_ui.picker_search.clear();
        screen_ui.picker = Some(UnitTarget::From);
    }
    if watch_unit_button(ui, to, state.to().symbol, state.to().name, metrics).clicked() {
        screen_ui.picker_search.clear();
        screen_ui.picker = Some(UnitTarget::To);
    }
    let amount = Rect::from_min_size(
        Pos2::new(rect.left() + pad, from.bottom() + metrics.fs(4.0)),
        Vec2::new(rect.width() - pad * 2.0, metrics.fs(28.0)),
    );
    painter.rect_filled(amount, metrics.fs(9.0), Color32::from_rgb(0x13, 0x10, 0x1A));
    painter.rect_stroke(
        amount,
        metrics.fs(9.0),
        Stroke::new(1.0, Color32::from_white_alpha(16)),
        StrokeKind::Inside,
    );
    let unit_width =
        theme::text_width(&painter, state.from().symbol, metrics.fs(9.0), Weight::Bold);
    let amount_text = state.input_text();
    theme::fitted_text(
        &painter,
        Rect::from_min_max(
            Pos2::new(
                amount.left() + metrics.fs(8.0),
                amount.top() + metrics.fs(2.0),
            ),
            Pos2::new(
                amount.right() - unit_width - metrics.fs(16.0),
                amount.bottom() - metrics.fs(2.0),
            ),
        ),
        Align2::RIGHT_CENTER,
        &amount_text,
        metrics.fs(20.0),
        metrics.fs(9.0),
        Weight::ExtraBold,
        Palette::TEXT,
    );
    theme::text(
        &painter,
        Pos2::new(amount.right() - metrics.fs(8.0), amount.center().y),
        Align2::RIGHT_CENTER,
        state.from().symbol,
        metrics.fs(9.0),
        Weight::Bold,
        Palette::TEAL_SOFT,
    );
    theme::fitted_text(
        &painter,
        Rect::from_min_max(
            Pos2::new(rect.left() + pad, amount.bottom() + metrics.fs(1.0)),
            Pos2::new(rect.right() - pad, rect.bottom() - metrics.fs(1.0)),
        ),
        Align2::RIGHT_CENTER,
        &format!("= {} {}", state.result_text(), state.to().symbol),
        metrics.fs(9.0),
        metrics.fs(6.0),
        Weight::Bold,
        Palette::TEAL_SOFT,
    );
}

fn quick_conversions_card(
    ui: &mut Ui,
    rect: Rect,
    state: &ConverterState,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    if rect.height() < metrics.fs(80.0) {
        return;
    }
    let painter = ui.painter().clone();
    theme::card(
        &painter,
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(18),
    );
    let pad = metrics.fs(14.0);
    theme::text(
        &painter,
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(16.0)),
        Align2::LEFT_CENTER,
        "QUICK CONVERSIONS",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let copy = Rect::from_min_size(
        Pos2::new(
            rect.right() - pad - metrics.fs(60.0),
            rect.top() + metrics.fs(5.0),
        ),
        Vec2::new(metrics.fs(60.0), metrics.fs(22.0)),
    );
    if widgets::pill(ui, copy, "Copy all", false, true, metrics).clicked() {
        let mut text = String::new();
        for unit in state.category().units.iter().take(8) {
            let value = units::convert(1.0, state.from(), *unit);
            text.push_str(&format!(
                "1 {} = {} {}\n",
                state.from().symbol,
                units::format_value(value),
                unit.symbol
            ));
        }
        ui.ctx().copy_text(text);
        feedback.show(ui.ctx(), "Conversions copied");
    }
    let mut y = rect.top() + metrics.fs(32.0);
    let row_height = metrics.fs(24.0);
    let mut count = 0;
    for unit in state.category().units {
        if *unit == state.from() || count >= 5 {
            continue;
        }
        if y + row_height > rect.bottom() - metrics.fs(6.0) {
            break;
        }
        let value = units::convert(1.0, state.from(), *unit);
        theme::text(
            &painter,
            Pos2::new(rect.left() + pad, y),
            Align2::LEFT_CENTER,
            format!(
                "1 {} = {} {}",
                state.from().symbol,
                units::format_value(value),
                unit.symbol
            ),
            metrics.fs(9.0),
            Weight::Medium,
            Palette::MUTED,
        );
        y += row_height;
        count += 1;
    }
}

fn field_amount(ui: &mut Ui, rect: Rect, state: &mut ConverterState, metrics: &Metrics) {
    ui.painter()
        .rect_filled(rect, metrics.fs(10.0), Color32::from_rgb(0x13, 0x10, 0x1A));
    let inner = rect.shrink2(Vec2::new(metrics.fs(44.0), metrics.fs(7.0)));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner)
            .layout(egui::Layout::right_to_left(egui::Align::Center)),
    );
    let response = child.add(
        egui::TextEdit::singleline(state.input_mut())
            .desired_width(inner.width())
            .char_limit(24)
            .margin(Margin::ZERO)
            .frame(egui::Frame::NONE)
            .font(theme::font_selection(metrics.fs(16.0), Weight::Bold))
            .text_color(Palette::TEXT)
            .align(egui::Align2::RIGHT_CENTER),
    );
    ui.painter().rect_stroke(
        rect,
        metrics.fs(10.0),
        Stroke::new(
            1.0,
            if response.has_focus() {
                Palette::VIOLET
            } else {
                Color32::from_white_alpha(16)
            },
        ),
        StrokeKind::Inside,
    );
}

fn amount_keypad(ui: &mut Ui, area: Rect, state: &mut ConverterState, metrics: &Metrics) {
    let gap = metrics.fs(4.0);
    let cells = if metrics.square_keys {
        common::square_grid(area, 4, 4, gap)
    } else {
        common::grid(area, 4, 4, gap)
    };
    let row3_last = if metrics.watch { "." } else { "\u{00B1}" };
    let labels: [(&str, KeyKind); 12] = [
        ("7", KeyKind::Digit),
        ("8", KeyKind::Digit),
        ("9", KeyKind::Digit),
        ("\u{232B}", KeyKind::Soft),
        ("4", KeyKind::Digit),
        ("5", KeyKind::Digit),
        ("6", KeyKind::Digit),
        ("C", KeyKind::Clear),
        ("1", KeyKind::Digit),
        ("2", KeyKind::Digit),
        ("3", KeyKind::Digit),
        (
            row3_last,
            if metrics.watch {
                KeyKind::Digit
            } else {
                KeyKind::Utility
            },
        ),
    ];
    for (index, (label, kind)) in labels.iter().enumerate() {
        let icon = (*label == "\u{232B}").then_some(Icon::Backspace);
        if widgets::key_with_icon(ui, cells[index], label, *kind, metrics, icon).clicked() {
            match *label {
                "\u{232B}" => state.backspace(),
                "C" => state.clear_input(),
                "\u{00B1}" => state.toggle_sign(),
                "." => state.append_decimal(),
                label if label.chars().next().is_some_and(|ch| ch.is_ascii_digit()) => {
                    state.append_digit(label.chars().next().unwrap());
                }
                _ => {}
            }
        }
    }
    // A wide zero centred under the pad. On the phone the spare right cell is
    // a decimal key; the watch keeps its decimal in row three and only the
    // wide zero at the bottom, exactly like the original amount pad.
    let zero_width = cells[12].width() * 2.0 + gap;
    let zero_left = area.center().x - zero_width / 2.0;
    let zero = Rect::from_min_size(
        Pos2::new(zero_left, cells[12].top()),
        Vec2::new(zero_width, cells[12].height()),
    );
    if widgets::key(ui, zero, "0", KeyKind::Digit, metrics).clicked() {
        state.append_digit('0');
    }
    if !metrics.watch {
        let decimal = Rect::from_min_size(
            Pos2::new(cells[15].left(), cells[15].top()),
            Vec2::new(cells[15].width(), cells[15].height()),
        );
        if widgets::key(ui, decimal, ".", KeyKind::Digit, metrics).clicked() {
            state.append_decimal();
        }
    }
}

fn reference_card(
    ui: &mut Ui,
    rect: Rect,
    state: &ConverterState,
    metrics: &Metrics,
    feedback: &mut Feedback,
) {
    let painter = ui.painter().clone();
    theme::card(
        &painter,
        rect,
        metrics.fs(16.0),
        Palette::CARD,
        Color32::from_white_alpha(18),
    );
    let pad = metrics.fs(12.0);
    theme::text(
        &painter,
        Pos2::new(rect.left() + pad, rect.top() + metrics.fs(16.0)),
        Align2::LEFT_CENTER,
        "REFERENCE",
        metrics.fs(8.0),
        Weight::Bold,
        Palette::FAINT,
    );
    let copy = Rect::from_min_size(
        Pos2::new(
            rect.right() - pad - metrics.fs(22.0),
            rect.top() + metrics.fs(6.0),
        ),
        Vec2::splat(metrics.fs(22.0)),
    );
    if widgets::icon_button(ui, copy, Icon::Copy, "Copy reference", metrics).clicked() {
        ui.ctx()
            .copy_text(units::format_reference(state.from(), state.to()));
        feedback.show(ui.ctx(), "Reference copied");
    }
    let list = Rect::from_min_max(
        Pos2::new(rect.left() + metrics.fs(6.0), rect.top() + metrics.fs(30.0)),
        Pos2::new(
            rect.right() - metrics.fs(6.0),
            rect.bottom() - metrics.fs(8.0),
        ),
    );
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(list)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );
    child.set_clip_rect(list);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(&mut child, |ui| {
            for unit in state.category().units {
                let (row, response) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), metrics.fs(32.0)),
                    Sense::click(),
                );
                if response.hovered() {
                    ui.painter()
                        .rect_filled(row, metrics.fs(8.0), Color32::from_white_alpha(7));
                }
                if *unit == state.from() {
                    ui.painter().rect_filled(
                        row,
                        metrics.fs(8.0),
                        Palette::VIOLET.gamma_multiply(0.12),
                    );
                }
                let value = units::convert(1.0, state.from(), *unit);
                theme::text(
                    ui.painter(),
                    Pos2::new(row.left() + metrics.fs(8.0), row.center().y),
                    Align2::LEFT_CENTER,
                    unit.symbol,
                    metrics.fs(9.5),
                    Weight::Bold,
                    if *unit == state.from() {
                        Palette::VIOLET_SOFT
                    } else {
                        Palette::TEXT
                    },
                );
                theme::fitted_text(
                    ui.painter(),
                    Rect::from_min_max(
                        Pos2::new(row.left() + metrics.fs(58.0), row.top() + metrics.fs(6.0)),
                        Pos2::new(
                            row.right() - metrics.fs(6.0),
                            row.bottom() - metrics.fs(6.0),
                        ),
                    ),
                    Align2::RIGHT_CENTER,
                    &units::format_value(value),
                    metrics.fs(9.0),
                    metrics.fs(6.5),
                    Weight::SemiBold,
                    Palette::TEAL_SOFT,
                );
            }
        });
}

fn category_picker(
    ui: &mut Ui,
    state: &mut ConverterState,
    screen_ui: &mut ConverterUi,
    metrics: &Metrics,
) {
    let compact = !metrics.watch;
    let controls = metrics.controls();
    let width = common::modal_width(ui, metrics.fs(360.0));
    let height = common::modal_max_height(ui);
    let margin = if metrics.watch { 9 } else { 16 };
    let frame = egui::Frame::NONE
        .fill(Palette::CARD)
        .stroke(Stroke::new(1.0, Palette::BORDER))
        .corner_radius(metrics.fs(16.0))
        .inner_margin(Margin::same(margin));
    let mut close = false;
    let mut selected = None;
    let response = egui::Modal::new(Id::unique("category-picker"))
        .frame(frame)
        .show(ui.ctx(), |ui| {
            ui.set_width(width);
            ui.set_max_height(height);
            let header_metrics = if compact { &controls } else { metrics };
            if widgets::modal_header(ui, "Category", "", header_metrics, true) {
                close = true;
            }
            ui.add_space(metrics.fs(4.0));
            egui::ScrollArea::vertical()
                .max_height(height - metrics.fs(40.0))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for (index, category) in CATEGORIES.iter().enumerate() {
                        let row_height = if compact {
                            controls.fs(40.0)
                        } else {
                            metrics.fs(28.0)
                        };
                        let (row, response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), row_height),
                            Sense::click(),
                        );
                        let current = state.category_index() == index;
                        if current {
                            ui.painter().rect_filled(
                                row,
                                metrics.fs(8.0),
                                Palette::VIOLET.gamma_multiply(0.15),
                            );
                        }
                        theme::text(
                            ui.painter(),
                            Pos2::new(row.left() + metrics.fs(8.0), row.center().y),
                            Align2::LEFT_CENTER,
                            category.label,
                            if compact {
                                controls.fs(12.0)
                            } else {
                                metrics.fs(10.0)
                            },
                            if current {
                                Weight::Bold
                            } else {
                                Weight::Medium
                            },
                            if current {
                                Palette::VIOLET_SOFT
                            } else {
                                Palette::TEXT
                            },
                        );
                        theme::text(
                            ui.painter(),
                            Pos2::new(row.right() - metrics.fs(8.0), row.center().y),
                            Align2::RIGHT_CENTER,
                            format!("{} units", category.units.len()),
                            if compact {
                                controls.fs(9.0)
                            } else {
                                metrics.fs(8.0)
                            },
                            Weight::Medium,
                            Palette::FAINT,
                        );
                        if response.clicked() {
                            selected = Some(index);
                        }
                    }
                });
        });
    if response.should_close() {
        close = true;
    }
    if let Some(index) = selected {
        state.select_category(index);
        close = true;
    }
    if close {
        screen_ui.category_picker = false;
    }
}

fn unit_picker(
    ui: &mut Ui,
    state: &mut ConverterState,
    screen_ui: &mut ConverterUi,
    target: UnitTarget,
    metrics: &Metrics,
) {
    let compact = !metrics.watch;
    let controls = metrics.controls();
    let width = common::modal_width(ui, metrics.fs(420.0));
    let height = common::modal_max_height(ui) * if compact { 0.84 } else { 0.78 };
    let frame = egui::Frame::NONE
        .fill(Palette::CARD)
        .stroke(Stroke::new(1.0, Palette::BORDER))
        .corner_radius(metrics.fs(20.0))
        .inner_margin(Margin::same(16));
    let title = match target {
        UnitTarget::From => "From unit",
        UnitTarget::To => "To unit",
    };
    let category_index = state.category_index();
    let mut close = false;
    let mut selected = None;
    let mut toggle_favorite = None;
    let favorites = screen_ui.favorites.clone();
    let response = egui::Modal::new(Id::unique("unit-picker"))
        .frame(frame)
        .show(ui.ctx(), |ui| {
            ui.set_width(width);
            ui.set_max_height(height);
            let header_metrics = if compact { &controls } else { metrics };
            if widgets::modal_header(ui, title, state.category().label, header_metrics, true) {
                close = true;
            }
            ui.add_space(if compact {
                controls.fs(4.0)
            } else {
                metrics.fs(6.0)
            });

            let search_height = if compact {
                controls.fs(40.0)
            } else {
                metrics.fs(34.0)
            };
            let (search, _) = ui.allocate_exact_size(
                Vec2::new(ui.available_width(), search_height),
                Sense::hover(),
            );
            let search_metrics = if compact { &controls } else { metrics };
            widgets::field(
                ui,
                search,
                &mut screen_ui.picker_search,
                "Search units",
                search_metrics,
            );
            ui.add_space(if compact {
                controls.fs(5.0)
            } else {
                metrics.fs(4.0)
            });

            let favorite_indices: Vec<usize> = favorites
                .iter()
                .filter_map(|&(category, unit)| (category == category_index).then_some(unit))
                .collect();
            if !favorite_indices.is_empty() {
                theme::text(
                    ui.painter(),
                    ui.cursor().min,
                    Align2::LEFT_TOP,
                    "FAVORITES",
                    if compact {
                        controls.fs(8.0)
                    } else {
                        metrics.fs(8.0)
                    },
                    Weight::Bold,
                    Palette::FAINT,
                );
                ui.add_space(if compact {
                    controls.fs(13.0)
                } else {
                    metrics.fs(11.0)
                });
                let chip_height = if compact {
                    controls.fs(28.0)
                } else {
                    metrics.fs(22.0)
                };
                let (chip_row, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), chip_height),
                    Sense::hover(),
                );
                let mut x = chip_row.left();
                for index in favorite_indices {
                    let unit = state.category().units[index];
                    let chip_width = (theme::text_width(
                        ui.painter(),
                        unit.symbol,
                        if compact {
                            controls.fs(10.5)
                        } else {
                            metrics.fs(10.0)
                        },
                        Weight::SemiBold,
                    ) + if compact {
                        controls.fs(22.0)
                    } else {
                        metrics.fs(18.0)
                    })
                    .max(if compact {
                        controls.fs(44.0)
                    } else {
                        metrics.fs(38.0)
                    });
                    if x + chip_width > chip_row.right() {
                        break;
                    }
                    let chip = Rect::from_min_size(
                        Pos2::new(x, chip_row.top()),
                        Vec2::new(chip_width, chip_height),
                    );
                    if widgets::pill(
                        ui,
                        chip,
                        unit.symbol,
                        false,
                        true,
                        if compact { &controls } else { metrics },
                    )
                    .clicked()
                    {
                        selected = Some(index);
                    }
                    x += chip_width
                        + if compact {
                            controls.fs(5.0)
                        } else {
                            metrics.fs(5.0)
                        };
                }
                ui.add_space(if compact {
                    controls.fs(4.0)
                } else {
                    metrics.fs(3.0)
                });
            }

            let query = screen_ui.picker_search.trim().to_lowercase();
            let shown: Vec<usize> = state
                .category()
                .units
                .iter()
                .enumerate()
                .filter(|(_, unit)| {
                    query.is_empty()
                        || unit.symbol.to_lowercase().contains(&query)
                        || unit.name.to_lowercase().contains(&query)
                })
                .map(|(index, _)| index)
                .collect();
            theme::text(
                ui.painter(),
                ui.cursor().min,
                Align2::LEFT_TOP,
                if query.is_empty() {
                    "ALL UNITS"
                } else {
                    "RESULTS"
                },
                if compact {
                    controls.fs(8.0)
                } else {
                    metrics.fs(8.0)
                },
                Weight::Bold,
                Palette::FAINT,
            );
            ui.add_space(if compact {
                controls.fs(12.0)
            } else {
                metrics.fs(9.0)
            });

            let row_height = if compact {
                controls.fs(48.0)
            } else {
                metrics.fs(34.0)
            };
            let list_height = (ui.available_height() - metrics.fs(4.0)).max(metrics.fs(70.0));
            egui::ScrollArea::vertical()
                .max_height(list_height)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if shown.is_empty() {
                        let (empty, _) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), metrics.fs(60.0)),
                            Sense::hover(),
                        );
                        widgets::empty_state(ui, empty, "No units match your search", metrics);
                    }
                    for index in shown {
                        let unit = state.category().units[index];
                        let (row, row_response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), row_height),
                            Sense::click(),
                        );
                        let current = match target {
                            UnitTarget::From => index == state.from_index(),
                            UnitTarget::To => index == state.to_index(),
                        };
                        if current {
                            ui.painter().rect_filled(
                                row,
                                if compact {
                                    controls.fs(11.0)
                                } else {
                                    metrics.fs(9.0)
                                },
                                Palette::VIOLET.gamma_multiply(0.15),
                            );
                        } else if row_response.hovered() {
                            ui.painter().rect_filled(
                                row,
                                if compact {
                                    controls.fs(11.0)
                                } else {
                                    metrics.fs(9.0)
                                },
                                Color32::from_white_alpha(7),
                            );
                        }
                        let text_left = row.left()
                            + if compact {
                                controls.fs(12.0)
                            } else {
                                metrics.fs(10.0)
                            };
                        let symbol_offset = if compact {
                            controls.fs(7.0)
                        } else {
                            metrics.fs(5.0)
                        };
                        theme::text(
                            ui.painter(),
                            Pos2::new(text_left, row.center().y - symbol_offset),
                            Align2::LEFT_CENTER,
                            unit.symbol,
                            if compact {
                                controls.fs(13.5)
                            } else {
                                metrics.fs(10.5)
                            },
                            Weight::Bold,
                            if current {
                                Palette::VIOLET_SOFT
                            } else {
                                Palette::TEXT
                            },
                        );
                        theme::text(
                            ui.painter(),
                            Pos2::new(text_left, row.center().y + symbol_offset),
                            Align2::LEFT_CENTER,
                            unit.name,
                            if compact {
                                controls.fs(9.5)
                            } else {
                                metrics.fs(7.5)
                            },
                            Weight::Medium,
                            Palette::MUTED,
                        );

                        let star_size = if compact {
                            controls.fs(28.0)
                        } else {
                            metrics.fs(22.0)
                        };
                        let star = Rect::from_center_size(
                            Pos2::new(row.right() - star_size / 2.0, row.center().y),
                            Vec2::splat(star_size),
                        );
                        let star_response = ui.interact(
                            star,
                            ui.scope_id().with(("unit-favorite", category_index, index)),
                            Sense::click(),
                        );
                        let is_favorite = favorites.contains(&(category_index, index));
                        icons::icon(
                            ui.painter(),
                            star.center(),
                            if compact {
                                controls.fs(14.0)
                            } else {
                                metrics.fs(11.0)
                            },
                            Icon::Spark,
                            if is_favorite {
                                Palette::AMBER
                            } else {
                                Palette::FAINT
                            },
                        );
                        if current {
                            icons::icon(
                                ui.painter(),
                                Pos2::new(
                                    row.right()
                                        - star_size
                                        - if compact {
                                            controls.fs(20.0)
                                        } else {
                                            metrics.fs(16.0)
                                        },
                                    row.center().y,
                                ),
                                if compact {
                                    controls.fs(14.0)
                                } else {
                                    metrics.fs(12.0)
                                },
                                Icon::Check,
                                Palette::TEAL_SOFT,
                            );
                        }
                        if star_response.clicked() {
                            toggle_favorite = Some(index);
                        } else if row_response.clicked() {
                            selected = Some(index);
                        }
                    }
                });
        });
    if response.should_close() {
        close = true;
    }
    if let Some(index) = toggle_favorite {
        let key = (category_index, index);
        if let Some(position) = screen_ui.favorites.iter().position(|entry| *entry == key) {
            screen_ui.favorites.remove(position);
        } else {
            screen_ui.favorites.push(key);
        }
    }
    if let Some(index) = selected {
        state.select_unit(target, index);
        close = true;
    }
    if close {
        screen_ui.picker = None;
    }
}
