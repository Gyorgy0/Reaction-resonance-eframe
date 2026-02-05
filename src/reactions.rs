use crate::material::{Material, VOID, tuple_to_rangeinclusive};
use crate::world::Board;
use crate::{particle::Particle, physics::Phase};
use egui::Color32;
use egui::epaint::Hsva;
use egui::lerp;
use grid::Grid;
use serde::{Deserialize, Serialize};
use std::mem::discriminant;
use std::ops::RangeInclusive;
use strum_macros::EnumIter;

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize, EnumIter)]
#[rustfmt::skip]
#[repr(u8)]
pub(crate) enum MaterialType {
    Acid,       // Corrosive material - everything with a pH value lower than 7.0
    Alloy,      // Mixture of metals
    Atmosphere, // Mixture of materials that are always present in the simulation
    Base,       // Corrosive material - everything with a pH value higher than 7.0
    CAutomata {neighbordhood: u8, birth: u8, survival:u8, stages: u8},  // Cellular automaton material defined by 4 rules (birth, survival, neighborhood and life stages)
    Ceramic,    // Hard, brittle, heat-resistant, and corrosion-resistant material
    Cloner,     // Material that clones the last new material it came in contact with
    Explosive,  // A material that generates a lot of energy and lot of gases
    Fuel,       // Flammable material under normal circumstances
    Glass,      // Amorphous material formed from a molten material and it's cooled without proper crystalization
    Oxidizer,   // This material can enhance the explosive power of explosives or the burning of fuels by aiding their combustion
    Decor,      // This material is indestructible and completely inert it's used for decoration purposes, mainly pixelart, map making, etc...
    Sink,       // This material swallows any material it comes in contact with it
    Solution,   // Material that contains other materials e.g. salts, if heated it leaves the dissolved materials behind
    Solvent,    // Dissolves certain materials
}

impl MaterialType {
    pub fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl MaterialType {
    pub fn get_neighborhood() {}
    pub fn get_birth() {}
    pub fn get_survival() {}
}
impl Board {
    #[inline(always)]
    pub(crate) fn solve_reactions(
        &mut self,
        prev_board: &Grid<Particle>,
        materials: &Vec<(String, Material)>,
        i: usize,
        j: usize,
        framedelta: f32,
        framecount: u64,
    ) {
        match &materials[self.contents[(i, j)].material_id].1.material_type {
            MaterialType::Fuel => {
                let rnd = rand::random_range(0_u8..4_u8);
                if std::mem::discriminant(
                    &materials[self
                        .contents
                        .get(i, j.wrapping_add(1))
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .phase,
                ) == std::mem::discriminant(&(Phase::Plasma))
                    && self.contents.get(i, j.wrapping_add(1)).is_some()
                    && rnd == 0_u8
                {
                    self.contents[(i, j)] = self.contents[(i, j.wrapping_add(1))].clone();
                    self.contents[(i, j)].material_id = 7_usize;
                    self.contents[(i, j)].energy = 70_f32;
                    self.contents[(i, j)].display_color = materials
                        [self.contents[(i, j.wrapping_add(1))].material_id]
                        .1
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials[self.contents[(i, j.wrapping_add(1))].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] = materials
                        [self.contents[(i, j)].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                } else if std::mem::discriminant(
                    &materials[self
                        .contents
                        .get(i, j.saturating_sub(1))
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .phase,
                ) == std::mem::discriminant(&(Phase::Plasma))
                    && self.contents.get(i, j.saturating_sub(1)).is_some()
                    && rnd == 1
                {
                    self.contents[(i, j)] = self.contents[(i, j.saturating_sub(1))].clone();
                    self.contents[(i, j)].material_id = 7_usize;
                    self.contents[(i, j)].energy = 70_f32;
                    self.contents[(i, j)].display_color = materials
                        [self.contents[(i, j.saturating_sub(1))].material_id]
                        .1
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials[self.contents[(i, j.saturating_sub(1))].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] = materials
                        [self.contents[(i, j.saturating_sub(1))].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                } else if std::mem::discriminant(
                    &materials[self
                        .contents
                        .get(i.wrapping_add(1), j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .phase,
                ) == std::mem::discriminant(&(Phase::Plasma))
                    && rnd == 2
                {
                    self.contents[(i, j)] = self.contents[(i.wrapping_add(1), j)].clone();
                    self.contents[(i, j)].material_id = 7_usize;
                    self.contents[(i, j)].energy = 70_f32;
                    self.contents[(i, j)].display_color = materials
                        [self.contents[(i.wrapping_add(1), j)].material_id]
                        .1
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials[self.contents[(i.wrapping_add(1), j)].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] = materials
                        [self.contents[(i.wrapping_add(1), j)].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                } else if std::mem::discriminant(
                    &materials[self
                        .contents
                        .get(i.saturating_sub(1), j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .phase,
                ) == std::mem::discriminant(&(Phase::Plasma))
                    && rnd == 3
                {
                    self.contents[(i, j)] = self.contents[(i.saturating_sub(1), j)].clone();
                    self.contents[(i, j)].material_id = 7_usize;
                    self.contents[(i, j)].energy = 70_f32;
                    self.contents[(i, j)].display_color = materials
                        [self.contents[(i.saturating_sub(1), j)].material_id]
                        .1
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials[self.contents[(i.saturating_sub(1), j)].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] = materials
                        [self.contents[(i.saturating_sub(1), j)].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                }
            }
            MaterialType::Cloner => {
                if self.contents[(i, j)].cloned_material == 0_usize {
                    if materials[self
                        .contents
                        .get(i, j.wrapping_add(1))
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        != VOID
                        && materials[self
                            .contents
                            .get(i, j.wrapping_add(1))
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .material_type
                            != MaterialType::Cloner
                    {
                        self.contents[(i, j)].cloned_material =
                            self.contents[(i, j.wrapping_add(1))].material_id;
                    } else if materials[self
                        .contents
                        .get(i, j.saturating_sub(1))
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        != VOID
                        && materials[self
                            .contents
                            .get(i, j.saturating_sub(1))
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .material_type
                            != MaterialType::Cloner
                    {
                        self.contents[(i, j)].cloned_material =
                            self.contents[(i, j.saturating_sub(1))].material_id;
                    } else if materials[self
                        .contents
                        .get(i.wrapping_add(1), j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        != VOID
                        && materials[self
                            .contents
                            .get(i.wrapping_add(1), j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .material_type
                            != MaterialType::Cloner
                    {
                        self.contents[(i, j)].cloned_material =
                            self.contents[(i.wrapping_add(1), j)].material_id;
                    } else if materials[self
                        .contents
                        .get(i.saturating_sub(1), j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        != VOID
                        && materials[self
                            .contents
                            .get(i.saturating_sub(1), j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .material_type
                            != MaterialType::Cloner
                    {
                        self.contents[(i, j)].cloned_material =
                            self.contents[(i.saturating_sub(1), j)].material_id;
                    }
                } else if self.contents[(i, j)].cloned_material != 0_usize {
                    let material = self.contents[(i, j)].cloned_material;
                    if self
                        .contents
                        .get(i, j.wrapping_add(1))
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id
                        == 0_usize
                    {
                        self.contents[(i, j.wrapping_add(1))].material_id = material;
                        self.contents[(i, j.wrapping_add(1))].display_color =
                            materials[material].1.material_color.color;
                        self.contents[(i, j.wrapping_add(1))].display_color = self.contents
                            [(i, j.wrapping_add(1))]
                            .display_color
                            .gamma_multiply(lerp(
                                tuple_to_rangeinclusive(
                                    materials[self.contents[(i, j.wrapping_add(1))].material_id]
                                        .1
                                        .material_color
                                        .shinyness,
                                ),
                                self.rngs[(i, j.wrapping_add(1))],
                            ));
                        self.contents[(i, j.wrapping_add(1))].display_color[3] = materials
                            [self.contents[(i, j.wrapping_add(1))].material_id]
                            .1
                            .material_color
                            .color
                            .a();
                    }
                    if self
                        .contents
                        .get(i, j.saturating_sub(1))
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id
                        == 0_usize
                    {
                        self.contents[(i, j.saturating_sub(1))].material_id = material;
                        self.contents[(i, j.saturating_sub(1))].display_color =
                            materials[material].1.material_color.color;
                        self.contents[(i, j.saturating_sub(1))].display_color = self.contents
                            [(i, j.saturating_sub(1))]
                            .display_color
                            .gamma_multiply(lerp(
                                tuple_to_rangeinclusive(
                                    materials[self.contents[(i, j.saturating_sub(1))].material_id]
                                        .1
                                        .material_color
                                        .shinyness,
                                ),
                                self.rngs[(i, j.saturating_sub(1))],
                            ));
                        self.contents[(i, j.saturating_sub(1))].display_color[3] = materials
                            [self.contents[(i, j.saturating_sub(1))].material_id]
                            .1
                            .material_color
                            .color
                            .a();
                    }
                    if self
                        .contents
                        .get(i.wrapping_add(1), j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id
                        == 0_usize
                    {
                        self.contents[(i.wrapping_add(1), j)].material_id = material;
                        self.contents[(i.wrapping_add(1), j)].display_color =
                            materials[material].1.material_color.color;
                        self.contents[(i.wrapping_add(1), j)].display_color = self.contents
                            [(i.wrapping_add(1), j)]
                            .display_color
                            .gamma_multiply(lerp(
                                tuple_to_rangeinclusive(
                                    materials[self.contents[(i.wrapping_add(1), j)].material_id]
                                        .1
                                        .material_color
                                        .shinyness,
                                ),
                                self.rngs[(i.wrapping_add(1), j)],
                            ));
                        self.contents[(i.wrapping_add(1), j)].display_color[3] = materials
                            [self.contents[(i.wrapping_add(1), j)].material_id]
                            .1
                            .material_color
                            .color
                            .a();
                    }
                    if self
                        .contents
                        .get(i.saturating_sub(1), j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id
                        == 0_usize
                    {
                        self.contents[(i.saturating_sub(1), j)].material_id = material;
                        self.contents[(i.saturating_sub(1), j)].display_color =
                            materials[material].1.material_color.color;
                        self.contents[(i.saturating_sub(1), j)].display_color = self.contents
                            [(i.saturating_sub(1), j)]
                            .display_color
                            .gamma_multiply(lerp(
                                tuple_to_rangeinclusive(
                                    materials[self.contents[(i.saturating_sub(1), j)].material_id]
                                        .1
                                        .material_color
                                        .shinyness,
                                ),
                                self.rngs[(i.saturating_sub(1), j)],
                            ));
                        self.contents[(i.saturating_sub(1), j)].display_color[3] = materials
                            [self.contents[(i.saturating_sub(1), j)].material_id]
                            .1
                            .material_color
                            .color
                            .a();
                    }
                }
            }
            MaterialType::Decor => {
                if self.contents[(i, j)].display_color
                    == Color32::from_rgba_unmultiplied(0, 0, 0, 0)
                {
                    self.contents[(i, j)].display_color = Hsva::new(
                        ((framecount / 4) % (355)) as f32 / (355_f32),
                        1_f32,
                        1_f32,
                        1_f32,
                    )
                    .into();
                    self.contents[(i, j)].display_color = Hsva::new(
                        ((framecount / 4) % (355)) as f32 / (355_f32),
                        1_f32,
                        1_f32,
                        1_f32,
                    )
                    .into();
                    self.contents[(i, j)].display_color =
                        self.contents[(i, j)].display_color.gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials[self.contents[(i, j)].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            self.rngs[(i, j)],
                        ));
                }
            }
            MaterialType::Sink => {
                if std::mem::discriminant(
                    &materials[self
                        .contents
                        .get(i, j.wrapping_add(1))
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .material_type,
                ) != std::mem::discriminant(&MaterialType::Sink)
                {
                    self.contents[(i, j.wrapping_add(1))].material_id = VOID.id;
                    self.contents[(i, j.wrapping_add(1))].display_color = VOID.material_color.color;
                }
                if std::mem::discriminant(
                    &materials[self
                        .contents
                        .get(i, j.saturating_sub(1))
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .material_type,
                ) != std::mem::discriminant(&MaterialType::Sink)
                {
                    self.contents[(i, j.saturating_sub(1))].material_id = VOID.id;
                    self.contents[(i, j.saturating_sub(1))].display_color =
                        VOID.material_color.color;
                }
                if std::mem::discriminant(
                    &materials[self
                        .contents
                        .get(i.wrapping_add(1), j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .material_type,
                ) != std::mem::discriminant(&MaterialType::Sink)
                {
                    self.contents[(i.wrapping_add(1), j)].material_id = VOID.id;
                    self.contents[(i.wrapping_add(1), j)].display_color = VOID.material_color.color;
                }
                if std::mem::discriminant(
                    &materials[self
                        .contents
                        .get(i.saturating_sub(1), j)
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .material_type,
                ) != std::mem::discriminant(&MaterialType::Sink)
                {
                    self.contents[(i.saturating_sub(1), j)].material_id = VOID.id;
                    self.contents[(i.saturating_sub(1), j)].display_color =
                        VOID.material_color.color;
                }
            }
            _ => {}
        }
    }
}
