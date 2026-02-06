use crate::material::tuple_to_rangeinclusive;
use crate::reactions::MaterialType;
use crate::{material::Material, particle::Particle, world::Board};
use egui::lerp;
use grid::Grid;
use std::mem::discriminant;

impl Board {
    #[inline(always)]
    pub(crate) fn solve_cells(
        &mut self,
        prev_board: &Grid<Particle>,
        materials: &Vec<(String, Material)>,
        i: usize,
        j: usize,
    ) {
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

        (0_usize..8_usize).into_iter().for_each(|pos: usize| {
            if discriminant(
                &materials[prev_board
                    .get(cell_positions[pos].0, cell_positions[pos].1)
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
                    .get(cell_positions[pos].0, cell_positions[pos].1)
                    .unwrap_or(&Particle::default())
                    .material_id,
            ) {
                automatons.push(
                    prev_board
                        .get(cell_positions[pos].0, cell_positions[pos].1)
                        .unwrap_or(&Particle::default())
                        .material_id,
                );
            }
        });
        automatons.iter().for_each(|automaton| {
            let mut alive_neighbours = 0_u8;
            let mut birth = materials[*automaton].1.material_type.get_birth();
            let mut survival = materials[*automaton].1.material_type.get_survival();
            (0_usize..8_usize).into_iter().for_each(|pos: usize| {
                if prev_board
                    .get(cell_positions[pos].0, cell_positions[pos].1)
                    .unwrap_or(&Particle::default())
                    .material_id
                    == *automaton
                {
                    alive_neighbours += 1_u8;
                }
            });
            if discriminant(&materials[self.contents[(i, j)].material_id].1.material_type)
                == discriminant(&MaterialType::CAutomata {
                    birth: 0_u8,
                    survival: 0_u8,
                    stages: 0_u8,
                })
            {
                self.contents[(i, j)].material_id = 0_usize;
                self.contents[(i, j)].display_color = materials[self.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .gamma_multiply(lerp(
                        tuple_to_rangeinclusive(
                            materials[self.contents[(i, j)].material_id]
                                .1
                                .material_color
                                .shinyness,
                        ),
                        self.rngs[(i, j)],
                    ));
                self.contents[(i, j)].display_color[3] = materials
                    [self.contents[(i, j)].material_id]
                    .1
                    .material_color
                    .color
                    .a();
            }
            (0_usize..8_usize).into_iter().for_each(|pos: usize| {
                if ((survival.reverse_bits() & 0b0000_0001_u8) * ((pos + 1_usize) as u8))
                    == alive_neighbours
                {
                    self.contents[(i, j)].material_id = prev_board[(i, j)].material_id;
                    self.contents[(i, j)].display_color = materials
                        [self.contents[(i, j)].material_id]
                        .1
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials[self.contents[(i, j)].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] = materials
                        [self.contents[(i, j)].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                }
                if ((birth.reverse_bits() & 0b0000_0001_u8) * ((pos + 1_usize) as u8))
                    == alive_neighbours
                {
                    self.contents[(i, j)].material_id = *automaton;
                    self.contents[(i, j)].display_color = materials
                        [self.contents[(i, j)].material_id]
                        .1
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            tuple_to_rangeinclusive(
                                materials[self.contents[(i, j)].material_id]
                                    .1
                                    .material_color
                                    .shinyness,
                            ),
                            self.rngs[(i, j)],
                        ));
                    self.contents[(i, j)].display_color[3] = materials
                        [self.contents[(i, j)].material_id]
                        .1
                        .material_color
                        .color
                        .a();
                }
                survival = survival << 1;
                birth = birth << 1;
            });
        });
    }
}
