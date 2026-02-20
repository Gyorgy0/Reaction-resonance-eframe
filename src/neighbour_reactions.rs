use crate::life_reactions::solve_cells;
use crate::material::tuple_to_rangeinclusive;
use crate::reactions::MaterialType;
use crate::{material::Material, particle::Particle, world::Board};
use egui::lerp;
use grid::Grid;
use std::cell;
use std::mem::discriminant;

#[inline(always)]
pub(crate) fn solve_by_neighbours(
    prev_board: &Grid<Particle>,
    board_rngs: &Grid<f32>,
    materials: &Vec<(String, Material)>,
    i: usize,
    j: usize,
) -> Particle {
    let mut new_particle = solve_cells(prev_board, board_rngs, materials, i, j);
    let cell_positions = [
        (i.wrapping_add(1), j),
        (i.saturating_sub(1), j),
        (i, j.wrapping_add(1)),
        (i, j.saturating_sub(1)),
    ];
    for pos in cell_positions {
        match &materials[prev_board
            .get(pos.0, pos.1)
            .unwrap_or(&prev_board[(i, j)])
            .material_id]
            .1
            .material_type
        {
            MaterialType::Cloner => {
                if prev_board[pos].cloned_material != 0_usize && new_particle.material_id == 0_usize
                {
                    new_particle.material_id = prev_board[pos].cloned_material;
                    new_particle.display_color = materials[prev_board[pos].cloned_material]
                        .1
                        .material_color
                        .color;
                    new_particle.display_color = new_particle.display_color.gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[new_particle.material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        board_rngs[(i, j)],
                    ));
                    new_particle.display_color[3] = materials[new_particle.material_id]
                        .1
                        .material_color
                        .color
                        .a();
                }
            }
            MaterialType::Sink => {
                if prev_board[(i, j)] != Particle::default()
                    && &materials[prev_board[(i, j)].material_id].1.material_type
                        != &MaterialType::Sink
                {
                    new_particle = Particle::default();
                    break;
                }
            }
            _ => {}
        }
    }
    new_particle
}
