use crate::chemistry::Material_Type;
use crate::physics::Phase;
use egui::Color32;
use egui::Vec2;
use rand::distr::Distribution;
use rand::distr::Uniform;
use serde::Deserialize;
use serde::Serialize;

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
}

#[derive(Clone)]
pub struct Board {
    pub rng: rand::rngs::SmallRng,
    pub width: u16,
    pub height: u16,
    pub contents: Vec<Vec<Particle>>,
    pub gravity: f32,
    pub brushsize: i32,
    pub cellsize: Vec2,
    pub rngs: Vec<Vec<f32>>,
    pub seeds: Vec<Vec<f32>>,
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
        let distribution = Uniform::new_inclusive(-1_f32, 1_f32).unwrap();
        self.contents = vec![
            vec![
                Particle {
                    material: VOID.clone(),
                    speed: Vec2::new(0.0, 0.0),
                    temperature: 20.0,
                    updated: false,
                };
                self.width as usize
            ];
            self.height as usize
        ];
        self.rngs = vec![vec![0_f32; self.width as usize]; self.height as usize];
        self.rngs.iter_mut().for_each(|row| {
            row.iter_mut()
                .for_each(|e| *e = distribution.sample(&mut self.rng))
        });
        self.seeds = vec![vec![0_f32; self.width as usize]; self.height as usize];
        self.seeds.iter_mut().for_each(|row| {
            row.iter_mut()
                .for_each(|e| *e = distribution.sample(&mut self.rng))
        });
    }
}

#[inline(always)]
pub fn update_board(
    game_board: &mut Board,
    is_stopped: bool,
    frame: &mut u8,
    framedelta: f32,
    rng: &mut rand::rngs::SmallRng,
) {
    let distribution = Uniform::new_inclusive(-1_f32, 1_f32).unwrap();
    game_board
        .rngs
        .iter_mut()
        .for_each(|row| row.iter_mut().for_each(|e| *e = distribution.sample(rng)));
    let row_count = game_board.height as i32;
    let col_count: i32 = game_board.width as i32;

    if !is_stopped {
        match *frame {
            0 => {
                (0..row_count * col_count).for_each(|count| {
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
