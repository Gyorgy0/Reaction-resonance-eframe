use std::i64;

use crate::chemistry::Material_Type;
use crate::physics::Phase;
use egui::Color32;
use egui::Vec2;
use rand;
use rand::Rng;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Material {
    pub name: String,                 // Name of the material
    pub density: f32,                 // Mass of a cm^3 volume of the material
    pub phase: Phase, // Phase of the material for, the implemented phases check the "Phase" enum
    pub material_type: Material_Type, // Type of the material for, the implemented types check the "Type" enum
    pub durability: i32, // Durability of a material - how much force it needs to disintegrate the material -> higher = more force
    pub color: Color32,  // Color of the material
}

#[derive(Clone)]
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
    // Returns only Option (it can be Some() or None()) - used for checking if we refer to a valid cell
    pub fn get_cell(&mut self, x: u16, y: u16) -> Option<&Particle> {
        self.contents
            .get((x as usize * self.width as usize) + y as usize)
    }
    // Returns a valid Particle
    pub fn get_particle(&mut self, x: u16, y: u16, fallback_position: usize) -> Particle {
        self.contents
            .get((x as usize * self.width as usize) + y as usize)
            .unwrap_or(&self.contents[fallback_position])
            .clone()
    }
}

#[inline(always)]
pub fn update_board(game_board: &mut Board, is_stopped: bool, frame: &mut u8) {
    let row_count = game_board.height as i32;
    let col_count: i32 = game_board.width as i32;
    let framedelta = 1.0 / 60.0;
    if !is_stopped {
        match *frame {
            0 => {
                (0..row_count * col_count).into_iter().for_each(|count| {
                    let i = count / col_count;
                    let j = count % col_count;
                    game_board.solve_particle(i, j, framedelta);
                    game_board.solve_reactions(i, j, framedelta);
                });
                *frame = 1;
            }
            1 => {
                (0..row_count * col_count).for_each(|count| {
                    let i = count / col_count;
                    let j = (col_count - 1) - (count % col_count);
                    game_board.solve_particle(i, j, framedelta);
                    game_board.solve_reactions(i, j, framedelta);
                });
                *frame = 0;
            }
            _ => {}
        }
    }
}
