use crate::material::tuple_to_rangeinclusive;
use crate::reactions::MaterialType;
use crate::world::get_safe_i;
use crate::{material::Material, particle::Particle};
use egui::lerp;
use std::mem::discriminant;

#[inline(always)]
pub(crate) fn solve_cells(
    prev_board: &Vec<Particle>,
    board_rngs: &Vec<f32>,
    materials: &Vec<(String, Material)>,
    height: &usize,
    width: &usize,
    i: usize,
    j: usize,
) -> Particle {
    // Cellular Automaton solving
    let cell_positions = [
        (i.wrapping_add(1), j.wrapping_add(1)),
        (i.wrapping_add(1), j),
        (i.wrapping_add(1), j.saturating_sub(1)),
        (i.saturating_sub(1), j.wrapping_add(1)),
        (i.saturating_sub(1), j),
        (i.saturating_sub(1), j.saturating_sub(1)),
        (i, j.wrapping_add(1)),
        (i, j.saturating_sub(1)),
    ];
    let mut automatons: Vec<usize> = vec![];
    let mut new_particle = prev_board[get_safe_i(height, width, &(i, j))];

    (0_usize..8_usize).for_each(|pos: usize| {
        if discriminant(
            &materials[prev_board
                .get(get_safe_i(
                    height,
                    width,
                    &(cell_positions[pos].0, cell_positions[pos].1),
                ))
                .unwrap_or(&Particle::default())
                .material_id]
                .1
                .material_type,
        ) == discriminant(&MaterialType::CAutomata {
            birth: 0_u8,
            survival: 0_u8,
            stages: 0_u8,
        }) && !automatons.contains(
            &prev_board
                .get(get_safe_i(
                    height,
                    width,
                    &(cell_positions[pos].0, cell_positions[pos].1),
                ))
                .unwrap_or(&Particle::default())
                .material_id,
        ) {
            automatons.push(
                prev_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(cell_positions[pos].0, cell_positions[pos].1),
                    ))
                    .unwrap_or(&Particle::default())
                    .material_id,
            );
        }
    });
    automatons.iter().for_each(|automaton| {
        let mut alive_neighbours = 0_u8;
        let mut birth = materials[*automaton].1.material_type.get_birth();
        let mut survival = materials[*automaton].1.material_type.get_survival();
        (0_usize..8_usize).for_each(|pos: usize| {
            if prev_board
                .get(get_safe_i(
                    height,
                    width,
                    &(cell_positions[pos].0, cell_positions[pos].1),
                ))
                .unwrap_or(&Particle::default())
                .material_id
                == *automaton
            {
                alive_neighbours += 1_u8;
            }
        });
        if discriminant(
            &materials[prev_board[get_safe_i(height, width, &(i, j))].material_id]
                .1
                .material_type,
        ) == discriminant(&MaterialType::CAutomata {
            birth: 0_u8,
            survival: 0_u8,
            stages: 0_u8,
        }) {
            new_particle.material_id = 0_usize;
            new_particle.display_color = materials[new_particle.material_id]
                .1
                .material_color
                .color
                .gamma_multiply(lerp(
                    tuple_to_rangeinclusive(
                        materials[new_particle.material_id]
                            .1
                            .material_color
                            .shinyness,
                    ),
                    board_rngs[get_safe_i(height, width, &(i, j))],
                ));
            new_particle.display_color[3] = materials[new_particle.material_id]
                .1
                .material_color
                .color
                .a();
        }
        (0_usize..8_usize).for_each(|pos: usize| {
            if ((survival.reverse_bits() & 0b0000_0001_u8) * ((pos + 1_usize) as u8))
                == alive_neighbours
            {
                new_particle.material_id =
                    prev_board[get_safe_i(height, width, &(i, j))].material_id;
                new_particle.display_color = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[new_particle.material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        board_rngs[get_safe_i(height, width, &(i, j))],
                    ));
                new_particle.display_color[3] = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .a();
            }
            if ((birth.reverse_bits() & 0b0000_0001_u8) * ((pos + 1_usize) as u8))
                == alive_neighbours
            {
                new_particle.material_id = *automaton;
                new_particle.display_color = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[new_particle.material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        board_rngs[get_safe_i(height, width, &(i, j))],
                    ));
                new_particle.display_color[3] = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .a();
            }
            survival <<= 1;
            birth <<= 1;
        });
    });
    new_particle
}
