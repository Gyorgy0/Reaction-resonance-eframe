mod app;
mod dialogs;
mod egui_input;
mod http_request;
mod life_reactions;
mod locale;
mod material;
mod particle;
mod physics;
mod reactions;
mod system_data;
mod system_ui;
mod world;
pub use app::EFrameApp;

// When compiling to Android
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    // Log to android output
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );

    let options = eframe::NativeOptions {
        android_app: Some(app),
        vsync: false,
        ..Default::default()
    };
    eframe::run_native(
        "Reaction resonance",
        options,
        Box::new(|cc| Ok(Box::new(EFrameApp::new(cc)))),
    )
    .unwrap()
}
