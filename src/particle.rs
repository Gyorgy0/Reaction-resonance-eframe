use std::sync::atomic::{AtomicBool, AtomicU8};

use egui::{Color32, Vec2};
use serde::{Deserialize, Serialize};

use crate::material::{AIR, Material};

#[rustfmt::skip]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Particle {
    pub material_id: usize,             // ID of material
    pub speed: Vec2,                    // Vectors of the particle (x, y)
    pub temperature: f32,               // Temperature of the particle
    pub cloned_material: usize,         // Cloned material for the Cloner material type
    pub life_stage: u8,                 // Life stage of the simulated cell (cellular automatons)
    pub updated: bool,                  // Is it updated?
    pub display_color: Color32,         // Displayed color
}

impl Particle {
    pub fn new(material: &Material, speed: Vec2, temperature: f32) -> Self {
        Self {
            material_id: material.id,
            speed,
            temperature,
            cloned_material: 0_usize,
            life_stage: 0_u8,
            updated: false,
            display_color: material.material_color.color,
        }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self::new(&AIR, Vec2::new(0_f32, 0_f32), 293.15_f32)
    }
}
#[rustfmt::skip]
pub struct AtomicParticle {
    pub physics_written: AtomicBool,    // Checks whether the AtomicParticle has been overwritten by a physics reaction
    pub life_written: AtomicBool,       // Checks whether the AtomicParticle has been overwritten by a life reaction
    pub reaction_written: AtomicBool,   // Checks whether the AtomicParticle has been overwritten by a chemical reaction
    pub temperature: AtomicBool,        // Checks whether the AtomicParticle's temperature has been overwritten
    pub speed_x: AtomicBool,            // Checks whether the AtomicParticle's speed's x componenet has been overwritten
    pub speed_y: AtomicBool,            // Checks whether the AtomicParticle's speed's y componenet has been overwritten
}

impl AtomicParticle {
    pub fn new(
        physics_written_x: bool,
        life_written: bool,
        reaction_written: bool,
        speed_x: bool,
        speed_y: bool,
        temperature: bool,
        _updated: bool,
    ) -> Self {
        Self {
            physics_written: AtomicBool::new(physics_written_x),
            life_written: AtomicBool::new(life_written),
            reaction_written: AtomicBool::new(reaction_written),
            speed_x: AtomicBool::new(speed_x),
            speed_y: AtomicBool::new(speed_y),
            temperature: AtomicBool::new(temperature),
        }
    }
}

impl Default for AtomicParticle {
    fn default() -> Self {
        Self::new(false, false, false, false, false, false, false)
    }
}
