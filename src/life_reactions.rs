use crate::material::tuple_to_rangeinclusive;
use crate::particle::AtomicParticle;
use crate::reactions::MaterialType;
use crate::world::{AtomicComparedSlice, get_safe_i, write_life_particle};
use crate::{material::Material, particle::Particle};
use egui::lerp;
use std::mem::discriminant;
use std::sync::Arc;

#[inline(never)]
pub(crate) fn solve_cells(
    slice_board: &AtomicComparedSlice<Particle>,
    check_board: &Arc<Vec<AtomicParticle>>,
    prev_board: &[Particle],
    board_rngs: &[f32],
    materials: &[(String, Material)],
    height: &usize,
    width: &usize,
    i: usize,
    j: usize,
    _framedelta: &f32,
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
    let _neumann_positions = [
        (i.wrapping_add(1), j),
        (i.saturating_sub(1), j),
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
        ) == discriminant(&MaterialType::cautomata_default())
            && !(automatons.contains(&Option::Some(
                prev_board
                    .get(get_safe_i(height, width, &cell_positions[pos]))
                    .unwrap_or(&Particle::default())
                    .material_id,
            )))
        {
            automatons[pos] = Option::Some(
                prev_board
                    .get(get_safe_i(height, width, &cell_positions[pos]))
                    .unwrap_or(&Particle::default())
                    .material_id,
            );
        }
    });
    if discriminant(&materials[new_particle.material_id].1.material_type)
        == discriminant(&MaterialType::cautomata_default())
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
                if prev_board
                    .get(get_safe_i(height, width, &cell_positions[pos]))
                    .unwrap_or(&Particle::default())
                    .material_id
                    == automatons[automaton].unwrap()
                    && materials[automatons[automaton].unwrap()]
                        .1
                        .material_type
                        .get_max_stage()
                        == prev_board[get_safe_i(height, width, &cell_positions[pos])]
                            .particle_health
                {
                    alive_neighbours += 1_u8;
                }
            });
            // We evaluate the cellular-automaton rulesets
            survival <<= alive_neighbours.saturating_sub(1_u8);
            birth <<= alive_neighbours.saturating_sub(1_u8);
            // Survival rule check
            if ((survival.reverse_bits() & 0b0000_0001_u8) * alive_neighbours) == alive_neighbours
                && prev_board[get_safe_i(height, width, &(i, j))].material_id
                    == automatons[automaton].unwrap()
                && prev_board[get_safe_i(height, width, &(i, j))].particle_health
                    == materials[automatons[automaton].unwrap()]
                        .1
                        .material_type
                        .get_max_stage()
            {
                new_particle = prev_board[get_safe_i(height, width, &(i, j))];
                new_particle.particle_health = new_particle.particle_health.saturating_sub(1_u16);
            }
            /*// Survive by health
            else if prev_board[get_safe_i(height, width, &(i, j))].life_stage > 0_u8
                && prev_board[get_safe_i(height, width, &(i, j))].material_id
                    == automatons[automaton].unwrap()
            {
                new_particle = prev_board[get_safe_i(height, width, &(i, j))];
                new_particle.life_stage = new_particle.life_stage.saturating_sub(1_u8);
            }*/
            // Birth rule check
            if ((birth.reverse_bits() & 0b0000_0001_u8) * alive_neighbours) == alive_neighbours
                && prev_board[get_safe_i(height, width, &(i, j))].material_id
                    != automatons[automaton].unwrap()
                && prev_board[get_safe_i(height, width, &(i, j))].particle_health == 0_u16
            {
                new_particle.material_id = automatons[automaton].unwrap();
                new_particle.particle_health = materials[automatons[automaton].unwrap()]
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
