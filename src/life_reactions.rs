use crate::material::tuple_to_rangeinclusive;
use crate::particle::AtomicParticle;
use crate::reactions::MaterialType;
use crate::world::{AtomicComparedSlice, get_safe_i, write_life_particle};
use crate::{material::Material, particle::Particle};
use egui::lerp;
use rayon::iter::ParallelIterator;
use std::mem::discriminant;
use std::sync::Arc;

#[inline(never)]
pub(crate) fn solve_cells(
    slice_board: &AtomicComparedSlice<Particle>,
    check_board: &Arc<Vec<AtomicParticle>>,
    prev_board: &Vec<Particle>,
    board_rngs: &Vec<f32>,
    materials: &Vec<(String, Material)>,
    height: &usize,
    width: &usize,
    i: usize,
    j: usize,
) {
    // Cellular Automaton solving (Moore neighborhood coordinates)
    // i - y value (current row)
    // j = x value (current column)
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
    // Length of the previous coordinate list
    let cell_positions_len = cell_positions.len();
    let mut automatons: [Option<usize>; 8] = [Option::None; 8];
    //
    let mut new_particle = prev_board[get_safe_i(height, width, &(i, j))];

    // Counting how many automatons we have in the Moore neighborhood
    // - we count them without duplicates and the invalid or duplicated automaton-types are Option::None values
    (0_usize..cell_positions_len).for_each(|pos: usize| {
        if discriminant(
            &materials[prev_board
                .get(get_safe_i(height, width, &cell_positions[pos]))
                .unwrap_or(&Particle::default())
                .material_id]
                .1
                .material_type,
        ) == discriminant(&MaterialType::CAutomata {
            birth: 0_u8,
            survival: 0_u8,
            stages: 0_u8,
        }) && !(automatons.contains(&Option::Some(
            prev_board
                .get(get_safe_i(height, width, &cell_positions[pos]))
                .unwrap_or(&Particle::default())
                .material_id,
        ))) {
            automatons[pos] = Option::Some(
                prev_board
                    .get(get_safe_i(height, width, &cell_positions[pos]))
                    .unwrap_or(&Particle::default())
                    .material_id,
            );
        }
    });
    if discriminant(&materials[new_particle.material_id].1.material_type)
        == discriminant(
            &(MaterialType::CAutomata {
                survival: 0_u8,
                birth: 0_u8,
                stages: 0_u8,
            }),
        )
    {
        new_particle = Particle::default();
    }
    // We evaluate each of the valid cellular-automatons found within the neighborhood
    (0_usize..cell_positions_len).for_each(|automaton| {
        if automatons[automaton].is_some() {
            new_particle = Particle::default();
            let mut alive_neighbours = 0_u8;
            let mut birth = materials[automatons[automaton].unwrap()]
                .1
                .material_type
                .get_birth();
            let mut survival = materials[automatons[automaton].unwrap()]
                .1
                .material_type
                .get_survival();

            // We count the number of alive neighbours (these neighbours are of the same type and they are "healthy")
            (0_usize..cell_positions_len).for_each(|pos: usize| {
                if discriminant(
                    &materials[prev_board
                        .get(get_safe_i(height, width, &cell_positions[pos]))
                        .unwrap_or(&Particle::default())
                        .material_id]
                        .1
                        .material_type,
                ) == discriminant(
                    &(MaterialType::CAutomata {
                        survival: 0_u8,
                        birth: 0_u8,
                        stages: 0_u8,
                    }),
                ) && materials[prev_board
                    .get(get_safe_i(height, width, &cell_positions[pos]))
                    .unwrap_or(&Particle::default())
                    .material_id]
                    .1
                    .material_type
                    .get_max_stage()
                    == prev_board[get_safe_i(height, width, &cell_positions[pos])].life_stage
                {
                    alive_neighbours += 1_u8;
                }
            });
            // We evaluate the cellular-automaton rulesets
            for pos in 0_usize..cell_positions_len {
                // Survival rule check
                if ((survival.reverse_bits() & 0b0000_0001_u8) * ((pos + 1_usize) as u8))
                    == alive_neighbours
                    && materials[prev_board[get_safe_i(height, width, &(i, j))].material_id]
                        .1
                        .material_type
                        .get_max_stage()
                        == prev_board[get_safe_i(height, width, &(i, j))].life_stage
                {
                    new_particle = prev_board[get_safe_i(height, width, &(i, j))];
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
                // Survival by health check
                else if ((survival.reverse_bits() & 0b0000_0001_u8) * ((pos + 1_usize) as u8))
                    != alive_neighbours
                    && prev_board[get_safe_i(height, width, &(i, j))].life_stage > 0_u8
                {
                    new_particle = prev_board[get_safe_i(height, width, &(i, j))];
                    new_particle.life_stage = new_particle.life_stage.saturating_sub(1_u8);
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
                // Birth rule check
                if ((birth.reverse_bits() & 0b0000_0001_u8) * ((pos + 1_usize) as u8))
                    == alive_neighbours
                    && prev_board[get_safe_i(height, width, &(i, j))].material_id
                        != automatons[automaton].unwrap()
                {
                    new_particle.material_id = automatons[automaton].unwrap();
                    new_particle.life_stage = materials[new_particle.material_id]
                        .1
                        .material_type
                        .get_max_stage();
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
            }
        }
    });
    unsafe {
        write_life_particle(
            slice_board,
            get_safe_i(height, width, &(i, j)),
            new_particle,
            check_board,
        )
    }
}
