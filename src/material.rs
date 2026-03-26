use std::ops::RangeInclusive;

use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::{physics::Phase, reactions::MaterialType, world::MaterialColor};

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
    id: 0,
    density: 0.0012041_f32,
    phase: Phase::Air,
    material_type: MaterialType::Oxidizer {
        combustion_speedup: 0.0_f32,
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
// Air temperature and it's density
pub static AIR_DENSITY_BY_TEMP: [(usize, f32); 32] = [
    (198_usize, 0.001783_f32),
    (223_usize, 0.001582),
    (248_usize, 0.001422),
    (258_usize, 0.001367),
    (263_usize, 0.001341),
    (268_usize, 0.001316),
    (273_usize, 0.001292),
    (5_usize + 273_usize, 0.001268),
    (10_usize + 273_usize, 0.001246),
    (15_usize + 273_usize, 0.001225),
    (20_usize + 273_usize, 0.001204),
    (25_usize + 273_usize, 0.001184),
    (30_usize + 273_usize, 0.001164),
    (40_usize + 273_usize, 0.001127),
    (50_usize + 273_usize, 0.001093),
    (60_usize + 273_usize, 0.001060),
    (80_usize + 273_usize, 0.001),
    (100_usize + 273_usize, 0.0009467),
    (125_usize + 273_usize, 0.0008868),
    (150_usize + 273_usize, 0.0008338),
    (175_usize + 273_usize, 0.0007868),
    (200_usize + 273_usize, 0.0007451),
    (225_usize + 273_usize, 0.0007078),
    (300_usize + 273_usize, 0.0006168),
    (400_usize + 273_usize, 0.0005238),
    (500_usize + 273_usize, 0.0004567),
    (600_usize + 273_usize, 0.0004043),
    (700_usize + 273_usize, 0.0003626),
    (800_usize + 273_usize, 0.0003289),
    (900_usize + 273_usize, 0.0003009),
    (1_000_usize + 273_usize, 0.0002773),
    (1_100_usize + 273_usize, 0.0002571),
];
