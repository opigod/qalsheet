//! The QalSheet window: responsive shell, tabs, keyboard shortcuts and feedback.

use std::time::Duration;

use eframe::egui::{
    self, Align2, Color32, Event, Id, Key, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, UiBuilder,
    Vec2,
};
use qalsheet_core::calculator::CalculatorState;
use qalsheet_core::units::ConverterState;

use crate::icons::{self, Icon};
use crate::screens::{algebra, calculator, converter, graph, tools};
use crate::storage;
use crate::theme::{self, Metrics, Palette, Weight};
use crate::widgets;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flavor {
    Desktop,
    Phone,
    Wear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tab {
    Calculator,
    Graph,
    Algebra,
    Converter,
    Tools,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Calculator => "Calculator",
            Self::Graph => "Graph",
            Self::Algebra => "Algebra",
            Self::Converter => "Converter",
            Self::Tools => "Tools",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Self::Calculator => Icon::Calculator,
            Self::Graph => Icon::Graph,
            Self::Algebra => Icon::Algebra,
            Self::Converter => Icon::Converter,
            Self::Tools => Icon::Tools,
        }
    }
}

const PHONE_TABS: [Tab; 5] = [
    Tab::Calculator,
    Tab::Graph,
    Tab::Algebra,
    Tab::Converter,
    Tab::Tools,
];

const WATCH_TABS: [Tab; 3] = [Tab::Calculator, Tab::Converter, Tab::Algebra];

pub struct Feedback {
    message: Option<String>,
    until: f64,
}

impl Default for Feedback {
    fn default() -> Self {
        Self {
            message: None,
            until: 0.0,
        }
    }
}

impl Feedback {
    pub fn show(&mut self, ctx: &egui::Context, text: impl Into<String>) {
        self.message = Some(text.into());
        self.until = ctx.input(|input| input.time) + 1.8;
        ctx.request_repaint_after(Duration::from_millis(120));
    }

    fn paint(&mut self, ctx: &egui::Context, metrics: &Metrics) {
        let Some(message) = self.message.as_deref() else {
            return;
        };
        let now = ctx.input(|input| input.time);
        if now > self.until {
            self.message = None;
            return;
        }
        // Draw on the root UI so the toast sits above the shell.
        let screen = ctx.content_rect();
        let ui = Ui::new(
            ctx.clone(),
            Id::unique("qal-toast-ui"),
            UiBuilder::new().max_rect(screen),
        );
        widgets::toast(&ui, message, metrics);
        ctx.request_repaint_after(Duration::from_millis(120));
    }
}

pub struct AppState {
    pub calculator: CalculatorState,
    pub graph: graph::GraphState,
    pub algebra: qalsheet_core::algebra::AlgebraState,
    pub converter: ConverterState,
    pub tools: tools::ToolsState,
    pub tab: Tab,
}

impl AppState {
    fn new(_flavor: Flavor) -> Self {
        Self {
            calculator: CalculatorState::new(),
            graph: graph::GraphState::new(),
            algebra: qalsheet_core::algebra::AlgebraState::new(),
            converter: ConverterState::new(),
            tools: tools::ToolsState::new(),
            tab: Tab::Calculator,
        }
    }
}

pub struct QalSheetApp {
    pub state: AppState,
    flavor: Flavor,
    metrics: Metrics,
    calc_ui: calculator::CalculatorUi,
    algebra_ui: algebra::AlgebraUi,
    converter_ui: converter::ConverterUi,
    feedback: Feedback,
    last_calc_history: (usize, Option<u64>),
    last_algebra_history: (usize, Option<u64>),
    notes_were_open: bool,
    last_note_save: f64,
    last_notes_digest: u64,
}

impl QalSheetApp {
    pub fn new(cc: &eframe::CreationContext<'_>, flavor: Flavor) -> Self {
        theme::install(&cc.egui_ctx);
        let mut app = Self {
            state: AppState::new(flavor),
            flavor,
            metrics: Metrics::new(1200.0, 800.0, flavor == Flavor::Wear),
            calc_ui: calculator::CalculatorUi::default(),
            algebra_ui: algebra::AlgebraUi::default(),
            converter_ui: converter::ConverterUi::default(),
            feedback: Feedback::default(),
            last_calc_history: (0, None),
            last_algebra_history: (0, None),
            notes_were_open: false,
            last_note_save: 0.0,
            last_notes_digest: 0,
        };
        storage::load(&mut app.state.calculator, &mut app.state.algebra);
        app.last_calc_history = history_signature(&app.state.calculator);
        app.last_algebra_history = (
            app.state.algebra.history.len(),
            app.state.algebra.history.first().map(|entry| entry.id),
        );
        app.last_notes_digest = notes_digest(&app.state);
        if let Ok(tab) = std::env::var("QALSHEET_TAB") {
            app.state.tab = match tab.to_lowercase().as_str() {
                "graph" => Tab::Graph,
                "algebra" => Tab::Algebra,
                "converter" | "convert" => Tab::Converter,
                "tools" => Tab::Tools,
                _ => Tab::Calculator,
            };
        }
        if std::env::var("QALSHEET_DEMO").is_ok() {
            app.seed_demo();
        }
        if let Ok(page) = std::env::var("QALSHEET_PAGE") {
            if page.eq_ignore_ascii_case("fx") {
                app.state.calculator.show_scientific_page(true);
            }
        }
        if let Ok(tool) = std::env::var("QALSHEET_TOOLS") {
            app.state.tools.tab = match tool.to_lowercase().as_str() {
                "programmer" | "program" => tools::ToolsTab::Programmer,
                "calculus" => tools::ToolsTab::Calculus,
                "constants" | "symbols" => tools::ToolsTab::Constants,
                _ => tools::ToolsTab::Data,
            };
        }
        app
    }

    fn seed_demo(&mut self) {
        for key in ["1", "2", "+", "3", "0", "="] {
            self.state.calculator.press(key);
        }
        self.state.calculator.press("\u{00D7}");
        self.state.calculator.press("2");
        self.state.calculator.press("=");
        self.state.algebra.set_equation("x^2 - 5x + 6 = 0");
        self.state.algebra.solve();
        self.state.converter.set_input("12.5");
        let expression = self.state.tools.calculus_expr.clone();
        let angle = self.state.calculator.angle_mode;
        if let Ok(compiled) = qalsheet_core::expr::compile(&expression) {
            self.state.tools.derivative_result = Some(
                qalsheet_core::calculus::derivative(&compiled, 1.5, angle)
                    .map_err(|error| error.message),
            );
            self.state.tools.integral_result = Some(
                qalsheet_core::calculus::integrate(&compiled, 0.0, 3.14, angle)
                    .map_err(|error| error.message),
            );
        }
    }

    pub fn flavor(&self) -> Flavor {
        self.flavor
    }

    fn tabs(&self) -> &'static [Tab] {
        if self.flavor == Flavor::Wear {
            &WATCH_TABS
        } else {
            &PHONE_TABS
        }
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let events = ctx.input(|input| input.events.clone());
        for event in events {
            match event {
                Event::Text(text) => match self.state.tab {
                    Tab::Calculator => self.state.calculator.type_text(&text),
                    Tab::Algebra => {
                        for ch in text.chars() {
                            match ch {
                                '0'..='9'
                                | '.'
                                | '+'
                                | '-'
                                | '*'
                                | '/'
                                | '^'
                                | '('
                                | ')'
                                | '['
                                | ']'
                                | '{'
                                | '}'
                                | '=' => {
                                    let key = if ch == '*' {
                                        "\u{00D7}"
                                    } else if ch == '/' {
                                        "\u{00F7}"
                                    } else if ch == '-' {
                                        "\u{2212}"
                                    } else {
                                        &ch.to_string()
                                    };
                                    self.state.algebra.press(key);
                                }
                                ch if ch.is_alphabetic() => {
                                    let key = ch.to_string();
                                    self.state.algebra.press(&key);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if modifiers.command || modifiers.ctrl || modifiers.alt {
                        continue;
                    }
                    match self.state.tab {
                        Tab::Calculator => match key {
                            Key::Enter => self.state.calculator.press("="),
                            Key::Backspace | Key::Delete => {
                                self.state.calculator.press("backspace")
                            }
                            Key::Escape => self.state.calculator.press("C"),
                            _ => {}
                        },
                        Tab::Algebra => match key {
                            Key::Enter => self.state.algebra.solve(),
                            Key::Backspace | Key::Delete => self.state.algebra.press("backspace"),
                            Key::Escape => self.state.algebra.press("C"),
                            _ => {}
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    fn draw_screen(&mut self, ui: &mut Ui, rect: Rect) {
        let mut child = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        match self.state.tab {
            Tab::Calculator => calculator::screen(
                &mut child,
                &mut self.state.calculator,
                &mut self.calc_ui,
                &self.metrics,
                &mut self.feedback,
            ),
            Tab::Graph => graph::screen(
                &mut child,
                &mut self.state.graph,
                &self.metrics,
                &mut self.feedback,
            ),
            Tab::Algebra => algebra::screen(
                &mut child,
                &mut self.state.algebra,
                &mut self.algebra_ui,
                &self.metrics,
                &mut self.feedback,
            ),
            Tab::Converter => converter::screen(
                &mut child,
                &mut self.state.converter,
                &mut self.converter_ui,
                &self.metrics,
                &mut self.feedback,
            ),
            Tab::Tools => tools::screen(
                &mut child,
                &mut self.state.tools,
                &mut self.state.calculator,
                &self.metrics,
                &mut self.feedback,
                &mut self.state.tab,
            ),
        }
    }

    fn desktop_shell(&mut self, ui: &mut Ui) {
        let screen = ui.ctx().content_rect();
        let sidebar_width = 216.0_f32.min(screen.width() * 0.28);
        let sidebar = Rect::from_min_size(screen.min, Vec2::new(sidebar_width, screen.height()));
        self.sidebar(ui, sidebar);

        let content = Rect::from_min_max(
            Pos2::new(sidebar.right() + 24.0, screen.top() + 18.0),
            Pos2::new(screen.right() - 24.0, screen.bottom() - 18.0),
        );
        self.draw_screen(ui, content);
    }

    fn sidebar(&mut self, ui: &mut Ui, rect: Rect) {
        let painter = ui.painter().clone();
        painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(16, 12, 24, 215));
        painter.line_segment(
            [
                Pos2::new(rect.right(), rect.top()),
                Pos2::new(rect.right(), rect.bottom()),
            ],
            Stroke::new(1.0, Color32::from_white_alpha(12)),
        );
        let pad = 18.0;
        let brand = Rect::from_min_size(
            Pos2::new(rect.left() + pad, rect.top() + 26.0),
            Vec2::new(rect.width() - pad * 2.0, 46.0),
        );
        draw_logo(ui, Rect::from_min_size(brand.min, Vec2::splat(36.0)));
        theme::text(
            ui.painter(),
            Pos2::new(brand.left() + 48.0, brand.center().y - 7.0),
            Align2::LEFT_CENTER,
            "QalSheet",
            19.0,
            Weight::ExtraBold,
            Palette::TEXT,
        );
        theme::text(
            ui.painter(),
            Pos2::new(brand.left() + 49.0, brand.center().y + 11.0),
            Align2::LEFT_CENTER,
            "PRO",
            8.0,
            Weight::Bold,
            Palette::TEAL,
        );

        let tabs = self.tabs();
        let mut y = rect.top() + 100.0;
        for (index, tab) in tabs.iter().copied().enumerate() {
            let item = Rect::from_min_size(
                Pos2::new(rect.left() + 12.0, y),
                Vec2::new(rect.width() - 24.0, 46.0),
            );
            if nav_item(ui, item, tab, self.state.tab == tab, index, &self.metrics) {
                self.state.tab = tab;
            }
            y += 53.0;
        }

        let divider = Rect::from_min_size(
            Pos2::new(rect.left() + pad, rect.bottom() - 124.0),
            Vec2::new(rect.width() - pad * 2.0, 1.0),
        );
        ui.painter().line_segment(
            [
                Pos2::new(divider.left(), divider.top()),
                Pos2::new(divider.right(), divider.top()),
            ],
            Stroke::new(1.0, Color32::from_white_alpha(10)),
        );
        let angle = Rect::from_min_size(
            Pos2::new(rect.left() + pad, rect.bottom() - 102.0),
            Vec2::new(76.0, 26.0),
        );
        if widgets::pill(
            ui,
            angle,
            &format!("Angle {}", self.state.calculator.angle_mode.label()),
            false,
            false,
            &self.metrics,
        )
        .clicked()
        {
            self.state.calculator.toggle_angle_mode();
        }
        theme::text(
            ui.painter(),
            Pos2::new(rect.left() + pad, rect.bottom() - 58.0),
            Align2::LEFT_CENTER,
            "Native Rust · no JVM",
            8.5,
            Weight::Medium,
            Palette::FAINT,
        );
        theme::text(
            ui.painter(),
            Pos2::new(rect.left() + pad, rect.bottom() - 42.0),
            Align2::LEFT_CENTER,
            "QalSheet 2.0",
            8.5,
            Weight::SemiBold,
            Palette::MUTED,
        );
    }

    fn compact_shell(&mut self, ui: &mut Ui) {
        let screen = ui.ctx().content_rect();
        let top_height = self.metrics.top_inset + 46.0;
        let top = Rect::from_min_size(screen.min, Vec2::new(screen.width(), top_height));
        let painter = ui.painter().clone();
        painter.rect_filled(
            Rect::from_min_size(
                top.min,
                Vec2::new(top.width(), top_height - self.metrics.top_inset),
            ),
            0.0,
            Color32::from_rgba_unmultiplied(16, 12, 24, 190),
        );
        painter.line_segment(
            [
                Pos2::new(top.left(), top.bottom() - self.metrics.top_inset),
                Pos2::new(top.right(), top.bottom() - self.metrics.top_inset),
            ],
            Stroke::new(1.0, Color32::from_white_alpha(10)),
        );
        let bar = Rect::from_min_size(
            Pos2::new(top.left() + 14.0, top.top() + self.metrics.top_inset),
            Vec2::new(top.width() - 28.0, 46.0),
        );
        draw_logo(
            ui,
            Rect::from_min_size(bar.left_center() + Vec2::new(0.0, -13.0), Vec2::splat(26.0)),
        );
        theme::text(
            ui.painter(),
            Pos2::new(bar.left() + 36.0, bar.center().y - 5.0),
            Align2::LEFT_CENTER,
            "QalSheet",
            15.0,
            Weight::ExtraBold,
            Palette::TEXT,
        );
        theme::text(
            ui.painter(),
            Pos2::new(bar.left() + 36.0, bar.center().y + 8.0),
            Align2::LEFT_CENTER,
            "PRO",
            7.0,
            Weight::Bold,
            Palette::TEAL,
        );
        theme::text(
            ui.painter(),
            bar.right_center(),
            Align2::RIGHT_CENTER,
            self.state.tab.label(),
            11.0,
            Weight::SemiBold,
            Palette::MUTED,
        );

        let nav_height = 62.0 + self.metrics.bottom_inset;
        let nav = Rect::from_min_max(
            Pos2::new(screen.left(), screen.bottom() - nav_height),
            screen.max,
        );
        let content = Rect::from_min_max(
            Pos2::new(screen.left() + 12.0, top.bottom() + 6.0),
            Pos2::new(screen.right() - 12.0, nav.top() - 6.0),
        );
        self.draw_screen(ui, content);
        let watch = self.flavor == Flavor::Wear;
        let tabs: &[Tab] = if watch { &WATCH_TABS } else { &PHONE_TABS };
        bottom_nav(ui, nav, &mut self.state.tab, tabs, watch, &self.metrics);
    }

    fn watch_shell(&mut self, ui: &mut Ui) {
        let screen = ui.ctx().content_rect();
        // Three compact, centred tabs fit the top of the circle exactly as
        // the Kotlin Wear layout did.
        let tabs = Rect::from_min_size(
            Pos2::new(screen.center().x - 51.0, screen.top() + 12.0),
            Vec2::new(102.0, 30.0),
        );
        watch_tabs(ui, tabs, &mut self.state.tab, &WATCH_TABS, &self.metrics);
        let content = Rect::from_min_max(
            Pos2::new(screen.left() + 18.0, tabs.bottom() + 2.0),
            Pos2::new(screen.right() - 18.0, screen.bottom() - 22.0),
        );
        self.draw_screen(ui, content);
    }
}

impl eframe::App for QalSheetApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        theme::paint_app_background(ui.ctx());
        let screen = ui.ctx().content_rect();
        self.metrics = Metrics::new(screen.width(), screen.height(), self.flavor == Flavor::Wear);
        self.handle_keyboard(ui.ctx());
        if self.flavor == Flavor::Wear {
            self.watch_shell(ui);
        } else if self.metrics.wide {
            self.desktop_shell(ui);
        } else {
            self.compact_shell(ui);
        }
        self.feedback.paint(ui.ctx(), &self.metrics);
        self.autosave(ui.ctx());
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [7.0 / 255.0, 6.0 / 255.0, 11.0 / 255.0, 1.0]
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        storage::save(&self.state.calculator, &self.state.algebra);
    }
}

impl QalSheetApp {
    fn autosave(&mut self, ctx: &egui::Context) {
        let calc_history = history_signature(&self.state.calculator);
        let alg_history = (
            self.state.algebra.history.len(),
            self.state.algebra.history.first().map(|entry| entry.id),
        );
        let notes_open = self.calc_ui.note_sheet || self.algebra_ui.note_sheet;
        let notes = notes_digest(&self.state);
        let now = ctx.input(|input| input.time);
        let history_changed =
            calc_history != self.last_calc_history || alg_history != self.last_algebra_history;
        let notes_changed = notes != self.last_notes_digest;
        let notes_closed = self.notes_were_open && !notes_open;
        let notes_due = notes_open && notes_changed && now - self.last_note_save >= 0.6;

        if history_changed || notes_closed || notes_due {
            storage::save(&self.state.calculator, &self.state.algebra);
            self.last_calc_history = calc_history;
            self.last_algebra_history = alg_history;
            self.last_notes_digest = notes;
            self.last_note_save = now;
        }
        self.notes_were_open = notes_open;
        if notes_open && notes_changed {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
    }
}

fn history_signature(state: &CalculatorState) -> (usize, Option<u64>) {
    (
        state.history.len(),
        state.history.first().map(|entry| entry.id),
    )
}

fn notes_digest(state: &AppState) -> u64 {
    let mut hash = 1_469_598_103_934_665_603_u64;
    for entry in &state.calculator.history {
        hash ^= entry.id;
        for byte in entry.note.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(1_099_511_628_211);
        }
    }
    for entry in &state.algebra.history {
        hash ^= entry.id;
        for byte in entry.note.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(1_099_511_628_211);
        }
    }
    hash
}

fn draw_logo(ui: &mut Ui, rect: Rect) {
    let painter = ui.painter();
    painter.rect_filled(rect, rect.width() * 0.28, Palette::VIOLET);
    painter.rect_filled(
        Rect::from_min_size(
            rect.left_top() + Vec2::new(1.0, 1.0),
            Vec2::new(rect.width() - 2.0, rect.height() * 0.46),
        ),
        rect.width() * 0.26,
        Palette::VIOLET_SOFT.gamma_multiply(0.20),
    );
    theme::text(
        painter,
        rect.center(),
        Align2::CENTER_CENTER,
        "Q",
        rect.height() * 0.58,
        Weight::ExtraBold,
        Palette::TEXT,
    );
}

fn nav_item(
    ui: &mut Ui,
    rect: Rect,
    tab: Tab,
    selected: bool,
    index: usize,
    metrics: &Metrics,
) -> bool {
    let id = ui.scope_id().with(("nav", index));
    let response = ui.interact(rect, id, Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("hover"), response.hovered(), 0.1);
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 12.0, Palette::VIOLET.gamma_multiply(0.15));
        painter.rect_stroke(
            rect,
            12.0,
            Stroke::new(1.0, Palette::VIOLET.gamma_multiply(0.28)),
            StrokeKind::Inside,
        );
    } else if hover > 0.0 {
        painter.rect_filled(rect, 12.0, Color32::from_white_alpha((hover * 8.0) as u8));
    }
    let icon_rect = Rect::from_min_size(
        Pos2::new(rect.left() + 13.0, rect.center().y - 9.0),
        Vec2::splat(18.0),
    );
    icons::draw(
        painter,
        icon_rect,
        tab.icon(),
        if selected {
            Palette::VIOLET_SOFT
        } else {
            Palette::MUTED
        },
    );
    theme::text(
        painter,
        Pos2::new(rect.left() + 42.0, rect.center().y),
        Align2::LEFT_CENTER,
        tab.label(),
        12.5,
        if selected {
            Weight::Bold
        } else {
            Weight::SemiBold
        },
        if selected {
            Palette::TEXT
        } else {
            Palette::MUTED
        },
    );
    let _ = metrics;
    response.clicked()
}

fn bottom_nav(
    ui: &mut Ui,
    rect: Rect,
    tab: &mut Tab,
    tabs: &[Tab],
    watch: bool,
    metrics: &Metrics,
) {
    let painter = ui.painter().clone();
    painter.rect_filled(
        Rect::from_min_size(
            rect.min,
            Vec2::new(rect.width(), rect.height() - metrics.bottom_inset),
        ),
        0.0,
        Color32::from_rgba_unmultiplied(16, 12, 24, 220),
    );
    painter.line_segment(
        [
            Pos2::new(rect.left(), rect.top()),
            Pos2::new(rect.right(), rect.top()),
        ],
        Stroke::new(1.0, Color32::from_white_alpha(10)),
    );
    let usable_height = rect.height() - metrics.bottom_inset;
    let width = rect.width() / tabs.len() as f32;
    for (index, item) in tabs.iter().copied().enumerate() {
        let cell = Rect::from_min_size(
            Pos2::new(rect.left() + width * index as f32, rect.top()),
            Vec2::new(width, usable_height),
        );
        let id = ui.scope_id().with(("bottom-nav", index));
        let response = ui.interact(cell, id, Sense::click());
        if response.clicked() {
            *tab = item;
        }
        let active = *tab == item;
        let icon_center = Pos2::new(cell.center().x, cell.top() + usable_height * 0.40);
        let (pill_width, pill_height, icon_size, label_size) = if metrics.watch {
            (38.0, 26.0, 19.0, 8.0)
        } else {
            (43.0, 29.0, 21.0, 9.0)
        };
        if active {
            painter.rect_filled(
                Rect::from_center_size(icon_center, Vec2::new(pill_width, pill_height)),
                pill_height / 2.0,
                Palette::VIOLET.gamma_multiply(0.20),
            );
        }
        icons::icon(
            &painter,
            icon_center,
            icon_size,
            item.icon(),
            if active {
                Palette::VIOLET_SOFT
            } else {
                Palette::FAINT
            },
        );
        painter.text(
            Pos2::new(cell.center().x, cell.top() + usable_height * 0.74),
            Align2::CENTER_CENTER,
            item.label(),
            theme::font(
                label_size,
                if active { Weight::Bold } else { Weight::Medium },
            ),
            if active {
                Palette::TEXT
            } else {
                Palette::MUTED
            },
        );
    }
    let _ = watch;
}

fn watch_tabs(ui: &mut Ui, rect: Rect, tab: &mut Tab, tabs: &[Tab], metrics: &Metrics) {
    let gap = 6.0;
    let width = (rect.width() - gap * (tabs.len() as f32 - 1.0)) / tabs.len() as f32;
    for (index, item) in tabs.iter().copied().enumerate() {
        let cell = Rect::from_min_size(
            Pos2::new(rect.left() + (width + gap) * index as f32, rect.top()),
            Vec2::new(width, rect.height()),
        );
        let id = ui.scope_id().with(("watch-tab", index));
        let response = ui.interact(cell, id, Sense::click());
        if response.clicked() {
            *tab = item;
        }
        let active = *tab == item;
        let center = cell.center();
        if active {
            ui.painter()
                .circle_filled(center, metrics.fs(13.0), Palette::VIOLET);
        }
        icons::icon(
            ui.painter(),
            center,
            metrics.fs(14.0),
            item.icon(),
            if active { Palette::INK } else { Palette::MUTED },
        );
    }
}
