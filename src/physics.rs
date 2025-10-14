use std::mem::discriminant;

use crate::world::{Board, VOID};
use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Phase {
    Void,
    Solid { melting_point: f32 },
    Powder { coarseness: f32, melting_point: f32 },     // Coarseness is the average diameter of a powder particle (between 0 and 1) (in cm), -> the smaller the diameter, the powder becomes more "clumpier"
    Liquid { viscosity: f32, boiling_point: f32 },      // Viscosity gives the rate, which the liquid spreads, for e.g. water has a viscosity of 1.0, the bigger the viscosity, the thicker the fluid is
    Gas {},                                             // Not fully implemented
    Plasma { energy: f32 },
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
            boiling_point: _,
        } = self
        {
            returnval = 1.0 / *viscosity
        };
        returnval
    }
    fn get_energy(&self) -> f32 {
        let mut returnval: f32 = 0.0;
        if let Phase::Plasma { energy } = self {
            returnval = *energy;
        };
        returnval
    }
}

impl Board {
    #[inline(always)]
    pub(crate) fn solve_particle(&mut self, i: usize, j: usize, framedelta: f32, framecount: u64) {
        match self.contents[(i, j)].material.phase {
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
                    if self.contents[(i, j)].material.density
                        > self
                            .contents
                            .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material
                            .density
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
                            &self
                                .contents
                                .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) == std::mem::discriminant(
                            &(Phase::Solid {
                                melting_point: 0_f32,
                            }),
                        )
                        || std::mem::discriminant(
                            &self
                                .contents
                                .get(
                                    i + ((self.gravity.signum() as i32 * _k)
                                        + self.gravity.signum() as i32)
                                        as usize,
                                    j,
                                )
                                .unwrap_or(&self.contents[(i, j)])
                                .material
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
                    && self
                        .contents
                        .get(
                            i + (self.gravity.signum() as i32) as usize,
                            j.wrapping_add(1),
                        )
                        .unwrap_or(&self.contents[(i, j)])
                        .material
                        .density
                        < self.contents[(i, j)].material.density
                    && std::mem::discriminant(
                        &self
                            .contents
                            .get(
                                i + (self.gravity.signum() as i32) as usize,
                                j.wrapping_add(1),
                            )
                            .unwrap_or(&self.contents[(i, j)])
                            .material
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    && self.contents[(i,j)].temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - self.contents[(i,j)]
                                .material
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                    && rnd.signum() == -1.0
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
                    && self
                        .contents
                        .get(
                            i + (self.gravity.signum() as i32) as usize,
                            j.wrapping_sub(1),
                        )
                        .unwrap_or(&self.contents[(i, j)])
                        .material
                        .density
                        < self.contents[(i, j)].material.density
                    && discriminant(
                        &self
                            .contents
                            .get(
                                i + (self.gravity.signum() as i32) as usize,
                                j.wrapping_sub(1),
                            )
                            .unwrap_or(&self.contents[(i, j)])
                            .material
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    && self.contents[(i,j)].temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - self.contents[(i,j)]
                                .material
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                    && rnd.signum() == 1.0
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
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            // LIQUID PHYSICS
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            Phase::Liquid {
                viscosity: _,
                boiling_point: _,
            } => {
                // Gravity simulation
                self.contents[(i, j)].speed.y += self.gravity * framedelta;
                let mut ychange = 0;
                for _k in 0..self.contents[(i, j)].speed.y.abs() as i32 {
                    // Falling and checking if there is a particle with a larger density
                    if (self.contents[(i, j)].material.density
                        > self
                            .contents
                            .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material
                            .density
                        && self.contents[(i, j)].updated
                        && discriminant(
                            &self
                                .contents
                                .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) != discriminant(&Phase::Solid {
                            melting_point: 0_f32,
                        }))
                        || (discriminant(
                            &self
                                .contents
                                .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) == discriminant(&Phase::Solid {
                            melting_point: 0_f32,
                        }) && self.contents[(i, j)].updated)
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
                            &self
                                .contents
                                .get(i + (self.gravity.signum() as i32 * _k) as usize, j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) == discriminant(
                            &(Phase::Solid {
                                melting_point: 0_f32,
                            }),
                        )
                        || discriminant(
                            &self
                                .contents
                                .get(
                                    i + (self.gravity.signum() as i32 * _k
                                        + self.gravity.signum() as i32)
                                        as usize,
                                    j,
                                )
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) == discriminant(
                            &(Phase::Powder {
                                coarseness: 0_f32,
                                melting_point: 0_f32,
                            }),
                        )
                        || discriminant(
                            &self
                                .contents
                                .get(
                                    i + (self.gravity.signum() as i32 * _k
                                        + self.gravity.signum() as i32)
                                        as usize,
                                    j,
                                )
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) == discriminant(
                            &(Phase::Liquid {
                                viscosity: 0_f32,
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
                if self.contents[(i, j)].speed.x.abs() > 1.0 {
                    self.contents[(i, j)].speed.x = 0.0;
                } else {
                    let rnd = self.rngs[(i, j)];
                    if rnd.abs()
                        >= (1_f32
                            - self.contents[(i, j)]
                                .material
                                .phase
                                .get_viscosity()
                                .sqrt()
                                .sqrt()
                                .sqrt())
                        .powi(16)
                    {
                        self.contents[(i, j)].speed.x += rnd.signum()
                            * (rnd.abs() + self.contents[(i, j)].material.phase.get_viscosity())
                                .powi(4);
                        orientation = (self.contents[(i, j)].speed.x.signum()
                            * (self.contents[(i, j)].speed.x.abs() + 1.0))
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
                            &self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material
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
                        && self
                            .contents
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&self.contents[(i, j)])
                            .material
                            .density
                            <= self.contents[(i, j)].material.density
                        && (discriminant(
                            &self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material
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
                        self.contents[(i, j)].speed.x *= -1.0;
                        break;
                    }
                }
                // This marks that the particle's position has been calculated
                self.contents[(i, j)].updated = true;
            }
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            // GAS PHYSICS
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            Phase::Gas {} => {
                // Rng determines which side should the particle fall
                let mut orientation: i32 = 0;
                // This calculates the position on the Y axis
                if self.contents[(i, j)].speed.y.abs() > 1.0 {
                    self.contents[(i, j)].speed.y = 0.0;
                } else {
                    // Rand range: (-1.0..1.0)
                    let rnd = self.rngs[(i, j)];
                    self.contents[(i, j)].speed.y += rnd.signum() * (rnd.abs() / 2_f32);
                    orientation = (self.contents[(i, j)].speed.y.signum()
                        * (self.contents[(i, j)].speed.y.abs() + 1.0))
                        as i32;
                }

                for _k in 0..orientation.abs() {
                    // This condition checks, whether the particle can fall to the determined side
                    if self
                        .contents
                        .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                        .is_some()
                        && std::mem::discriminant(
                            &self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
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
                        && (self
                            .contents
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material
                            .density
                            <= self.contents[(i, j)].material.density)
                        && (std::mem::discriminant(
                            &self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Powder {
                                coarseness: 0_f32,
                                melting_point: 0_f32,
                            }),
                        ) || std::mem::discriminant(
                            &self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) != std::mem::discriminant(&Phase::Liquid {
                            viscosity: 0_f32,
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
                if self.contents[(i, j)].speed.x.abs() > 1.0 {
                    self.contents[(i, j)].speed.x = 0.0;
                } else {
                    let rnd = self.rngs[(i, j)] * self.seeds[(i, j)];
                    self.contents[(i, j)].speed.x += rnd.signum() * (rnd.abs());
                    orientation = (self.contents[(i, j)].speed.x.signum()
                        * (self.contents[(i, j)].speed.x.abs() + 1.0))
                        as i32;
                }

                for _k in 0..orientation.abs() {
                    // This condition checks, whether the particle can fall to the determined side
                    if self
                        .contents
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .is_some()
                        && std::mem::discriminant(
                            &self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material
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
                        && self
                            .contents
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&self.contents[(i, j)])
                            .material
                            .density
                            <= self.contents[(i, j)].material.density
                        && (std::mem::discriminant(
                            &self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Powder {
                                coarseness: 0_f32,
                                melting_point: 0_f32,
                            }),
                        ) || std::mem::discriminant(
                            &self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Liquid {
                                viscosity: 0_f32,
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
            Phase::Plasma { energy: _f32 } => {
                let cellenergy = self.contents[(i, j)].material.phase.get_energy();
                if cellenergy > 1.0 {
                    self.contents[(i, j)].material.phase = Phase::Plasma {
                        energy: cellenergy - 1.0,
                    };
                } else {
                    self.contents[(i, j)].material = VOID.clone();
                    self.contents[(i, j)].display_color = VOID.material_color.color;
                }

                // Rng determines which side should the particle fall
                let mut orientation: i32 = 0;
                // This calculates the position on the Y axis
                if self.contents[(i, j)].speed.y.abs() > 1.0 {
                    self.contents[(i, j)].speed.y = 0.0;
                } else {
                    // Rand range: (-1.0..1.0)
                    let rnd = self.rngs[(i, j)];
                    self.contents[(i, j)].speed.y += rnd.signum() * (rnd.abs() / 2_f32);
                    orientation = (self.contents[(i, j)].speed.y.signum()
                        * (self.contents[(i, j)].speed.y.abs() + 1.0))
                        as i32;
                }

                for _k in 0..orientation.abs() {
                    // This condition checks, whether the particle can fall to the determined side
                    if self
                        .contents
                        .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                        .is_some()
                        && std::mem::discriminant(
                            &self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
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
                        && (self
                            .contents
                            .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                            .unwrap_or(&self.contents[(i, j)])
                            .material
                            .density
                            <= self.contents[(i, j)].material.density)
                        && (std::mem::discriminant(
                            &self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Powder {
                                coarseness: 0_f32,
                                melting_point: 0_f32,
                            }),
                        ) || std::mem::discriminant(
                            &self
                                .contents
                                .get(i.wrapping_add((orientation.signum() * _k) as usize), j)
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) != std::mem::discriminant(&Phase::Liquid {
                            viscosity: 0_f32,
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
                if self.contents[(i, j)].speed.x.abs() > 1.0 {
                    self.contents[(i, j)].speed.x = 0.0;
                } else {
                    // Rand range: (-1.0..1.0)
                    let rnd = self.rngs[(i, j)] * self.seeds[(i, j)];
                    self.contents[(i, j)].speed.x += rnd.signum() * (rnd.abs());
                    orientation = (self.contents[(i, j)].speed.x.signum()
                        * (self.contents[(i, j)].speed.x.abs() + 1.0))
                        as i32;
                }

                for _k in 0..orientation.abs() {
                    // This condition checks, whether the particle can fall to the determined side
                    if self
                        .contents
                        .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                        .is_some()
                        && std::mem::discriminant(
                            &self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material
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
                        && self
                            .contents
                            .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                            .unwrap_or(&self.contents[(i, j)])
                            .material
                            .density
                            <= self.contents[(i, j)].material.density
                        || (std::mem::discriminant(
                            &self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Powder {
                                coarseness: 0_f32,
                                melting_point: 0_f32,
                            }),
                        ) || std::mem::discriminant(
                            &self
                                .contents
                                .get(i, j.wrapping_add((orientation.signum() * _k) as usize))
                                .unwrap_or(&self.contents[(i, j)])
                                .material
                                .phase,
                        ) != std::mem::discriminant(
                            &(Phase::Liquid {
                                viscosity: 0_f32,
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
        }
    }
}
