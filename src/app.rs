use std::default;

use egui::Widget;

use crate::world::{vec2_f32, Board, Material};
// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EFrameApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    wasm_fullscreen: bool,
    #[serde(skip)]
    game_board: Board,
    materials: Vec<Material>,
}

impl Default for EFrameApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            wasm_fullscreen: false,
            game_board: Board {
                width: 512,
                height: 384,
                contents: vec![],
                gravity: 9.81,
                brushsize: 1,
                cellsize: vec2_f32::new(0.0, 0.0),
            },
            materials: vec![],
        }
    }
}

impl EFrameApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for EFrameApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(target_arch = "wasm32")]

            //egui::Image::from_bytes(uri, draw_boa).ui(ui);
                if ui.button("Fullscreen").clicked() {
                        let Some(window) = web_sys::window() else {
                            return;
                        };
                        let Some(document) = window.document() else {
                            return;
                        };
                    if self.wasm_fullscreen {
                        let _ = document.exit_fullscreen();

                        let Ok(screen) = window.screen() else {
                            return;
                        };
                        let _ = screen.orientation().unlock();

                        self.wasm_fullscreen = false;
                    } else {
                        let Some(element) = document.document_element() else {
                            return;
                        };
                        let _ = element.request_fullscreen();

                        let Ok(screen) = window.screen() else {
                            return;
                        };
                        let _ = screen
                            .orientation()
                            .lock(web_sys::OrientationLockType::Landscape);
                        self.wasm_fullscreen = true;
                    }
                }
        });
    }
}
