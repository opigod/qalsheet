//! Custom widgets painted in the Wear app's visual language.

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Id, Layout, Margin, Pos2, Rect, Response, RichText,
    Sense, Stroke, StrokeKind, TextEdit, Ui, UiBuilder, Vec2,
};

use crate::icons::{self, Icon};
use crate::theme::{self, Metrics, Palette, Weight};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyKind {
    Digit,
    Operator,
    Soft,
    Constant,
    Clear,
    Equals,
    Utility,
    Accent,
}

impl KeyKind {
    fn colors(self) -> (Color32, Color32, Color32) {
        match self {
            Self::Digit => (Palette::KEY, Palette::KEY_LIFT, Palette::TEXT),
            Self::Operator => (
                Palette::OPERATOR,
                Palette::OPERATOR_LIFT,
                Palette::VIOLET_SOFT,
            ),
            Self::Soft => (Palette::KEY_LIFT, Palette::OPERATOR_LIFT, Palette::TEXT),
            Self::Constant => (Palette::CARD_LIFT, Palette::OPERATOR, Palette::TEAL_SOFT),
            Self::Clear => (Palette::CLEAR, Palette::CLEAR_LIFT, Palette::PINK),
            Self::Equals => (Palette::VIOLET, Palette::VIOLET_SOFT, Palette::INK),
            Self::Utility => (
                Palette::CARD_LIFT,
                Palette::OPERATOR_LIFT,
                Palette::VIOLET_SOFT,
            ),
            Self::Accent => (Palette::VIOLET_DEEP, Palette::VIOLET, Palette::VIOLET_SOFT),
        }
    }
}

/// Draws a key and returns its response. The caller decides what a click means.
pub fn key(ui: &mut Ui, rect: Rect, label: &str, kind: KeyKind, metrics: &Metrics) -> Response {
    key_with_icon(ui, rect, label, kind, metrics, None)
}

pub fn key_with_icon(
    ui: &mut Ui,
    rect: Rect,
    label: &str,
    kind: KeyKind,
    metrics: &Metrics,
    icon: Option<Icon>,
) -> Response {
    let id = ui.scope_id().with((
        "qal-key",
        label,
        rect.left().to_bits(),
        rect.top().to_bits(),
    ));
    let response = ui.interact(rect, id, Sense::click());
    let pressed = ui.ctx().animate_bool_with_time(
        id.with("down"),
        response.is_pointer_button_down_on(),
        0.05,
    );
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.10);

    let (base, pressed_color, text_color) = kind.colors();
    let mut fill = theme::blend(base, pressed_color, pressed.max(hover * 0.55));
    if response.enabled() == false {
        fill = fill.gamma_multiply(0.5);
    }
    let shrink = pressed * metrics.fs(1.4);
    let visual = rect.shrink(shrink);
    let radius = (visual.height() * 0.5).min(metrics.fs(26.0));
    let border = if pressed > 0.2 {
        Color32::from_white_alpha(40)
    } else {
        Color32::from_white_alpha(18)
    };

    let painter = ui.painter();
    if kind == KeyKind::Equals {
        painter.rect_filled(visual, radius, Palette::VIOLET);
        painter.rect_filled(
            Rect::from_min_max(
                visual.left_top() + Vec2::new(2.0, 1.0),
                visual.right_top() + Vec2::new(-2.0, visual.height() * 0.52),
            ),
            CornerRadius {
                nw: radius as u8,
                ne: radius as u8,
                sw: 0,
                se: 0,
            },
            Palette::VIOLET_SOFT.gamma_multiply(0.24),
        );
        if pressed > 0.0 {
            painter.rect_filled(
                visual,
                radius,
                Palette::VIOLET_SOFT.gamma_multiply(pressed * 0.28),
            );
        }
    } else {
        painter.rect_filled(visual, radius, fill);
    }
    painter.rect_stroke(visual, radius, Stroke::new(1.0, border), StrokeKind::Inside);

    if let Some(icon) = icon {
        let size = visual.height() * 0.42;
        icons::icon(painter, visual.center(), size, icon, text_color);
    } else {
        let base_size = (if label.chars().count() <= 2 {
            metrics.fs(21.0)
        } else if label.chars().count() <= 4 {
            metrics.fs(14.5)
        } else {
            metrics.fs(11.5)
        }) * metrics.key_label_scale;
        let size = theme::fit_size(
            painter,
            label,
            visual.width() - metrics.fs(8.0),
            base_size,
            metrics.fs(7.0),
            if kind == KeyKind::Digit || kind == KeyKind::Equals {
                Weight::ExtraBold
            } else {
                Weight::Bold
            },
        );
        painter.text(
            visual.center(),
            Align2::CENTER_CENTER,
            label,
            theme::font(
                size,
                if kind == KeyKind::Digit || kind == KeyKind::Equals {
                    Weight::ExtraBold
                } else {
                    Weight::Bold
                },
            ),
            text_color,
        );
    }
    response
}

pub fn pill(
    ui: &mut Ui,
    rect: Rect,
    label: &str,
    selected: bool,
    accent: bool,
    metrics: &Metrics,
) -> Response {
    let id = ui.scope_id().with((
        "qal-pill",
        label,
        rect.left().to_bits(),
        rect.top().to_bits(),
    ));
    let response = ui.interact(rect, id, Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.10);
    let fill = if selected {
        Palette::VIOLET
    } else if accent {
        Palette::VIOLET.gamma_multiply(0.18)
    } else {
        theme::blend(Palette::CARD_LIFT, Palette::KEY_LIFT, hover * 0.7)
    };
    let foreground = if selected {
        Palette::INK
    } else if accent {
        Palette::VIOLET_SOFT
    } else {
        Palette::MUTED
    };
    let radius = rect.height() * 0.5;
    let painter = ui.painter();
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(
            1.0,
            if selected {
                Color32::TRANSPARENT
            } else {
                Color32::from_white_alpha(16)
            },
        ),
        StrokeKind::Inside,
    );
    let size = theme::fit_size(
        painter,
        label,
        rect.width() - metrics.fs(12.0),
        metrics.fs(11.0),
        metrics.fs(7.0),
        Weight::SemiBold,
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        theme::font(size, Weight::SemiBold),
        foreground,
    );
    response
}

pub fn icon_button(
    ui: &mut Ui,
    rect: Rect,
    icon: Icon,
    label: &str,
    metrics: &Metrics,
) -> Response {
    let id = ui.scope_id().with((
        "qal-icon",
        label,
        rect.left().to_bits(),
        rect.top().to_bits(),
    ));
    let response = ui.interact(rect, id, Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.10);
    let pressed = ui.ctx().animate_bool_with_time(
        id.with("down"),
        response.is_pointer_button_down_on(),
        0.05,
    );
    let fill = theme::blend(
        Color32::from_white_alpha(10),
        Color32::from_white_alpha(28),
        hover.max(pressed),
    );
    let visual = rect.shrink(pressed * metrics.fs(1.0));
    let painter = ui.painter();
    painter.circle_filled(visual.center(), visual.width() * 0.5, fill);
    painter.circle_stroke(
        visual.center(),
        visual.width() * 0.5,
        Stroke::new(1.0, Color32::from_white_alpha(18)),
    );
    icons::icon(
        painter,
        visual.center(),
        visual.width() * 0.48,
        icon,
        Palette::TEXT,
    );
    response
}

pub fn text_button(
    ui: &mut Ui,
    rect: Rect,
    label: &str,
    accent: bool,
    metrics: &Metrics,
) -> Response {
    let id = ui.scope_id().with((
        "qal-text-button",
        label,
        rect.left().to_bits(),
        rect.top().to_bits(),
    ));
    let response = ui.interact(rect, id, Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.10);
    let fill = if accent {
        theme::blend(Palette::VIOLET, Palette::VIOLET_SOFT, hover * 0.2)
    } else {
        theme::blend(Palette::CARD_LIFT, Palette::KEY_LIFT, hover)
    };
    let fg = if accent { Palette::INK } else { Palette::TEXT };
    let radius = rect.height() * 0.5;
    let painter = ui.painter();
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, Color32::from_white_alpha(16)),
        StrokeKind::Inside,
    );
    let size = theme::fit_size(
        painter,
        label,
        rect.width() - metrics.fs(14.0),
        metrics.fs(12.0),
        metrics.fs(7.0),
        Weight::Bold,
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        theme::font(size, Weight::Bold),
        fg,
    );
    response
}

pub fn segmented(
    ui: &mut Ui,
    rect: Rect,
    labels: &[&str],
    selected: usize,
    metrics: &Metrics,
) -> Option<usize> {
    let radius = rect.height() * 0.5;
    let painter = ui.painter();
    painter.rect_filled(rect, radius, Palette::CARD_LIFT);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, Color32::from_white_alpha(16)),
        StrokeKind::Inside,
    );
    let gap = metrics.fs(2.0);
    let segment_width = (rect.width() - gap * (labels.len() as f32 + 1.0)) / labels.len() as f32;
    let mut chosen = None;
    for (index, label) in labels.iter().enumerate() {
        let x = rect.left() + gap + index as f32 * (segment_width + gap);
        let segment = Rect::from_min_size(
            Pos2::new(x, rect.top() + gap),
            Vec2::new(segment_width, rect.height() - gap * 2.0),
        );
        let id = ui.scope_id().with(("seg", index, rect.left().to_bits()));
        let response = ui.interact(segment, id, Sense::click());
        if response.clicked() {
            chosen = Some(index);
        }
        if index == selected {
            painter.rect_filled(segment, segment.height() * 0.5, Palette::VIOLET);
        } else if response.hovered() {
            painter.rect_filled(
                segment,
                segment.height() * 0.5,
                Color32::from_white_alpha(8),
            );
        }
        let color = if index == selected {
            Palette::INK
        } else {
            Palette::MUTED
        };
        let size = theme::fit_size(
            painter,
            label,
            segment.width() - metrics.fs(8.0),
            metrics.fs(10.5),
            metrics.fs(6.5),
            Weight::Bold,
        );
        painter.text(
            segment.center(),
            Align2::CENTER_CENTER,
            *label,
            theme::font(size, Weight::Bold),
            color,
        );
    }
    chosen
}

/// A single-line text field with the app's card treatment.
pub fn field(
    ui: &mut Ui,
    rect: Rect,
    text: &mut String,
    hint: &str,
    metrics: &Metrics,
) -> Response {
    field_impl(ui, rect, text, hint, metrics, false, 80)
}

pub fn field_multiline(
    ui: &mut Ui,
    rect: Rect,
    text: &mut String,
    hint: &str,
    metrics: &Metrics,
) -> Response {
    field_impl(ui, rect, text, hint, metrics, true, 4000)
}

fn field_impl(
    ui: &mut Ui,
    rect: Rect,
    text: &mut String,
    hint: &str,
    metrics: &Metrics,
    multiline: bool,
    limit: usize,
) -> Response {
    let id = ui
        .scope_id()
        .with(("qal-field", rect.left().to_bits(), rect.top().to_bits()));
    ui.painter()
        .rect_filled(rect, metrics.fs(12.0), Color32::from_rgb(0x13, 0x10, 0x1A));
    let inner = rect.shrink2(Vec2::new(metrics.fs(11.0), metrics.fs(7.0)));
    let mut child = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::left_to_right(Align::Center)),
    );
    let edit = if multiline {
        TextEdit::multiline(text)
            .desired_rows(4)
            .desired_width(inner.width())
            .char_limit(limit)
            .margin(Margin::ZERO)
            .frame(egui::Frame::NONE)
            .font(theme::font_selection(metrics.fs(13.5), Weight::Medium))
            .text_color(Palette::TEXT)
            .hint_text(RichText::new(hint).color(Palette::FAINT))
            .id(id)
    } else {
        TextEdit::singleline(text)
            .desired_width(inner.width())
            .char_limit(limit)
            .margin(Margin::ZERO)
            .frame(egui::Frame::NONE)
            .font(theme::font_selection(metrics.fs(14.0), Weight::Medium))
            .text_color(Palette::TEXT)
            .hint_text(RichText::new(hint).color(Palette::FAINT))
            .id(id)
    };
    let response = child.add(edit);
    let border = if response.has_focus() {
        Palette::VIOLET
    } else if response.hovered() {
        Palette::BORDER
    } else {
        Color32::from_white_alpha(16)
    };
    ui.painter().rect_stroke(
        rect,
        metrics.fs(12.0),
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
    response
}

pub fn dropdown(ui: &mut Ui, rect: Rect, symbol: &str, name: &str, metrics: &Metrics) -> Response {
    let id = ui
        .scope_id()
        .with(("qal-dropdown", rect.left().to_bits(), rect.top().to_bits()));
    let response = ui.interact(rect, id, Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.10);
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        metrics.fs(12.0),
        theme::blend(Palette::CARD_LIFT, Palette::KEY_LIFT, hover),
    );
    painter.rect_stroke(
        rect,
        metrics.fs(12.0),
        Stroke::new(1.0, Color32::from_white_alpha(18)),
        StrokeKind::Inside,
    );
    let left = rect.left() + metrics.fs(11.0);
    let right = rect.right() - metrics.fs(28.0);
    let symbol_rect = Rect::from_min_max(
        Pos2::new(left, rect.top() + metrics.fs(3.0)),
        Pos2::new(right, rect.center().y + metrics.fs(0.5)),
    );
    let name_rect = Rect::from_min_max(
        Pos2::new(left, rect.center().y + metrics.fs(0.5)),
        Pos2::new(right, rect.bottom() - metrics.fs(2.0)),
    );
    theme::fitted_text(
        painter,
        symbol_rect,
        Align2::LEFT_CENTER,
        symbol,
        metrics.fs(12.5),
        metrics.fs(7.0),
        Weight::Bold,
        Palette::TEXT,
    );
    theme::fitted_text(
        painter,
        name_rect,
        Align2::LEFT_CENTER,
        name,
        metrics.fs(7.5),
        metrics.fs(5.0),
        Weight::Medium,
        Palette::MUTED,
    );
    icons::icon(
        painter,
        Pos2::new(rect.right() - metrics.fs(16.0), rect.center().y),
        metrics.fs(12.0),
        Icon::ChevronDown,
        Palette::MUTED,
    );
    response
}

pub fn icon_toggle(ui: &mut Ui, rect: Rect, icon: Icon, active: bool, label: &str) -> Response {
    let id = ui
        .scope_id()
        .with(("toggle", label, rect.left().to_bits(), rect.top().to_bits()));
    let response = ui.interact(rect, id, Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.1);
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        rect.height() * 0.5,
        if active {
            Palette::VIOLET.gamma_multiply(0.22)
        } else {
            theme::blend(Color32::TRANSPARENT, Color32::from_white_alpha(10), hover)
        },
    );
    icons::icon(
        painter,
        rect.center(),
        rect.width() * 0.52,
        icon,
        if active {
            Palette::VIOLET_SOFT
        } else {
            Palette::MUTED
        },
    );
    response
}

pub fn toast(ui: &Ui, text: &str, metrics: &Metrics) {
    if text.is_empty() {
        return;
    }
    let screen = ui.ctx().content_rect();
    let painter = ui.ctx().layer_painter(egui::LayerId::new(
        egui::Order::Tooltip,
        Id::unique("qal-toast"),
    ));
    let width =
        theme::text_width(&painter, text, metrics.fs(11.0), Weight::SemiBold) + metrics.fs(26.0);
    let rect = Rect::from_center_size(
        screen.center_bottom() - Vec2::new(0.0, metrics.fs(86.0)),
        Vec2::new(width.min(screen.width() - 20.0), metrics.fs(28.0)),
    );
    painter.rect_filled(rect, rect.height() * 0.5, Palette::CARD_LIFT);
    painter.rect_stroke(
        rect,
        rect.height() * 0.5,
        Stroke::new(1.0, Palette::VIOLET.gamma_multiply(0.5)),
        StrokeKind::Inside,
    );
    theme::fitted_text(
        &painter,
        rect.shrink2(Vec2::new(metrics.fs(12.0), 0.0)),
        Align2::CENTER_CENTER,
        text,
        metrics.fs(11.0),
        metrics.fs(7.0),
        Weight::SemiBold,
        Palette::TEXT,
    );
}

/// A card header with an optional close button. Returns true when closed.
pub fn modal_header(
    ui: &mut Ui,
    title: &str,
    subtitle: &str,
    metrics: &Metrics,
    close: bool,
) -> bool {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, metrics.fs(30.0)), Sense::hover());
    let title_y = if subtitle.is_empty() {
        rect.center().y
    } else {
        rect.center().y - metrics.fs(5.0)
    };
    theme::text(
        ui.painter(),
        Pos2::new(rect.left(), title_y),
        Align2::LEFT_CENTER,
        title,
        metrics.fs(16.0),
        Weight::Bold,
        Palette::TEXT,
    );
    if !subtitle.is_empty() {
        theme::text(
            ui.painter(),
            Pos2::new(rect.left(), rect.center().y + metrics.fs(7.0)),
            Align2::LEFT_CENTER,
            subtitle,
            metrics.fs(9.0),
            Weight::Medium,
            Palette::MUTED,
        );
    }
    if close {
        let size = metrics.fs(28.0);
        let button = Rect::from_min_size(
            Pos2::new(rect.right() - size, rect.center().y - size / 2.0),
            Vec2::splat(size),
        );
        icon_button(ui, button, Icon::Close, "Close", metrics).clicked()
    } else {
        false
    }
}

pub fn empty_state(ui: &mut Ui, rect: Rect, text: &str, metrics: &Metrics) {
    theme::text(
        ui.painter(),
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        metrics.fs(11.0),
        Weight::Medium,
        Palette::MUTED,
    );
}

pub fn row_separator(painter: &egui::Painter, rect: Rect) {
    painter.line_segment(
        [
            Pos2::new(rect.left(), rect.center().y),
            Pos2::new(rect.right(), rect.center().y),
        ],
        Stroke::new(1.0, Color32::from_white_alpha(10)),
    );
}

// Alias used by screens that want the palette without importing theme.
pub use crate::theme::Palette as Colors;

pub fn set_clip(ui: &mut Ui, rect: Rect) {
    ui.set_clip_rect(rect);
}
