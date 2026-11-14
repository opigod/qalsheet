use eframe::egui;
use qalsheet_ui::{Flavor, QalSheetApp};

fn main() -> eframe::Result {
    qalsheet_ui::storage::set_state_path(qalsheet_ui::storage::desktop_state_path());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("QalSheet Pro")
            .with_app_id("qalsheet")
            .with_inner_size([1180.0, 800.0])
            .with_min_inner_size([380.0, 560.0])
            .with_position([40.0, 40.0]),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "QalSheet Pro",
        options,
        Box::new(|cc| Ok(Box::new(QalSheetApp::new(cc, Flavor::Desktop)))),
    )
}
