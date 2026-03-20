use crate::material::tuple_to_rangeinclusive;
use crate::particle::{AtomicParticle, Particle};
use crate::physics::Phase;
use crate::world::{get_safe_i, write_particle};
use crate::{material::Material, world::AtomicComparedSlice};
use egui::epaint::Hsva;
use egui::{Color32, lerp};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum_macros::EnumIter;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, EnumIter)]
#[rustfmt::skip]
#[repr(u8)]
pub(crate) enum MaterialType {
    /// Corrosive material - everything with a pH value lower than 7.0
    ///                      everything with a pH value higher than 7.0
    Corrosive {ph_value: f32, blacklist: bool, material_list: Vec<usize>},
    /// Mixture of metals - on reaction with corrosive materials the corrosion resistant metals leave a powder-type material behind
    Alloy,
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
    // A material that generates a lot of energy and lot of gases
    Explosive,
    // Flammable material under normal circumstances
    Fuel,
    // Machines e.g. cloners, sinks, pumps, conveyor belts, etc...
    Machine {machine: MachineTypes},
    // Conductive materials, they react based on their reactivity series
    // They are capable of coloring flames 
    Metal,
    // This material can enhance the explosive power of
    // explosives or the burning of fuels by aiding their combustion
    Oxidizer,
    // This material is indestructible and completely inert it's used for 
    // decoration purposes, mainly pixelart, map making, etc...
    Decor,
    // Material that contains other materials e.g. salts, if heated it leaves the
    // dissolved materials behind
    // Also dissolves certain materials
    Solution,
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize, EnumIter, Default)]
pub(crate) enum MachineTypes {
    #[default]
    Cloner,
    Sink,
}

impl MaterialType {
    pub fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
    pub fn cautomata_default() -> Self {
        MaterialType::CAutomata {
            survival: u8::default(),
            birth: u8::default(),
            stages: u8::default(),
        }
    }
    pub fn machine_default() -> Self {
        MaterialType::Machine {
            machine: MachineTypes::default(),
        }
    }
    pub fn corrosive_default() -> Self {
        MaterialType::Corrosive {
            ph_value: f32::default(),
            blacklist: bool::default(),
            material_list: vec![],
        }
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

    pub fn get_max_stage(&self) -> u8 {
        let mut returnval: u8 = 0_u8;
        if let MaterialType::CAutomata {
            birth: _,
            survival: _,
            stages,
        } = self
        {
            returnval = *stages;
        };
        returnval
    }

    pub fn get_machine_type(&self) -> MachineTypes {
        let mut returnval: MachineTypes = MachineTypes::default();
        if let MaterialType::Machine { machine } = self {
            returnval = *machine;
        };
        returnval
    }
}
#[inline(always)]
pub(crate) fn solve_reactions(
    slice_board: &AtomicComparedSlice<Particle>,
    check_board: &Arc<Vec<AtomicParticle>>,
    prev_board: &Vec<Particle>,
    materials: &Vec<(String, Material)>,
    rngs: &Vec<f32>,
    _seeds: &Vec<f32>,
    height: &usize,
    width: &usize,
    i: usize,
    j: usize,
    framecount: u64,
) {
    let neumann_positions = [
        (i.wrapping_add(1), j),
        (i.saturating_sub(1), j),
        (i, j.wrapping_add(1)),
        (i, j.saturating_sub(1)),
    ];
    let _moore_positions = [
        (i.wrapping_add(1), j.wrapping_add(1)),
        (i.wrapping_add(1), j),
        (i.wrapping_add(1), j.saturating_sub(1)),
        (i.saturating_sub(1), j.wrapping_add(1)),
        (i.saturating_sub(1), j),
        (i.saturating_sub(1), j.saturating_sub(1)),
        (i, j.wrapping_add(1)),
        (i, j.saturating_sub(1)),
    ];
    let mut new_particle = *slice_board.get_elem(get_safe_i(height, width, &(i, j)));
    match &materials[prev_board[get_safe_i(height, width, &(i, j))].material_id]
        .1
        .material_type
    {
        MaterialType::Fuel => {
            let rnd = rand::random_range(0..4_u8);
            if std::mem::discriminant(
                &materials[prev_board
                    .get(get_safe_i(height, width, &neumann_positions[rnd as usize]))
                    .unwrap_or(&prev_board[get_safe_i(height, width, &(i, j))])
                    .material_id]
                    .1
                    .phase,
            ) == std::mem::discriminant(&Phase::plasma_default())
                && std::mem::discriminant(
                    &materials[prev_board
                        .get(get_safe_i(height, width, &(i, j)))
                        .unwrap_or(&prev_board[get_safe_i(height, width, &(i, j))])
                        .material_id]
                        .1
                        .phase,
                ) != std::mem::discriminant(&Phase::plasma_default())
                && prev_board
                    .get(get_safe_i(height, width, &neumann_positions[rnd as usize]))
                    .is_some()
            {
                new_particle = prev_board[get_safe_i(height, width, &(i, j))];
                new_particle.material_id = 7_usize;
                new_particle.display_color = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[new_particle.material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        rngs[get_safe_i(height, width, &(i, j))],
                    ));
                new_particle.display_color[3] = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .a();
                if !check_board[get_safe_i(height, width, &(i, j))]
                    .reaction_written
                    .load(std::sync::atomic::Ordering::Relaxed)
                {
                    unsafe {
                        write_particle(
                            slice_board,
                            get_safe_i(height, width, &(i, j)),
                            new_particle,
                            check_board,
                        )
                    };
                }
            }
        }
        MaterialType::Machine {
            machine: machine_type,
        } => match machine_type {
            MachineTypes::Cloner => {
                for pos in neumann_positions.into_iter() {
                    if new_particle.cloned_material == 0_usize
                        && new_particle.material_id
                            != prev_board
                                .get(get_safe_i(height, width, &pos))
                                .unwrap_or(&prev_board[get_safe_i(height, width, &(i, j))])
                                .material_id
                    {
                        new_particle.cloned_material =
                            prev_board[get_safe_i(height, width, &pos)].material_id;
                        unsafe {
                            write_particle(
                                slice_board,
                                get_safe_i(height, width, &(i, j)),
                                new_particle,
                                check_board,
                            )
                        };
                    } else if new_particle.cloned_material != 0_usize
                        && prev_board[get_safe_i(height, width, &pos)].material_id == 0_usize
                    {
                        new_particle.material_id =
                            prev_board[get_safe_i(height, width, &(i, j))].cloned_material;
                        new_particle.display_color =
                            materials[new_particle.material_id].1.material_color.color;
                        new_particle.display_color =
                            new_particle.display_color.gamma_multiply(lerp(
                                tuple_to_rangeinclusive(
                                    materials[new_particle.material_id]
                                        .1
                                        .material_color
                                        .shinyness,
                                ),
                                rngs[get_safe_i(height, width, &(i, j))],
                            ));
                        new_particle.display_color[3] = materials[new_particle.material_id]
                            .1
                            .material_color
                            .color
                            .a();
                        unsafe {
                            write_particle(
                                slice_board,
                                get_safe_i(height, width, &pos),
                                new_particle,
                                check_board,
                            )
                        };
                    }
                }
            }
            MachineTypes::Sink => {
                for pos in neumann_positions.into_iter() {
                    if materials[prev_board[get_safe_i(height, width, &pos)].material_id]
                        .1
                        .material_type
                        .get_machine_type()
                        != MachineTypes::Sink
                    {
                        new_particle = Particle::default();
                        unsafe {
                            write_particle(
                                slice_board,
                                get_safe_i(height, width, &pos),
                                new_particle,
                                check_board,
                            )
                        };
                    }
                }
            }
        },
        MaterialType::Decor => {
            if new_particle.display_color == Color32::from_rgba_unmultiplied(0, 0, 0, 0) {
                new_particle.display_color = Hsva::new(
                    ((framecount / 4) % (356)) as f32 / (356_f32),
                    1_f32,
                    1_f32,
                    1_f32,
                )
                .into();
                new_particle.display_color = new_particle.display_color.gamma_multiply(lerp(
                    tuple_to_rangeinclusive(
                        materials[prev_board[get_safe_i(height, width, &(i, j))].material_id]
                            .1
                            .material_color
                            .shinyness,
                    ),
                    rngs[get_safe_i(height, width, &(i, j))],
                ));
                unsafe {
                    write_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        new_particle,
                        check_board,
                    )
                };
            }
        }
        _ => {}
    }
}
