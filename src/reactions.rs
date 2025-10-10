use crate::{physics::Phase, world::Board};
use egui::{Color32, epaint::Hsva, lerp};
use serde::{Deserialize, Serialize};

#[derive(Copy, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[rustfmt::skip]
pub(crate) enum MaterialType {
    Acid,       // Corrosive material - everything with a pH value lower than 7.0
    Alloy,      // Mixture of metals
    Atmosphere, // Mixture of materials that are always present in the simulation*/
    Base,       // Corrosive material - everything with a pH value higher than 7.0
    Ceramic,    // Hard, brittle, heat-resistant, and corrosion-resistant material
    Cloner,     // Material that clones the last new material it came in contact with
    Explosive,  // A material that generates a lot of energy and lot of gases
    Fuel,       // Flammable material under normal circumstances
    Glass,      // Amorphous material formed from a molten material and it's cooled without proper crystalization
    Oxidizer,   // This material can enhance the explosive power of explosives or the burning of fuels by aiding their combustion
    Decor,      // This material is indestructible and completely inert it's used for decoration purposes, mainly pixelart, map making, etc...
    Sink,       // This material swallows any material it comes in contact with it
    Solution,   // Material that contains other materials e.g. salts, on heat it leaves the dissolved materials behind
    Solvent,    // Dissolves certain materials
}

impl Board {
    #[inline(always)]
    pub(crate) fn solve_reactions(&mut self, i: usize, j: usize, framedelta: f32, framecount: u64) {
        match self.contents[(i, j)].material.material_type {
            MaterialType::Fuel => {
                let rnd = rand::random_range(0_u8..4_u8);
                if std::mem::discriminant(
                    &self
                        .contents
                        .get(i, j + 1)
                        .unwrap_or(&self.contents[(i, j)])
                        .material
                        .phase,
                ) == std::mem::discriminant(&(Phase::Plasma { energy: 0_f32 }))
                    && self.contents.get(i, j + 1).is_some()
                    && rnd == 0
                {
                    self.contents[(i, j)] = self.contents[(i, j + 1)].clone();
                    self.contents[(i, j)].material.phase = Phase::Plasma { energy: 70.0 };
                    self.contents[(i, j)].display_color = self.contents[(i, j + 1)]
                        .material
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            self.contents[(i, j + 1)]
                                .material
                                .material_color
                                .shinyness
                                .clone(),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] =
                        self.contents[(i, j)].material.material_color.color.a();
                } else if std::mem::discriminant(
                    &self
                        .contents
                        .get(i, j - 1)
                        .unwrap_or(&self.contents[(i, j)])
                        .material
                        .phase,
                ) == std::mem::discriminant(&(Phase::Plasma { energy: 0_f32 }))
                    && self.contents.get(i, j - 1).is_some()
                    && rnd == 1
                {
                    self.contents[(i, j)] = self.contents[(i, j - 1)].clone();
                    self.contents[(i, j)].material.phase = Phase::Plasma { energy: 70.0 };
                    self.contents[(i, j)].display_color = self.contents[(i, j - 1)]
                        .material
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            self.contents[(i, j - 1)]
                                .material
                                .material_color
                                .shinyness
                                .clone(),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] =
                        self.contents[(i, j - 1)].material.material_color.color.a();
                } else if std::mem::discriminant(
                    &self
                        .contents
                        .get(i + 1, j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material
                        .phase,
                ) == std::mem::discriminant(&(Phase::Plasma { energy: 0_f32 }))
                    && rnd == 2
                {
                    self.contents[(i, j)] = self.contents[(i + 1, j)].clone();
                    self.contents[(i, j)].material.phase = Phase::Plasma { energy: 70.0 };
                    self.contents[(i, j)].display_color = self.contents[(i + 1, j)]
                        .material
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            self.contents[(i + 1, j)]
                                .material
                                .material_color
                                .shinyness
                                .clone(),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] =
                        self.contents[(i + 1, j)].material.material_color.color.a();
                } else if std::mem::discriminant(
                    &self
                        .contents
                        .get(i - 1, j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material
                        .phase,
                ) == std::mem::discriminant(&(Phase::Plasma { energy: 0_f32 }))
                    && rnd == 3
                {
                    self.contents[(i, j)] = self.contents[(i - 1, j)].clone();
                    self.contents[(i, j)].material.phase = Phase::Plasma { energy: 70.0 };
                    self.contents[(i, j)].display_color = self.contents[(i - 1, j)]
                        .material
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            self.contents[(i - 1, j)]
                                .material
                                .material_color
                                .shinyness
                                .clone(),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] =
                        self.contents[(i - 1, j)].material.material_color.color.a();
                }
            }
            MaterialType::Decor => {
                if self.contents[(i, j)].material.material_color.color
                    == Color32::from_rgba_unmultiplied(0, 0, 0, 0)
                {
                    self.contents[(i, j)].display_color =
                        Hsva::new(((framecount / 4) % (355)) as f32 / (355.0), 1.0, 1.0, 1.0)
                            .into();
                    self.contents[(i, j)].material.material_color.color =
                        Hsva::new(((framecount / 4) % (355)) as f32 / (355.0), 1.0, 1.0, 1.0)
                            .into();
                    self.contents[(i, j)].display_color =
                        self.contents[(i, j)].display_color.gamma_multiply(lerp(
                            self.contents[(i, j)]
                                .material
                                .material_color
                                .shinyness
                                .clone(),
                            self.rngs[(i, j)],
                        ));
                }
            }
            _ => {}
        }
    }
}
