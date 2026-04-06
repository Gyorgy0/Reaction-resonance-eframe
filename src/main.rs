#![warn(clippy::all, rust_2018_idioms)]
//  #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    use egui::{IconData, Image, include_image};
    use reaction_resonance_eframe::EFrameApp;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        vsync: false,
        viewport: egui::ViewportBuilder::default().with_icon(eframe::icon_data::from_png_bytes(include_bytes!("assets/icon_falling_sand.png")).unwrap())
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Reaction resonance",
        native_options,
        Box::new(|cc| Ok(Box::new(EFrameApp::new(cc)))),
    )
}

// When compiling to WebAssembly
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    use web_sys::Document;
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions {
        webgl_context_option: eframe::WebGlContextOption::CompatibilityFirst,
        ..Default::default()
    };

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");
        let canvas = document
            .get_element_by_id("glcanvas")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(reaction_resonance_eframe::EFrameApp::new(cc)))),
            )
            .await;
        // LOADING SCREEN NOT YET IMPLEMENTED
        /*// Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }*/
    });
}
