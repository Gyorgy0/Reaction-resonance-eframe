use std::sync::atomic::AtomicBool;

use crate::life_reactions::solve_cells;
use crate::material::Material;
use crate::particle::Particle;
use crate::reactions::solve_reactions;
use egui::Color32;
use egui::Vec2;
use grid::Grid;
use grid::grid;
use rand::distr::Distribution;
use rand::distr::Uniform;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelIterator;
use rayon::iter::IntoParallelRefMutIterator;
use rayon::iter::ParallelIterator;
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
#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, Debug)]
pub(crate) struct MaterialColor{
    pub color: Color32,                 // Color of the material
    pub shinyness: (f32,f32),           // Shinyness of the material (<1_f32 - darker colors)
                                        //                           (>1_f32 - lighter colors)
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

impl Board {
    pub fn create_board(&mut self) {
        let distribution = Uniform::new_inclusive(-1_f32, 1_f32).unwrap();
        self.contents = Grid::from_vec(
            vec![Particle::default(); self.height as usize * self.width as usize],
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
    materials: &Vec<(String, Material)>,
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
        (0..row_count * col_count).for_each(|count| {
            let i = (count / col_count) as usize;
            let j = (count % col_count) as usize;
            game_board.solve_particle(materials, i, j, framedelta);
        });
        let prev_board: Grid<Particle> = game_board.contents.clone();
        let new_board: Vec<Particle> = (0..row_count * col_count)
            .into_par_iter()
            .enumerate()
            .map(|particle| {
                let i = particle.0 / col_count as usize;
                let j = particle.0 % col_count as usize;
                solve_reactions(
                    &prev_board,
                    &game_board.rngs,
                    materials,
                    i,
                    j,
                    framedelta,
                    *framecount,
                )
            })
            .collect();
        game_board.contents = Grid::from_vec(new_board, col_count as usize);
    }
}
