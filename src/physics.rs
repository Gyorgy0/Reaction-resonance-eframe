use crate::{
    material::{Material, VOID},
    particle::Particle,
    world::{AtomicComparedSlice, Board, get_i, write_slice},
};
use crossbeam::epoch::Atomic;
use egui::Vec2;
use serde::{Deserialize, Serialize};
use std::{
    mem::discriminant,
    sync::{Arc, atomic::AtomicU8},
};

#[rustfmt::skip]
#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
#[repr(u8)]
pub enum Phase {
    Void,
    Solid { melting_point: f32 },
    Powder { coarseness: f32, melting_point: f32 },                         // Coarseness is the average diameter of a powder particle (between 0 and 1) (in cm), -> the smaller the diameter, the powder becomes more "clumpier"
    Liquid { viscosity: f32, melting_point: f32, boiling_point: f32 },      // Viscosity gives the rate, which the liquid spreads, for e.g. water has a viscosity of 1_f32, the bigger the viscosity, the thicker the fluid is
    Gas {boiling_point: f32},                                               // Not fully implemented
    Plasma,
}

impl Phase {
    fn get_coarseness(&self) -> f32 {
        let mut returnval: f32 = 0.0;
        if let Phase::Powder {
            coarseness,
            melting_point: _,
        } = self
        {
            returnval = *coarseness
        };
        returnval
    }
    fn get_viscosity(&self) -> f32 {
        let mut returnval: f32 = 0.0;
        if let Phase::Liquid {
            viscosity,
            melting_point: _,
            boiling_point: _,
        } = self
        {
            returnval = 1_f32 / *viscosity
        };
        returnval
    }
}

#[inline(always)]
pub unsafe fn solve_particle(
    slice_board: &AtomicComparedSlice<Particle>,
    check_board: &Arc<Vec<AtomicU8>>,
    prev_board: &Vec<Particle>,
    materials: &Vec<(String, Material)>,
    rngs: &Vec<f32>,
    width: u16,
    i: usize,
    j: usize,
    gravity: f32,
    framedelta: f32,
) {
    match &materials[prev_board[get_i(width, (i, j))].material_id]
        .1
        .phase
    {
        Phase::Void => {}

        Phase::Solid { melting_point: _ } => {}
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // POWDER PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Powder {
            coarseness: _,
            melting_point: _,
        } => {
            // Gravity simulation
            //prev_board[get_i(width, (i,j))].speed.y += gravity * framedelta;
            let mut ychange = 0;
            for _k in 0..prev_board[get_i(width, (i, j))].speed.y.abs() as i32 {
                // Falling and checking if there is a particle with a larger density
                if materials[prev_board[get_i(width, (i, j))].material_id]
                    .1
                    .density
                    > materials[prev_board
                        .get(get_i(
                            width,
                            (i + (gravity.signum() as i32 * _k) as usize, j),
                        ))
                        .unwrap_or(&prev_board[get_i(width, (i, j))])
                        .material_id]
                        .1
                        .density
                    && std::mem::discriminant(
                        &materials[prev_board
                            .get(get_i(
                                width,
                                (i + (gravity.signum() as i32 * _k) as usize, j),
                            ))
                            .unwrap_or(&prev_board[get_i(width, (i, j))])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    && prev_board[get_i(width, (i, j))].updated
                {
                    ychange = _k;
                }
                // Checks if the particle falls inside bounds
                // Checks, whether there is another denser particle in the path of the falling particle
                else if prev_board
                    .get(get_i(
                        width,
                        (i + (gravity.signum() as i32 * _k) as usize, j),
                    ))
                    .is_none()
                    || std::mem::discriminant(
                        &materials[prev_board
                            .get(get_i(
                                width,
                                (i + (gravity.signum() as i32 * _k) as usize, j),
                            ))
                            .unwrap_or(&prev_board[get_i(width, (i, j))])
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    || std::mem::discriminant(
                        &materials[prev_board
                            .get(get_i(
                                width,
                                (i + (gravity.signum() as i32 * _k) as usize, j),
                            ))
                            .unwrap_or(&prev_board[get_i(width, (i, j))])
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    )
                {
                    //prev_board[get_i(width, (i,j))].speed.y -= gravity * framedelta;
                    break;
                }
            }
            if ychange != 0 {
                unsafe {
                    write_slice(
                        slice_board,
                        get_i(width, (i, j)),
                        prev_board[get_i(
                            width,
                            (i + ((gravity.signum() as i32 * ychange) as usize), j),
                        )],
                        check_board[get_i(width, (i, j))]
                            .fetch_add(1_u8, std::sync::atomic::Ordering::AcqRel),
                        AtomicU8::default(),
                    );
                    write_slice(
                        slice_board,
                        get_i(
                            width,
                            (i + ((gravity.signum() as i32 * ychange) as usize), j),
                        ),
                        prev_board[get_i(width, (i, j))],
                        check_board[get_i(width, (i, j))]
                            .fetch_add(1_u8, std::sync::atomic::Ordering::AcqRel),
                        AtomicU8::default(),
                    );
                }
                /*prev_board.swap(
                    get_i(width ,(i, j)),
                    get_i(width, (i + ((gravity.signum() as i32 * ychange) as usize), j)),
                );
                prev_board[get_i( width, (i + ((gravity.signum() as i32 * ychange) as usize), j))]
                    .updated = false;*/
            }
            // This decides where the particle falls (left or right)
            let rnd = rngs[get_i(width, (i, j))];
            if prev_board[get_i(width, (i, j))].updated
                && prev_board
                    .get(get_i(
                        width,
                        (i + (gravity.signum() as i32) as usize, j.wrapping_add(1)),
                    ))
                    .is_some()
                && materials[prev_board
                    .get(get_i(
                        width,
                        (i + (gravity.signum() as i32) as usize, j.wrapping_add(1)),
                    ))
                    .unwrap_or(&prev_board[get_i(width, (i, j))])
                    .material_id]
                    .1
                    .density
                    < materials[prev_board[get_i(width, (i, j))].material_id]
                        .1
                        .density
                && std::mem::discriminant(
                    &materials[prev_board
                        .get(get_i(
                            width,
                            (i + (gravity.signum() as i32) as usize, j.wrapping_add(1)),
                        ))
                        .unwrap_or(&prev_board[get_i(width, (i, j))])
                        .material_id]
                        .1
                        .phase,
                ) != std::mem::discriminant(
                    &(Phase::Solid {
                        melting_point: 0_f32,
                    }),
                )
                && prev_board[get_i(width, (i,j))].temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - materials[prev_board[get_i(width, (i,j))]
                                .material_id].1
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                && rnd.signum() == -1_f32
            {
                /*prev_board.swap(
                    get_i(width,(i, j)),
                    get_i(width,(
                        i + (gravity.signum() as i32) as usize,
                        j.wrapping_add(1)),
                    ),
                );
                prev_board[get_i(width, (
                    i + (gravity.signum() as i32) as usize,
                    j.wrapping_add(1))
                )]
                    .updated = false;*/
            }
            if prev_board[get_i(width, (i, j))].updated
                && prev_board
                    .get(get_i(
                        width,
                        (i + (gravity.signum() as i32) as usize, j.wrapping_sub(1)),
                    ))
                    .is_some()
                && materials[prev_board
                    .get(get_i(
                        width,
                        (i + (gravity.signum() as i32) as usize, j.wrapping_sub(1)),
                    ))
                    .unwrap_or(&prev_board[get_i(width, (i, j))])
                    .material_id]
                    .1
                    .density
                    < materials[prev_board[get_i(width, (i, j))].material_id]
                        .1
                        .density
                && discriminant(
                    &materials[prev_board
                        .get(get_i(
                            width,
                            (i + (gravity.signum() as i32) as usize, j.wrapping_sub(1)),
                        ))
                        .unwrap_or(&prev_board[get_i(width, (i, j))])
                        .material_id]
                        .1
                        .phase,
                ) != std::mem::discriminant(
                    &(Phase::Solid {
                        melting_point: 0_f32,
                    }),
                )
                && prev_board[get_i(width, (i,j))].temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - materials[prev_board[get_i(width, (i,j))]
                                .material_id].1
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                && rnd.signum() == 1_f32
            {
                /*prev_board.swap(
                    get_i(width, (i, j)),
                    get_i(width, (
                        i + (gravity.signum() as i32) as usize,
                        j.wrapping_sub(1),
                    )),
                );
                prev_board[get_i(width, (
                    i + (gravity.signum() as i32) as usize,
                    j.wrapping_sub(1),
                ))]
                    .updated = false;*/
            }
            // This marks that the particle's position has been calculated
            //prev_board[get_i(width, (i,j))].updated = true;
        }
        _ => {}
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// LIQUID PHYSICS
/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/*Phase::Liquid {
            viscosity: _,
            melting_point: _,
            boiling_point: _,
        } => {
            // Gravity simulation
            prev_board[get_i(width, (i,j))].speed.y += gravity * framedelta;
            let mut ychange = 0;
            for _k in 0..prev_board[get_i(width, (i,j))].speed.y.abs() as i32 {
                // Falling and checking if there is a particle with a larger density
                if materials[prev_board[get_i(width, (i,j))].material_id].1.density
                    > materials[prev_board
                        .get(get_i(width, (i + (gravity.signum() as i32 * _k) as usize, j)))
                        .unwrap_or(&prev_board[get_i(width, (i,j))])
                        .material_id]
                        .1
                        .density
                    && prev_board[get_i(width, (i,j))].updated
                    && discriminant(
                        &materials[prev_board
                            .get(get_i(width, (i + (gravity.signum() as i32 * _k) as usize, j)))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != discriminant(&Phase::Solid {
                        melting_point: 0_f32,
                    })
                {
                    ychange = _k;
                }
                // Checks if the particle falls inside bounds
                // Checks, whether there is another denser particle in the path of the falling particle
                else if prev_board
                    .get(get_i(width, (i + (gravity.signum() as i32 * _k) as usize, j)))
                    .is_none()
                    && discriminant(
                        &materials[prev_board
                            .get(get_i(width, (i + (gravity.signum() as i32 * _k) as usize, j)))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) == discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    || discriminant(
                        &materials[prev_board
                            .get(
                                i + (gravity.signum() as i32 * _k
                                    + gravity.signum() as i32)
                                    as usize,
                                j,
                            )
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) == discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    )
                    || discriminant(
                        &materials[prev_board
                            .get(
                                i + (gravity.signum() as i32 * _k
                                    + gravity.signum() as i32)
                                    as usize,
                                j,
                            )
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) == discriminant(
                        &(Phase::Liquid {
                            viscosity: 0_f32,
                            melting_point: 0_f32,
                            boiling_point: 0_f32,
                        }),
                    )
                {
                    prev_board[get_i(width, (i,j))].speed.y -= gravity * framedelta;
                    break;
                }
            }
            if ychange != 0 {
                prev_board.swap(
                    (i, j),
                    (i + (gravity.signum() as i32 * ychange) as usize, j),
                );
                prev_board[(i + ((gravity.signum() as i32 * ychange) as usize), j)]
                    .updated = false;
            }
            // Rng determines which side should the particle fall
            let mut orientation: i32 = 0;
            if prev_board[get_i(width, (i,j))].speed.x.abs() > 1_f32 {
                prev_board[get_i(width, (i,j))].speed.x = 0.0;
            } else {
                let rnd = rngs[(i, j)];
                if rnd.abs()
                    >= (1_f32
                        - materials[prev_board[get_i(width, (i,j))].material_id]
                            .1
                            .phase
                            .get_viscosity())
                    .powi(16)
                {
                    prev_board[get_i(width, (i,j))].speed.x += rnd.signum()
                        * (rnd.abs()
                            + materials[prev_board[get_i(width, (i,j))].material_id]
                                .1
                                .phase
                                .get_viscosity()
                                .sqrt());
                    orientation = (prev_board[get_i(width, (i,j))].speed.x.signum()
                        * (prev_board[get_i(width, (i,j))].speed.x.abs() + 1_f32))
                        as i32;
                }
            }

            for _k in 0..orientation.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if prev_board
                    .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                    .is_some()
                    && discriminant(
                        &materials[prev_board
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) == discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                {
                    continue;
                } else if prev_board
                    .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                    .is_some()
                    && materials[prev_board
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .unwrap_or(&prev_board[get_i(width, (i,j))])
                        .material_id]
                        .1
                        .density
                        <= materials[prev_board[get_i(width, (i,j))].material_id].1.density
                    && (discriminant(
                        &materials[prev_board
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    ))
                {
                    prev_board.swap(
                        (i, j),
                        (i, j.wrapping_add((orientation.signum() * _k) as usize)),
                    );
                    prev_board[(i, j.wrapping_add((orientation.signum() * _k) as usize))]
                        .updated = true;
                } else {
                    prev_board[get_i(width, (i,j))].speed.x *= -1_f32;
                    break;
                }
            }
            // This marks that the particle's position has been calculated
            prev_board[get_i(width, (i,j))].updated = true;
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // GAS PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Gas { boiling_point: _ } => {
            // Rng determines which side should the particle fall
            let mut orientation: i32 = 0;
            // This calculates the position on the Y axis
            if prev_board[get_i(width, (i,j))].speed.y.abs() > 1_f32 {
                prev_board[get_i(width, (i,j))].speed.y = 0.0;
            } else {
                // Rand range: (-1_f32..1_f32)
                let rnd = rngs[(i, j)];
                prev_board[get_i(width, (i,j))].speed.y += rnd.signum() * (rnd.abs() / 2_f32);
                orientation = (prev_board[get_i(width, (i,j))].speed.y.signum()
                    * (prev_board[get_i(width, (i,j))].speed.y.abs() + 1_f32))
                    as i32;
            }

            for _k in 0..orientation.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if prev_board
                    .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                    .is_some()
                    && std::mem::discriminant(
                        &materials[prev_board
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::Solid {
                        melting_point: 0_f32,
                    })
                {
                    continue;
                } else if prev_board
                    .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                    .is_some()
                    && (materials[prev_board
                        .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                        .unwrap_or(&prev_board[get_i(width, (i,j))])
                        .material_id]
                        .1
                        .density
                        <= materials[prev_board[get_i(width, (i,j))].material_id].1.density)
                    && (std::mem::discriminant(
                        &materials[prev_board
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    ) || std::mem::discriminant(
                        &materials[prev_board
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(&Phase::Liquid {
                        viscosity: 0_f32,
                        melting_point: 0_f32,
                        boiling_point: 0_f32,
                    }))
                {
                    prev_board.swap(
                        (i, j),
                        (i.wrapping_add((orientation.signum() * _k) as usize), j),
                    );
                    prev_board[(i.wrapping_add((orientation.signum() * _k) as usize), j)]
                        .updated = true;
                }
            }
            orientation = 0;
            // This calculates the position on the X axis
            if prev_board[get_i(width, (i,j))].speed.x.abs() > 1_f32 {
                prev_board[get_i(width, (i,j))].speed.x = 0.0;
            } else {
                let rnd = rngs[(i, j)] * self.seeds[(i, j)];
                prev_board[get_i(width, (i,j))].speed.x += rnd.signum() * (rnd.abs());
                orientation = (prev_board[get_i(width, (i,j))].speed.x.signum()
                    * (prev_board[get_i(width, (i,j))].speed.x.abs() + 1_f32))
                    as i32;
            }

            for _k in 0..orientation.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if prev_board
                    .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                    .is_some()
                    && std::mem::discriminant(
                        &materials[prev_board
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::Solid {
                        melting_point: 0_f32,
                    })
                {
                    continue;
                } else if prev_board
                    .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                    .is_some()
                    && materials[prev_board
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .unwrap_or(&prev_board[get_i(width, (i,j))])
                        .material_id]
                        .1
                        .density
                        <= materials[prev_board[get_i(width, (i,j))].material_id].1.density
                    && (std::mem::discriminant(
                        &materials[prev_board
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    ) || std::mem::discriminant(
                        &materials[prev_board
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Liquid {
                            viscosity: 0_f32,
                            melting_point: 0_f32,
                            boiling_point: 0_f32,
                        }),
                    ))
                {
                    prev_board.swap(
                        (i, j),
                        (i, j.wrapping_add((orientation.signum() * _k) as usize)),
                    );
                    prev_board[(i, j.wrapping_add((orientation.signum() * _k) as usize))]
                        .updated = true;
                }
            }
            // This marks that the particle's position has been calculated
            prev_board[get_i(width, (i,j))].updated = true;
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // PLASMA PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Plasma => {
            let cellenergy = prev_board[get_i(width, (i,j))].energy;
            if cellenergy > 1_f32 {
                prev_board[get_i(width, (i,j))].material_id = 7_usize;
                prev_board[get_i(width, (i,j))].energy -= 1_f32;
            } else {
                prev_board[get_i(width, (i,j))].material_id = 0_usize;
                prev_board[get_i(width, (i,j))].display_color = VOID.material_color.color;
            }

            // Rng determines which side should the particle fall
            let mut orientation: i32 = 0;
            // This calculates the position on the Y axis
            if prev_board[get_i(width, (i,j))].speed.y.abs() > 1_f32 {
                prev_board[get_i(width, (i,j))].speed.y = 0.0;
            } else {
                // Rand range: (-1_f32..1_f32)
                let rnd = rngs[(i, j)];
                prev_board[get_i(width, (i,j))].speed.y += rnd.signum() * (rnd.abs() / 2_f32);
                orientation = (prev_board[get_i(width, (i,j))].speed.y.signum()
                    * (prev_board[get_i(width, (i,j))].speed.y.abs() + 1_f32))
                    as i32;
            }

            for _k in 0..orientation.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if prev_board
                    .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                    .is_some()
                    && std::mem::discriminant(
                        &materials[prev_board
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::Solid {
                        melting_point: 0_f32,
                    })
                {
                    continue;
                } else if prev_board
                    .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                    .is_some()
                    && (materials[prev_board
                        .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                        .unwrap_or(&prev_board[get_i(width, (i,j))])
                        .material_id]
                        .1
                        .density
                        <= materials[prev_board[get_i(width, (i,j))].material_id].1.density)
                    && (std::mem::discriminant(
                        &materials[prev_board
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    ) || std::mem::discriminant(
                        &materials[prev_board
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(&Phase::Liquid {
                        viscosity: 0_f32,
                        melting_point: 0_f32,
                        boiling_point: 0_f32,
                    }))
                {
                    prev_board.swap(
                        (i, j),
                        (i.wrapping_add((orientation.signum() * _k) as usize), j),
                    );
                    prev_board[(i.wrapping_add((orientation.signum() * _k) as usize), j)]
                        .updated = true;
                }
            }
            orientation = 0;
            // This calculates the position on the X axis
            if prev_board[get_i(width, (i,j))].speed.x.abs() > 1_f32 {
                prev_board[get_i(width, (i,j))].speed.x = 0.0;
            } else {
                let rnd = rngs[(i, j)] * self.seeds[(i, j)];
                prev_board[get_i(width, (i,j))].speed.x += rnd.signum() * (rnd.abs());
                orientation = (prev_board[get_i(width, (i,j))].speed.x.signum()
                    * (prev_board[get_i(width, (i,j))].speed.x.abs() + 1_f32))
                    as i32;
            }

            for _k in 0..orientation.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if prev_board
                    .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                    .is_some()
                    && std::mem::discriminant(
                        &materials[prev_board
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::Solid {
                        melting_point: 0_f32,
                    })
                {
                    continue;
                } else if prev_board
                    .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                    .is_some()
                    && materials[prev_board
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .unwrap_or(&prev_board[get_i(width, (i,j))])
                        .material_id]
                        .1
                        .density
                        <= materials[prev_board[get_i(width, (i,j))].material_id].1.density
                    && (std::mem::discriminant(
                        &materials[prev_board
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    ) || std::mem::discriminant(
                        &materials[prev_board
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&prev_board[get_i(width, (i,j))])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Liquid {
                            viscosity: 0_f32,
                            melting_point: 0_f32,
                            boiling_point: 0_f32,
                        }),
                    ))
                {
                    prev_board.swap(
                        (i, j),
                        (i, j.wrapping_add((orientation.signum() * _k) as usize)),
                    );
                    prev_board[(i, j.wrapping_add((orientation.signum() * _k) as usize))]
                        .updated = true;
                }
            }
            // This marks that the particle's position has been calculated
            prev_board[get_i(width, (i,j))].updated = true;
        }
    }
}*/
