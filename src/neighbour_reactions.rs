use crate::life_reactions::solve_cells;
use crate::{material::Material, particle::Particle};

#[inline(always)]
pub(crate) fn solve_by_neighbours(
    prev_board: &Vec<Particle>,
    board_rngs: &Vec<f32>,
    materials: &Vec<(String, Material)>,
    height: &usize,
    width: &usize,
    i: usize,
    j: usize,
) -> Particle {
    let new_particle = solve_cells(prev_board, board_rngs, materials, height, width, i, j);
    let _cell_positions = [
        (i.wrapping_add(1), j),
        (i.saturating_sub(1), j),
        (i, j.wrapping_add(1)),
        (i, j.saturating_sub(1)),
    ];
    /*for pos in cell_positions {
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
                    && materials[prev_board[(i, j)].material_id].1.material_type
                        != MaterialType::Sink
                {
                    new_particle = Particle::default();
                    break;
                }
            }
            _ => {}
        }
    }*/
    new_particle
}
