use std::ops::RangeInclusive;

use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::{
    physics::Phase,
    reactions::{MaterialType, OxidizingAgent},
    world::MaterialColor,
};

#[rustfmt::skip]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub(crate) struct Material {
    pub id: usize,                          // ID of the material
    pub density: f32,                       // Mass of a cm^3 volume of the material
    pub phase: Phase,                       // Phase of the material for, the implemented phases check the "Phase" enum
    pub material_type: MaterialType,        // Type of the material for, the implemented types check the "Type" enum
    pub initial_temperature: f32,           // The initial temperature of the material
    pub heat_capacity: f32,                 // The material's heat capacity in J/(g*K)
    pub heat_conductivity: f32,             // The material's heat conductivity W/(cm*K)
    pub durability: i32,                    // Durability of a material - how much force it needs to disintegrate the material -> higher = more force
    pub material_color: MaterialColor,      // Color of the material
}

pub fn tuple_to_rangeinclusive(range: (f32, f32)) -> RangeInclusive<f32> {
    RangeInclusive::new(range.0, range.1)
}

pub static AIR: Material = Material {
    id: 0_usize,
    density: 0.0012041_f32,
    phase: Phase::Air,
    material_type: MaterialType::Oxidizer {
        oxidizing_agent: OxidizingAgent::Oxygen,
        combustion_speedup: 1.0_f32,
    },
    initial_temperature: 293.15_f32,
    heat_capacity: 1.006_f32,
    heat_conductivity: 0.00025_f32,
    durability: -1_i32,
    material_color: MaterialColor {
        color: Color32::from_rgba_premultiplied(0_u8, 0_u8, 0_u8, 100_u8),
        shinyness: (1_f32, 1_f32),
    },
};
// Material temperature and it's density
// 0.001204
pub static MATERIAL_DENSITY_BY_TEMP: [(usize, f32); 32] = [
    (198_usize, 1.48_f32),
    (223_usize, 1.31_f32),
    (248_usize, 1.18_f32),
    (258_usize, 1.14_f32),
    (263_usize, 1.11_f32),
    (268_usize, 1.09_f32),
    (273_usize, 1.07_f32),
    (278_usize, 1.05_f32),
    (283_usize, 1.03_f32),
    (288_usize, 1.02_f32),
    (293_usize, 1_f32),
    (298_usize, 0.98_f32),
    (303_usize, 0.97_f32),
    (313_usize, 0.94_f32),
    (323_usize, 0.91_f32),
    (333_usize, 0.88_f32),
    (353_usize, 0.83_f32),
    (373_usize, 0.79_f32),
    (398_usize, 0.74_f32),
    (423_usize, 0.69_f32),
    (448_usize, 0.65_f32),
    (473_usize, 0.62_f32),
    (498_usize, 0.59_f32),
    (573_usize, 0.51_f32),
    (673_usize, 0.44_f32),
    (773_usize, 0.38_f32),
    (873_usize, 0.34_f32),
    (973_usize, 0.3_f32),
    (1_073_usize, 0.27_f32),
    (1_173_usize, 0.25_f32),
    (1_273_usize, 0.23_f32),
    (1_373_usize, 0.21_f32),
];
