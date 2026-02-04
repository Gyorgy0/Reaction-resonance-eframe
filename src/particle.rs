use egui::{Color32, Vec2};
use serde::{Deserialize, Serialize};

use crate::material::{Material, VOID};

#[rustfmt::skip]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Particle {
    pub material_id: usize,             // ID of material
    pub speed: Vec2,                    // Vectors of the particle (x, y)
    pub temperature: f32,               // Temperature of the particle
    pub storage: Vec<(f32,usize)>,      // Storage for the particles (used in Solution and Alloy material types)
    pub cloned_material: usize,         // Cloned material for the Cloner material type
    pub updated: bool,                  // Is it updated?
    pub display_color: Color32,         // Displayed color
}
impl Particle {
    pub fn new(material: &Material, speed: Vec2, temperature: f32) -> Self {
        Self {
            material_id: material.id,
            speed,
            temperature,
            storage: vec![],
            cloned_material: 0_usize,
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
