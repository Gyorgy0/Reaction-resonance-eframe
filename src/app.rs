use core::f32;
use std::mem::discriminant;

use crate::EFrameApp;
use crate::dialogs::{BoardSize, OptionsMenuDialog};
use crate::egui_input::BrushTool;
use crate::locale::get_text;
use crate::material::Material;
use crate::system_data::{ApplicationOptions, get_sign, get_temperature, import_materials};
use crate::system_ui::{debug_text_rendering, get_particle};
use crate::{
    egui_input::{BrushShape, handle_key_inputs, handle_mouse_input, resize_brush},
    reactions::MaterialType,
    system_ui::draw_brush_outlines,
    world::update_board,
};
use egui::util::hash;
use egui::{
    Color32, ColorImage, Id, Image, LayerId, Rect, RichText, Sense, Stroke, TextureOptions, Theme,
    Vec2, load, pos2, vec2,
};
use egui_dialogs::DialogDetails;
use strum::IntoEnumIterator;

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
            && let Ok(options) = res.reply::<(f32, ApplicationOptions, BoardSize)>()
        {
            self.game_board.gravity = options.0;
            self.program_options = options.1;
            self.dialogopen = false;
            if self.game_board.board_size != options.2 {
                self.game_board.board_size = options.2;
                self.game_board.width = self.game_board.board_size.get_size().0;
                self.game_board.height = self.game_board.board_size.get_size().1;
                self.game_board.create_board();
            }
        }
        /*
        egui::Panel::left("navbar_placeholder")
        .exact_size(15_f32)
        .show(ctx, |ui| {});
        */
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
                            self.game_board.board_size,
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
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .rectangle_brush_tooltip
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.game_board.brush_shape = BrushShape::Rectangle;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .rhombus_brush_tooltip
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.game_board.brush_shape = BrushShape::Rhombus;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .ellipse_brush_tooltip
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.game_board.brush_shape = BrushShape::Ellipse;
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
                    #[cfg(not(any(
                        target_os = "android",
                        target_arch = "wasm32",
                        target_os = "ios"
                    )))]
                    {
                        if ui
                            .add(
                                egui::widgets::Button::new(
                                    RichText::new(
                                        get_text(
                                            &self.program_options.locale,
                                            self.program_options.selected_locale,
                                        )
                                        .reload_button
                                        .as_str(),
                                    )
                                    .heading()
                                    .strong(),
                                )
                                .stroke(Stroke::new(1_f32, Color32::WHITE))
                                .fill(Color32::PURPLE),
                            )
                            .clicked()
                        {
                            // Reimports the materials

                            use crate::system_data::{
                                import_locales, import_reactions, import_transitions,
                            };
                            self.materials = import_materials(&mut vec![]);
                            // Sorts the elements by their Id's and outputs them to the category list
                            self.materials.sort_by_key(|elem| elem.1.id);
                            self.material_categories = vec![];
                            for category in MaterialType::iter() {
                                let mut category_vec: Vec<(String, Material)> = vec![];
                                for material in self.materials.iter() {
                                    if discriminant(&category)
                                        == discriminant(&material.1.material_type)
                                    {
                                        category_vec.push(material.clone());
                                    }
                                }
                                self.material_categories.push(category_vec);
                            }
                            self.program_options.locale =
                                import_locales(&mut self.program_options.locale);
                            self.physical_transitions = import_transitions();
                            self.chemical_reactions = import_reactions();
                        }
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
                            particle_name = self.program_options.locale
                                [self.program_options.selected_locale]
                                .element_names
                                .get(&self.materials[self.viewed_particle.material_id].0)
                                .unwrap_or(&self.materials[self.viewed_particle.material_id].0),
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
                                                RichText::new(
                                                    self.program_options.locale
                                                        [self.program_options.selected_locale]
                                                        .element_names
                                                        .get(&material.0)
                                                        .unwrap_or(&material.0),
                                                )
                                                .size(20_f32)
                                                .color(Color32::WHITE)
                                                .strong(),
                                            )
                                            .min_size(vec2(70_f32, 35_f32))
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
                                    egui::widgets::Button::new(RichText::new("01234567891"))
                                        .stroke(Stroke::new(1_f32, Color32::WHITE))
                                        .min_size(Vec2::new(button_width, button_width / 2_f32));
                                let placed_button = ui.add_visible(false, button);
                                ui.horizontal_centered(|ui| {
                                    ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                        Color32::GRAY;
                                    ui.style_mut().visuals.widgets.hovered.expansion = 1_f32;
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
                                    == discriminant(&BrushTool::ThermalBrush {
                                        temp_delta: 0_f32,
                                        default_temp: false,
                                    })
                                    && !self.selected_tool.get_default_temp()
                                {
                                    self.selected_tool = BrushTool::ThermalBrush {
                                        temp_delta: (self.selected_tool.get_temp_delta().signum()
                                            * (self.game_board.brush_size.max_elem() + 1_f32))
                                            / 16_f32,
                                        default_temp: false,
                                    };
                                }
                                ui.horizontal_centered(|ui| {
                                    ui.style_mut().visuals.widgets.hovered.expansion = 1_f32;
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(
                                                    get_text(
                                                        &self.program_options.locale,
                                                        self.program_options.selected_locale,
                                                    )
                                                    .mix_button
                                                    .as_str(),
                                                )
                                                .heading()
                                                .strong()
                                                .monospace(),
                                            )
                                            .fill(Color32::from_rgba_unmultiplied(
                                                69_u8, 28_u8, 10_u8, 255_u8,
                                            ))
                                            .stroke(Stroke::new(1_f32, Color32::WHITE))
                                            .min_size(placed_button.intrinsic_size().unwrap()),
                                        )
                                        .clicked()
                                    {
                                        self.selected_tool = BrushTool::MixBrush;
                                    }
                                });
                                ui.horizontal_centered(|ui| {
                                    ui.style_mut().visuals.widgets.hovered.expansion = 1_f32;
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
                                        self.selected_tool = BrushTool::ThermalBrush {
                                            temp_delta: 1_f32,
                                            default_temp: false,
                                        };
                                    }
                                });
                                ui.horizontal_centered(|ui| {
                                    ui.style_mut().visuals.widgets.hovered.expansion = 1_f32;
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                RichText::new(
                                                    get_text(
                                                        &self.program_options.locale,
                                                        self.program_options.selected_locale,
                                                    )
                                                    .default_heat_button
                                                    .as_str(),
                                                )
                                                .heading()
                                                .strong()
                                                .monospace(),
                                            )
                                            .fill(Color32::DARK_GREEN)
                                            .stroke(Stroke::new(1_f32, Color32::WHITE))
                                            .min_size(placed_button.intrinsic_size().unwrap()),
                                        )
                                        .clicked()
                                    {
                                        self.selected_tool = BrushTool::ThermalBrush {
                                            temp_delta: 0_f32,
                                            default_temp: true,
                                        };
                                    }
                                });
                                ui.horizontal_centered(|ui| {
                                    ui.style_mut().visuals.widgets.hovered.expansion = 1_f32;
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
                                        self.selected_tool = BrushTool::ThermalBrush {
                                            temp_delta: -1_f32,
                                            default_temp: false,
                                        };
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
                        &mut self.rng,
                        &self.selected_tool,
                        board.clone(),
                    );
                    handle_key_inputs(
                        &mut self.game_board,
                        &self.materials,
                        &self.physical_transitions,
                        &self.chemical_reactions,
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
                        &mut self.rng,
                        &self.selected_tool,
                        board.clone(),
                    );
                    handle_key_inputs(
                        &mut self.game_board,
                        &self.materials,
                        &self.physical_transitions,
                        &self.chemical_reactions,
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
                &self.chemical_reactions,
                self.program_options.simulation_stopped,
                &mut self.framecount,
                ctx.input(|time| time.unstable_dt),
            );
            egui::Context::request_repaint(ctx);
        });
    }
}
