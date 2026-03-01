use crate::{
    material::{Material, VOID},
    particle::{AtomicParticle, Particle},
    world::{
        AtomicComparedSlice, Board, get_i, swap_particle, write_particle, write_speed_field,
        write_updated_field,
    },
};
use crossbeam::epoch::Atomic;
use egui::{Vec2, vec2};
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
pub fn solve_particle(
    slice_board: &AtomicComparedSlice<Particle>,
    check_board: &Arc<Vec<AtomicParticle>>,
    materials: &Vec<(String, Material)>,
    rngs: &Vec<f32>,
    width: u16,
    i: usize,
    j: usize,
    gravity: f32,
    framedelta: f32,
) {
        match &materials[self.contents[(i, j)].material_id].1.phase {
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
                self.contents[(i, j)].speed.y += self.gravity * framedelta;
                let mut ychange = 0;
                for _k in 0..self.contents[(i, j)].speed.y.abs() as i32 {
                    // Falling and checking if there is a particle with a larger density
                    if materials[self.contents[(i, j)].material_id].1.density
                        > materials[self
                            .contents
                            .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .density
                        && std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Solid {
                                melting_point: 0_f32,
                            }),
                        )
                        && self.contents[(i, j)].updated
                    {
                        ychange = _k;
                    }
                    // Checks if the particle falls inside bounds
                    // Checks, whether there is another denser particle in the path of the falling particle
                    else if self
                        .contents
                        .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                        .is_none()
                        || std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) == std::mem::discriminant(
                            &(Phase::Solid {
                                melting_point: 0_f32,
                            }),
                        )
                        || std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(
                                    i + ((self.gravity.signum() as i32 * _k)
                                        + self.gravity.signum() as i32)
                                        as usize,
                                    j,
                                )
                                .unwrap_or(&self.contents[(i, j)])
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
                        self.contents[(i, j)].speed.y -= self.gravity * framedelta;
                        break;
                    }
                }
                if ychange != 0 {
                    self.contents.swap(
                        (i, j),
                        (i + ((self.gravity.signum() as i32 * ychange) as usize), j),
                    );
                    self.contents[(i + ((self.gravity.signum() as i32 * ychange) as usize), j)]
                        .updated = false;
                }
                // This decides where the particle falls (left or right)
                let rnd = self.rngs[(i, j)];
                if self.contents[(i, j)].updated
                    && self
                        .contents
                        .get(
                            i + (self.gravity.signum() as i32) as usize,
                            j.wrapping_add(1),
                        )
                        .is_some()
                    && materials[self
                        .contents
                        .get(
                            i + (self.gravity.signum() as i32) as usize,
                            j.wrapping_add(1),
                        )
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .density
                        < materials[self.contents[(i, j)].material_id].1.density
                    && std::mem::discriminant(
                        &materials[self
                            .contents
                            .get(
                                i + (self.gravity.signum() as i32) as usize,
                                j.wrapping_add(1),
                            )
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    && self.contents[(i,j)].temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - materials[self.contents[(i,j)]
                                .material_id].1
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                    && rnd.signum() == -1_f32
                {
                    self.contents.swap(
                        (i, j),
                        (
                            i + (self.gravity.signum() as i32) as usize,
                            j.wrapping_add(1),
                        ),
                    );
                    self.contents[(
                        i + (self.gravity.signum() as i32) as usize,
                        j.wrapping_add(1),
                    )]
                        .updated = false;
                }
                if self.contents[(i, j)].updated
                    && self
                        .contents
                        .get(
                            i + (self.gravity.signum() as i32) as usize,
                            j.wrapping_sub(1),
                        )
                        .is_some()
                    && materials[self
                        .contents
                        .get(
                            i + (self.gravity.signum() as i32) as usize,
                            j.wrapping_sub(1),
                        )
                        .unwrap_or(&self.contents[(i, j)])
                        .material_id]
                        .1
                        .density
                        < materials[self.contents[(i, j)].material_id].1.density
                    && discriminant(
                        &materials[self
                            .contents
                            .get(
                                i + (self.gravity.signum() as i32) as usize,
                                j.wrapping_sub(1),
                            )
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    && self.contents[(i,j)].temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - materials[self.contents[(i,j)]
                                .material_id].1
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                    && rnd.signum() == 1_f32
                {
                    self.contents.swap(
                        (i, j),
                        (
                            i + (self.gravity.signum() as i32) as usize,
                            j.wrapping_sub(1),
                        ),
                    );
                    self.contents[(
                        i + (self.gravity.signum() as i32) as usize,
                        j.wrapping_sub(1),
                    )]
                        .updated = false;
                }
                // This marks that the particle's position has been calculated
                self.contents[(i, j)].updated = true;
            }
            _ => {}

            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            // LIQUID PHYSICS
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            /*Phase::Liquid {
                viscosity: _,
                melting_point: _,
                boiling_point: _,
            } => {
                // Gravity simulation
                self.contents[(i, j)].speed.y += self.gravity * framedelta;
                let mut ychange = 0;
                for _k in 0..self.contents[(i, j)].speed.y.abs() as i32 {
                    // Falling and checking if there is a particle with a larger density
                    if materials[self.contents[(i, j)].material_id].1.density
                        > materials[self
                            .contents
                            .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .density
                        && self.contents[(i, j)].updated
                        && discriminant(
                            &materials[self
                                .contents
                                .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                                .unwrap_or(&self.contents[(i, j)])
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
                    else if self
                        .contents
                        .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                        .is_none()
                        && discriminant(
                            &materials[self
                                .contents
                                .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) == discriminant(
                            &(Phase::Solid {
                                melting_point: 0_f32,
                            }),
                        )
                        || discriminant(
                            &materials[self
                                .contents
                                .get(
                                    i + (self.gravity.signum() as i32 * _k
                                        + self.gravity.signum() as i32)
                                        as usize,
                                    j,
                                )
                                .unwrap_or(&self.contents[(i, j)])
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
                            &materials[self
                                .contents
                                .get(
                                    i + (self.gravity.signum() as i32 * _k
                                        + self.gravity.signum() as i32)
                                        as usize,
                                    j,
                                )
                                .unwrap_or(&self.contents[(i, j)])
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
                        self.contents[(i, j)].speed.y -= self.gravity * framedelta;
                        break;
                    }
                }
                if ychange != 0 {
                    self.contents.swap(
                        (i, j),
                        (i + (self.gravity.signum() as i32 * ychange) as usize, j),
                    );
                    self.contents[(i + ((self.gravity.signum() as i32 * ychange) as usize), j)]
                        .updated = false;
                }
                // Rng determines which side should the particle fall
                let mut orientation: i32 = 0;
                if self.contents[(i, j)].speed.x.abs() > 1_f32 {
                    self.contents[(i, j)].speed.x = 0.0;
                } else {
                    let rnd = self.rngs[(i, j)];
                    if rnd.abs()
                        >= (1_f32
                            - materials[self.contents[(i, j)].material_id]
                                .1
                                .phase
                                .get_viscosity())
                        .powi(16)
                    {
                        self.contents[(i, j)].speed.x += rnd.signum()
                            * (rnd.abs()
                                + materials[self.contents[(i, j)].material_id]
                                    .1
                                    .phase
                                    .get_viscosity()
                                    .sqrt());
                        orientation = (self.contents[(i, j)].speed.x.signum()
                            * (self.contents[(i, j)].speed.x.abs() + 1_f32))
                            as i32;
                    }
                }

                for _k in 0..orientation.abs() {
                    // This condition checks, whether the particle can fall to the determined side
                    if self
                        .contents
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .is_some()
                        && discriminant(
                            &materials[self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
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
                    } else if self
                        .contents
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .is_some()
                        && materials[self
                            .contents
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .density
                            <= materials[self.contents[(i, j)].material_id].1.density
                        && (discriminant(
                            &materials[self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
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
                        self.contents.swap(
                            (i, j),
                            (i, j.wrapping_add((orientation.signum() * _k) as usize)),
                        );
                        self.contents[(i, j.wrapping_add((orientation.signum() * _k) as usize))]
                            .updated = true;
                    } else {
                        self.contents[(i, j)].speed.x *= -1_f32;
                        break;
                    }
                }
                // This marks that the particle's position has been calculated
                self.contents[(i, j)].updated = true;
            }
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            // GAS PHYSICS
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            Phase::Gas { boiling_point: _ } => {
                // Rng determines which side should the particle fall
                let mut orientation: i32 = 0;
                // This calculates the position on the Y axis
                if self.contents[(i, j)].speed.y.abs() > 1_f32 {
                    self.contents[(i, j)].speed.y = 0.0;
                } else {
                    // Rand range: (-1_f32..1_f32)
                    let rnd = self.rngs[(i, j)];
                    self.contents[(i, j)].speed.y += rnd.signum() * (rnd.abs() / 2_f32);
                    orientation = (self.contents[(i, j)].speed.y.signum()
                        * (self.contents[(i, j)].speed.y.abs() + 1_f32))
                        as i32;
                }

                for _k in 0..orientation.abs() {
                    // This condition checks, whether the particle can fall to the determined side
                    if self
                        .contents
                        .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                        .is_some()
                        && std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) == std::mem::discriminant(&Phase::Solid {
                            melting_point: 0_f32,
                        })
                    {
                        continue;
                    } else if self
                        .contents
                        .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                        .is_some()
                        && (materials[self
                            .contents
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .density
                            <= materials[self.contents[(i, j)].material_id].1.density)
                        && (std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Powder {
                                coarseness: 0_f32,
                                melting_point: 0_f32,
                            }),
                        ) || std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) != std::mem::discriminant(&Phase::Liquid {
                            viscosity: 0_f32,
                            melting_point: 0_f32,
                            boiling_point: 0_f32,
                        }))
                    {
                        self.contents.swap(
                            (i, j),
                            (i.wrapping_add((orientation.signum() * _k) as usize), j),
                        );
                        self.contents[(i.wrapping_add((orientation.signum() * _k) as usize), j)]
                            .updated = true;
                    }
                }
                orientation = 0;
                // This calculates the position on the X axis
                if self.contents[(i, j)].speed.x.abs() > 1_f32 {
                    self.contents[(i, j)].speed.x = 0.0;
                } else {
                    let rnd = self.rngs[(i, j)] * self.seeds[(i, j)];
                    self.contents[(i, j)].speed.x += rnd.signum() * (rnd.abs());
                    orientation = (self.contents[(i, j)].speed.x.signum()
                        * (self.contents[(i, j)].speed.x.abs() + 1_f32))
                        as i32;
                }

                for _k in 0..orientation.abs() {
                    // This condition checks, whether the particle can fall to the determined side
                    if self
                        .contents
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .is_some()
                        && std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) == std::mem::discriminant(&Phase::Solid {
                            melting_point: 0_f32,
                        })
                    {
                        continue;
                    } else if self
                        .contents
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .is_some()
                        && materials[self
                            .contents
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .density
                            <= materials[self.contents[(i, j)].material_id].1.density
                        && (std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Powder {
                                coarseness: 0_f32,
                                melting_point: 0_f32,
                            }),
                        ) || std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
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
                        self.contents.swap(
                            (i, j),
                            (i, j.wrapping_add((orientation.signum() * _k) as usize)),
                        );
                        self.contents[(i, j.wrapping_add((orientation.signum() * _k) as usize))]
                            .updated = true;
                    }
                }
                // This marks that the particle's position has been calculated
                self.contents[(i, j)].updated = true;
            }
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            // PLASMA PHYSICS
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            Phase::Plasma => {
                let cellenergy = self.contents[(i, j)].energy;
                if cellenergy > 1_f32 {
                    self.contents[(i, j)].material_id = 7_usize;
                    self.contents[(i, j)].energy -= 1_f32;
                } else {
                    self.contents[(i, j)].material_id = 0_usize;
                    self.contents[(i, j)].display_color = VOID.material_color.color;
                }

                // Rng determines which side should the particle fall
                let mut orientation: i32 = 0;
                // This calculates the position on the Y axis
                if self.contents[(i, j)].speed.y.abs() > 1_f32 {
                    self.contents[(i, j)].speed.y = 0.0;
                } else {
                    // Rand range: (-1_f32..1_f32)
                    let rnd = self.rngs[(i, j)];
                    self.contents[(i, j)].speed.y += rnd.signum() * (rnd.abs() / 2_f32);
                    orientation = (self.contents[(i, j)].speed.y.signum()
                        * (self.contents[(i, j)].speed.y.abs() + 1_f32))
                        as i32;
                }

                for _k in 0..orientation.abs() {
                    // This condition checks, whether the particle can fall to the determined side
                    if self
                        .contents
                        .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                        .is_some()
                        && std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) == std::mem::discriminant(&Phase::Solid {
                            melting_point: 0_f32,
                        })
                    {
                        continue;
                    } else if self
                        .contents
                        .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                        .is_some()
                        && (materials[self
                            .contents
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .density
                            <= materials[self.contents[(i, j)].material_id].1.density)
                        && (std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Powder {
                                coarseness: 0_f32,
                                melting_point: 0_f32,
                            }),
                        ) || std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) != std::mem::discriminant(&Phase::Liquid {
                            viscosity: 0_f32,
                            melting_point: 0_f32,
                            boiling_point: 0_f32,
                        }))
                    {
                        self.contents.swap(
                            (i, j),
                            (i.wrapping_add((orientation.signum() * _k) as usize), j),
                        );
                        self.contents[(i.wrapping_add((orientation.signum() * _k) as usize), j)]
                            .updated = true;
                    }
                }
                orientation = 0;
                // This calculates the position on the X axis
                if self.contents[(i, j)].speed.x.abs() > 1_f32 {
                    self.contents[(i, j)].speed.x = 0.0;
                } else {
                    let rnd = self.rngs[(i, j)] * self.seeds[(i, j)];
                    self.contents[(i, j)].speed.x += rnd.signum() * (rnd.abs());
                    orientation = (self.contents[(i, j)].speed.x.signum()
                        * (self.contents[(i, j)].speed.x.abs() + 1_f32))
                        as i32;
                }

                for _k in 0..orientation.abs() {
                    // This condition checks, whether the particle can fall to the determined side
                    if self
                        .contents
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .is_some()
                        && std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) == std::mem::discriminant(&Phase::Solid {
                            melting_point: 0_f32,
                        })
                    {
                        continue;
                    } else if self
                        .contents
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .is_some()
                        && materials[self
                            .contents
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&self.contents[(i, j)])
                            .material_id]
                            .1
                            .density
                            <= materials[self.contents[(i, j)].material_id].1.density
                        && (std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material_id]
                                .1
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Powder {
                                coarseness: 0_f32,
                                melting_point: 0_f32,
                            }),
                        ) || std::mem::discriminant(
                            &materials[self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
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
                        self.contents.swap(
                            (i, j),
                            (i, j.wrapping_add((orientation.signum() * _k) as usize)),
                        );
                        self.contents[(i, j.wrapping_add((orientation.signum() * _k) as usize))]
                            .updated = true;
                    }
                }
                // This marks that the particle's position has been calculated
                self.contents[(i, j)].updated = true;
            }*/
        }
    }
