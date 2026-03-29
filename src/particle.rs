use std::sync::atomic::{AtomicBool, AtomicU8};

use egui::{Color32, Vec2, lerp};
use serde::{Deserialize, Serialize};

use crate::material::{AIR, Material, tuple_to_rangeinclusive};

#[rustfmt::skip]
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Particle {
    pub material_id: usize,             // ID of material
    pub speed: Vec2,                    // Vectors of the particle (x, y)
    pub temperature: f32,               // Temperature of the particle
    pub cloned_material: usize,         // Cloned material for the Cloner material type
    pub particle_health: u16,           // "Health" of the simulated cell (does it need to exist?)
    pub burning: bool,                  // Is the particle burning?
    pub display_color: Color32,         // Displayed color
}

impl Particle {
    pub fn new(material: &Material, speed: Vec2, temperature: f32) -> Self {
        Self {
            material_id: material.id,
            speed,
            temperature,
            cloned_material: 0_usize,
            particle_health: 0_u16,
            burning: false,
            display_color: material.material_color.color,
        }
    }
    // Applies the material's color and shinyness to the particle's display color
    pub fn set_color(&mut self, materials: &[(String, Material)], noise_val: f32) -> Self {
        self.display_color = materials[self.material_id].1.material_color.color;
        self.display_color = self.display_color.gamma_multiply(lerp(
            tuple_to_rangeinclusive(materials[self.material_id].1.material_color.shinyness),
            noise_val,
        ));
        self.display_color[3] = materials[self.material_id].1.material_color.color.a();
        *self
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self::new(&AIR, Vec2::new(0_f32, 0_f32), 293.15_f32)
    }
}
#[rustfmt::skip]
pub struct AtomicParticle {
    pub physics_written: AtomicBool,        // Checks whether the AtomicParticle has been overwritten by a physics reaction
    pub life_written: AtomicBool,           // Checks whether the AtomicParticle has been overwritten by a life reaction
    pub reaction_written: AtomicBool,       // Checks whether the AtomicParticle has been overwritten by a chemical reaction
    pub temperature_write_count: AtomicU8,  // Checks how many times the temperature has been exchanged (max. 4 per particle)
    pub speed_x: AtomicBool,                // Checks whether the AtomicParticle's speed's x componenet has been overwritten
    pub speed_y: AtomicBool,                // Checks whether the AtomicParticle's speed's y componenet has been overwritten
}

impl AtomicParticle {
    pub fn new(
        physics_written_x: bool,
        life_written: bool,
        reaction_written: bool,
        thread_count: u8,
        speed_x: bool,
        speed_y: bool,
    ) -> Self {
        Self {
            physics_written: AtomicBool::new(physics_written_x),
            life_written: AtomicBool::new(life_written),
            reaction_written: AtomicBool::new(reaction_written),
            temperature_write_count: AtomicU8::new(thread_count),
            speed_x: AtomicBool::new(speed_x),
            speed_y: AtomicBool::new(speed_y),
        }
    }
}

impl Default for AtomicParticle {
    fn default() -> Self {
        Self::new(false, false, false, 0_u8, false, false)
    }
}
