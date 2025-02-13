use crate::chemistry::Material_Type;
use crate::physics::Phase;
use egui::Vec2;
use rand;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Material {
    pub name: & 'static str,                 // Name of the material
    pub density: f32,                 // Mass of a cm^3 volume of the material
    pub phase: Phase, // Phase of the material for, the implemented phases check the "Phase" enum
    pub material_type: Material_Type, // Type of the material for, the implemented types check the "Type" enum
    pub durability: i32, // Durability of a material - how much force it needs to disintegrate the material -> higher = more force
    pub color: color32_u8,  // Color of the material
}
#[derive(Clone)]
pub struct Particle {
    pub material: Material, // Material of the particle
    pub speed: Vec2,        // Vectors of the particle (x, y)
    pub temperature: f32,   // Temperature of the material
    pub updated: bool,      // Is it updated?
    pub seed: f32,          // Seed of particle
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct vec2_f32 {
    pub x: f32,
    pub y: f32,
}
impl vec2_f32 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

// Conversions:
// vec2_f32 -> egui::Vec2
impl From<vec2_f32> for Vec2 {
    #[inline(always)]
    fn from(v: vec2_f32) -> Self {
        Self { x: v.x, y: v.y }
    }
}

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct color32_u8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl color32_u8 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub const fn get_val(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

// Conversions:
// color32_u8 -> [u8;4]
impl From<color32_u8> for [u8;4] {
    #[inline(always)]
    fn from(v: color32_u8) -> Self {
        [v.r, v.g, v.b, v.a]
    }
}


#[derive(Clone)]
pub struct Board {
    pub width: u16,
    pub height: u16,
    pub contents: Vec<Particle>,
    pub gravity: f32,
    pub brushsize: i32,
    pub cellsize: vec2_f32,
}

pub static VOID:Material = Material {
        name: "Void",
        density: 0.0,
        phase: Phase::Void,
        material_type: Material_Type::Atmosphere,
        durability: -1,
        color: color32_u8::new(0, 0, 0, 100),
    };

impl Board {
    pub fn create_board(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.contents = vec![
            Particle {
                material: VOID,
                speed: egui::Vec2::from(vec2_f32::new(0.0, 0.0)),
                temperature: 20.0,
                updated: false,
                seed: rand::thread_rng().gen_range(0.0, 1.0),
            };
            (self.width as usize) * (self.height as usize)
        ];
    }
}
