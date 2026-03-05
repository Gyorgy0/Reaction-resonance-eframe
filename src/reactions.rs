use crate::particle::{AtomicParticle, Particle};
use crate::world::get_safe_i;
use crate::{material::Material, world::AtomicComparedSlice};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum_macros::EnumIter;

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize, EnumIter)]
#[rustfmt::skip]
#[repr(u8)]
pub(crate) enum MaterialType {
    /// Corrosive material - everything with a pH value lower than 7.0
    ///                      everything with a pH value higher than 7.0
    Corrosive,
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
    // Amorphous material formed from a molten material and it's cooled without proper crystalization
    Glass,
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

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize, EnumIter)]
pub(crate) enum MachineTypes {
    Cloner,
    Sink,
}

impl Default for MachineTypes {
    fn default() -> Self {
        Self::Cloner
    }
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
    slice_board: &AtomicComparedSlice<Particle>,
    check_board: &Arc<Vec<AtomicParticle>>,
    prev_board: &Vec<Particle>,
    materials: &Vec<(String, Material)>,
    rngs: &Vec<f32>,
    seeds: &Vec<f32>,
    height: &usize,
    width: &usize,
    i: usize,
    j: usize,
    gravity: f32,
    framedelta: f32,
) {
    let neumann_positions = [
        (i.wrapping_add(1), j),
        (i.saturating_sub(1), j),
        (i, j.wrapping_add(1)),
        (i, j.saturating_sub(1)),
    ];
    let moore_positions = [
        (i.wrapping_add(1), j.wrapping_add(1)),
        (i.wrapping_add(1), j),
        (i.wrapping_add(1), j.saturating_sub(1)),
        (i.saturating_sub(1), j.wrapping_add(1)),
        (i.saturating_sub(1), j),
        (i.saturating_sub(1), j.saturating_sub(1)),
        (i, j.wrapping_add(1)),
        (i, j.saturating_sub(1)),
    ];
    match &materials[prev_board[get_safe_i(height, width, &(i, j))].material_id]
        .1
        .material_type
    {
        MaterialType::Fuel => {
            let rnd = rand::random_range(0_u8..4_u8);
        }
        MaterialType::Machine {
            machine: machine_type,
        } => match machine_type {
            MachineTypes::Cloner => {}
            MachineTypes::Sink => {}
        },
        MaterialType::Decor => {}
        _ => {}
    }
}
