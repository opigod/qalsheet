//! QalSheet's native Rust user interface, shared by Linux, phones and watches.

pub mod app;
pub mod icons;
pub mod screens;
pub mod storage;
pub mod theme;
pub mod widgets;

pub use app::{Flavor, QalSheetApp};

#[cfg(target_os = "android")]
pub fn run_android(app: winit::platform::android::activity::AndroidApp, flavor: Flavor) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let state_path = app.internal_data_path().map(|path| path.join("state.txt"));
    log::info!("QalSheet state path: {state_path:?}");
    storage::set_state_path(state_path);
    let options = eframe::NativeOptions {
        android_app: Some(app),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    let title = if flavor == Flavor::Wear {
        "QalSheet"
    } else {
        "QalSheet Pro"
    };
    let _ = eframe::run_native(
        title,
        options,
        Box::new(move |cc| Ok(Box::new(QalSheetApp::new(cc, flavor)))),
    );
}
