use crate::{
    egui_input::{handle_key_inputs, handle_mouse_input},
    world::{Board, Material, VOID, update_board},
};
use egui::{
    Color32, ColorImage, Id, Image, LayerId, Rect, RichText, Sense, Stroke, TextureHandle,
    TextureOptions, Vec2, load, pos2, util::hash, vec2,
};
use rand::SeedableRng;
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
    rng: rand::rngs::SmallRng,
    #[serde(skip)]
    response_text: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl Default for EFrameApp {
    fn default() -> Self {
        let mut game_board = Board {
            rng: rand::rngs::SmallRng::from_os_rng(),
            width: 512,
            height: 256,
            contents: vec![],
            gravity: 9.81,
            brushsize: 10,
            cellsize: Vec2::new(2.0, 2.0),
            rngs: vec![],
            seeds: vec![],
        };
        game_board.create_board();
        let ctx = egui::Context::default();
        let texture = ctx.load_texture(
            "Board".to_string(),
            ColorImage::example(),
            TextureOptions::NEAREST,
        );
        let mut materials: Vec<Material> = vec![];
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            use std::fs;

            let paths = fs::read_dir("src/materials/").unwrap();
            for path in paths {
                let materials_per_phase =
                    fs::read(path.unwrap().path().display().to_string()).unwrap();
                let mut serialized_materials: Vec<Material> =
                    serde_json::from_slice(materials_per_phase.as_slice()).unwrap();
                materials.append(&mut serialized_materials);
            }
        }
        let response_text = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        #[cfg(target_arch = "wasm32")]
        {
            use crate::http_request::get_req;
            get_req(response_text.clone());
        }
        #[cfg(target_os = "android")]
        {
            let powder_materials= include_str!("materials/powder.json");
            let mut serialized_materials: Vec<Material> = serde_json::from_str(&powder_materials).unwrap();
            materials.append(&mut serialized_materials);
        }
        let selected_material = VOID.clone();
        Self {
            fullscreen: false,
            game_board,
            materials,
            texture,
            selected_material,
            is_stopped: false,
            frame: 0,
            rng: rand::rngs::SmallRng::from_os_rng(),
            response_text: response_text.clone(),
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
        #[cfg(target_arch = "wasm32")]
        // Passed values of http requests
        {
            if !self.response_text.lock().unwrap().is_empty() {
                let mut materials_response: Vec<Material> =
                    serde_json::from_str(&self.response_text.lock().unwrap().pop().unwrap())
                        .unwrap();
                self.materials.append(&mut materials_response);
                self.selected_material = self.materials.last().unwrap().clone();
            }
        }
        egui::TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button(RichText::new("Fullscreen").size(20.0)).clicked() {
                    #[cfg(target_arch = "wasm32")]
                    {
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
                    {
                        if !self.fullscreen {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                            self.fullscreen = true;
                        } else {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                            self.fullscreen = false
                        }
                    }
                }
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("<").size(20.0)).clicked()
                        && self.game_board.brushsize > 0
                    {
                        self.game_board.brushsize -= 2;
                    }
                    ui.label(
                        RichText::new(format!("Brush size: {:03}", self.game_board.brushsize))
                            .size(20.0),
                    );
                    if ui.button(RichText::new(">").size(20.0)).clicked()
                        && self.game_board.brushsize < 256
                    {
                        self.game_board.brushsize += 2;
                    }
                });
                if ui
                    .button(
                        RichText::new("Reset")
                            .size(20.0)
                            .background_color(Color32::DARK_RED),
                    )
                    .clicked()
                {
                    self.game_board.create_board();
                }
            });
            ui.label("FPS: ".to_owned() + &ui.input(|i| (1.0 / i.unstable_dt).to_string()));
        });
        egui::TopBottomPanel::bottom(Id::new("bottom panel"))
            .exact_height(50.0)
            .show(ctx, |ui| {
                egui::ScrollArea::new([false, true]).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        self.materials.iter().for_each(|material| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(material.name.clone())
                                            .size(20.0)
                                            .color(Color32::WHITE)
                                            .strong(),
                                    )
                                    .min_size(vec2(Default::default(), 35.0))
                                    .stroke(Stroke::new(1.0, material.color)),
                                )
                                .clicked()
                            {
                                self.selected_material = material.clone();
                            }
                        });
                    });
                });
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            let pixels: Vec<Color32> = self.game_board.draw_board();
            let frameimage: ColorImage = ColorImage::new(
                [
                    self.game_board.width as usize,
                    self.game_board.height as usize,
                ],
                pixels,
            );
            self.texture = ctx.load_texture("Board", frameimage.clone(), TextureOptions::NEAREST);
            self.texture
                .set(frameimage.clone(), TextureOptions::NEAREST);
            let sized_texture =
                load::SizedTexture::new(self.texture.id(), self.texture.size_vec2());
            let binding = Image::from_texture(sized_texture);
            let board_display = Image::new(Image::source(&binding, ui.ctx()))
                .fit_to_exact_size(Vec2::new(
                    self.game_board.width as f32 * self.game_board.cellsize.x,
                    self.game_board.height as f32 * self.game_board.cellsize.y,
                ))
                .sense(Sense::click_and_drag());
            let width = ui.max_rect().width();
            let height = ui.max_rect().height();
            if width
                < (height * (self.game_board.width as f32 / self.game_board.height as f32)).ceil()
            {
                ui.horizontal_centered(|ui| {
                    let board = ui.add(board_display);
                    self.game_board.cellsize = vec2(
                        width / self.game_board.width as f32,
                        width / self.game_board.width as f32,
                    );
                    ui.painter()
                        .clone()
                        .with_layer_id(LayerId::new(egui::Order::Foreground, Id::new(hash(0))))
                        .with_clip_rect(ctx.screen_rect())
                        .rect(
                            Rect::from_min_size(
                                ((((board
                                    .hover_pos()
                                    .unwrap_or(pos2(-1024.0, -1024.0))
                                    .to_vec2()
                                    - board.interact_rect.min.to_vec2())
                                    / vec2(
                                        self.game_board.cellsize.x,
                                        self.game_board.cellsize.y,
                                    ))
                                .floor())
                                    * vec2(self.game_board.cellsize.x, self.game_board.cellsize.y))
                                .to_pos2()
                                .floor()
                                    + board.interact_rect.min.to_vec2()
                                    - vec2(
                                        self.game_board.cellsize.x
                                            * self.game_board.brushsize as f32
                                            * 0.5,
                                        self.game_board.cellsize.y
                                            * self.game_board.brushsize as f32
                                            * 0.5,
                                    ),
                                Vec2::new(
                                    self.game_board.brushsize as f32 * self.game_board.cellsize.x
                                        + self.game_board.cellsize.x,
                                    self.game_board.brushsize as f32 * self.game_board.cellsize.y
                                        + self.game_board.cellsize.y,
                                ),
                            ),
                            1.0,
                            Color32::from_black_alpha(100),
                            Stroke::new(2.0, Color32::WHITE),
                            egui::StrokeKind::Outside,
                        );
                    handle_mouse_input(
                        &mut self.game_board,
                        &mut self.selected_material,
                        board.clone(),
                    );
                    handle_key_inputs(&mut self.game_board, &mut self.is_stopped, board);
                });
            } else {
                ui.vertical_centered(|ui| {
                    let board = ui.add(board_display);
                    self.game_board.cellsize = vec2(
                        height / self.game_board.height as f32,
                        height / self.game_board.height as f32,
                    );
                    ui.painter()
                        .clone()
                        .with_layer_id(LayerId::new(egui::Order::Foreground, Id::new(hash(0))))
                        .with_clip_rect(ctx.screen_rect())
                        .rect(
                            Rect::from_min_size(
                                ((((board
                                    .hover_pos()
                                    .unwrap_or(pos2(-1024.0, -1024.0))
                                    .to_vec2()
                                    - board.interact_rect.min.to_vec2())
                                    / vec2(
                                        self.game_board.cellsize.x,
                                        self.game_board.cellsize.y,
                                    ))
                                .floor())
                                    * vec2(self.game_board.cellsize.x, self.game_board.cellsize.y))
                                .to_pos2()
                                .floor()
                                    + board.interact_rect.min.to_vec2()
                                    - vec2(
                                        self.game_board.cellsize.x
                                            * self.game_board.brushsize as f32
                                            * 0.5,
                                        self.game_board.cellsize.y
                                            * self.game_board.brushsize as f32
                                            * 0.5,
                                    ),
                                Vec2::new(
                                    self.game_board.brushsize as f32 * self.game_board.cellsize.x
                                        + self.game_board.cellsize.x,
                                    self.game_board.brushsize as f32 * self.game_board.cellsize.y
                                        + self.game_board.cellsize.y,
                                ),
                            ),
                            1.0,
                            Color32::from_black_alpha(100),
                            Stroke::new(2.0, Color32::WHITE),
                            egui::StrokeKind::Outside,
                        );
                    handle_mouse_input(
                        &mut self.game_board,
                        &mut self.selected_material,
                        board.clone(),
                    );
                    handle_key_inputs(&mut self.game_board, &mut self.is_stopped, board);
                });
            }
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
