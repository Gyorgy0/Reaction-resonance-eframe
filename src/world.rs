use std::ops::RangeInclusive;

use crate::physics::Phase;
use crate::reactions::MaterialType;
use egui::Color32;
use egui::Vec2;
use grid::Grid;
use rand::distr::Distribution;
use rand::distr::Uniform;
use serde::Deserialize;
use serde::Serialize;

#[rustfmt::skip]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub(crate) struct Chunk {
    pub size: u16,                 // Size of the chunk (it's width and height)
    pub position: (i64, i64),      // Position of a chunk (absolute position of a chunk from (0,0))
    pub particles: Grid<Particle>, // The particles that need to be simulated/loaded in/displayed
    pub rngs: Grid<f32>,           // Rngs for particle behaviour
    pub seeds: Grid<f32>,          // Seeds for the particles
    pub unloaded_time: u64,        // The time elapsed from the last load-in
}

#[rustfmt::skip]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub(crate) struct Material {
    pub name: String,                       // Name of the material
    pub density: f32,                       // Mass of a cm^3 volume of the material
    pub phase: Phase,                       // Phase of the material for, the implemented phases check the "Phase" enum
    pub material_type: MaterialType,        // Type of the material for, the implemented types check the "Type" enum
    pub durability: i32,                    // Durability of a material - how much force it needs to disintegrate the material -> higher = more force
    pub material_color: MaterialColor,      // Color of the material
}

#[rustfmt::skip]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub(crate) struct MaterialColor{
    pub color: Color32,                 // Color of the material
    pub shinyness: RangeInclusive<f32>, // Shinyness of the material (<1.0 - darker colors)
                                        //                           (>1.0 - lighter colors)
}

#[rustfmt::skip]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Particle {
    pub material: Material,     // Material of the particle
    pub speed: Vec2,            // Vectors of the particle (x, y)
    pub temperature: f32,       // Temperature of the materialy
    pub updated: bool,          // Is it updated?
    pub display_color: Color32, // Displayed color
}

#[rustfmt::skip]
#[derive(Clone)]
pub struct Board {
    pub rng: rand::rngs::SmallRng,
    pub width: u16,
    pub height: u16,
    pub contents: Grid<Particle>,
    pub gravity: f32,
    pub brushsize: i32,
    pub cellsize: Vec2,
    pub rngs: Grid<f32>,
    pub seeds: Grid<f32>,
}

pub static VOID: Material = Material {
    name: String::new(),
    density: 0.0,
    phase: Phase::Void,
    material_type: MaterialType::Atmosphere,
    durability: -1,
    material_color: MaterialColor {
        color: Color32::from_rgba_premultiplied(0, 0, 0, 100),
        shinyness: 1.0..=1.0,
    },
};

impl Board {
    pub fn create_board(&mut self) {
        let distribution = Uniform::new_inclusive(-1_f32, 1_f32).unwrap();
        self.contents = Grid::from_vec(
            vec![
                Particle {
                    material: VOID.clone(),
                    speed: Vec2::new(0.0, 0.0),
                    temperature: 20.0,
                    updated: false,
                    display_color: VOID.material_color.color,
                };
                self.height as usize * self.width as usize
            ],
            self.width as usize,
        );
        self.rngs = grid::Grid::from_vec(
            vec![0_f32; self.height as usize * self.width as usize],
            self.width as usize,
        );
        self.rngs
            .iter_mut()
            .for_each(|e| *e = distribution.sample(&mut self.rng));
        self.seeds = grid::Grid::from_vec(
            vec![0_f32; self.height as usize * self.width as usize],
            self.width as usize,
        );
        self.seeds
            .iter_mut()
            .for_each(|e| *e = distribution.sample(&mut self.rng));
    }
}

#[inline(always)]
pub fn update_board(
    game_board: &mut Board,
    is_stopped: bool,
    framecount: &mut u64,
    framedelta: f32,
) {
    *framecount = framecount.wrapping_add(1);
    let distribution = Uniform::new_inclusive(-1_f32, 1_f32).unwrap();
    game_board
        .rngs
        .iter_mut()
        .for_each(|e| *e = distribution.sample(&mut game_board.rng));
    let row_count = game_board.height as i32;
    let col_count: i32 = game_board.width as i32;

    if !is_stopped {
        match *framecount % 2 {
            0 => {
                (0..row_count * col_count).for_each(|count| {
                    let i = (count / col_count) as usize;
                    let j = (count % col_count) as usize;
                    game_board.solve_particle(i, j, framedelta, *framecount);
                    game_board.solve_reactions(i, j, framedelta, *framecount);
                });
            }
            1 => {
                (0..row_count * col_count).for_each(|count| {
                    let i = (count / col_count) as usize;
                    let j = ((col_count - 1) - (count % col_count)) as usize;
                    game_board.solve_particle(i, j, framedelta, *framecount);
                    game_board.solve_reactions(i, j, framedelta, *framecount);
                });
            }
            _ => {}
        }
    }
}
