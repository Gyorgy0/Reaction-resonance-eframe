use std::{f32, ops::RangeInclusive};

use egui::{Layout, Separator, Vec2};
use egui_dialogs::{Dialog, dialog_window};

use crate::{
    locale::{Locale, get_text},
    material::Material,
    system_data::ApplicationOptions,
};

pub struct OptionsMenuDialog {
    pub picked_gravity: f32,
    pub default_gravity: f32,
    pub original_program_options: ApplicationOptions,
    pub program_options: ApplicationOptions,
    pub materials: Vec<(String, Material)>,
    pub locale: Vec<Locale>,
    pub selected_locale: usize,
}

impl OptionsMenuDialog {
    pub fn new(
        gravity: f32,
        program_options: ApplicationOptions,
        materials: Vec<(String, Material)>,
        locale: &Vec<Locale>,
        selected_locale: usize,
    ) -> Self {
        Self {
            picked_gravity: gravity,
            default_gravity: 9.81_f32,
            original_program_options: program_options.clone(),
            program_options: program_options.clone(),
            materials,
            locale: locale.clone(),
            selected_locale,
        }
    }
}
impl Dialog<(f32, ApplicationOptions)> for OptionsMenuDialog {
    fn show(
        &mut self,
        ctx: &egui::Context,
        dctx: &egui_dialogs::DialogContext,
    ) -> Option<(f32, ApplicationOptions)> {
        // Return None if the user hasn't selected something
        let mut res: Option<(f32, ApplicationOptions)> = None;
        dialog_window(
            ctx,
            dctx,
            get_text(&self.locale, self.selected_locale)
                .options_title
                .as_str(),
        )
        .max_size(Vec2::splat(ctx.content_rect().size().max_elem() / 4_f32))
        .show(ctx, |ui| {
            ui.with_layout(Layout::top_down(egui::Align::TOP), |ui| {
                ui.label(
                    get_text(
                        &self.program_options.locale,
                        self.program_options.selected_locale,
                    )
                    .gravity_option_label
                    .as_str(),
                );
                egui::Grid::new("gravity_controls").show(ui, |ui| {
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button("-1").clicked() {
                            self.picked_gravity -= 1_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("-5").clicked() {
                            self.picked_gravity -= 5_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("-10").clicked() {
                            self.picked_gravity -= 10_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("-50").clicked() {
                            self.picked_gravity -= 50_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                    });

                    ui.vertical_centered_justified(|ui| {
                        ui.add(
                            egui::widgets::Slider::new(
                                &mut self.picked_gravity,
                                RangeInclusive::new(-1_000_f32, 1_000_f32),
                            )
                            .suffix(" m/s²"),
                        );
                    });

                    ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                        if ui.button("+1").clicked() {
                            self.picked_gravity += 1_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("+5").clicked() {
                            self.picked_gravity += 5_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("+10").clicked() {
                            self.picked_gravity += 10_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("+50").clicked() {
                            self.picked_gravity += 50_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                    });
                    ui.end_row();

                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button("-0.01").clicked() {
                            self.picked_gravity -= 0.01_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("-0.05").clicked() {
                            self.picked_gravity -= 0.05_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("-0.10").clicked() {
                            self.picked_gravity -= 0.1_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("-0.50").clicked() {
                            self.picked_gravity -= 0.5_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                    });
                    ui.label("");
                    ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                        if ui.button("+0.01").clicked() {
                            self.picked_gravity += 0.01_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("+0.05").clicked() {
                            self.picked_gravity += 0.05_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("+0.10").clicked() {
                            self.picked_gravity += 0.1_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                        if ui.button("+0.50").clicked() {
                            self.picked_gravity += 0.5_f32;
                            self.picked_gravity = self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                        }
                    });
                    ui.end_row();

                    ui.separator();
                    ui.label("");
                    ui.separator();
                    ui.end_row();

                    ui.horizontal_centered(|ui| {
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .sun_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 274_f32;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .mercury_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 3.7_f32;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .venus_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 8.87_f32;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .moon_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 1.622_f32;
                        }
                    });
                    ui.horizontal_centered(|ui| {
                        ui.add(Separator::default().vertical().spacing(25_f32));
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .earth_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 9.81_f32;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .mars_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 3.71_f32;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .jupiter_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 24.79_f32;
                        }
                        ui.add(Separator::default().vertical().spacing(25_f32));
                    });
                    ui.horizontal_centered(|ui| {
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .saturn_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 10.44_f32;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .uranus_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 8.87_f32;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .neptune_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 11.15_f32;
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .pluto_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            self.picked_gravity = 0.62_f32;
                        }
                    });
                    ui.end_row();
                });
                ui.add_space(25_f32);
                ui.horizontal(|ui| {
                    ui.label(
                        get_text(
                            &self.program_options.locale,
                            self.program_options.selected_locale,
                        )
                        .screen_option_label
                        .as_str(),
                    );
                    if !self.program_options.fullscreen {
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .fullscreen_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            #[cfg(target_arch = "wasm32")]
                            {
                                use web_sys::OrientationLockType;
                                use web_sys::window;

                                let Some(window) = window() else {
                                    return;
                                };
                                let Some(document) = window.document() else {
                                    return;
                                };
                                if self.program_options.fullscreen {
                                    let _ = document.exit_fullscreen();

                                    let Ok(screen) = window.screen() else {
                                        return;
                                    };
                                    let _ = screen.orientation().unlock();

                                    self.program_options.fullscreen = false;
                                } else {
                                    let Some(element) = document.document_element() else {
                                        return;
                                    };
                                    let _ = element.request_fullscreen();

                                    let Ok(screen) = window.screen() else {
                                        return;
                                    };
                                    let _ =
                                        screen.orientation().lock(OrientationLockType::Landscape);
                                    self.program_options.fullscreen = true;
                                }
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                if !self.program_options.fullscreen {
                                    use egui::ViewportCommand;

                                    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
                                    self.program_options.fullscreen = true;
                                } else {
                                    use egui::ViewportCommand;

                                    ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
                                    self.program_options.fullscreen = false
                                }
                            }
                        }
                    } else if self.program_options.fullscreen
                        && ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .windowed_button
                                .as_str(),
                            )
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
                            if self.program_options.fullscreen {
                                let _ = document.exit_fullscreen();

                                let Ok(screen) = window.screen() else {
                                    return;
                                };
                                let _ = screen.orientation().unlock();

                                self.program_options.fullscreen = false;
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
                                self.program_options.fullscreen = true;
                            }
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                            self.program_options.fullscreen = false
                        }
                    }
                });
                ui.add_space(25_f32);
                ui.horizontal(|ui| {
                    ui.label(
                        get_text(
                            &self.program_options.locale,
                            self.program_options.selected_locale,
                        )
                        .temperature_scale_label
                        .as_str(),
                    );
                    ui.radio_value(
                        &mut self.program_options.temperature_scale,
                        crate::system_data::TemperatureScale::Kelvin,
                        get_text(
                            &self.program_options.locale,
                            self.program_options.selected_locale,
                        )
                        .temperature_kelvin_option
                        .as_str(),
                    );
                    ui.radio_value(
                        &mut self.program_options.temperature_scale,
                        crate::system_data::TemperatureScale::Celsius,
                        get_text(
                            &self.program_options.locale,
                            self.program_options.selected_locale,
                        )
                        .temperature_celsius_option
                        .as_str(),
                    );
                    ui.radio_value(
                        &mut self.program_options.temperature_scale,
                        crate::system_data::TemperatureScale::Fahrenheit,
                        get_text(
                            &self.program_options.locale,
                            self.program_options.selected_locale,
                        )
                        .temperature_fahrenheit_option
                        .as_str(),
                    );
                });
                ui.add_space(25_f32);
                ui.horizontal(|ui| {
                    ui.label(
                        get_text(
                            &self.program_options.locale,
                            self.program_options.selected_locale,
                        )
                        .language_label
                        .as_str(),
                    );
                    for index in 0..self.original_program_options.clone().locale.len() {
                        if ui
                            .button(
                                self.original_program_options.clone().locale[index]
                                    .get_language_name(),
                            )
                            .clicked()
                        {
                            self.program_options.selected_locale = index;
                            res = Some((self.picked_gravity, self.program_options.clone()));
                        }
                    }
                    // This is for the PC platform (locale and materials and their reactions are serialized from files)
                    #[cfg(not(any(
                        target_os = "android",
                        target_arch = "wasm32",
                        target_os = "ios"
                    )))]
                    {
                        ui.separator();
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .generate_default_locale_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            use std::fs;

                            use crate::locale::Locale;

                            let mut default_locale = Locale::default();
                            default_locale.fill_elements(self.materials.clone());
                            let serialized_default_locale =
                                serde_json::to_string_pretty(&default_locale).unwrap();
                            fs::write("src/locale/default_locale.json", serialized_default_locale)
                                .unwrap();
                        }
                    }
                });
                ui.add_space(25_f32);
                ui.horizontal(|ui| {
                    ui.label(
                        get_text(
                            &self.program_options.locale,
                            self.program_options.selected_locale,
                        )
                        .board_size_label
                        .as_str(),
                    );
                });
                ui.separator();
                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .options_ok_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            res = Some((self.picked_gravity, self.program_options.clone()));
                        }
                        if ui
                            .button(
                                get_text(
                                    &self.program_options.locale,
                                    self.program_options.selected_locale,
                                )
                                .options_cancel_button
                                .as_str(),
                            )
                            .clicked()
                        {
                            res =
                                Some((self.default_gravity, self.original_program_options.clone()));
                        }
                    });
                });
            });
        });
        res
    }
}
