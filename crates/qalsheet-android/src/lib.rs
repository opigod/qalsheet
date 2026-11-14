//! QalSheet Pro for Android. NativeActivity, no Java or DEX code.

#[cfg(target_os = "android")]
mod android {
    use qalsheet_ui::Flavor;
    use winit::platform::android::activity::AndroidApp;

    #[unsafe(no_mangle)]
    fn android_main(app: AndroidApp) {
        qalsheet_ui::run_android(app, Flavor::Phone);
    }
}
