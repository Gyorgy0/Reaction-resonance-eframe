use crate::chemistry::Material_Type;
use crate::physics::Phase;
use egui::Color32;
use egui::Vec2;
use serde::Deserialize;
use serde::Serialize;
use xorshift::Xoroshiro128;
use xorshift::Xorshift1024;
use xorshift::Xorshift128;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub(crate) struct Material {
    pub name: String,                 // Name of the material
    pub density: f32,                 // Mass of a cm^3 volume of the material
    pub phase: Phase, // Phase of the material for, the implemented phases check the "Phase" enum
    pub material_type: Material_Type, // Type of the material for, the implemented types check the "Type" enum
    pub durability: i32, // Durability of a material - how much force it needs to disintegrate the material -> higher = more force
    pub color: Color32,  // Color of the material
}

#[derive(Clone, Debug)]
pub struct Particle {
    pub material: Material, // Material of the particle
    pub speed: Vec2,        // Vectors of the particle (x, y)
    pub temperature: f32,   // Temperature of the material
    pub updated: bool,      // Is it updated?
    pub seed: f32,          // Seed of particle
}

#[derive(Clone)]
pub struct Board {
    pub width: u16,
    pub height: u16,
    pub contents: Vec<Particle>,
    pub gravity: f32,
    pub brushsize: i32,
    pub cellsize: Vec2,
}

pub static VOID: Material = Material {
    name: String::new(),
    density: 0.0,
    phase: Phase::Void,
    material_type: Material_Type::Atmosphere,
    durability: -1,
    color: Color32::from_rgba_premultiplied(0, 0, 0, 100),
};

impl Board {
    pub fn create_board(&mut self) {
        self.contents = vec![
            Particle {
                material: VOID.clone(),
                speed: Vec2::new(0.0, 0.0),
                temperature: 20.0,
                updated: false,
                seed: rand::random_range(0.0..1.0),
            };
            (self.width as usize) * (self.height as usize)
        ];
    }
}

#[inline(always)]
pub fn update_board(
    game_board: &mut Board,
    is_stopped: bool,
    frame: &mut u8,
    framedelta: f32,
    rng: &mut Xorshift128,
) {
    let row_count = game_board.height as i32;
    let col_count: i32 = game_board.width as i32;
    if !is_stopped {
        match *frame {
            0 => {
                (0..row_count * col_count).for_each(|count| {
                    let i = count / col_count;
                    let j = count % col_count;
                    game_board.solve_particle(i, j, framedelta, rng);
                    game_board.solve_reactions(i, j, framedelta, rng);
                });
                *frame = 1;
            }
            1 => {
                (0..row_count * col_count).for_each(|count| {
                    let i = count / col_count;
                    let j = (col_count - 1) - (count % col_count);
                    game_board.solve_particle(i, j, framedelta, rng);
                    game_board.solve_reactions(i, j, framedelta, rng);
                });
                *frame = 0;
            }
            _ => {}
        }
    }
}
