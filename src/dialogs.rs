use std::{f32, ops::RangeInclusive};

use crossbeam::epoch::Pointable;
use egui::{Align2, Layout, Separator, Vec2};
use egui_dialogs::{Dialog, dialog_window};

pub struct OptionsMenuDialog {
    pub picked_gravity: f32,
    pub default_gravity: f32,
    pub fullscreen: bool,
}

impl OptionsMenuDialog {
    pub fn new(gravity: f32, fullscreen: bool) -> Self {
        Self {
            picked_gravity: gravity,
            default_gravity: 9.81_f32,
            fullscreen,
        }
    }
}
impl Dialog<(f32, bool)> for OptionsMenuDialog {
    fn show(
        &mut self,
        ctx: &egui::Context,
        dctx: &egui_dialogs::DialogContext,
    ) -> Option<(f32, bool)> {
        // Return None if the user hasn't selected something
        let mut res: Option<(f32, bool)> = None;
        dialog_window(ctx, dctx, "Options")
            .auto_sized()
            .show(ctx, |ui| {
                ui.with_layout(Layout::top_down(egui::Align::TOP), |ui| {
                    ui.label("Gravity:");
                    egui::Grid::new("gravity_controls").show(ui, |ui| {
                        ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button("-1").clicked() {
                                self.picked_gravity -= 1_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("-5").clicked() {
                                self.picked_gravity -= 5_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("-10").clicked() {
                                self.picked_gravity -= 10_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("-50").clicked() {
                                self.picked_gravity -= 50_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                        });

                        ui.vertical_centered_justified(|ui| {
                            ui.add(
                                egui::widgets::Slider::new(
                                    &mut self.picked_gravity,
                                    RangeInclusive::new(-1_000_f32, 1_000_f32),
                                )
                                .suffix(" m/s^2"),
                            );
                        });

                        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                            if ui.button("+1").clicked() {
                                self.picked_gravity += 1_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("+5").clicked() {
                                self.picked_gravity += 5_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("+10").clicked() {
                                self.picked_gravity += 10_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("+50").clicked() {
                                self.picked_gravity += 50_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                        });
                        ui.end_row();

                        ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                            if ui.button("-0.01").clicked() {
                                self.picked_gravity -= 0.01_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("-0.05").clicked() {
                                self.picked_gravity -= 0.05_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("-0.10").clicked() {
                                self.picked_gravity -= 0.1_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("-0.50").clicked() {
                                self.picked_gravity -= 0.5_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                        });
                        ui.label("");
                        ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                            if ui.button("+0.01").clicked() {
                                self.picked_gravity += 0.01_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("+0.05").clicked() {
                                self.picked_gravity += 0.05_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("+0.10").clicked() {
                                self.picked_gravity += 0.1_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                            if ui.button("+0.50").clicked() {
                                self.picked_gravity += 0.5_f32;
                                self.picked_gravity =
                                    self.picked_gravity.clamp(-1_000_f32, 1_000_f32);
                            }
                        });
                        ui.end_row();

                        ui.separator();
                        ui.label("");
                        ui.separator();
                        ui.end_row();

                        ui.horizontal_centered(|ui| {
                            if ui.button("Sun").clicked() {
                                self.picked_gravity = 274_f32;
                            }
                            if ui.button("Mercury").clicked() {
                                self.picked_gravity = 3.7_f32;
                            }
                            if ui.button("Venus").clicked() {
                                self.picked_gravity = 8.87_f32;
                            }
                            if ui.button("Moon").clicked() {
                                self.picked_gravity = 1.622_f32;
                            }
                        });
                        ui.horizontal_centered(|ui| {
                            ui.add(Separator::default().vertical().spacing(25_f32));
                            if ui.button("Earth").clicked() {
                                self.picked_gravity = 9.81_f32;
                            }
                            if ui.button("Mars").clicked() {
                                self.picked_gravity = 3.71_f32;
                            }
                            if ui.button("Jupiter").clicked() {
                                self.picked_gravity = 24.79_f32;
                            }
                            ui.add(Separator::default().vertical().spacing(25_f32));
                        });
                        ui.horizontal_centered(|ui| {
                            if ui.button("Saturn").clicked() {
                                self.picked_gravity = 10.44_f32;
                            }
                            if ui.button("Uranus").clicked() {
                                self.picked_gravity = 8.87_f32;
                            }
                            if ui.button("Neptune").clicked() {
                                self.picked_gravity = 11.15_f32;
                            }
                            if ui.button("Pluto").clicked() {
                                self.picked_gravity = 0.62_f32;
                            }
                        });
                        ui.end_row();
                    });
                    ui.add_space(25_f32);
                    ui.horizontal(|ui| {
                        ui.label("Window: ");
                        if !self.fullscreen {
                            if ui.button("Fullscreen").clicked() {
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
                                            .lock(OrientationLockType::Landscape);
                                        self.fullscreen = true;
                                    }
                                }
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    if !self.fullscreen {
                                        use egui::ViewportCommand;

                                        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(true));
                                        self.fullscreen = true;
                                    } else {
                                        use egui::ViewportCommand;

                                        ctx.send_viewport_cmd(ViewportCommand::Fullscreen(false));
                                        self.fullscreen = false
                                    }
                                }
                            }
                        } else if self.fullscreen && ui.button("Windowed").clicked() {
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
                                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
                                self.fullscreen = false
                            }
                        }
                    });
                    ui.separator();
                    ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Ok").clicked() {
                                res = Some((self.picked_gravity, self.fullscreen));
                            }
                            if ui.button("Cancel").clicked() {
                                res = Some((self.default_gravity, self.fullscreen));
                            }
                        });
                    });
                });
            });
        res
    }
}
