//! Theme tokens, fonts and small painting helpers.
//!
//! The palette and Nunito typography are the same as the Wear app. The extra
//! graph colours only appear on screens the watch does not have.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, FontData, FontDefinitions, FontFamily, FontId, FontSelection, Painter, Pos2,
    Rect, Stroke, StrokeKind, Ui, Vec2,
};

#[derive(Clone, Copy, Debug)]
pub struct Palette;

impl Palette {
    pub const INK: Color32 = Color32::from_rgb(0x07, 0x06, 0x0B);
    pub const INK_LIFT: Color32 = Color32::from_rgb(0x10, 0x0C, 0x18);
    pub const CARD: Color32 = Color32::from_rgb(0x17, 0x14, 0x1F);
    pub const CARD_LIFT: Color32 = Color32::from_rgb(0x21, 0x1D, 0x2B);
    pub const KEY: Color32 = Color32::from_rgb(0x24, 0x21, 0x2E);
    pub const KEY_LIFT: Color32 = Color32::from_rgb(0x30, 0x2C, 0x3D);
    pub const OPERATOR: Color32 = Color32::from_rgb(0x35, 0x30, 0x4A);
    pub const OPERATOR_LIFT: Color32 = Color32::from_rgb(0x43, 0x3C, 0x5C);
    pub const CLEAR: Color32 = Color32::from_rgb(0x3F, 0x24, 0x33);
    pub const CLEAR_LIFT: Color32 = Color32::from_rgb(0x51, 0x2D, 0x42);

    pub const BORDER: Color32 = Color32::from_rgb(0x34, 0x2F, 0x41);
    pub const BORDER_SOFT: Color32 = Color32::from_rgba_premultiplied(0x14, 0x14, 0x18, 0x20);
    pub const TEXT: Color32 = Color32::from_rgb(0xF7, 0xF3, 0xFF);
    pub const MUTED: Color32 = Color32::from_rgb(0xA3, 0x9B, 0xB7);
    pub const FAINT: Color32 = Color32::from_rgb(0x75, 0x6C, 0x88);

    pub const VIOLET: Color32 = Color32::from_rgb(0x8B, 0x5C, 0xF6);
    pub const VIOLET_SOFT: Color32 = Color32::from_rgb(0xC4, 0xB5, 0xFD);
    pub const VIOLET_DEEP: Color32 = Color32::from_rgb(0x4F, 0x37, 0x8B);
    pub const SLATE: Color32 = Color32::from_rgb(0x44, 0x45, 0x59);
    pub const TEAL: Color32 = Color32::from_rgb(0x2D, 0xD4, 0xBF);
    pub const TEAL_SOFT: Color32 = Color32::from_rgb(0x99, 0xF6, 0xE4);
    pub const PINK: Color32 = Color32::from_rgb(0xFF, 0x9E, 0xC4);
    pub const ERROR: Color32 = Color32::from_rgb(0xFF, 0x6B, 0x81);
    pub const AMBER: Color32 = Color32::from_rgb(0xFF, 0xD1, 0x66);
    pub const BLUE: Color32 = Color32::from_rgb(0x60, 0xA5, 0xFA);
    pub const LIME: Color32 = Color32::from_rgb(0xA3, 0xE6, 0x35);
}

/// Android phone key labels are 20% larger than the shared baseline.
const ANDROID_KEY_LABEL_SCALE: f32 = 1.2;
/// The small Android controls (mode switch, unit picker, stat labels) need
/// more than the 0.94 phone density scale to stay legible.
const ANDROID_CONTROL_SCALE: f32 = 1.3;
/// Desktop controls use the same baseline as Android but have more room.
const DESKTOP_CONTROL_SCALE: f32 = 1.2;

pub const GRAPH_COLORS: [Color32; 6] = [
    Palette::VIOLET,
    Palette::TEAL,
    Palette::PINK,
    Palette::AMBER,
    Palette::BLUE,
    Palette::LIME,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
}

impl Weight {
    fn name(self) -> &'static str {
        match self {
            Self::Regular => "nunito-regular",
            Self::Medium => "nunito-medium",
            Self::SemiBold => "nunito-semibold",
            Self::Bold => "nunito-bold",
            Self::ExtraBold => "nunito-extrabold",
        }
    }

    pub fn family(self) -> FontFamily {
        FontFamily::Name(Arc::from(self.name()))
    }
}

pub fn font(size: f32, weight: Weight) -> FontId {
    FontId::new(size, weight.family())
}

pub fn font_selection(size: f32, weight: Weight) -> FontSelection {
    FontSelection::FontId(font(size, weight))
}

/// Installs Nunito and a dark, transparent style. Call once at startup.
pub fn install(ctx: &egui::Context) {
    install_fonts(ctx);
    ctx.set_theme(egui::ThemePreference::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.override_text_color = Some(Palette::TEXT);
    style.visuals.panel_fill = Color32::TRANSPARENT;
    style.visuals.window_fill = Palette::CARD;
    style.visuals.window_stroke = Stroke::new(1.0, Palette::BORDER);
    style.visuals.extreme_bg_color = Palette::CARD;
    style.visuals.faint_bg_color = Palette::CARD_LIFT;
    style.visuals.code_bg_color = Palette::CARD_LIFT;
    style.visuals.selection.bg_fill = Palette::VIOLET.gamma_multiply(0.55);
    style.visuals.selection.stroke = Stroke::new(1.0, Palette::VIOLET_SOFT);
    style.visuals.hyperlink_color = Palette::TEAL;
    style.visuals.widgets.noninteractive.bg_fill = Palette::CARD;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Palette::BORDER);
    style.visuals.widgets.inactive.bg_fill = Palette::KEY;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Palette::BORDER_SOFT);
    style.visuals.widgets.hovered.bg_fill = Palette::KEY_LIFT;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Palette::BORDER);
    style.visuals.widgets.active.bg_fill = Palette::OPERATOR_LIFT;
    style.visuals.widgets.open.bg_fill = Palette::CARD_LIFT;
    style.spacing.item_spacing = Vec2::new(8.0, 8.0);
    style.spacing.button_padding = Vec2::new(10.0, 6.0);
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.spacing.scroll.bar_width = 4.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_outer_margin = 2.0;
    style.spacing.scroll.floating = false;
    style.spacing.interact_size.y = 28.0;
    ctx.set_style_of(egui::Theme::Dark, style);
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let faces: [(&str, &[u8]); 5] = [
        (
            "nunito-regular",
            include_bytes!("../fonts/nunito_regular.ttf"),
        ),
        (
            "nunito-medium",
            include_bytes!("../fonts/nunito_medium.ttf"),
        ),
        (
            "nunito-semibold",
            include_bytes!("../fonts/nunito_semibold.ttf"),
        ),
        ("nunito-bold", include_bytes!("../fonts/nunito_bold.ttf")),
        (
            "nunito-extrabold",
            include_bytes!("../fonts/nunito_extrabold.ttf"),
        ),
    ];
    for (name, bytes) in faces {
        fonts
            .font_data
            .insert(name.to_string(), Arc::new(FontData::from_static(bytes)));
    }

    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let monospace_fallback = fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();

    for weight in [
        Weight::Regular,
        Weight::Medium,
        Weight::SemiBold,
        Weight::Bold,
        Weight::ExtraBold,
    ] {
        let mut chain = vec![weight.name().to_string()];
        chain.extend(fallback.iter().cloned());
        fonts.families.insert(weight.family(), chain);
    }

    // Text fields and default widgets use the medium face.
    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        proportional.retain(|name| !name.starts_with("nunito"));
        proportional.insert(0, "nunito-medium".to_string());
    }
    if let Some(monospace) = fonts.families.get_mut(&FontFamily::Monospace) {
        monospace.extend(monospace_fallback);
    }
    ctx.set_fonts(fonts);
}

#[derive(Clone, Copy, Debug)]
pub struct Metrics {
    pub watch: bool,
    pub wide: bool,
    pub compact: bool,
    pub scale: f32,
    pub top_inset: f32,
    pub bottom_inset: f32,
    /// Android phone keypads use square keys instead of filling the height.
    pub square_keys: bool,
    /// Extra size for key labels on Android phones. Other platforms keep 1.
    pub key_label_scale: f32,
}

impl Metrics {
    pub fn new(width: f32, height: f32, flavor_wear: bool) -> Self {
        let watch = flavor_wear || (width < 320.0 && height < 430.0);
        let wide = !watch && width >= 780.0;
        let compact = !watch && !wide;
        let scale = if watch {
            0.62
        } else if compact {
            if width >= 420.0 { 1.0 } else { 0.94 }
        } else {
            1.0
        };
        let top_inset = if flavor_wear {
            2.0
        } else if cfg!(target_os = "android") {
            4.0
        } else {
            0.0
        };
        let bottom_inset = if cfg!(target_os = "android") {
            22.0
        } else {
            0.0
        };
        // The stretched-key problem only happens on the Android phone layout.
        // The watch keeps its own keypad geometry and desktop windows keep the
        // wider keys.
        let android_phone = cfg!(target_os = "android") && !watch && compact;
        Self {
            watch,
            wide,
            compact,
            scale,
            top_inset,
            bottom_inset,
            square_keys: android_phone,
            key_label_scale: if android_phone {
                ANDROID_KEY_LABEL_SCALE
            } else {
                1.0
            },
        }
    }

    pub fn fs(&self, base: f32) -> f32 {
        (base * self.scale).max(6.0)
    }

    /// Returns a copy scaled for a specific control group. Layout callers use
    /// this for compact Android headers and lists without scaling the keycaps.
    pub fn scaled(&self, factor: f32) -> Self {
        let mut scaled = *self;
        scaled.scale *= factor;
        scaled
    }

    /// Metrics for small controls: larger on Android, moderately larger on
    /// desktop, unchanged on the watch.
    pub fn controls(&self) -> Self {
        if self.watch {
            *self
        } else if self.square_keys {
            self.scaled(ANDROID_CONTROL_SCALE)
        } else {
            self.scaled(DESKTOP_CONTROL_SCALE)
        }
    }

    pub fn gap(&self) -> f32 {
        self.fs(8.0)
    }

    pub fn pad(&self) -> f32 {
        self.fs(14.0)
    }

    pub fn radius(&self) -> f32 {
        self.fs(20.0)
    }
}

pub fn text(
    painter: &Painter,
    position: Pos2,
    anchor: egui::Align2,
    value: impl ToString,
    size: f32,
    weight: Weight,
    color: Color32,
) {
    painter.text(position, anchor, value, font(size, weight), color);
}

pub fn text_width(painter: &Painter, value: &str, size: f32, weight: Weight) -> f32 {
    painter
        .layout_no_wrap(value.to_string(), font(size, weight), Palette::TEXT)
        .size()
        .x
}

/// Shrinks a single line until it fits `max_width`.
pub fn fit_size(
    painter: &Painter,
    value: &str,
    max_width: f32,
    max_size: f32,
    min_size: f32,
    weight: Weight,
) -> f32 {
    let mut size = max_size;
    while size > min_size {
        if text_width(painter, value, size, weight) <= max_width {
            return size;
        }
        size = (size - 0.5).max(min_size);
    }
    min_size
}

pub fn fitted_text(
    painter: &Painter,
    rect: Rect,
    anchor: egui::Align2,
    value: &str,
    max_size: f32,
    min_size: f32,
    weight: Weight,
    color: Color32,
) -> f32 {
    let size = fit_size(painter, value, rect.width(), max_size, min_size, weight);
    let galley = painter.layout_no_wrap(value.to_string(), font(size, weight), color);
    let text_size = galley.size();
    let offset = match anchor {
        egui::Align2::LEFT_CENTER | egui::Align2::LEFT_TOP | egui::Align2::LEFT_BOTTOM => {
            Pos2::new(rect.left(), rect.center().y - text_size.y / 2.0)
        }
        egui::Align2::RIGHT_CENTER | egui::Align2::RIGHT_TOP | egui::Align2::RIGHT_BOTTOM => {
            Pos2::new(
                rect.right() - text_size.x,
                rect.center().y - text_size.y / 2.0,
            )
        }
        _ => rect.center() - text_size / 2.0,
    };
    painter.galley(offset, galley, color);
    size
}

pub fn blend(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_unmultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}

pub fn lighten(color: Color32, amount: f32) -> Color32 {
    blend(color, Color32::WHITE, amount)
}

pub fn card(painter: &Painter, rect: Rect, radius: f32, fill: Color32, border: Color32) {
    painter.rect_filled(rect, radius, fill);
    painter.rect_stroke(rect, radius, Stroke::new(1.0, border), StrokeKind::Inside);
}

pub fn card_with_shadow(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    fill: Color32,
    border: Color32,
) {
    let shadow = Color32::from_black_alpha(60);
    painter.rect_filled(rect.translate(Vec2::new(0.0, 3.0)), radius + 2.0, shadow);
    card(painter, rect, radius, fill, border);
}

pub fn gradient_background(painter: &Painter, rect: Rect) {
    let mut mesh = egui::Mesh::default();
    let top = Palette::INK_LIFT;
    let middle = Palette::INK;
    let bottom = Color32::from_rgb(0x06, 0x04, 0x0A);
    mesh.colored_vertex(rect.left_top(), top);
    mesh.colored_vertex(rect.right_top(), top);
    mesh.colored_vertex(rect.left_bottom(), bottom);
    mesh.colored_vertex(rect.right_bottom(), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 2, 3);
    painter.add(egui::Shape::mesh(mesh));

    radial_glow(
        painter,
        rect.left_top() + Vec2::new(rect.width() * 0.55, -rect.height() * 0.05),
        rect.width().max(rect.height()) * 0.62,
        Palette::VIOLET.gamma_multiply(0.20),
    );
    radial_glow(
        painter,
        rect.left_bottom() + Vec2::new(-rect.width() * 0.03, -rect.height() * 0.06),
        rect.width().max(rect.height()) * 0.52,
        Palette::TEAL.gamma_multiply(0.07),
    );
    // Keep the unused middle variable meaningful for future tweaking.
    let _ = middle;
}

fn radial_glow(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    if radius <= 1.0 || color.a() == 0 {
        return;
    }
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(center, color);
    let segments = 40;
    for index in 0..segments {
        let angle = std::f32::consts::TAU * index as f32 / segments as f32;
        let position = center + Vec2::new(angle.cos(), angle.sin()) * radius;
        mesh.colored_vertex(position, Color32::TRANSPARENT);
    }
    for index in 0..segments {
        mesh.add_triangle(0, 1 + index as u32, 1 + ((index + 1) % segments) as u32);
    }
    painter.add(egui::Shape::mesh(mesh));
}

/// A rounded backdrop for the whole application. Paint before the rest.
pub fn paint_app_background(ctx: &egui::Context) {
    let painter = ctx.layer_painter(egui::LayerId::background());
    let rect = ctx.content_rect();
    painter.rect_filled(rect, 0.0, Palette::INK);
    gradient_background(&painter, rect);
}

pub fn screen_rect(ui: &Ui) -> Rect {
    ui.ctx().content_rect()
}
