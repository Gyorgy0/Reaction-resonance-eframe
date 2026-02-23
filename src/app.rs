use std::{mem::discriminant, u8};

use crate::{
    egui_input::{BrushShape, handle_key_inputs, handle_mouse_input},
    material::{Material, VOID},
    reactions::MaterialType,
    system_ui::draw_brush_outlines,
    world::{Board, update_board},
};
use egui::{
    Color32, ColorImage, Id, Image, RichText, Sense, Stroke, TextureHandle, TextureOptions, Theme,
    Vec2, load, vec2,
};
use rand::SeedableRng;
use strum::IntoEnumIterator;
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
    materials: Vec<(String, Material)>,
    #[serde(skip)]
    material_categories: Vec<Vec<(String, Material)>>,
    #[serde(skip)]
    selected_material: usize,
    #[serde(skip)]
    selected_category: MaterialType,
    #[serde(skip)]
    is_stopped: bool,
    #[serde(skip)]
    framecount: u64,
    #[serde(skip)]
    rng: rand::rngs::SmallRng,
    #[serde(skip)]
    response_text: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl Default for EFrameApp {
    fn default() -> Self {
        let mut game_board = Board {
            rng: rand::rngs::SmallRng::seed_from_u64(0_u64),
            width: 512_u16,
            height: 256_u16,
            contents: grid::Grid::from_vec(vec![], 0_usize),
            gravity: 9.81_f32,
            brush_size: vec2(25_f32, 9_f32),
            brush_shape: BrushShape::Rectangle,
            cellsize: Vec2::new(2_f32, 2_f32),
            rngs: grid::Grid::from_vec(vec![], 0_usize),
            seeds: grid::Grid::from_vec(vec![], 0_usize),
        };
        game_board.create_board();
        let ctx = egui::Context::default();
        let texture = ctx.load_texture(
            "Board".to_string(),
            ColorImage::example(),
            TextureOptions::NEAREST,
        );
        let mut materials: Vec<(String, Material)> = vec![(String::new(), VOID.clone())];
        #[cfg(any(target_os = "windows", target_os = "linux"))]
        {
            use std::fs;

            /*// This is for serializing particles/components with new fields and enums - testing purposes

            let mut number = 0b1111_0000_u8;
            let mut option = String::new();

            for i in 0..u8::BITS {
                option.push_str(&(number & 0b0000_0001_u8).to_string());
                number = number >> 1;
            }
            let data = serde_json::to_string(&option).unwrap();
            println!("{:?}", data);
            fs::write("src/new.json", data).unwrap();*/

            let paths = fs::read_dir("src/materials/").unwrap();
            for path in paths {
                let materials_per_phase: Result<Vec<u8>, std::io::Error> =
                    fs::read(path.unwrap().path().display().to_string().as_str());
                let mut serialized_materials: Vec<(String, Material)> =
                    serde_json::from_slice(materials_per_phase.unwrap().as_slice()).unwrap();
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
            let solid_materials = include_str!("materials/solid.json");
            let mut serialized_materials: Vec<(String, Material)> =
                serde_json::from_str(&solid_materials).unwrap();
            materials.append(&mut serialized_materials);

            let powder_materials = include_str!("materials/powder.json");
            let mut serialized_materials: Vec<(String, Material)> =
                serde_json::from_str(&powder_materials).unwrap();
            materials.append(&mut serialized_materials);

            let plasma_materials = include_str!("materials/plasma.json");
            let mut serialized_materials: Vec<(String, Material)> =
                serde_json::from_str(&plasma_materials).unwrap();
            materials.append(&mut serialized_materials);

            let liquid_materials = include_str!("materials/liquid.json");
            let mut serialized_materials: Vec<(String, Material)> =
                serde_json::from_str(&liquid_materials).unwrap();
            materials.append(&mut serialized_materials);

            let gas_materials = include_str!("materials/gas.json");
            let mut serialized_materials: Vec<(String, Material)> =
                serde_json::from_str(&gas_materials).unwrap();
            materials.append(&mut serialized_materials);
        }
        materials.sort_by_key(|material| material.1.id);
        let mut material_categories: Vec<Vec<(String, Material)>> = vec![];
        for category in MaterialType::iter() {
            let mut category_vec: Vec<(String, Material)> = vec![];
            for material in materials.iter() {
                if discriminant(&category) == discriminant(&material.1.material_type) {
                    category_vec.push(material.clone());
                }
            }
            material_categories.push(category_vec);
        }
        let selected_material = 0_usize;
        let selected_category = MaterialType::Fuel;
        Self {
            fullscreen: false,
            game_board,
            materials,
            material_categories,
            texture,
            selected_material,
            selected_category,
            is_stopped: false,
            framecount: 0,
            rng: rand::rngs::SmallRng::seed_from_u64(0_u64),
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
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.set_theme(Theme::Dark);
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
                let mut materials_response: Vec<(String, Material)> =
                    serde_json::from_str(&self.response_text.lock().unwrap().pop().unwrap())
                        .unwrap();
                self.materials.append(&mut materials_response);
                self.materials.sort_by_key(|material| material.1.id);
                let mut material_categories: Vec<Vec<(String, Material)>> = vec![];
                for category in MaterialType::iter() {
                    let mut category_vec: Vec<(String, Material)> = vec![];
                    for material in self.materials.iter() {
                        if discriminant(&category) == discriminant(&material.1.material_type) {
                            category_vec.push(material.clone());
                        }
                    }
                    material_categories.push(category_vec);
                }
                self.material_categories = material_categories;
                self.selected_material = 0_usize;
                self.selected_category = MaterialType::Fuel;
            }
        }
        egui::TopBottomPanel::top("top panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new("Fullscreen").size(20_f32))
                    .clicked()
                {
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
                    if ui.button(RichText::new("<").size(20_f32)).clicked()
                        && self.game_board.brush_size.x > 0_f32
                    {
                        self.game_board.brush_size -= vec2(2_f32, 2_f32);
                    }
                    ui.label(
                        RichText::new(format!("Brush size: {:03}", self.game_board.brush_size))
                            .size(20_f32),
                    );
                    if ui.button(RichText::new(">").size(20_f32)).clicked()
                        && self.game_board.brush_size.x < 256_f32
                    {
                        self.game_board.brush_size += vec2(2_f32, 2_f32);
                    }
                });
                if ui
                    .button(
                        RichText::new("Reset")
                            .size(20_f32)
                            .background_color(Color32::DARK_RED),
                    )
                    .clicked()
                {
                    self.game_board.create_board();
                }
            });
            ui.label("FPS: ".to_owned() + &ui.input(|i| (1_f32 / i.unstable_dt).to_string()));
        });
        egui::TopBottomPanel::bottom(Id::new("materials"))
            .exact_height(50_f32)
            .show(ctx, |ui| {
                egui::ScrollArea::new([true, false]).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        self.material_categories[self.selected_category.discriminant() as usize]
                            .iter()
                            .for_each(|material| {
                                if material.1.id != 0_usize
                                    && ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(material.0.clone())
                                                    .size(20_f32)
                                                    .color(Color32::WHITE)
                                                    .strong(),
                                            )
                                            .min_size(vec2(Default::default(), 35_f32))
                                            .stroke(
                                                Stroke::new(1_f32, material.1.material_color.color),
                                            ),
                                        )
                                        .clicked()
                                {
                                    self.selected_material = material.1.id;
                                }
                            });
                    });
                    ui.add(egui::Separator::default().spacing(10_f32));
                });
            });
        egui::SidePanel::right(Id::new("material categories"))
            .resizable(false)
            .default_width(32_f32)
            .show_separator_line(false)
            .show(ctx, |ui| {
                egui::ScrollArea::new([false, true])
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                if ui.add(egui::Button::new(RichText::new("Erase"))).clicked() {
                                    self.selected_material = 0_usize;
                                }
                                MaterialType::iter().for_each(|category| {
                                    if ui
                                        .add(egui::Button::new(RichText::new(format!(
                                            "{}",
                                            category
                                        ))))
                                        .clicked()
                                    {
                                        self.selected_category = category;
                                    }
                                });
                            });
                            ui.add(
                                egui::Separator::default()
                                    .spacing(10_f32)
                                    .vertical()
                                    .grow(f32::INFINITY),
                            );
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
                    draw_brush_outlines(&self.game_board, &board, ui, ctx);
                    handle_mouse_input(
                        &mut self.game_board,
                        &self.materials,
                        self.selected_material,
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
                    draw_brush_outlines(&self.game_board, &board, ui, ctx);
                    handle_mouse_input(
                        &mut self.game_board,
                        &self.materials,
                        self.selected_material,
                        board.clone(),
                    );
                    handle_key_inputs(&mut self.game_board, &mut self.is_stopped, board);
                });
            }
            update_board(
                &mut self.game_board,
                &self.materials,
                self.is_stopped,
                &mut self.framecount,
                ctx.input(|time| time.unstable_dt),
            );
            egui::Context::request_repaint(ctx);
        });
    }
}
