use std::sync::atomic::AtomicBool;

use egui::{Color32, Vec2};
use serde::{Deserialize, Serialize};

use crate::material::{Material, VOID};

#[rustfmt::skip]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Particle {
    pub material_id: usize,             // ID of material
    pub speed: Vec2,                    // Vectors of the particle (x, y)
    pub temperature: f32,               // Temperature of the particle
    pub cloned_material: usize,         // Cloned material for the Cloner material type
    pub life_stage: u8,                 // Life stage of the simulated cell (cellular automatons)
    pub energy: f32,                    // Energy for the Plasma phase
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
            energy: 0_f32,
            updated: false,
            display_color: material.material_color.color,
        }
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self::new(&VOID, Vec2::new(0_f32, 0_f32), 293.15_f32)
    }
}

pub struct AtomicParticle {
    pub written: AtomicBool, // Checks whether the AtomicParticle has been overwritten
    //pub material_id: AtomicBool,
    pub speed: AtomicBool,
    //pub temperature: AtomicBool,
    //pub cloned_material: AtomicBool,
    //pub life_stage: AtomicBool,
    //pub energy: AtomicBool,
    pub updated: AtomicBool,
    //pub display_color: AtomicBool,
}

impl AtomicParticle {
    pub fn new(
        written: bool,
        material_id: bool,
        speed: bool,
        temperature: bool,
        cloned_material: bool,
        life_stage: bool,
        energy: bool,
        updated: bool,
        display_color: bool,
    ) -> Self {
        Self {
            written: AtomicBool::new(written),
            //material_id: AtomicBool::new(material_id),
            speed: AtomicBool::new(speed),
            //temperature: AtomicBool::new(temperature),
            //cloned_material: AtomicBool::new(cloned_material),
            //life_stage: AtomicBool::new(life_stage),
            //energy: AtomicBool::new(energy),
            updated: AtomicBool::new(updated),
            //display_color: AtomicBool::new(display_color),
        }
    }
}

impl Default for AtomicParticle {
    fn default() -> Self {
        Self::new(
            false, false, false, false, false, false, false, false, false,
        )
    }
}
