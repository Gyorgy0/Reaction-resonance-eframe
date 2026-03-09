use std::cell::UnsafeCell;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use egui::vec2;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use crate::egui_input::BrushShape;
use crate::life_reactions::solve_cells;
use crate::material::Material;
use crate::particle::AtomicParticle;
use crate::particle::Particle;
use crate::physics::solve_particle;
use crate::reactions::solve_reactions;
use egui::Color32;
use egui::Vec2;
use rand::distr::Distribution;
use rand::distr::Uniform;
use serde::Deserialize;
use serde::Serialize;

#[rustfmt::skip]
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub(crate) struct Chunk {
    pub size: u16,                 // Size of the chunk (it's width and height)
    pub position: (i64, i64),      // Position of a chunk (absolute position of a chunk from (0,0))
    pub particles: Vec<Particle>, // The particles that need to be simulated/loaded in/displayed
    pub rngs: Vec<f32>,           // Rngs for particle behaviour
    pub seeds: Vec<f32>,          // Seeds for the particles
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
    pub contents: Vec<Particle>,
    pub gravity: f32,
    pub brush_size: Vec2,
    pub brush_shape: BrushShape,
    pub cellsize: Vec2,
    pub rngs: Vec<f32>,
    pub seeds: Vec<f32>,
}

impl Board {
    pub fn create_board(&mut self) {
        let distribution = Uniform::new_inclusive(-1_f32, 1_f32).unwrap();
        self.contents = vec![Particle::default(); self.height as usize * self.width as usize];
        self.rngs = vec![0_f32; self.height as usize * self.width as usize];
        self.rngs
            .iter_mut()
            .for_each(|e| *e = distribution.sample(&mut self.rng));
        self.seeds = vec![0_f32; self.height as usize * self.width as usize];
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
        let board_slice: AtomicComparedSlice<Particle> =
            AtomicComparedSlice::new(game_board.contents.clone());
        let prev_board: Vec<Particle> = game_board.contents.clone();
        let mut check_board: Arc<Vec<AtomicParticle>> = Arc::new(vec![]);
        let height = game_board.height as usize;
        let width = game_board.width as usize;
        let mut atomicvec: Vec<AtomicParticle> = vec![];
        (0_usize..(row_count * col_count) as usize).for_each(|_count| {
            atomicvec.push(AtomicParticle::default());
        });
        check_board = Arc::new(atomicvec);
        (0_usize..(row_count * col_count) as usize)
            .into_par_iter()
            .for_each(|count: usize| {
                let i = count / width;
                let j = count % width;
                solve_cells(
                    &board_slice,
                    &check_board,
                    &prev_board,
                    &game_board.rngs,
                    materials,
                    &height,
                    &width,
                    i,
                    j,
                );
            });
        (0_usize..(row_count * col_count) as usize)
            .into_par_iter()
            .for_each(|count: usize| {
                let i = count / width;
                let j = count % width;
                solve_particle(
                    &board_slice,
                    &check_board,
                    materials,
                    &game_board.rngs,
                    &game_board.seeds,
                    &height,
                    &width,
                    i,
                    j,
                    game_board.gravity,
                    framedelta,
                );
                solve_reactions(
                    &board_slice,
                    &check_board,
                    &prev_board,
                    materials,
                    &game_board.rngs,
                    &game_board.seeds,
                    &height,
                    &width,
                    i,
                    j,
                    *framecount,
                );
            });
        /*(0_usize..(row_count * col_count) as usize)
        .into_par_iter()
        .for_each(|count: usize| {
            let i = count / width as usize;
            let j = count % width as usize;

        });*/
        game_board.contents = board_slice.data.into_inner();
    }
}

/// A method that returns an index inside the specified height and width
#[inline(always)]
pub fn get_safe_i(rows: &usize, cols: &usize, pos: &(usize, usize)) -> usize {
    let row = pos.0.clamp(0_usize, *rows);
    let mut col = pos.1;
    if col > usize::MAX - cols {
        col = 0_usize;
    }
    col = col.clamp(0_usize, *cols - 1_usize);
    (row * cols) + col
}

/// A thread-safe wrapper for a slice, allowing concurrent writes to distinct indexes.
#[derive(Debug)]
pub struct AtomicComparedSlice<T> {
    data: UnsafeCell<Vec<T>>, // Use Vec<T> for owned data (easier lifetime management)
}

// unsafe impls: Manually mark ThreadSafeSlice as Send and Sync.
unsafe impl<T: Send> Send for AtomicComparedSlice<T> {}
unsafe impl<T: Send> Sync for AtomicComparedSlice<T> {}

impl<T> AtomicComparedSlice<T> {
    /// Create a new ThreadSafeSlice from a Vec<T>.
    pub fn new(data: Vec<T>) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    /// Get the length of the slice.
    pub fn len(&self) -> usize {
        unsafe { (*self.data.get()).len() } // Safe: Read-only access to len
    }
    /// Get the element of the slice.
    pub fn get_elem(&self, index: usize) -> &T {
        unsafe { &self.data.get().as_ref().unwrap()[index] } // Safe: Read-only access to element
    }
    /// Get the element of the slice.
    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        let element = unsafe { self.data.get().as_ref().unwrap() }; // Safe: Read-only access to element
        element.get(index)
    }
}

/// Write a value to a specific index's "updated" field
pub unsafe fn swap_particle(
    slice: &AtomicComparedSlice<Particle>,
    index_1: usize,
    index_2: usize,
    check_board: &Arc<Vec<AtomicParticle>>,
) {
    unsafe {
        // Get a raw pointer to the underlying Vec
        let data_ptr = slice.data.get();
        let vec = &mut *data_ptr; // Dereference to &mut Vec<T> (unsafe!)

        if !check_board[index_1].written.swap(true, Ordering::Relaxed)
            && !check_board[index_2].written.swap(true, Ordering::Relaxed)
        {
            // Get a mutable pointer to the element at `index`
            let elem_1_ptr = vec.as_mut_ptr().add(index_1);
            let particle_1 = slice.data.get().as_ref().unwrap()[index_1];
            let elem_2_ptr = vec.as_mut_ptr().add(index_2);
            let particle_2 = slice.data.get().as_ref().unwrap()[index_2];
            *elem_1_ptr = particle_2;
            *elem_2_ptr = particle_1;
        }
    }
}

/// Write a value to a specific index.
pub unsafe fn write_life_particle(
    slice: &AtomicComparedSlice<Particle>,
    index: usize,
    value: Particle,
    check_board: &Arc<Vec<AtomicParticle>>,
) {
    unsafe {
        // Get a raw pointer to the underlying Vec
        let data_ptr = slice.data.get();
        let vec = &mut *data_ptr; // Dereference to &mut Vec<T> (unsafe!)

        // Checks whether the particle was overwritten
        if !check_board[index]
            .life_written
            .swap(true, Ordering::Relaxed)
        {
            // Get a mutable pointer to the element at `index`
            let elem_ptr = vec.as_mut_ptr().add(index);

            // Write the value into the element (replaces the old value)
            *elem_ptr = value;
        }
    }
}

/// Write a value to a specific index.
pub unsafe fn write_particle(
    slice: &AtomicComparedSlice<Particle>,
    index: usize,
    value: Particle,
    check_board: &Arc<Vec<AtomicParticle>>,
) {
    unsafe {
        // Get a raw pointer to the underlying Vec
        let data_ptr = slice.data.get();
        let vec = &mut *data_ptr; // Dereference to &mut Vec<T> (unsafe!)

        // Checks whether the particle was overwritten
        if !check_board[index]
            .reaction_written
            .swap(true, Ordering::Relaxed)
        {
            // Get a mutable pointer to the element at `index`
            let elem_ptr = vec.as_mut_ptr().add(index);

            // Write the value into the element (replaces the old value)
            *elem_ptr = value;
        }
    }
}

/// Write a value to a specific index's "updated" field
pub unsafe fn write_updated_field(
    slice: &AtomicComparedSlice<Particle>,
    index: usize,
    value: bool,
    check_board: &Arc<Vec<AtomicParticle>>,
) {
    unsafe {
        // Get a raw pointer to the underlying Vec
        let data_ptr = slice.data.get();
        let vec = &mut *data_ptr; // Dereference to &mut Vec<T> (unsafe!)

        if !check_board[index].updated.swap(true, Ordering::Relaxed) {
            // Get a mutable pointer to the element at `index`
            let elem_ptr = vec.as_mut_ptr().add(index);
            let mut prev_particle: Particle = slice.data.get().as_ref().unwrap()[index];
            prev_particle.updated = value;
            // Write the value into the element (replaces the old value)
            *elem_ptr = prev_particle;
        }
    }
}

/// Write a value to a specific index's "speed" field's x component
pub unsafe fn write_x_speed_field(
    slice: &AtomicComparedSlice<Particle>,
    index: usize,
    value: f32,
    check_board: &Arc<Vec<AtomicParticle>>,
) {
    unsafe {
        // Get a raw pointer to the underlying Vec
        let data_ptr = slice.data.get();
        let vec = &mut *data_ptr; // Dereference to &mut Vec<T> (unsafe!)

        if !check_board[index]
            .speed_x
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            check_board[index]
                .speed_x
                .store(true, std::sync::atomic::Ordering::Relaxed);
            // Get a mutable pointer to the element at `index`
            let elem_ptr = vec.as_mut_ptr().add(index);
            let mut prev_particle: Particle = slice.data.get().as_ref().unwrap()[index];
            prev_particle.speed = vec2(value, prev_particle.speed.y);
            // Write the value into the element (replaces the old value)
            *elem_ptr = prev_particle;
        }
    }
}

/// Write a value to a specific index's "speed" field's y component
pub unsafe fn write_y_speed_field(
    slice: &AtomicComparedSlice<Particle>,
    index: usize,
    value: f32,
    check_board: &Arc<Vec<AtomicParticle>>,
) {
    unsafe {
        // Get a raw pointer to the underlying Vec
        let data_ptr = slice.data.get();
        let vec = &mut *data_ptr; // Dereference to &mut Vec<T> (unsafe!)

        if !check_board[index]
            .speed_y
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            check_board[index]
                .speed_y
                .store(true, std::sync::atomic::Ordering::Relaxed);
            // Get a mutable pointer to the element at `index`
            let elem_ptr = vec.as_mut_ptr().add(index);
            let mut prev_particle: Particle = slice.data.get().as_ref().unwrap()[index];
            prev_particle.speed = vec2(prev_particle.speed.x, value);
            // Write the value into the element (replaces the old value)
            *elem_ptr = prev_particle;
        }
    }
}
