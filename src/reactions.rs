use crate::material::{Material, VOID, tuple_to_rangeinclusive};
use crate::particle::Particle;
use crate::physics::Phase;
use crate::world::Board;
use egui::Color32;
use egui::epaint::Hsva;
use egui::lerp;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize, EnumIter)]
#[rustfmt::skip]
#[repr(u8)]
pub(crate) enum MaterialType {
    Acid,       // Corrosive material - everything with a pH value lower than 7.0
    Alloy,      // Mixture of metals
    Base,       // Corrosive material - everything with a pH value higher than 7.0
    CAutomata {survival: u8, birth:u8, stages: u8},  // Cellular automaton material defined by 4 rules (birth, survival, neighborhood and life stages)
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
    pub fn get_birth(&self) -> u8 {
        let mut returnval: u8 = 0_u8;
        if let MaterialType::CAutomata {
            birth,
            survival: _,
            stages: _,
        } = self
        {
            returnval = *birth;
        };
        returnval
    }

    pub fn get_survival(&self) -> u8 {
        let mut returnval: u8 = 0_u8;
        if let MaterialType::CAutomata {
            birth: _,
            survival,
            stages: _,
        } = self
        {
            returnval = *survival;
        };
        returnval
    }
}
#[inline(always)]
pub(crate) fn solve_reactions(
    prev_board: &Board,
    materials: &Vec<(String, Material)>,
    i: usize,
    j: usize,
    _framedelta: f32,
    framecount: u64,
) -> Particle {
    let mut new_particle = prev_board.contents[(i, j)];
    match &materials[prev_board.contents[(i, j)].material_id]
        .1
        .material_type
    {
        MaterialType::Fuel => {
            let rnd = rand::random_range(0_u8..4_u8);
            if std::mem::discriminant(
                &materials[prev_board
                    .contents
                    .get(i, j.wrapping_add(1))
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma))
                && prev_board.contents.get(i, j.wrapping_add(1)).is_some()
                && rnd == 0_u8
            {
                new_particle = prev_board.contents[(i, j.wrapping_add(1))];
                new_particle.material_id = 7_usize;
                new_particle.energy = 20_f32;
                new_particle.display_color = materials[prev_board.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[prev_board.contents[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        prev_board.rngs[(i, j)],
                    ));
                new_particle.display_color[3] = materials[prev_board.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .a();
            } else if std::mem::discriminant(
                &materials[prev_board
                    .contents
                    .get(i, j.saturating_sub(1))
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma))
                && prev_board.contents.get(i, j.saturating_sub(1)).is_some()
                && rnd == 1
            {
                new_particle.material_id = 7_usize;
                new_particle.energy = 20_f32;
                new_particle.display_color = materials[prev_board.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[prev_board.contents[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        prev_board.rngs[(i, j)],
                    ));
                new_particle.display_color[3] = materials[prev_board.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .a();
            } else if std::mem::discriminant(
                &materials[prev_board
                    .contents
                    .get(i.wrapping_add(1), j)
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma))
                && rnd == 2
            {
                new_particle.material_id = 7_usize;
                new_particle.energy = 20_f32;
                new_particle.display_color = materials[prev_board.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[prev_board.contents[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        prev_board.rngs[(i, j)],
                    ));
                new_particle.display_color[3] = materials[prev_board.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .a();
            } else if std::mem::discriminant(
                &materials[prev_board
                    .contents
                    .get(i.saturating_sub(1), j)
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma))
                && rnd == 3
            {
                new_particle.material_id = 7_usize;
                new_particle.energy = 20_f32;
                new_particle.display_color = materials[prev_board.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[prev_board.contents[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        prev_board.rngs[(i, j)],
                    ));
                new_particle.display_color[3] = materials[prev_board.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .a();
            }
            new_particle
        }
        /*MaterialType::Cloner => {
            if prev_board.contents[(i, j)].cloned_material == 0_usize {
                if materials[prev_board
                    .contents
                    .get(i, j.wrapping_add(1))
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    != VOID
                    && materials[prev_board
                        .contents
                        .get(i, j.wrapping_add(1))
                        .unwrap_or(&prev_board.contents[(i, j)])
                        .material_id]
                        .1
                        .material_type
                        != MaterialType::Cloner
                {
                    new_particle.cloned_material =
                        prev_board.contents[(i, j.wrapping_add(1))].material_id;
                } else if materials[prev_board
                    .contents
                    .get(i, j.saturating_sub(1))
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    != VOID
                    && materials[prev_board
                        .contents
                        .get(i, j.saturating_sub(1))
                        .unwrap_or(&prev_board.contents[(i, j)])
                        .material_id]
                        .1
                        .material_type
                        != MaterialType::Cloner
                {
                    new_particle.cloned_material =
                        prev_board.contents[(i, j.saturating_sub(1))].material_id;
                } else if materials[prev_board
                    .contents
                    .get(i.wrapping_add(1), j)
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    != VOID
                    && materials[prev_board
                        .contents
                        .get(i.wrapping_add(1), j)
                        .unwrap_or(&prev_board.contents[(i, j)])
                        .material_id]
                        .1
                        .material_type
                        != MaterialType::Cloner
                {
                    new_particle.cloned_material =
                        prev_board.contents[(i.wrapping_add(1), j)].material_id;
                } else if materials[prev_board
                    .contents
                    .get(i.saturating_sub(1), j)
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    != VOID
                    && materials[prev_board
                        .contents
                        .get(i.saturating_sub(1), j)
                        .unwrap_or(&prev_board.contents[(i, j)])
                        .material_id]
                        .1
                        .material_type
                        != MaterialType::Cloner
                {
                    new_particle.cloned_material =
                        prev_board.contents[(i.saturating_sub(1), j)].material_id;
                }
            } else if prev_board.contents[(i, j)].cloned_material != 0_usize {
                let material = prev_board.contents[(i, j)].cloned_material;
                if prev_board
                    .contents
                    .get(i, j.wrapping_add(1))
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id
                    == 0_usize
                {
                    prev_board.contents[(i, j.wrapping_add(1))].material_id = material;
                    prev_board.contents[(i, j.wrapping_add(1))].display_color =
                        materials[material].1.material_color.color;
                    prev_board.contents[(i, j.wrapping_add(1))].display_color = prev_board.contents
                        [(i, j.wrapping_add(1))]
                        .display_color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials[prev_board.contents[(i, j.wrapping_add(1))].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            prev_board.rngs[(i, j.wrapping_add(1))],
                        ));
                    prev_board.contents[(i, j.wrapping_add(1))].display_color[3] = materials
                        [prev_board.contents[(i, j.wrapping_add(1))].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                }
                if prev_board
                    .contents
                    .get(i, j.saturating_sub(1))
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id
                    == 0_usize
                {
                    prev_board.contents[(i, j.saturating_sub(1))].material_id = material;
                    prev_board.contents[(i, j.saturating_sub(1))].display_color =
                        materials[material].1.material_color.color;
                    prev_board.contents[(i, j.saturating_sub(1))].display_color = prev_board
                        .contents[(i, j.saturating_sub(1))]
                        .display_color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials
                                    [prev_board.contents[(i, j.saturating_sub(1))].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            prev_board.rngs[(i, j.saturating_sub(1))],
                        ));
                    prev_board.contents[(i, j.saturating_sub(1))].display_color[3] = materials
                        [prev_board.contents[(i, j.saturating_sub(1))].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                }
                if prev_board
                    .contents
                    .get(i.wrapping_add(1), j)
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id
                    == 0_usize
                {
                    prev_board.contents[(i.wrapping_add(1), j)].material_id = material;
                    prev_board.contents[(i.wrapping_add(1), j)].display_color =
                        materials[material].1.material_color.color;
                    prev_board.contents[(i.wrapping_add(1), j)].display_color = prev_board.contents
                        [(i.wrapping_add(1), j)]
                        .display_color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials[prev_board.contents[(i.wrapping_add(1), j)].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            prev_board.rngs[(i.wrapping_add(1), j)],
                        ));
                    prev_board.contents[(i.wrapping_add(1), j)].display_color[3] = materials
                        [prev_board.contents[(i.wrapping_add(1), j)].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                }
                if prev_board
                    .contents
                    .get(i.saturating_sub(1), j)
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id
                    == 0_usize
                {
                    prev_board.contents[(i.saturating_sub(1), j)].material_id = material;
                    prev_board.contents[(i.saturating_sub(1), j)].display_color =
                        materials[material].1.material_color.color;
                    prev_board.contents[(i.saturating_sub(1), j)].display_color = prev_board
                        .contents[(i.saturating_sub(1), j)]
                        .display_color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials
                                    [prev_board.contents[(i.saturating_sub(1), j)].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            prev_board.rngs[(i.saturating_sub(1), j)],
                        ));
                    prev_board.contents[(i.saturating_sub(1), j)].display_color[3] = materials
                        [prev_board.contents[(i.saturating_sub(1), j)].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                }
            }
        }*/
        MaterialType::Decor => {
            if prev_board.contents[(i, j)].display_color
                == Color32::from_rgba_unmultiplied(0, 0, 0, 0)
            {
                new_particle.display_color = Hsva::new(
                    ((framecount / 4) % (355)) as f32 / (355_f32),
                    1_f32,
                    1_f32,
                    1_f32,
                )
                .into();
                new_particle.display_color = Hsva::new(
                    ((framecount / 4) % (355)) as f32 / (355_f32),
                    1_f32,
                    1_f32,
                    1_f32,
                )
                .into();
                new_particle.display_color = prev_board.contents[(i, j)]
                    .display_color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[prev_board.contents[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        prev_board.rngs[(i, j)],
                    ));
            }
            new_particle
        }
        /*MaterialType::Sink => {
            if std::mem::discriminant(
                &materials[prev_board
                    .contents
                    .get(i, j.wrapping_add(1))
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    .material_type,
            ) != std::mem::discriminant(&MaterialType::Sink)
            {
                prev_board.contents[(i, j.wrapping_add(1))].material_id = VOID.id;
                prev_board.contents[(i, j.wrapping_add(1))].display_color =
                    VOID.material_color.color;
            }
            if std::mem::discriminant(
                &materials[prev_board
                    .contents
                    .get(i, j.saturating_sub(1))
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    .material_type,
            ) != std::mem::discriminant(&MaterialType::Sink)
            {
                prev_board.contents[(i, j.saturating_sub(1))].material_id = VOID.id;
                prev_board.contents[(i, j.saturating_sub(1))].display_color =
                    VOID.material_color.color;
            }
            if std::mem::discriminant(
                &materials[prev_board
                    .contents
                    .get(i.wrapping_add(1), j)
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    .material_type,
            ) != std::mem::discriminant(&MaterialType::Sink)
            {
                prev_board.contents[(i.wrapping_add(1), j)].material_id = VOID.id;
                prev_board.contents[(i.wrapping_add(1), j)].display_color =
                    VOID.material_color.color;
            }
            if std::mem::discriminant(
                &materials[prev_board
                    .contents
                    .get(i.saturating_sub(1), j)
                    .unwrap_or(&prev_board.contents[(i, j)])
                    .material_id]
                    .1
                    .material_type,
            ) != std::mem::discriminant(&MaterialType::Sink)
            {
                prev_board.contents[(i.saturating_sub(1), j)].material_id = VOID.id;
                prev_board.contents[(i.saturating_sub(1), j)].display_color =
                    VOID.material_color.color;
            }
        }*/
        _ => new_particle,
    }
}
