//! Hand-drawn line icons. No icon font and no extra dependency.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Shape, Stroke, StrokeKind, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Calculator,
    Graph,
    Algebra,
    Converter,
    Tools,
    History,
    Close,
    Copy,
    Backspace,
    ChevronDown,
    Swap,
    Plus,
    Minus,
    Trash,
    Check,
    Search,
    Eye,
    EyeOff,
    ZoomIn,
    ZoomOut,
    Fit,
    ArrowRight,
    Warning,
    Spark,
}

pub fn draw(painter: &Painter, rect: Rect, icon: Icon, tint: Color32) {
    let w = rect.width();
    let h = rect.height();
    let stroke = Stroke::new((w.min(h) * 0.09).max(1.0), tint);
    let p = |x: f32, y: f32| rect.min + Vec2::new(x * w, y * h);
    let line = |a: (f32, f32), b: (f32, f32)| {
        painter.line_segment([p(a.0, a.1), p(b.0, b.1)], stroke);
    };
    let poly = |points: &[(f32, f32)]| {
        painter.add(Shape::line(
            points.iter().map(|(x, y)| p(*x, *y)).collect(),
            stroke,
        ));
    };
    let radius = (w.min(h) * 0.055).max(1.1);

    match icon {
        Icon::Calculator => {
            painter.rect_stroke(
                Rect::from_min_max(p(0.20, 0.08), p(0.80, 0.92)),
                w * 0.12,
                stroke,
                StrokeKind::Inside,
            );
            painter.rect_filled(
                Rect::from_min_max(p(0.31, 0.21), p(0.69, 0.35)),
                radius,
                tint,
            );
            for (x, y) in [
                (0.33, 0.48),
                (0.50, 0.48),
                (0.67, 0.48),
                (0.33, 0.65),
                (0.50, 0.65),
                (0.67, 0.65),
                (0.33, 0.81),
                (0.50, 0.81),
            ] {
                painter.circle_filled(p(x, y), radius, tint);
            }
        }
        Icon::Graph => {
            line((0.16, 0.84), (0.86, 0.84));
            line((0.16, 0.84), (0.16, 0.14));
            let points: Vec<(f32, f32)> = (0..=14)
                .map(|index| {
                    let x = 0.24 + 0.62 * index as f32 / 14.0;
                    let y = 0.52 - 0.24 * ((index as f32 * 0.58).sin());
                    (x, y)
                })
                .collect();
            poly(&points);
            painter.circle_filled(p(0.78, 0.31), radius * 1.3, tint);
        }
        Icon::Algebra => {
            line((0.16, 0.28), (0.46, 0.72));
            line((0.46, 0.28), (0.16, 0.72));
            line((0.58, 0.40), (0.84, 0.40));
            line((0.58, 0.62), (0.84, 0.62));
        }
        Icon::Converter => {
            line((0.16, 0.36), (0.84, 0.36));
            line((0.70, 0.23), (0.84, 0.36));
            line((0.70, 0.49), (0.84, 0.36));
            line((0.84, 0.66), (0.16, 0.66));
            line((0.30, 0.53), (0.16, 0.66));
            line((0.30, 0.79), (0.16, 0.66));
        }
        Icon::Tools => {
            for (y, knob) in [(0.28, 0.64), (0.50, 0.38), (0.72, 0.58)] {
                line((0.16, y), (0.84, y));
                painter.circle_filled(p(knob, y), radius * 1.6, tint);
            }
        }
        Icon::History => {
            painter.circle_stroke(p(0.50, 0.52), w * 0.33, stroke);
            line((0.50, 0.52), (0.50, 0.30));
            line((0.50, 0.52), (0.66, 0.60));
        }
        Icon::Close => {
            line((0.26, 0.26), (0.74, 0.74));
            line((0.74, 0.26), (0.26, 0.74));
        }
        Icon::Copy => {
            painter.rect_stroke(
                Rect::from_min_max(p(0.30, 0.12), p(0.82, 0.64)),
                w * 0.08,
                stroke,
                StrokeKind::Inside,
            );
            painter.rect_stroke(
                Rect::from_min_max(p(0.18, 0.34), p(0.68, 0.88)),
                w * 0.08,
                stroke,
                StrokeKind::Inside,
            );
        }
        Icon::Backspace => {
            poly(&[
                (0.42, 0.18),
                (0.88, 0.18),
                (0.88, 0.82),
                (0.42, 0.82),
                (0.12, 0.50),
                (0.42, 0.18),
            ]);
            line((0.54, 0.37), (0.74, 0.63));
            line((0.74, 0.37), (0.54, 0.63));
        }
        Icon::ChevronDown => {
            line((0.25, 0.38), (0.50, 0.64));
            line((0.50, 0.64), (0.75, 0.38));
        }
        Icon::Swap => {
            line((0.38, 0.18), (0.38, 0.82));
            line((0.38, 0.82), (0.25, 0.68));
            line((0.38, 0.82), (0.51, 0.68));
            line((0.62, 0.82), (0.62, 0.18));
            line((0.62, 0.18), (0.49, 0.32));
            line((0.62, 0.18), (0.75, 0.32));
        }
        Icon::Plus => {
            line((0.50, 0.22), (0.50, 0.78));
            line((0.22, 0.50), (0.78, 0.50));
        }
        Icon::Minus => {
            line((0.22, 0.50), (0.78, 0.50));
        }
        Icon::Trash => {
            line((0.18, 0.28), (0.82, 0.28));
            line((0.38, 0.28), (0.40, 0.16));
            line((0.62, 0.28), (0.60, 0.16));
            line((0.40, 0.16), (0.60, 0.16));
            line((0.26, 0.28), (0.32, 0.86));
            line((0.74, 0.28), (0.68, 0.86));
            line((0.32, 0.86), (0.68, 0.86));
            line((0.46, 0.40), (0.46, 0.74));
            line((0.58, 0.40), (0.58, 0.74));
        }
        Icon::Check => {
            line((0.18, 0.52), (0.42, 0.76));
            line((0.42, 0.76), (0.82, 0.26));
        }
        Icon::Search => {
            painter.circle_stroke(p(0.44, 0.42), w * 0.27, stroke);
            line((0.64, 0.62), (0.84, 0.84));
        }
        Icon::Eye => {
            let mut lens = Vec::new();
            for index in 0..=20 {
                let t = index as f32 / 20.0;
                let x = 0.10 + 0.80 * t;
                let y = 0.50 + 0.22 * (std::f32::consts::PI * t).sin();
                lens.push((x, y));
            }
            for index in (0..=20).rev() {
                let t = index as f32 / 20.0;
                let x = 0.10 + 0.80 * t;
                let y = 0.50 - 0.22 * (std::f32::consts::PI * t).sin();
                lens.push((x, y));
            }
            poly(&lens);
            painter.circle_filled(p(0.50, 0.50), w * 0.10, tint);
        }
        Icon::EyeOff => {
            let mut lens = Vec::new();
            for index in 0..=20 {
                let t = index as f32 / 20.0;
                lens.push((
                    0.10 + 0.80 * t,
                    0.50 + 0.22 * (std::f32::consts::PI * t).sin(),
                ));
            }
            for index in (0..=20).rev() {
                let t = index as f32 / 20.0;
                lens.push((
                    0.10 + 0.80 * t,
                    0.50 - 0.22 * (std::f32::consts::PI * t).sin(),
                ));
            }
            poly(&lens);
            line((0.20, 0.82), (0.80, 0.18));
        }
        Icon::ZoomIn => {
            painter.circle_stroke(p(0.42, 0.42), w * 0.26, stroke);
            line((0.61, 0.61), (0.84, 0.84));
            line((0.26, 0.42), (0.58, 0.42));
            line((0.42, 0.26), (0.42, 0.58));
        }
        Icon::ZoomOut => {
            painter.circle_stroke(p(0.42, 0.42), w * 0.26, stroke);
            line((0.61, 0.61), (0.84, 0.84));
            line((0.26, 0.42), (0.58, 0.42));
        }
        Icon::Fit => {
            line((0.18, 0.36), (0.18, 0.18));
            line((0.18, 0.18), (0.36, 0.18));
            line((0.64, 0.18), (0.82, 0.18));
            line((0.82, 0.18), (0.82, 0.36));
            line((0.82, 0.64), (0.82, 0.82));
            line((0.82, 0.82), (0.64, 0.82));
            line((0.36, 0.82), (0.18, 0.82));
            line((0.18, 0.82), (0.18, 0.64));
        }
        Icon::ArrowRight => {
            line((0.18, 0.50), (0.82, 0.50));
            line((0.64, 0.32), (0.82, 0.50));
            line((0.64, 0.68), (0.82, 0.50));
        }
        Icon::Warning => {
            poly(&[(0.50, 0.12), (0.90, 0.84), (0.10, 0.84), (0.50, 0.12)]);
            line((0.50, 0.38), (0.50, 0.62));
            painter.circle_filled(p(0.50, 0.72), radius, tint);
        }
        Icon::Spark => {
            let points: Vec<(f32, f32)> = (0..8)
                .map(|index| {
                    let angle = std::f32::consts::TAU * index as f32 / 8.0;
                    let radius = if index % 2 == 0 { 0.38 } else { 0.14 };
                    (0.50 + radius * angle.cos(), 0.50 + radius * angle.sin())
                })
                .collect();
            poly(&points);
            poly(&[points[points.len() - 1], points[0]]);
        }
    }
}

pub fn icon(painter: &Painter, center: Pos2, size: f32, icon: Icon, tint: Color32) {
    draw(
        painter,
        Rect::from_center_size(center, Vec2::splat(size)),
        icon,
        tint,
    );
}

pub fn dot(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    painter.circle_filled(center, radius, color);
}

pub fn dashed_line(painter: &Painter, a: Pos2, b: Pos2, stroke: Stroke) {
    let delta = b - a;
    let length = delta.length();
    if length <= 0.0 {
        return;
    }
    let direction = delta / length;
    let dash = 4.0;
    let gap = 4.0;
    let mut start = 0.0;
    while start < length {
        let end = (start + dash).min(length);
        painter.line_segment([a + direction * start, a + direction * end], stroke);
        start += dash + gap;
    }
}

pub fn crosshair(painter: &Painter, center: Pos2, size: f32, color: Color32) {
    let stroke = Stroke::new(1.0, color);
    painter.circle_stroke(center, size, stroke);
    painter.line_segment(
        [
            center - Vec2::new(size + 3.0, 0.0),
            center + Vec2::new(size + 3.0, 0.0),
        ],
        stroke,
    );
    painter.line_segment(
        [
            center - Vec2::new(0.0, size + 3.0),
            center + Vec2::new(0.0, size + 3.0),
        ],
        stroke,
    );
}

pub fn rounded_icon_button(
    painter: &Painter,
    rect: Rect,
    icon_kind: Icon,
    fill: Color32,
    tint: Color32,
) {
    painter.rect_filled(rect, rect.height() / 2.0, fill);
    draw(
        painter,
        Rect::from_center_size(rect.center(), Vec2::splat(rect.width() * 0.46)),
        icon_kind,
        tint,
    );
}

// Keep `egui` imported for future helpers without an unused warning.
#[allow(unused_imports)]
use egui as _egui;
