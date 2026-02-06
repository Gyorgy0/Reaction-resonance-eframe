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
    pub durability: i32,                    // Durability of a material - how much force it needs to disintegrate the material -> higher = more force
    pub material_color: MaterialColor,      // Color of the material
}

pub fn tuple_to_rangeinclusive(range: (f32, f32)) -> RangeInclusive<f32> {
    RangeInclusive::new(range.0, range.1)
}

pub static VOID: Material = Material {
    id: 0,
    density: 0_f32,
    phase: Phase::Void,
    material_type: MaterialType::Solution,
    durability: -1,
    material_color: MaterialColor {
        color: Color32::from_rgba_premultiplied(0, 0, 0, 100),
        shinyness: (1_f32, 1_f32),
    },
};
