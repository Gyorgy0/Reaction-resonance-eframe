use crate::material::{Material, VOID, tuple_to_rangeinclusive};
use crate::neighbour_reactions::solve_by_neighbours;
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
    /// Corrosive material - everything with a pH value lower than 7.0
    Acid,
    /// Mixture of metals
    Alloy,
    /// Corrosive material - everything with a pH value higher than 7.0
    Base,
    /// Cellular automaton material defined by 3 rules (survival, birth and life stages)
    /// - survival ruleset -> the rule is encoded by the number's binary format
    ///                      0- false   1 - true
    ///
    ///           Rule form: 1 1 1 1 _ 1 1 1 1 -> 255
    ///                      ^ ^ ^ ^   ^ ^ ^ ^
    ///                            . . .
    ///                      | | | - etc....
    ///                      | | - the cell lives through the new iteration if 2 of
    ///                      | |   it's neighbours are alive
    ///                      | - the cell lives through the new iteration if 1 of it's neighbour is alive
    /// - birth ruleset -> works just like the survival ruleset, but it specifies the
    ///   number of neighbour required for a cell to become "alive"
    /// - stages -> specifies how many life stages a cell has the 
    ///   cell "starts to age" from the first generation when it can't survive, every iteration the value
    ///   of the life stages decreases until zero, then the cell dies
    /// The neighbours are checked in a Moore neighbourhood pattern:
    /// 
    ///     |-----|-----|-----|
    ///     |  #  |  #  |  #  |
    ///     |-----|-----|-----|
    ///     |  #  |     |  #  |
    ///     |-----|-----|-----|
    ///     |  #  |  #  |  #  |
    ///     |-----|-----|-----|
    /// 
    CAutomata {survival: u8, birth:u8, stages: u8},
    // Hard, brittle, heat-resistant, and corrosion-resistant material
    Ceramic,
    // Material that clones the last new material it came in contact with
    Cloner,
    // A material that generates a lot of energy and lot of gases
    Explosive,
    // Flammable material under normal circumstances
    Fuel,
    // Amorphous material formed from a molten material and it's cooled without proper crystalization
    Glass,
    // This material can enhance the explosive power of
    // explosives or the burning of fuels by aiding their combustion
    Oxidizer,
    // This material is indestructible and completely inert it's used for 
    // decoration purposes, mainly pixelart, map making, etc...
    Decor,
    // This material swallows any material it comes in contact with it
    Sink,
    // Material that contains other materials e.g. salts, if heated it leaves the
    // dissolved materials behind
    Solution,
    // Dissolves certain materials
    Solvent,
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
    prev_board: &Vec<Particle>,
    board_rngs: &Vec<f32>,
    materials: &Vec<(String, Material)>,
    width: u16,
    i: usize,
    j: usize,
    _framedelta: f32,
    framecount: u64,
) -> Particle {
    let mut new_particle = solve_by_neighbours(prev_board, board_rngs, materials, width, i, j);
    /*match &materials[prev_board[(i, j)].material_id].1.material_type {
        MaterialType::Fuel => {
            let rnd = rand::random_range(0_u8..4_u8);
            if std::mem::discriminant(
                &materials[prev_board
                    .get(i, j.wrapping_add(1))
                    .unwrap_or(&prev_board[(i, j)])
                    .material_id]
                    .1
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma))
                && prev_board.get(i, j.wrapping_add(1)).is_some()
                && rnd == 0_u8
            {
                new_particle = prev_board[(i, j.wrapping_add(1))];
                new_particle.material_id = 7_usize;
                new_particle.energy = 20_f32;
                new_particle.display_color = materials[prev_board[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[prev_board[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        board_rngs[(i, j)],
                    ));
                new_particle.display_color[3] = materials[prev_board[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .a();
            } else if std::mem::discriminant(
                &materials[prev_board
                    .get(i, j.saturating_sub(1))
                    .unwrap_or(&prev_board[(i, j)])
                    .material_id]
                    .1
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma))
                && prev_board.get(i, j.saturating_sub(1)).is_some()
                && rnd == 1
            {
                new_particle.material_id = 7_usize;
                new_particle.energy = 20_f32;
                new_particle.display_color = materials[prev_board[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[prev_board[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        board_rngs[(i, j)],
                    ));
                new_particle.display_color[3] = materials[prev_board[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .a();
            } else if std::mem::discriminant(
                &materials[prev_board
                    .get(i.wrapping_add(1), j)
                    .unwrap_or(&prev_board[(i, j)])
                    .material_id]
                    .1
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma))
                && rnd == 2
            {
                new_particle.material_id = 7_usize;
                new_particle.energy = 20_f32;
                new_particle.display_color = materials[prev_board[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[prev_board[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        board_rngs[(i, j)],
                    ));
                new_particle.display_color[3] = materials[prev_board[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .a();
            } else if std::mem::discriminant(
                &materials[prev_board
                    .get(i.saturating_sub(1), j)
                    .unwrap_or(&prev_board[(i, j)])
                    .material_id]
                    .1
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma))
                && rnd == 3
            {
                new_particle.material_id = 7_usize;
                new_particle.energy = 20_f32;
                new_particle.display_color = materials[prev_board[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[prev_board[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        board_rngs[(i, j)],
                    ));
                new_particle.display_color[3] = materials[prev_board[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .a();
            }
        }
        MaterialType::Cloner => {
            if prev_board[(i, j)].cloned_material == 0_usize {
                if materials[prev_board
                    .get(i, j.wrapping_add(1))
                    .unwrap_or(&prev_board[(i, j)])
                    .material_id]
                    .1
                    != VOID
                    && materials[prev_board
                        .get(i, j.wrapping_add(1))
                        .unwrap_or(&prev_board[(i, j)])
                        .material_id]
                        .1
                        .material_type
                        != MaterialType::Cloner
                {
                    new_particle.cloned_material = prev_board[(i, j.wrapping_add(1))].material_id;
                } else if materials[prev_board
                    .get(i, j.saturating_sub(1))
                    .unwrap_or(&prev_board[(i, j)])
                    .material_id]
                    .1
                    != VOID
                    && materials[prev_board
                        .get(i, j.saturating_sub(1))
                        .unwrap_or(&prev_board[(i, j)])
                        .material_id]
                        .1
                        .material_type
                        != MaterialType::Cloner
                {
                    new_particle.cloned_material = prev_board[(i, j.saturating_sub(1))].material_id;
                } else if materials[prev_board
                    .get(i.wrapping_add(1), j)
                    .unwrap_or(&prev_board[(i, j)])
                    .material_id]
                    .1
                    != VOID
                    && materials[prev_board
                        .get(i.wrapping_add(1), j)
                        .unwrap_or(&prev_board[(i, j)])
                        .material_id]
                        .1
                        .material_type
                        != MaterialType::Cloner
                {
                    new_particle.cloned_material = prev_board[(i.wrapping_add(1), j)].material_id;
                } else if materials[prev_board
                    .get(i.saturating_sub(1), j)
                    .unwrap_or(&prev_board[(i, j)])
                    .material_id]
                    .1
                    != VOID
                    && materials[prev_board
                        .get(i.saturating_sub(1), j)
                        .unwrap_or(&prev_board[(i, j)])
                        .material_id]
                        .1
                        .material_type
                        != MaterialType::Cloner
                {
                    new_particle.cloned_material = prev_board[(i.saturating_sub(1), j)].material_id;
                }
            }
        }
        MaterialType::Decor => {
            if prev_board[(i, j)].display_color == Color32::from_rgba_unmultiplied(0, 0, 0, 0) {
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
                new_particle.display_color = prev_board[(i, j)].display_color.gamma_multiply(lerp(
                    tuple_to_rangeinclusive(
                        materials[prev_board[(i, j)].material_id]
                            .1
                            .material_color
                            .shinyness,
                    ),
                    board_rngs[(i, j)],
                ));
            }
        }
        _ => {}
    }*/
    new_particle
}
