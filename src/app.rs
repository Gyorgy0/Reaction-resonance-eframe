use core::f32;
use std::{mem::discriminant, u8};

use crate::dialogs::OptionsMenuDialog;
use crate::egui_input::BrushTool;
use crate::locale::{Locale, get_text};
use crate::particle::Particle;
use crate::physics::{BLACK_BODY_RADIATION_COLORS, PhysicalReactions};
use crate::system_data::{ApplicationOptions, get_sign, get_temperature};
use crate::system_ui::{debug_text_rendering, get_particle};
use crate::{
    egui_input::{BrushShape, handle_key_inputs, handle_mouse_input, resize_brush},
    material::{AIR, Material},
    reactions::MaterialType,
    system_ui::draw_brush_outlines,
    world::{Board, update_board},
};
use egui::ahash::AHashMap;
use egui::epaint::Hsva;
use egui::text::LayoutJob;
use egui::util::hash;
use egui::{
    Color32, ColorImage, Id, Image, LayerId, Rect, RichText, Sense, Stroke, TextureHandle,
    TextureOptions, Theme, Vec2, load, pos2, vec2,
};
use egui_colorgradient::ColorInterpolator;
use egui_dialogs::{DialogDetails, Dialogs};
use rand::SeedableRng;
use strum::IntoEnumIterator;
// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // If we add new fields, give them default values when deserializing old state
pub struct EFrameApp<'a> {
    #[serde(skip)]
    viewed_particle: Particle,
    #[serde(skip)]
    fps_values: Vec<f32>,
    #[serde(skip)]
    physical_transitions: PhysicalReactions,
    #[serde(skip)]
    black_body_gradient: ColorInterpolator,
    #[serde(skip)]
    dialogs: Dialogs<'a>,
    #[serde(skip)]
    debug_text_job: LayoutJob,
    #[serde(skip)]
    material_texture: TextureHandle,
    #[serde(skip)]
    heatmap_texture: TextureHandle,
    #[serde(skip)]
    game_board: Board,
    #[serde(skip)]
    materials: Vec<(String, Material)>,
    #[serde(skip)]
    material_categories: Vec<Vec<(String, Material)>>,
    #[serde(skip)]
    program_options: ApplicationOptions,
    #[serde(skip)]
    selected_tool: BrushTool,
    #[serde(skip)]
    selected_category: MaterialType,
    #[serde(skip)]
    framecount: u64,
    #[serde(skip)]
    rng: rand::rngs::SmallRng,
    #[serde(skip)]
    dialogopen: bool,
}

impl Default for EFrameApp<'_> {
    fn default() -> Self {
        // Initializes the default values
        let mut game_board: Board = Board::default();
        let ctx: egui::Context = egui::Context::default();
        let mut program_options: ApplicationOptions = ApplicationOptions::default();

        // Generates the game's board
        game_board.create_board();

        // Initializes the texture handles
        let material_texture = ctx.load_texture(
            "Board".to_string(),
            ColorImage::example(),
            TextureOptions::NEAREST,
        );
        let heatmap_texture = ctx.load_texture(
            "Board_heatmap".to_string(),
            ColorImage::example(),
            TextureOptions::LINEAR,
        );
        let mut locales: Vec<Locale> = vec![];
        let mut materials: Vec<(String, Material)> = vec![(String::new(), AIR.clone())];
        let serialized_transition_melting: AHashMap<usize, usize>;
        let serialized_transition_boiling: AHashMap<usize, Vec<(usize, f32)>>;
        let serialized_transition_sublimation: AHashMap<usize, usize>;
        let serialized_transition_ionization: AHashMap<usize, usize>;
        // This is for the PC platform (locale and materials and their reactions are serialized from files)
        #[cfg(not(any(target_os = "android", target_arch = "wasm32", target_os = "ios")))]
        {
            use std::fs;

            /*// This is for serializing particles/components with new fields and enums - testing purposes

            //let mut materialhash: AHashMap<String, Material> = AHashMap::from([(String::new(), VOID.clone())]);
            let mut types: Vec<MaterialType> = vec![];
            for material_type in MaterialType::iter() {
                types.push(material_type);
            }
            let data = serde_json::to_string(&types).unwrap();
            println!("{:?}", data);
            fs::write("src/new.json", data).unwrap();
            let serialized_data: AHashMap<String, Vec<(usize, f32)>> =
                serde_json::from_reader(fs::read("src/new.json").unwrap().as_slice()).unwrap();
            println!("{:?}", serialized_data);
            */

            // Locale
            let paths = fs::read_dir("src/locale").unwrap();
            for path in paths {
                if path
                    .as_ref()
                    .is_ok_and(|path| path.file_name() != "default_locale.json")
                {
                    let locale: Result<Vec<u8>, std::io::Error> =
                        fs::read(path.as_ref().unwrap().path().display().to_string().as_str());
                    let serialized_locale: Locale =
                        serde_json::from_reader(locale.unwrap().as_slice())
                            .unwrap_or(Locale::default());
                    locales.push(serialized_locale);
                }
            }

            // Materials
            let paths = fs::read_dir("src/materials/").unwrap();
            for path in paths {
                let materials_per_phase: Result<Vec<u8>, std::io::Error> =
                    fs::read(path.as_ref().unwrap().path().display().to_string().as_str());
                let mut serialized_materials: Vec<(String, Material)> =
                    serde_json::from_reader(materials_per_phase.unwrap().as_slice()).unwrap();
                materials.append(&mut serialized_materials);
            }

            // Sorts the elements by their Id's and outputs them to a list
            materials.sort_by_key(|elem| elem.1.id);
            let mut material_ids: Vec<(usize, String)> = vec![];
            materials.iter().for_each(|element| {
                material_ids.push((element.1.id, element.0.clone()));
            });
            fs::write(
                "src/material_ids.json",
                serde_json::to_string(&material_ids).unwrap(),
            )
            .unwrap();

            // Physical transitions
            let transition_path_melting = fs::read("src/physics/phase_transitions_melting.json");
            serialized_transition_melting =
                serde_json::from_reader(transition_path_melting.unwrap().as_slice()).unwrap();
            let transition_path_boiling = fs::read("src/physics/phase_transitions_boiling.json");
            serialized_transition_boiling =
                serde_json::from_reader(transition_path_boiling.unwrap().as_slice()).unwrap();
            let transition_path_sublimation =
                fs::read("src/physics/phase_transitions_sublimation.json");
            serialized_transition_sublimation =
                serde_json::from_reader(transition_path_sublimation.unwrap().as_slice()).unwrap();
            let transition_path_ionization =
                fs::read("src/physics/phase_transitions_ionization.json");
            serialized_transition_ionization =
                serde_json::from_reader(transition_path_ionization.unwrap().as_slice()).unwrap();
        }

        #[cfg(any(target_os = "android", target_arch = "wasm32", target_os = "ios"))]
        {
            use crate::included_files::FILES;

            // Locale
            locales.push(serde_json::from_str(&FILES.locales.locale_en).unwrap());
            locales.push(serde_json::from_str(&FILES.locales.locale_hu).unwrap());
            locales.push(serde_json::from_str(&FILES.locales.locale_sk).unwrap());

            // Materials
            let mut serialized_materials: Vec<(String, Material)> =
                serde_json::from_str(&FILES.materials.solid_materials).unwrap();

            serialized_materials
                .push(serde_json::from_str(&FILES.materials.powder_materials).unwrap());

            serialized_materials
                .push(serde_json::from_str(&FILES.materials.liquid_materials).unwrap());

            serialized_materials
                .push(serde_json::from_str(&FILES.materials.gas_materials).unwrap());

            serialized_materials
                .push(serde_json::from_str(&FILES.materials.plasma_materials).unwrap());

            serialized_materials
                .push(serde_json::from_str(&FILES.materials.life_materials).unwrap());

            materials.append(&mut serialized_materials);

            // Physical transitions
            serialized_transition_melting =
                serde_json::from_str(&FILES.physics_transition.melting_transitions).unwrap();
            serialized_transition_boiling =
                serde_json::from_str(&FILES.physics_transition.boiling_transitions).unwrap();
            serialized_transition_sublimation =
                serde_json::from_str(&FILES.physics_transition.sublimation_transitions).unwrap();
            serialized_transition_ionization =
                serde_json::from_str(&FILES.physics_transition.ionization_transitions).unwrap();
        }

        program_options.locale = locales;
        let mut material_categories: Vec<Vec<(String, Material)>> = vec![];
        // Sort material by their ID's
        materials.sort_by_key(|elem| elem.1.id);
        for category in MaterialType::iter() {
            let mut category_vec: Vec<(String, Material)> = vec![];
            for material in materials.iter() {
                if discriminant(&category) == discriminant(&material.1.material_type) {
                    category_vec.push(material.clone());
                }
            }
            material_categories.push(category_vec);
        }
        let selected_tool = BrushTool::MaterialBrush {
            selected_material: 0_usize,
        };
        let selected_category = MaterialType::fuel_default();
        let debug_text_job = LayoutJob::default();
        let stops: Vec<(f32, Hsva)> = BLACK_BODY_RADIATION_COLORS
            .iter_mut()
            .map(|stop| (stop.0, stop.1.into()))
            .collect();
        Self {
            viewed_particle: Particle::default(),
            fps_values: vec![0_f32; 256_usize],
            black_body_gradient: egui_colorgradient::Gradient::new(
                egui_colorgradient::InterpolationMethod::Constant,
                stops,
            )
            .interpolator(),
            physical_transitions: PhysicalReactions::new(
                serialized_transition_melting,
                serialized_transition_boiling,
                serialized_transition_sublimation,
                serialized_transition_ionization,
            ),
            debug_text_job,
            game_board,
            materials,
            material_categories,
            material_texture,
            heatmap_texture,
            selected_tool,
            selected_category,
            program_options,
            framecount: 0,
            rng: rand::rngs::SmallRng::seed_from_u64(0_u64),
            dialogs: Dialogs::default(),
            dialogopen: false,
        }
    }
}

impl EFrameApp<'_> {
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

impl eframe::App for EFrameApp<'_> {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();
        const OPTIONS_MENU: &str = "options_menu";
        // Logic for showing the dialogs and handling the reply is there is one
        if let Some(res) = self.dialogs.show(ctx)
            && res.is_reply_of(OPTIONS_MENU)
            && let Ok(options) = res.reply::<(f32, ApplicationOptions)>()
        {
            self.game_board.gravity = options.0;
            self.program_options = options.1;
            self.dialogopen = false;
        }
        egui::Panel::top("top panel").show(ctx, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {
                ui.add_space(5_f32);
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .options_title
                                .as_str(),
                            )
                            .size(20_f32),
                        )
                        .clicked()
                    {
                        DialogDetails::new(OptionsMenuDialog::new(
                            self.game_board.gravity,
                            self.program_options.clone(),
                            self.materials.clone(),
                            &self.program_options.locale,
                            self.program_options.selected_locale,
                        ))
                        .with_id(OPTIONS_MENU)
                        .show(&mut self.dialogs);
                        self.dialogopen = true;
                    }
                    ui.horizontal_centered(|ui| {
                        ui.label(
                            RichText::new(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .brush_size_label
                                .as_str(),
                            )
                            .size(20_f32),
                        );
                        if ui.button(RichText::new("–").size(20_f32)).clicked() {
                            resize_brush(&mut self.game_board.brush_size, vec2(-1_f32, 0_f32));
                        }
                        ui.label(
                            RichText::new(format!(
                                "{x_label} {size_x:03}",
                                x_label = get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .x_axis_label,
                                size_x = self.game_board.brush_size.x
                            ))
                            .size(20_f32),
                        );
                        if ui.button(RichText::new("+").size(20_f32)).clicked() {
                            resize_brush(&mut self.game_board.brush_size, vec2(1_f32, 0_f32));
                        }
                        ui.separator();
                        if ui.button(RichText::new("–").size(20_f32)).clicked() {
                            resize_brush(&mut self.game_board.brush_size, vec2(0_f32, -1_f32));
                        }
                        ui.label(
                            RichText::new(format!(
                                "{y_label} {size_y:03}",
                                y_label = get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .y_axis_label,
                                size_y = self.game_board.brush_size.y
                            ))
                            .size(20_f32),
                        );
                        if ui.button(RichText::new("+").size(20_f32)).clicked() {
                            resize_brush(&mut self.game_board.brush_size, vec2(0_f32, 1_f32));
                        }
                        ui.separator();
                        if ui.button(RichText::new("–").size(20_f32)).clicked() {
                            resize_brush(&mut self.game_board.brush_size, vec2(-1_f32, -1_f32));
                        }
                        if ui.button(RichText::new("+").size(20_f32)).clicked() {
                            resize_brush(&mut self.game_board.brush_size, vec2(1_f32, 1_f32));
                        }
                        ui.separator();
                        for brush_shapes in BrushShape::iter() {
                            if ui.button(format!("{:?}", brush_shapes)).clicked() {
                                self.game_board.brush_shape = brush_shapes;
                            }
                        }
                    });

                    if ui
                        .add(
                            egui::widgets::Button::new(
                                RichText::new(
                                    get_text(
                                        &self.program_options.locale,
                                        self.program_options.selected_locale,
                                    )
                                    .reset_button
                                    .as_str(),
                                )
                                .heading()
                                .strong(),
                            )
                            .stroke(Stroke::new(1_f32, Color32::WHITE))
                            .fill(Color32::DARK_RED),
                        )
                        .clicked()
                    {
                        self.game_board.create_board();
                    }
                });
                ui.add_space(5_f32);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{particle_temp:>16.2} {temperature_sign}   |   {particle_name}",
                            particle_temp = get_temperature(
                                self.program_options.temperature_scale,
                                self.viewed_particle.temperature
                            ),
                            temperature_sign = get_sign(self.program_options.temperature_scale),
                            particle_name = self.materials[self.viewed_particle.material_id].0,
                        ))
                        .size(15_f32)
                        .strong()
                        .monospace(),
                    );
                });
            });
        });
        egui::Panel::bottom(Id::new("materials"))
            .exact_size(50_f32)
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
                                    self.selected_tool = BrushTool::MaterialBrush {
                                        selected_material: material.1.id,
                                    };
                                }
                            });
                    });
                    ui.add(egui::Separator::default().spacing(10_f32));
                });
            });
        egui::Panel::right(Id::new("material categories"))
            .resizable(false)
            .default_size(32_f32)
            .show_separator_line(false)
            .show(ctx, |ui| {
                egui::ScrollArea::new([false, true])
                    .max_height(f32::INFINITY)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                let button_width = ui.content_rect().width() * 0.035;
                                let button =
                                    egui::widgets::Button::new(RichText::new("0123456789"))
                                        .stroke(Stroke::new(1_f32, Color32::WHITE))
                                        .min_size(Vec2::new(button_width, button_width / 2_f32));
                                let placed_button = ui.add_visible(false, button);
                                ui.horizontal_centered(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(
                                                    get_text(
                                                        &self.program_options.locale,
                                                        self.program_options.selected_locale,
                                                    )
                                                    .eraser_button
                                                    .as_str(),
                                                )
                                                .heading()
                                                .strong()
                                                .monospace(),
                                            )
                                            .stroke(Stroke::new(1_f32, Color32::WHITE))
                                            .min_size(placed_button.intrinsic_size().unwrap()),
                                        )
                                        .clicked()
                                    {
                                        self.selected_tool = BrushTool::MaterialBrush {
                                            selected_material: 0_usize,
                                        };
                                    }
                                });
                                if discriminant(&self.selected_tool)
                                    == discriminant(&BrushTool::ThermalBrush { temp_delta: 0_f32 })
                                {
                                    self.selected_tool = BrushTool::ThermalBrush {
                                        temp_delta: (self.selected_tool.get_temp_delta().signum()
                                            * (self.game_board.brush_size.max_elem() + 1_f32))
                                            / 16_f32,
                                    };
                                }
                                ui.horizontal_centered(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(
                                                    get_text(
                                                        &self.program_options.locale,
                                                        self.program_options.selected_locale,
                                                    )
                                                    .heat_button
                                                    .as_str(),
                                                )
                                                .heading()
                                                .strong()
                                                .monospace(),
                                            )
                                            .fill(Color32::DARK_RED)
                                            .stroke(Stroke::new(1_f32, Color32::WHITE))
                                            .min_size(placed_button.intrinsic_size().unwrap()),
                                        )
                                        .clicked()
                                    {
                                        self.selected_tool =
                                            BrushTool::ThermalBrush { temp_delta: 1_f32 };
                                    }
                                });
                                ui.horizontal_centered(|ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(
                                                    get_text(
                                                        &self.program_options.locale,
                                                        self.program_options.selected_locale,
                                                    )
                                                    .cool_button
                                                    .as_str(),
                                                )
                                                .heading()
                                                .strong()
                                                .monospace(),
                                            )
                                            .fill(Color32::DARK_BLUE)
                                            .stroke(Stroke::new(1_f32, Color32::WHITE))
                                            .min_size(placed_button.intrinsic_size().unwrap()),
                                        )
                                        .clicked()
                                    {
                                        self.selected_tool =
                                            BrushTool::ThermalBrush { temp_delta: -1_f32 };
                                    }
                                });
                                MaterialType::iter().for_each(|category| {
                                    let category_button = ui.add(
                                        egui::Button::new(
                                            Image::new(category.get_icon()).fit_to_exact_size(
                                                Vec2::splat(
                                                    placed_button.intrinsic_size().unwrap().x
                                                        * 0.9_f32,
                                                ),
                                            ),
                                        )
                                        .corner_radius(10_u8),
                                    );
                                    if category_button.hovered() {
                                        category_button.show_tooltip_text(
                                            get_text(
                                                &self.program_options.locale,
                                                self.program_options.selected_locale,
                                            )
                                            .category_names
                                            .get(&category.discriminant())
                                            .unwrap()
                                            .as_str(),
                                        );
                                    }
                                    if category_button.clicked() {
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
            let frame_image: ColorImage = ColorImage::new(
                [
                    self.game_board.width as usize,
                    self.game_board.height as usize,
                ],
                pixels,
            );
            self.material_texture =
                ctx.load_texture("Board", frame_image.clone(), TextureOptions::NEAREST);
            self.material_texture
                .set(frame_image.clone(), TextureOptions::NEAREST);
            let sized_texture = load::SizedTexture::new(
                self.material_texture.id(),
                self.material_texture.size_vec2(),
            );
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
                    self.viewed_particle = get_particle(&self.game_board, &board);
                    self.game_board.cellsize = vec2(
                        width / self.game_board.width as f32,
                        width / self.game_board.width as f32,
                    );
                    if !self.dialogopen {
                        let heatmap_pixels = self
                            .game_board
                            .draw_board_temperature(&self.black_body_gradient);
                        let frame_image: ColorImage = ColorImage::new(
                            [
                                self.game_board.width as usize,
                                self.game_board.height as usize,
                            ],
                            heatmap_pixels,
                        );
                        self.heatmap_texture = ctx.load_texture(
                            "Board_heatmap",
                            frame_image.clone(),
                            TextureOptions::LINEAR,
                        );
                        self.heatmap_texture
                            .set(frame_image.clone(), TextureOptions::LINEAR);
                        ui.painter()
                            .clone()
                            .with_clip_rect(board.interact_rect)
                            .with_layer_id(LayerId::new(egui::Order::Middle, Id::new(hash(3_i32))))
                            .image(
                                self.heatmap_texture.id(),
                                board.interact_rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                    }
                    if self.program_options.debug_mode {
                        debug_text_rendering(
                            &self.game_board,
                            &self.materials,
                            &mut self.debug_text_job,
                            &mut self.fps_values,
                            &board,
                            ctx,
                            ui,
                        );
                    }
                    draw_brush_outlines(&self.game_board, &board, ui, ctx);
                    handle_mouse_input(
                        &mut self.game_board,
                        &self.materials,
                        &self.selected_tool,
                        board.clone(),
                    );
                    handle_key_inputs(
                        &mut self.game_board,
                        &self.materials,
                        &self.physical_transitions,
                        &mut self.program_options,
                        &mut self.framecount,
                        ctx.input(|time| time.unstable_dt),
                        board,
                    );
                });
            } else {
                ui.vertical_centered(|ui| {
                    let board = ui.add(board_display);
                    self.viewed_particle = get_particle(&self.game_board, &board);
                    self.game_board.cellsize = vec2(
                        height / self.game_board.height as f32,
                        height / self.game_board.height as f32,
                    );
                    if !self.dialogopen {
                        let heatmap_pixels = self
                            .game_board
                            .draw_board_temperature(&self.black_body_gradient);
                        let frame_image: ColorImage = ColorImage::new(
                            [
                                self.game_board.width as usize,
                                self.game_board.height as usize,
                            ],
                            heatmap_pixels,
                        );
                        self.heatmap_texture = ctx.load_texture(
                            "Board_heatmap",
                            frame_image.clone(),
                            TextureOptions::LINEAR,
                        );
                        self.heatmap_texture
                            .set(frame_image.clone(), TextureOptions::LINEAR);
                        ui.painter()
                            .clone()
                            .with_clip_rect(board.interact_rect)
                            .with_layer_id(LayerId::new(egui::Order::Middle, Id::new(hash(3_i32))))
                            .image(
                                self.heatmap_texture.id(),
                                board.interact_rect,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );
                    }
                    if self.program_options.debug_mode {
                        debug_text_rendering(
                            &self.game_board,
                            &self.materials,
                            &mut self.debug_text_job,
                            &mut self.fps_values,
                            &board,
                            ctx,
                            ui,
                        );
                    }
                    draw_brush_outlines(&self.game_board, &board, ui, ctx);
                    handle_mouse_input(
                        &mut self.game_board,
                        &self.materials,
                        &self.selected_tool,
                        board.clone(),
                    );
                    handle_key_inputs(
                        &mut self.game_board,
                        &self.materials,
                        &self.physical_transitions,
                        &mut self.program_options,
                        &mut self.framecount,
                        ctx.input(|time| time.unstable_dt),
                        board,
                    );
                });
            }
            update_board(
                &mut self.game_board,
                &self.materials,
                &self.physical_transitions,
                self.program_options.simulation_stopped,
                &mut self.framecount,
                ctx.input(|time| time.unstable_dt),
            );
            egui::Context::request_repaint(ctx);
        });
    }
}
