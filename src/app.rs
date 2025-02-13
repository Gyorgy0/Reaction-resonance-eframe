
use egui::{epaint::TextureManager, frame, Color32, ColorImage, Image, TextureHandle, Vec2};

use crate::world::{vec2_f32, Board, Material, color32_u8};
// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EFrameApp {
    fullscreen: bool,
    #[serde(skip)]
    game_board: Board,
    #[serde(skip)]
    materials: Vec<Material>,
}

impl Default for EFrameApp {
    fn default() -> Self {
        Self {
            fullscreen: false,
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
            if ui.button("Fullscreen").clicked() {
                let Some(window) = web_sys::window() else {
                    return;
                };
                let Some(document) = window.document() else {
                    return;
                };
                if self.fullscreen {
                    let _ = document.exit_fullscreen();

                    let Ok(screen) = window.screen() else {
                        return;
                    };
                    let _ = screen.orientation().unlock();

                    self.fullscreen = false;
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
                    self.fullscreen = true;
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Fullscreen").clicked() {
                if !self.fullscreen {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen((true)));
                    self.fullscreen = true;
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen((false)));
                    self.fullscreen = false
                }
            }
            let pixels: Vec<u8> = self.game_board.draw_board();
            print!("{}", pixels.len());
            //let frameimage: ColorImage = ColorImage::from_rgba_unmultiplied([self.game_board.height as usize, self.game_board.width as usize], &mut pixels);
            


        });
    }
}
