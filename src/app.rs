use crate::{
    chemistry::Material_Type,
    egui_input::{handle_key_inputs, handle_mouse_input},
    physics::Phase,
    world::{update_board, Board, Material},
};
use egui::{load, Color32, ColorImage, Image, Sense, TextureHandle, TextureOptions, Vec2};
use xorshift::{xorshift128, Rng, SeedableRng, Xoroshiro128, Xorshift1024, Xorshift128};
// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct EFrameApp {
    fullscreen: bool,
    #[serde(skip)]
    texture: TextureHandle,
    #[serde(skip)]
    game_board: Board,
    #[serde(skip)]
    materials: Vec<Material>,
    #[serde(skip)]
    selected_material: Material,
    #[serde(skip)]
    is_stopped: bool,
    #[serde(skip)]
    frame: u8,
    #[serde(skip)]
    rng: Xorshift128,
}

impl Default for EFrameApp {
    fn default() -> Self {
        let mut game_board = Board {
            width: 512,
            height: 384,
            contents: vec![],
            gravity: 9.81,
            brushsize: 10,
            cellsize: Vec2::new(3.0, 3.0),
        };
        game_board.create_board();
        let ctx = egui::Context::default();
        let texture = ctx.load_texture(
            "Board".to_string(),
            ColorImage::example(),
            TextureOptions::NEAREST,
        );
        let states: [u64; 16] = [rand::random(); 16];
        Self {
            fullscreen: false,
            game_board,
            materials: vec![],
            texture,
            selected_material: Material {
                name: "Methane".to_string(),
                density: 0.657,
                phase: Phase::Powder { coarseness: 0.5 },
                material_type: Material_Type::Fuel,
                durability: -1,
                color: Color32::from_rgba_unmultiplied(252, 250, 0, 255),
            },
            is_stopped: false,
            frame: 0,
            rng: SeedableRng::from_seed(&states[..]),
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
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    self.fullscreen = true;
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                    self.fullscreen = false
                }
            }
            ui.label("FPS: ".to_owned() + &ui.input(|i| (1.0 / i.unstable_dt).to_string()));

            let mut pixels: Vec<u8> = self.game_board.draw_board();
            let frameimage: ColorImage = ColorImage::from_rgba_unmultiplied(
                [
                    self.game_board.width as usize,
                    self.game_board.height as usize,
                ],
                &mut pixels,
            );
            self.texture = ctx.load_texture("Board", frameimage.clone(), TextureOptions::NEAREST);
            self.texture
                .set(frameimage.clone(), TextureOptions::NEAREST);
            let sized_texture =
                load::SizedTexture::new(self.texture.id(), self.texture.size_vec2());
            let board = ui.add(
                Image::new(Image::source(&Image::from_texture(sized_texture), ui.ctx()))
                    .fit_to_exact_size(Vec2::new(
                        self.game_board.width as f32 * self.game_board.cellsize.x,
                        self.game_board.height as f32 * self.game_board.cellsize.y,
                    ))
                    .sense(Sense::click_and_drag()),
            );

            handle_mouse_input(
                &mut self.game_board,
                &mut self.selected_material,
                board.clone(),
            );
            handle_key_inputs(&mut self.game_board, &mut self.is_stopped, board);
            update_board(
                &mut self.game_board,
                self.is_stopped,
                &mut self.frame,
                ctx.input(|time| time.unstable_dt),
                &mut self.rng,
            );
            egui::Context::request_repaint(ctx);
        });
    }
}
