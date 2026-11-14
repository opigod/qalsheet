use eframe::egui::{self, Align2, Color32, Pos2, Rect, Stroke, StrokeKind, Ui, Vec2};

use crate::theme::{self, Metrics, Palette, Weight};

/// Splits an area into a uniform row-major grid.
pub fn grid(area: Rect, rows: usize, cols: usize, gap: f32) -> Vec<Rect> {
    if rows == 0 || cols == 0 {
        return Vec::new();
    }
    let width = (area.width() - gap * (cols as f32 - 1.0)) / cols as f32;
    let height = (area.height() - gap * (rows as f32 - 1.0)) / rows as f32;
    let mut cells = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        for col in 0..cols {
            cells.push(Rect::from_min_size(
                Pos2::new(
                    area.left() + col as f32 * (width + gap),
                    area.top() + row as f32 * (height + gap),
                ),
                Vec2::new(width, height),
            ));
        }
    }
    cells
}

/// The side length of square key cells that fit inside `area`.
pub fn square_key_size(area: Rect, rows: usize, cols: usize, gap: f32) -> f32 {
    if rows == 0 || cols == 0 {
        return 0.0;
    }
    let by_width = (area.width() - gap * (cols as f32 - 1.0)) / cols as f32;
    let by_height = (area.height() - gap * (rows as f32 - 1.0)) / rows as f32;
    by_width.min(by_height).max(0.0)
}

/// Splits `area` into equally sized square cells, centred as one block.
///
/// Android phones use this instead of [`grid`] so keys keep a 1:1 shape rather
/// than stretching to fill the extra height.
pub fn square_grid(area: Rect, rows: usize, cols: usize, gap: f32) -> Vec<Rect> {
    if rows == 0 || cols == 0 {
        return Vec::new();
    }
    let key = square_key_size(area, rows, cols, gap);
    let width = key * cols as f32 + gap * (cols as f32 - 1.0);
    let height = key * rows as f32 + gap * (rows as f32 - 1.0);
    let left = area.center().x - width / 2.0;
    let top = area.center().y - height / 2.0;
    let mut cells = Vec::with_capacity(rows * cols);
    for row in 0..rows {
        for col in 0..cols {
            cells.push(Rect::from_min_size(
                Pos2::new(
                    left + col as f32 * (key + gap),
                    top + row as f32 * (key + gap),
                ),
                Vec2::splat(key),
            ));
        }
    }
    cells
}

/// A row with `count` square cells centred under a square `total`-column grid.
pub fn square_centered_row(
    area: Rect,
    row: usize,
    rows: usize,
    count: usize,
    total: usize,
    gap: f32,
) -> Vec<Rect> {
    let key = square_key_size(area, rows, total, gap);
    let width = key * total as f32 + gap * (total as f32 - 1.0);
    let grid_left = area.center().x - width / 2.0;
    let grid_height = key * rows as f32 + gap * (rows as f32 - 1.0);
    let grid_top = area.center().y - grid_height / 2.0;
    let row_width = key * count as f32 + gap * (count as f32 - 1.0);
    let left = grid_left + width / 2.0 - row_width / 2.0;
    let top = grid_top + row as f32 * (key + gap);
    (0..count)
        .map(|index| {
            Rect::from_min_size(
                Pos2::new(left + index as f32 * (key + gap), top),
                Vec2::splat(key),
            )
        })
        .collect()
}

/// A row with `count` centred cells out of `total` columns.
pub fn centered_row(
    area: Rect,
    row: usize,
    rows: usize,
    count: usize,
    total: usize,
    gap: f32,
) -> Vec<Rect> {
    let width = (area.width() - gap * (total as f32 - 1.0)) / total as f32;
    let height = (area.height() - gap * (rows as f32 - 1.0)) / rows as f32;
    let row_width = width * count as f32 + gap * (count as f32 - 1.0);
    let left = area.center().x - row_width / 2.0;
    (0..count)
        .map(|index| {
            Rect::from_min_size(
                Pos2::new(
                    left + index as f32 * (width + gap),
                    area.top() + row as f32 * (height + gap),
                ),
                Vec2::new(width, height),
            )
        })
        .collect()
}

pub fn page_title(
    ui: &mut Ui,
    rect: Rect,
    title: &str,
    subtitle: &str,
    metrics: &Metrics,
    action: Option<&str>,
) {
    let painter = ui.painter();
    theme::text(
        painter,
        Pos2::new(rect.left(), rect.center().y - metrics.fs(6.0)),
        Align2::LEFT_CENTER,
        title,
        metrics.fs(22.0),
        Weight::ExtraBold,
        Palette::TEXT,
    );
    if !subtitle.is_empty() {
        theme::text(
            painter,
            Pos2::new(rect.left(), rect.center().y + metrics.fs(13.0)),
            Align2::LEFT_CENTER,
            subtitle,
            metrics.fs(9.5),
            Weight::Medium,
            Palette::MUTED,
        );
    }
    if let Some(action) = action {
        let width = metrics.fs(80.0);
        let pill = Rect::from_min_size(
            Pos2::new(rect.right() - width, rect.center().y - metrics.fs(13.0)),
            Vec2::new(width, metrics.fs(26.0)),
        );
        crate::widgets::pill(ui, pill, action, false, true, metrics);
    }
}

pub fn section_label(painter: &egui::Painter, pos: Pos2, text: &str, metrics: &Metrics) {
    painter.text(
        pos,
        Align2::LEFT_CENTER,
        text,
        theme::font(metrics.fs(8.0), Weight::Bold),
        Palette::FAINT,
    );
}

pub fn accent_line(painter: &egui::Painter, rect: Rect, color: Color32) {
    painter.line_segment(
        [
            Pos2::new(rect.left(), rect.bottom()),
            Pos2::new(rect.right(), rect.bottom()),
        ],
        Stroke::new(1.0, color),
    );
}

pub fn stat_tile(
    ui: &mut Ui,
    rect: Rect,
    label: &str,
    value: &str,
    accent: Color32,
    metrics: &Metrics,
) {
    let painter = ui.painter();
    theme::card(
        painter,
        rect,
        metrics.fs(12.0),
        Palette::CARD,
        Color32::from_white_alpha(14),
    );
    theme::text(
        painter,
        Pos2::new(
            rect.left() + metrics.fs(10.0),
            rect.top() + metrics.fs(13.0),
        ),
        Align2::LEFT_CENTER,
        label,
        metrics.fs(8.0),
        Weight::SemiBold,
        Palette::MUTED,
    );
    theme::fitted_text(
        painter,
        Rect::from_min_max(
            Pos2::new(
                rect.left() + metrics.fs(10.0),
                rect.top() + metrics.fs(20.0),
            ),
            Pos2::new(
                rect.right() - metrics.fs(8.0),
                rect.bottom() - metrics.fs(6.0),
            ),
        ),
        Align2::LEFT_CENTER,
        value,
        metrics.fs(16.0),
        metrics.fs(8.0),
        Weight::Bold,
        accent,
    );
}

pub fn horizontal_rule(ui: &Ui, rect: Rect, metrics: &Metrics) {
    let y = rect.center().y;
    ui.painter().line_segment(
        [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
        Stroke::new(1.0, Color32::from_white_alpha(12)),
    );
    let _ = metrics;
}

pub fn badge(painter: &egui::Painter, rect: Rect, text: &str, color: Color32, metrics: &Metrics) {
    painter.rect_filled(rect, rect.height() * 0.5, color.gamma_multiply(0.16));
    theme::fitted_text(
        painter,
        rect,
        Align2::CENTER_CENTER,
        text,
        metrics.fs(8.0),
        metrics.fs(6.0),
        Weight::Bold,
        color,
    );
}

pub fn error_card(ui: &mut Ui, rect: Rect, message: &str, metrics: &Metrics) {
    let painter = ui.painter();
    theme::card(
        painter,
        rect,
        metrics.fs(12.0),
        Palette::ERROR.gamma_multiply(0.10),
        Palette::ERROR.gamma_multiply(0.35),
    );
    theme::text(
        painter,
        Pos2::new(rect.left() + metrics.fs(12.0), rect.center().y),
        Align2::LEFT_CENTER,
        message,
        metrics.fs(10.0),
        Weight::Medium,
        Palette::ERROR,
    );
}

pub fn cursor_rect(ui: &Ui) -> Rect {
    ui.max_rect()
}

pub fn fill(ui: &Ui, rect: Rect, color: Color32, radius: f32) {
    ui.painter().rect_filled(rect, radius, color);
}

pub fn stroke(ui: &Ui, rect: Rect, color: Color32, radius: f32) {
    ui.painter()
        .rect_stroke(rect, radius, Stroke::new(1.0, color), StrokeKind::Inside);
}

pub fn modal_width(ui: &Ui, max: f32) -> f32 {
    let screen = ui.ctx().content_rect();
    if screen.width() < 320.0 {
        (screen.width() * 0.68).min(max).max(140.0)
    } else {
        max.min(screen.width() - 28.0).max(150.0)
    }
}

pub fn modal_max_height(ui: &Ui) -> f32 {
    let screen = ui.ctx().content_rect();
    if screen.width() < 320.0 {
        screen.height() * 0.68
    } else {
        (screen.height() * 0.82).max(180.0)
    }
}

pub fn close_on_escape(ctx: &egui::Context) -> bool {
    ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
}

/// Rows of keys shaped for a round watch face. The usable width at a given
/// vertical position is the circle chord, so lower rows are narrower instead
/// of being clipped by the bezel.
pub fn watch_row(
    ui: &Ui,
    area: Rect,
    row: usize,
    rows: usize,
    cols: usize,
    count: usize,
    gap: f32,
) -> Vec<Rect> {
    let screen = ui.ctx().content_rect();
    let center = screen.center();
    let radius = screen.width().min(screen.height()) * 0.5 - gap * 1.2;
    let row_height = (area.height() - gap * (rows as f32 - 1.0)) / rows as f32;
    let top = area.top() + row as f32 * (row_height + gap);
    let bottom = top + row_height;
    let dy = (top - center.y).abs().max((bottom - center.y).abs());
    let half = (radius * radius - dy * dy).max(0.0).sqrt();
    let available = ((half * 2.0) - gap * 2.0).clamp(80.0, area.width());
    let key_width = (available - gap * (cols as f32 - 1.0)) / cols as f32;
    let group_width = key_width * count as f32 + gap * (count as f32 - 1.0);
    let start = center.x - group_width / 2.0;
    (0..count)
        .map(|index| {
            Rect::from_min_size(
                Pos2::new(start + index as f32 * (key_width + gap), top),
                Vec2::new(key_width, row_height),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn square_grid_keeps_cells_square_and_centres_the_block() {
        let area = Rect::from_min_size(Pos2::new(12.0, 254.0), Vec2::new(366.0, 465.0));
        let cells = square_grid(area, 5, 5, 4.7);

        assert_eq!(cells.len(), 25);
        for cell in &cells {
            assert!(close(cell.width(), cell.height()));
        }

        let key = cells[0].width();
        let width = key * 5.0 + 4.7 * 4.0;
        let height = key * 5.0 + 4.7 * 4.0;
        assert!(close(cells[0].left(), area.center().x - width / 2.0));
        assert!(close(cells[0].top(), area.center().y - height / 2.0));
        assert!(close(cells[24].right(), area.center().x + width / 2.0));
        assert!(close(cells[24].bottom(), area.center().y + height / 2.0));
    }

    #[test]
    fn square_key_size_shrinks_to_fit_the_shorter_side() {
        let wide = Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 300.0));
        let cells = square_grid(wide, 5, 5, 4.0);
        assert!(close(cells[0].width(), (300.0 - 16.0) / 5.0));
        assert!(close(cells[0].height(), cells[0].width()));
    }

    #[test]
    fn square_centered_row_matches_the_main_grid() {
        let area = Rect::from_min_size(Pos2::new(12.0, 238.0), Vec2::new(366.0, 460.0));
        let grid = square_grid(area, 4, 5, 4.7);
        let row = square_centered_row(area, 3, 4, 4, 5, 4.7);

        assert_eq!(row.len(), 4);
        assert!(close(row[0].top(), grid[15].top()));
        assert!(close(row[3].top(), grid[15].top()));
        assert!(close(row[0].width(), grid[15].width()));

        let row_width = row[3].right() - row[0].left();
        assert!(close(row[0].left(), area.center().x - row_width / 2.0));
    }
}
