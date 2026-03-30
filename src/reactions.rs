use crate::material::tuple_to_rangeinclusive;
use crate::particle::{AtomicParticle, Particle};
use crate::physics::Phase;
use crate::world::{get_safe_i, write_particle};
use crate::{material::Material, world::AtomicComparedSlice};
use egui::ahash::AHashMap;
use egui::epaint::Hsva;
use egui::{Color32, lerp};
use serde::{Deserialize, Serialize};
use std::mem::discriminant;
use std::sync::Arc;
use strum_macros::EnumIter;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, EnumIter)]
#[rustfmt::skip]
#[repr(u8)]
pub(crate) enum MaterialType {
    /// Corrosive material - everything with a pH value lower than 7.0
    ///                      everything with a pH value higher than 7.0
    Corrosive {ph_value: f32,blacklist: bool, material_list: Vec<usize>},
    /// Mixture of metals - on reaction with corrosive materials the corrosion resistant metals leave a powder-type material behind
    Alloy {metals: Vec<(usize, f32)>},
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
    CAutomata {survival: u8, birth:u8, stages: u16},
    // Hard, brittle, heat-resistant, and corrosion-resistant material
    Ceramic {chemical_resistance: f32},
    // A material that generates a lot of energy and lot of gases
    Explosive {ignition_temperature: f32, explosion_power: f32, flame_temperature: f32},
    // Flammable material under normal circumstances
    Fuel {burn_time: u16, ignition_temperature: f32, flame_temperature: f32},
    // Machines e.g. cloners, sinks, pumps, conveyor belts, etc...
    Machine {machine: MachineTypes},
    // Conductive materials, they react based on their reactivity series
    // They are capable of coloring flames 
    Metal {reactivity: MetalReactivity},
    // This material can enhance the explosive power of
    // explosives or the burning of fuels by aiding their combustion
    Oxidizer {oxidizing_agent: OxidizingAgent, combustion_speedup: f32},
    // This material is indestructible and completely inert it's used for 
    // decoration purposes, mainly pixelart, map making, etc...
    Decor,
    // Material that contains other materials e.g. salts, if heated it leaves the
    // dissolved materials behind
    // Also dissolves certain materials
    Solution,
}

pub struct ChemicalReactions {
    pub(crate) burning: AHashMap<usize, Vec<(usize, f32)>>,
    pub(crate) mingling: AHashMap<String, Vec<(usize, f32)>>,
}
impl ChemicalReactions {
    pub(crate) fn new(
        burning: AHashMap<usize, Vec<(usize, f32)>>,
        mingling: AHashMap<String, Vec<(usize, f32)>>,
    ) -> Self {
        ChemicalReactions { burning, mingling }
    }
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize, EnumIter, Default)]
pub(crate) enum MachineTypes {
    #[default]
    Cloner,
    Sink,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, EnumIter, Default)]
pub(crate) enum OxidizingAgent {
    None,
    #[default]
    Oxygen,
    Fluorine,
    Chlorine,
    Bromine,
    Iodine,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, EnumIter, Default)]
pub(crate) enum MetalReactivity {
    Air,
    Water,
    #[default]
    WeakAcids,
    StrongAcids,
    AquaRegia,
}

impl MaterialType {
    pub fn discriminant(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }
    pub fn cautomata_default() -> Self {
        MaterialType::CAutomata {
            survival: u8::default(),
            birth: u8::default(),
            stages: u16::default(),
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
    pub fn fuel_default() -> Self {
        MaterialType::Fuel {
            burn_time: u16::default(),
            ignition_temperature: f32::default(),
            flame_temperature: f32::default(),
        }
    }
    pub fn explosive_default() -> Self {
        MaterialType::Explosive {
            ignition_temperature: f32::default(),
            explosion_power: f32::default(),
            flame_temperature: f32::default(),
        }
    }

    pub(crate) fn alloy_default() -> Self {
        MaterialType::Alloy { metals: vec![] }
    }

    pub(crate) fn metal_default() -> Self {
        MaterialType::Metal {
            reactivity: MetalReactivity::default(),
        }
    }

    pub(crate) fn oxidizer_default() -> Self {
        MaterialType::Oxidizer {
            oxidizing_agent: OxidizingAgent::default(),
            combustion_speedup: f32::default(),
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

    pub fn get_max_stage(&self) -> u16 {
        let mut returnval: u16 = 0_u16;
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
    prev_board: &[Particle],
    materials: &[(String, Material)],
    chemical_reactions: &ChemicalReactions,
    rngs: &[f32],
    _seeds: &[f32],
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
    let mut current_particle = *slice_board.get_elem(get_safe_i(height, width, &(i, j)));
    let current_rng = rngs.get(get_safe_i(height, width, &(i, j))).unwrap().abs();
    match &materials[prev_board[get_safe_i(height, width, &(i, j))].material_id]
        .1
        .material_type
    {
        MaterialType::Fuel {
            burn_time,
            ignition_temperature,
            flame_temperature,
        } => {
            if current_particle.burning && current_particle.particle_health > 0_u16 {
                current_particle.particle_health -= 1_u16;
            } else if current_particle.burning && current_particle.particle_health == 0_u16 {
                current_particle.burning = false;
                if chemical_reactions
                    .burning
                    .get(&current_particle.material_id)
                    .is_some()
                {
                    for products in chemical_reactions
                        .burning
                        .get(&current_particle.material_id)
                        .unwrap()
                    {
                        if current_rng > (1_f32 - products.1) {
                            current_particle.material_id = products.0;
                            current_particle.temperature = *flame_temperature;
                            current_particle.display_color =
                                materials[products.0].1.material_color.color;
                            break;
                        }
                    }
                }
            }
            for pos in neumann_positions {
                if slice_board.get(get_safe_i(height, width, &pos)).is_some() {
                    let mut checked_particle =
                        *slice_board.get(get_safe_i(height, width, &pos)).unwrap();
                    let rng = rngs.get(get_safe_i(height, width, &pos)).unwrap().abs();
                    if checked_particle.temperature > *ignition_temperature
                        && !current_particle.burning
                        && materials[checked_particle.material_id].1.phase == Phase::Air
                    {
                        current_particle.burning = true;
                        current_particle.particle_health = *burn_time;
                    }
                    if discriminant(&materials[checked_particle.material_id].1.phase)
                        == discriminant(&Phase::Air)
                        && current_particle.burning
                        && current_particle.particle_health > 0_u16
                        && chemical_reactions
                            .burning
                            .get(&current_particle.material_id)
                            .is_some()
                    {
                        for products in chemical_reactions
                            .burning
                            .get(&current_particle.material_id)
                            .unwrap()
                        {
                            if rng > (1_f32 - products.1) {
                                checked_particle.material_id = products.0;
                                checked_particle.temperature = *flame_temperature;
                                checked_particle
                                    .set_color(materials, _seeds[get_safe_i(height, width, &pos)]);
                                unsafe {
                                    write_particle(
                                        slice_board,
                                        get_safe_i(height, width, &pos),
                                        checked_particle,
                                        check_board,
                                    )
                                };
                                break;
                            }
                        }
                    }
                }
            }
            current_particle.set_color(materials, _seeds[get_safe_i(height, width, &(i, j))]);
            unsafe {
                write_particle(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    current_particle,
                    check_board,
                )
            }
        }
        MaterialType::Explosive {
            ignition_temperature,
            explosion_power,
            flame_temperature,
        } => {
            if current_particle.temperature >= *ignition_temperature {
                for y in -explosion_power.floor().abs() as i32..explosion_power.ceil().abs() as i32
                {
                    for x in
                        -explosion_power.floor().abs() as i32..explosion_power.ceil().abs() as i32
                    {
                        if (x.abs().pow(2_u32) + y.abs().pow(2_u32))
                            <= explosion_power.abs().powi(2_i32) as i32
                        {
                            let pos = (i + y as usize, j + x as usize);
                            if slice_board.get(get_safe_i(height, width, &pos)).is_some() {
                                let mut checked_particle =
                                    *slice_board.get(get_safe_i(height, width, &pos)).unwrap();
                                let rng = rngs.get(get_safe_i(height, width, &pos)).unwrap().abs();
                                if checked_particle.temperature > *ignition_temperature {
                                    current_particle.burning = true;
                                }
                                if chemical_reactions
                                    .burning
                                    .get(&current_particle.material_id)
                                    .is_some()
                                {
                                    for products in chemical_reactions
                                        .burning
                                        .get(&current_particle.material_id)
                                        .unwrap()
                                    {
                                        if rng > (1_f32 - products.1) {
                                            checked_particle.material_id = products.0;
                                            checked_particle.temperature = *flame_temperature;
                                            checked_particle.set_color(
                                                materials,
                                                _seeds[get_safe_i(height, width, &pos)],
                                            );
                                            unsafe {
                                                write_particle(
                                                    slice_board,
                                                    get_safe_i(height, width, &pos),
                                                    checked_particle,
                                                    check_board,
                                                )
                                            };
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        MaterialType::Machine {
            machine: machine_type,
        } => match machine_type {
            MachineTypes::Cloner => {
                for pos in neumann_positions.into_iter() {
                    if current_particle.cloned_material == 0_usize
                        && current_particle.material_id
                            != prev_board
                                .get(get_safe_i(height, width, &pos))
                                .unwrap_or(&prev_board[get_safe_i(height, width, &(i, j))])
                                .material_id
                    {
                        current_particle.cloned_material =
                            prev_board[get_safe_i(height, width, &pos)].material_id;
                        unsafe {
                            write_particle(
                                slice_board,
                                get_safe_i(height, width, &(i, j)),
                                current_particle,
                                check_board,
                            )
                        };
                    } else if current_particle.cloned_material != 0_usize
                        && prev_board[get_safe_i(height, width, &pos)].material_id == 0_usize
                    {
                        current_particle.material_id =
                            prev_board[get_safe_i(height, width, &(i, j))].cloned_material;
                        current_particle
                            .set_color(materials, _seeds[get_safe_i(height, width, &pos)]);
                        unsafe {
                            write_particle(
                                slice_board,
                                get_safe_i(height, width, &pos),
                                current_particle,
                                check_board,
                            )
                        };
                    }
                }
            }
            MachineTypes::Sink => {
                for pos in neumann_positions.into_iter() {
                    if slice_board.get(get_safe_i(height, width, &pos)).is_some()
                        && materials[prev_board[get_safe_i(height, width, &pos)].material_id]
                            .1
                            .material_type
                            .get_machine_type()
                            != MachineTypes::Sink
                    {
                        current_particle = Particle::default();
                        unsafe {
                            write_particle(
                                slice_board,
                                get_safe_i(height, width, &pos),
                                current_particle,
                                check_board,
                            )
                        };
                    }
                }
            }
        },
        MaterialType::Decor => {
            if current_particle.display_color == Color32::from_rgba_unmultiplied(0, 0, 0, 0) {
                current_particle.display_color = Hsva::new(
                    ((framecount / 4) % (356)) as f32 / (356_f32),
                    1_f32,
                    1_f32,
                    1_f32,
                )
                .into();
                current_particle.display_color =
                    current_particle.display_color.gamma_multiply(lerp(
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
                        current_particle,
                        check_board,
                    )
                };
            }
        }
        _ => {
            for pos in neumann_positions {
                let checked_particle = slice_board.get(get_safe_i(height, width, &pos));
                if checked_particle.is_some() {
                    let mut neighboring_particle = *checked_particle.unwrap();
                    let current_rng = rngs[get_safe_i(height, width, &(i, j))];
                    let rng = rngs[get_safe_i(height, width, &pos)];
                    let checked_id_pair = (
                        current_particle.material_id,
                        neighboring_particle.material_id,
                    );
                    let checked_reverse_pair = (checked_id_pair.1, checked_id_pair.0);

                    if chemical_reactions
                        .mingling
                        .get(&format!("{:?}", checked_id_pair))
                        .is_some()
                    {
                        for products in chemical_reactions
                            .mingling
                            .get(&format!("{:?}", checked_id_pair))
                            .unwrap()
                        {
                            if rng > (1_f32 - products.1) {
                                neighboring_particle.material_id = products.0;
                                current_particle.temperature =
                                    materials[products.0].1.initial_temperature;
                                neighboring_particle
                                    .set_color(materials, _seeds[get_safe_i(height, width, &pos)]);
                                unsafe {
                                    write_particle(
                                        slice_board,
                                        get_safe_i(height, width, &pos),
                                        neighboring_particle,
                                        check_board,
                                    )
                                };
                                break;
                            }
                        }
                        for products in chemical_reactions
                            .mingling
                            .get(&format!("{:?}", checked_id_pair))
                            .unwrap()
                        {
                            if current_rng > (1_f32 - products.1) {
                                current_particle.material_id = products.0;
                                current_particle.temperature =
                                    materials[products.0].1.initial_temperature;
                                current_particle
                                    .set_color(materials, _seeds[get_safe_i(height, width, &pos)]);
                                unsafe {
                                    write_particle(
                                        slice_board,
                                        get_safe_i(height, width, &(i, j)),
                                        current_particle,
                                        check_board,
                                    )
                                };
                                break;
                            }
                        }
                    } else if chemical_reactions
                        .mingling
                        .get(&format!("{:?}", checked_reverse_pair))
                        .is_some()
                    {
                        for products in chemical_reactions
                            .mingling
                            .get(&format!("{:?}", checked_reverse_pair))
                            .unwrap()
                        {
                            if rng > (1_f32 - products.1) {
                                neighboring_particle.material_id = products.0;
                                current_particle.temperature =
                                    materials[products.0].1.initial_temperature;
                                neighboring_particle
                                    .set_color(materials, _seeds[get_safe_i(height, width, &pos)]);
                                unsafe {
                                    write_particle(
                                        slice_board,
                                        get_safe_i(height, width, &pos),
                                        neighboring_particle,
                                        check_board,
                                    )
                                };
                                break;
                            }
                        }
                        for products in chemical_reactions
                            .mingling
                            .get(&format!("{:?}", checked_reverse_pair))
                            .unwrap()
                        {
                            if current_rng > (1_f32 - products.1) {
                                current_particle.material_id = products.0;
                                current_particle.temperature =
                                    materials[products.0].1.initial_temperature;
                                current_particle
                                    .set_color(materials, _seeds[get_safe_i(height, width, &pos)]);
                                unsafe {
                                    write_particle(
                                        slice_board,
                                        get_safe_i(height, width, &(i, j)),
                                        current_particle,
                                        check_board,
                                    )
                                };
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
