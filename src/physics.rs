use crate::{
    material::{Material, tuple_to_rangeinclusive},
    particle::{AtomicParticle, Particle},
    world::{
        AtomicComparedSlice, get_safe_i, swap_particle, write_particle, write_updated_field,
        write_x_speed_field, write_y_speed_field,
    },
};
use ahash::AHashMap;
use egui::lerp;
use serde::{Deserialize, Serialize};
use std::{mem::discriminant, sync::Arc};

pub struct PhysicalReactions {
    pub(crate) melting: AHashMap<usize, usize>,
    pub(crate) boiling: AHashMap<usize, usize>,
    pub(crate) sublimation: AHashMap<usize, usize>,
    pub(crate) ionization: AHashMap<usize, usize>,
}

impl PhysicalReactions {
    pub fn new(
        melting: AHashMap<usize, usize>,
        boiling: AHashMap<usize, usize>,
        sublimation: AHashMap<usize, usize>,
        ionization: AHashMap<usize, usize>,
    ) -> Self {
        Self {
            melting,
            boiling,
            sublimation,
            ionization,
        }
    }
}

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
    fn get_melting_point_sld(&self) -> f32 {
        let mut returnval: f32 = 0.0;
        if let Phase::Solid { melting_point } = self {
            returnval = *melting_point
        };
        returnval
    }
    fn get_melting_point_pwdr(&self) -> f32 {
        let mut returnval: f32 = 0.0;
        if let Phase::Powder {
            coarseness: _,
            melting_point,
        } = self
        {
            returnval = *melting_point
        };
        returnval
    }
    fn get_melting_point_liqd(&self) -> f32 {
        let mut returnval: f32 = 0.0;
        if let Phase::Liquid {
            viscosity: _,
            melting_point,
            boiling_point: _,
        } = self
        {
            returnval = *melting_point
        };
        returnval
    }
    fn get_boiling_point_gas(&self) -> f32 {
        let mut returnval: f32 = 0.0;
        if let Phase::Gas { boiling_point } = self {
            returnval = *boiling_point;
        };
        returnval
    }
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
    physical_transitions: &PhysicalReactions,
    rngs: &Vec<f32>,
    seeds: &Vec<f32>,
    height: &usize,
    width: &usize,
    i: usize,
    j: usize,
    gravity: f32,
    framedelta: f32,
) {
    match &materials[slice_board
        .get_elem(get_safe_i(height, width, &(i, j)))
        .material_id]
        .1
        .phase
    {
        Phase::Void => {}

        Phase::Solid { melting_point } => {
            let current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
            if *melting_point < current_particle.temperature {
                let mut new_particle = *current_particle;
                new_particle.material_id = *physical_transitions
                    .melting
                    .get(&current_particle.material_id)
                    .unwrap_or(&current_particle.material_id);
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
                        rngs[get_safe_i(height, width, &(i, j))],
                    ));
                new_particle.display_color[3] = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .a();
                unsafe {
                    write_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        new_particle,
                        check_board,
                    )
                };
            }
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // POWDER PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Powder {
            coarseness: _,
            melting_point: _,
        } => {
            // Gravity simulation
            unsafe {
                write_y_speed_field(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .speed
                        .y
                        + (gravity * framedelta),
                    check_board,
                )
            };
            let mut ychange = 0;
            for _k in 0..slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .speed
                .y
                .abs() as i32
            {
                // Falling and checking if there is a particle with a larger density
                if materials[slice_board
                    .get_elem(get_safe_i(height, width, &(i, j)))
                    .material_id]
                    .1
                    .density
                    > materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i + (gravity.signum() as i32 * _k) as usize, j),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .density
                    && std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i + (gravity.signum() as i32 * _k) as usize, j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    && slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .updated
                {
                    ychange = _k;
                }
                // Checks if the particle falls inside bounds
                // Checks, whether there is another denser particle in the path of the falling particle
                else if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i + (gravity.signum() as i32 * _k) as usize, j),
                    ))
                    .is_none()
                    || std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i + (gravity.signum() as i32 * _k) as usize, j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    || std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(
                                    i + ((gravity.signum() as i32 * _k) + gravity.signum() as i32)
                                        as usize,
                                    j,
                                ),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
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
                    unsafe {
                        write_y_speed_field(
                            slice_board,
                            get_safe_i(height, width, &(i, j)),
                            slice_board
                                .get_elem(get_safe_i(height, width, &(i, j)))
                                .speed
                                .y
                                - (gravity * framedelta),
                            check_board,
                        )
                    };
                    break;
                }
            }
            if ychange != 0 {
                unsafe {
                    swap_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        get_safe_i(
                            height,
                            width,
                            &(i + ((gravity.signum() as i32 * ychange) as usize), j),
                        ),
                        check_board,
                    );
                    write_updated_field(
                        slice_board,
                        get_safe_i(
                            height,
                            width,
                            &(i + ((gravity.signum() as i32 * ychange) as usize), j),
                        ),
                        false,
                        check_board,
                    );
                }
            }
            // This decides where the particle falls (left or right)
            let rnd = rngs[get_safe_i(height, width, &(i, j))];
            if slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .updated
                && slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i + (gravity.signum() as i32) as usize, j.wrapping_add(1)),
                    ))
                    .is_some()
                && materials[slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i + (gravity.signum() as i32) as usize, j.wrapping_add(1)),
                    ))
                    .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                    .material_id]
                    .1
                    .density
                    < materials[slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .material_id]
                        .1
                        .density
                && std::mem::discriminant(
                    &materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i + (gravity.signum() as i32) as usize, j.wrapping_add(1)),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .phase,
                ) != std::mem::discriminant(
                    &(Phase::Solid {
                        melting_point: 0_f32,
                    }),
                )
                && slice_board.get_elem(get_safe_i(height, width,&(i,j))).temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - materials[slice_board.get_elem(get_safe_i(height, width, &(i,j)))
                                .material_id].1
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                && rnd.signum() == -1_f32
            {
                unsafe {
                    swap_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        get_safe_i(
                            height,
                            width,
                            &(i + (gravity.signum() as i32) as usize, j.wrapping_add(1)),
                        ),
                        check_board,
                    );
                    write_updated_field(
                        slice_board,
                        get_safe_i(
                            height,
                            width,
                            &(i + (gravity.signum() as i32) as usize, j.wrapping_add(1)),
                        ),
                        false,
                        check_board,
                    );
                }
            }
            if slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .updated
                && slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i + (gravity.signum() as i32) as usize, j.wrapping_sub(1)),
                    ))
                    .is_some()
                && materials[slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i + (gravity.signum() as i32) as usize, j.wrapping_sub(1)),
                    ))
                    .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                    .material_id]
                    .1
                    .density
                    < materials[slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .material_id]
                        .1
                        .density
                && discriminant(
                    &materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i + (gravity.signum() as i32) as usize, j.wrapping_sub(1)),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .phase,
                ) != std::mem::discriminant(
                    &(Phase::Solid {
                        melting_point: 0_f32,
                    }),
                )
                && slice_board.get_elem(get_safe_i(height, width,&(i,j))).temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - materials[slice_board.get_elem(get_safe_i(height, width,&(i,j)))
                                .material_id].1
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                && rnd.signum() == 1_f32
            {
                unsafe {
                    swap_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        get_safe_i(
                            height,
                            width,
                            &(i + (gravity.signum() as i32) as usize, j.wrapping_sub(1)),
                        ),
                        check_board,
                    );
                    write_updated_field(
                        slice_board,
                        get_safe_i(
                            height,
                            width,
                            &(i + (gravity.signum() as i32) as usize, j.wrapping_sub(1)),
                        ),
                        false,
                        check_board,
                    )
                };
            }
            // This marks that the particle's position has been calculated
            unsafe {
                write_updated_field(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    true,
                    check_board,
                );
            }
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // LIQUID PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Liquid {
            viscosity: _,
            melting_point,
            boiling_point,
        } => {
            // Phase change from liquid to solid and liquid to gas
            let current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
            if *melting_point > current_particle.temperature {
                let mut new_particle = *current_particle;
                new_particle.material_id = *physical_transitions
                    .melting
                    .get(&current_particle.material_id)
                    .unwrap_or(&current_particle.material_id);
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
                        rngs[get_safe_i(height, width, &(i, j))],
                    ));
                new_particle.display_color[3] = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .a();
                unsafe {
                    write_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        new_particle,
                        check_board,
                    )
                };
            } else if *boiling_point < current_particle.temperature {
                let mut new_particle = *current_particle;
                new_particle.material_id = *physical_transitions
                    .boiling
                    .get(&current_particle.material_id)
                    .unwrap_or(&current_particle.material_id);
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
                        rngs[get_safe_i(height, width, &(i, j))],
                    ));
                new_particle.display_color[3] = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .a();
                unsafe {
                    write_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        new_particle,
                        check_board,
                    )
                };
            }
            // Gravity simulation
            unsafe {
                write_y_speed_field(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .speed
                        .y
                        + (gravity * framedelta),
                    check_board,
                )
            };
            let mut ychange = 0;
            for _k in 0..slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .speed
                .y
                .abs() as i32
            {
                // Falling and checking if there is a particle with a larger density
                if materials[slice_board
                    .get_elem(get_safe_i(height, width, &(i, j)))
                    .material_id]
                    .1
                    .density
                    > materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i + (gravity.signum() as i32 * _k) as usize, j),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .density
                    && slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .updated
                    && discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i + (gravity.signum() as i32 * _k) as usize, j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
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
                else if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i + (gravity.signum() as i32 * _k) as usize, j),
                    ))
                    .is_none()
                    && discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i + (gravity.signum() as i32 * _k) as usize, j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) == discriminant(
                        &(Phase::Solid {
                            melting_point: 0_f32,
                        }),
                    )
                    || discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(
                                    i + (gravity.signum() as i32 * _k + gravity.signum() as i32)
                                        as usize,
                                    j,
                                ),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
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
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(
                                    i + (gravity.signum() as i32 * _k + gravity.signum() as i32)
                                        as usize,
                                    j,
                                ),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
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
                    unsafe {
                        write_y_speed_field(
                            slice_board,
                            get_safe_i(height, width, &(i, j)),
                            slice_board
                                .get_elem(get_safe_i(height, width, &(i, j)))
                                .speed
                                .y
                                - (gravity * framedelta),
                            check_board,
                        )
                    };
                    break;
                }
            }
            if ychange != 0 {
                unsafe {
                    swap_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        get_safe_i(
                            height,
                            width,
                            &(i + (gravity.signum() as i32 * ychange) as usize, j),
                        ),
                        check_board,
                    );
                    write_updated_field(
                        slice_board,
                        get_safe_i(
                            height,
                            width,
                            &(i + ((gravity.signum() as i32 * ychange) as usize), j),
                        ),
                        false,
                        check_board,
                    );
                };
            }
            // Rng determines which side should the particle fall
            let mut orientation: i32 = 0;
            if slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .speed
                .x
                .abs()
                > 1_f32
            {
                unsafe {
                    write_x_speed_field(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        0.0_f32,
                        check_board,
                    );
                }
            } else {
                let rnd = rngs[get_safe_i(height, width, &(i, j))];
                if rnd.abs()
                    >= (1_f32
                        - materials[slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .material_id]
                            .1
                            .phase
                            .get_viscosity())
                    .powi(16)
                {
                    unsafe {
                        write_x_speed_field(
                            slice_board,
                            get_safe_i(height, width, &(i, j)),
                            slice_board
                                .get_elem(get_safe_i(height, width, &(i, j)))
                                .speed
                                .x
                                + (rnd.signum()
                                    * (rnd.abs()
                                        + materials[slice_board
                                            .get_elem(get_safe_i(height, width, &(i, j)))
                                            .material_id]
                                            .1
                                            .phase
                                            .get_viscosity()
                                            .sqrt())),
                            check_board,
                        );
                    }
                    orientation = (slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .speed
                        .x
                        .signum()
                        * (slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .speed
                            .x
                            .abs()
                            + 1_f32)) as i32;
                }
            }
            let mut xchange = 0_i32;
            for _k in 0_i32..orientation.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i, j.wrapping_add((orientation.signum() * _k) as usize)),
                    ))
                    .is_some()
                    && discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation.signum() * _k) as usize)),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
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
                } else if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i, j.wrapping_add((orientation.signum() * _k) as usize)),
                    ))
                    .is_some()
                    && materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i, j.wrapping_add((orientation.signum() * _k) as usize)),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .density
                        <= materials[slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .material_id]
                            .1
                            .density
                    && (discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation.signum() * _k) as usize)),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
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
                    xchange = _k;
                    continue;
                } else {
                    unsafe {
                        write_x_speed_field(
                            slice_board,
                            get_safe_i(height, width, &(i, j)),
                            -slice_board
                                .get_elem(get_safe_i(height, width, &(i, j)))
                                .speed
                                .x,
                            check_board,
                        );
                    }
                    break;
                }
            }
            unsafe {
                swap_particle(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    get_safe_i(
                        height,
                        width,
                        &(
                            i,
                            (j.wrapping_add((orientation.signum() * xchange) as usize)),
                        ),
                    ),
                    check_board,
                );
                write_updated_field(
                    slice_board,
                    get_safe_i(
                        height,
                        width,
                        &(i, j.wrapping_add((orientation.signum() * xchange) as usize)),
                    ),
                    true,
                    check_board,
                )
            };
            // This marks that the particle's position has been calculated
            unsafe {
                write_updated_field(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    true,
                    check_board,
                );
            }
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // GAS PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Gas { boiling_point } => {
            // Phase transition fromg as to liquid
            let current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
            if *boiling_point > current_particle.temperature {
                let mut new_particle = *current_particle;
                new_particle.material_id = *physical_transitions
                    .boiling
                    .get(&current_particle.material_id)
                    .unwrap_or(&current_particle.material_id);
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
                        rngs[get_safe_i(height, width, &(i, j))],
                    ));
                new_particle.display_color[3] = materials[new_particle.material_id]
                    .1
                    .material_color
                    .color
                    .a();
                unsafe {
                    write_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        new_particle,
                        check_board,
                    )
                };
            }
            // Rng determines which side should the particle fall
            let mut orientation_y: i32 = 0_i32;
            // This calculates the position on the Y axis
            if slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .speed
                .y
                .abs()
                > 1_f32
            {
                unsafe {
                    write_y_speed_field(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        0_f32,
                        check_board,
                    );
                }
            } else {
                // Rand range: (-1_f32..1_f32)
                let rnd = rngs[get_safe_i(height, width, &(i, j))];
                unsafe {
                    write_y_speed_field(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .speed
                            .y
                            + (rnd.signum() * (rnd.abs() / 2_f32)),
                        check_board,
                    );
                }
                orientation_y = (slice_board
                    .get_elem(get_safe_i(height, width, &(i, j)))
                    .speed
                    .y
                    .signum()
                    * (slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .speed
                        .y
                        .abs()
                        + 1_f32)) as i32;
            }
            let mut ychange = 0_i32;
            for k in 0_i32..orientation_y.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                    ))
                    .is_some()
                    && std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::Solid {
                        melting_point: 0_f32,
                    })
                {
                    continue;
                } else if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                    ))
                    .is_some()
                    && (materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .density
                        <= materials[slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .material_id]
                            .1
                            .density)
                    && (std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    ) || std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(&Phase::Liquid {
                        viscosity: 0_f32,
                        melting_point: 0_f32,
                        boiling_point: 0_f32,
                    }))
                {
                    ychange = k;
                    unsafe {
                        write_updated_field(
                            slice_board,
                            get_safe_i(
                                height,
                                width,
                                &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                            ),
                            true,
                            check_board,
                        )
                    };
                }
            }
            let mut orientation_x = 0;
            // This calculates the position on the X axis
            if slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .speed
                .x
                .abs()
                > 1_f32
            {
                unsafe {
                    write_x_speed_field(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        0_f32,
                        check_board,
                    );
                }
            } else {
                let rnd = rngs[get_safe_i(height, width, &(i, j))]
                    * seeds[get_safe_i(height, width, &(i, j))];
                unsafe {
                    write_x_speed_field(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .speed
                            .x
                            + (seeds[get_safe_i(height, width, &(i, j))].signum() * (rnd / 2_f32)),
                        check_board,
                    );
                };
                orientation_x = (rnd.signum()
                    * (slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .speed
                        .x
                        .abs()
                        + 1_f32)) as i32;
            }

            let mut xchange = 0_i32;
            for _k in 0..orientation_x.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                    ))
                    .is_some()
                    && std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::Solid {
                        melting_point: 0_f32,
                    })
                {
                    continue;
                } else if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                    ))
                    .is_some()
                    && materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .density
                        <= materials[slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .material_id]
                            .1
                            .density
                    && (std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    ) || std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
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
                    xchange = _k;
                    unsafe {
                        write_updated_field(
                            slice_board,
                            get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                            ),
                            true,
                            check_board,
                        );
                    }
                }
            }
            unsafe {
                swap_particle(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    get_safe_i(
                        height,
                        width,
                        &(
                            i.wrapping_add((orientation_y.signum() * ychange) as usize),
                            j.wrapping_add((orientation_x.signum() * xchange) as usize),
                        ),
                    ),
                    check_board,
                )
            }
            // This marks that the particle's position has been calculated
            unsafe {
                write_updated_field(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    true,
                    check_board,
                );
            }
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // PLASMA PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Plasma => {
            let _cellenergy = slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .energy;
            /*if cellenergy > 1_f32 {
                slice_board
                    .get_elem(get_safe_i(height, width, &(i, j)))
                    .material_id = 7_usize;
                slice_board
                    .get_elem(get_safe_i(height, width, &(i, j)))
                    .energy -= 1_f32;
            } else {
                slice_board
                    .get_elem(get_safe_i(height, width, &(i, j)))
                    .material_id = 0_usize;
                slice_board
                    .get_elem(get_safe_i(height, width, &(i, j)))
                    .display_color = VOID.material_color.color;
            }*/

            // Rng determines which side should the particle fall
            let mut orientation_y: i32 = 0_i32;
            // This calculates the position on the Y axis
            if slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .speed
                .y
                .abs()
                > 1_f32
            {
                unsafe {
                    write_y_speed_field(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        0_f32,
                        check_board,
                    );
                }
            } else {
                // Rand range: (-1_f32..1_f32)
                let rnd = rngs[get_safe_i(height, width, &(i, j))]
                    * seeds[get_safe_i(height, width, &(i, j))];
                unsafe {
                    write_y_speed_field(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .speed
                            .y
                            + (rnd.signum() * (rnd.abs() / 2_f32)),
                        check_board,
                    );
                }
                orientation_y = (slice_board
                    .get_elem(get_safe_i(height, width, &(i, j)))
                    .speed
                    .y
                    .signum()
                    * (slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .speed
                        .y
                        .abs()
                        + 1_f32)) as i32;
            }
            let mut ychange = 0_i32;
            for k in 0_i32..orientation_y.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                    ))
                    .is_some()
                    && std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::Solid {
                        melting_point: 0_f32,
                    })
                {
                    continue;
                } else if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                    ))
                    .is_some()
                    && (materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .density
                        <= materials[slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .material_id]
                            .1
                            .density)
                    && (std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    ) || std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(&Phase::Liquid {
                        viscosity: 0_f32,
                        melting_point: 0_f32,
                        boiling_point: 0_f32,
                    }))
                {
                    ychange = k;
                    unsafe {
                        write_updated_field(
                            slice_board,
                            get_safe_i(
                                height,
                                width,
                                &(i.wrapping_add((orientation_y.signum() * k) as usize), j),
                            ),
                            true,
                            check_board,
                        )
                    };
                }
            }
            let mut orientation_x = 0;
            // This calculates the position on the X axis
            if slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .speed
                .x
                .abs()
                > 1_f32
            {
                unsafe {
                    write_x_speed_field(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        0_f32,
                        check_board,
                    );
                }
            } else {
                let rnd = rngs[get_safe_i(height, width, &(i, j))];
                unsafe {
                    write_x_speed_field(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .speed
                            .x
                            + (seeds[get_safe_i(height, width, &(i, j))].signum() * (rnd / 2_f32)),
                        check_board,
                    );
                };
                orientation_x = (rnd.signum()
                    * (slice_board
                        .get_elem(get_safe_i(height, width, &(i, j)))
                        .speed
                        .x
                        .abs()
                        + 1_f32)) as i32;
            }

            let mut xchange = 0_i32;
            for _k in 0..orientation_x.abs() {
                // This condition checks, whether the particle can fall to the determined side
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                    ))
                    .is_some()
                    && std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::Solid {
                        melting_point: 0_f32,
                    })
                {
                    continue;
                } else if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                    ))
                    .is_some()
                    && materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .density
                        <= materials[slice_board
                            .get_elem(get_safe_i(height, width, &(i, j)))
                            .material_id]
                            .1
                            .density
                    && (std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                            .material_id]
                            .1
                            .phase,
                    ) != std::mem::discriminant(
                        &(Phase::Powder {
                            coarseness: 0_f32,
                            melting_point: 0_f32,
                        }),
                    ) || std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                            ))
                            .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
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
                    xchange = _k;
                    unsafe {
                        write_updated_field(
                            slice_board,
                            get_safe_i(
                                height,
                                width,
                                &(i, j.wrapping_add((orientation_x.signum() * _k) as usize)),
                            ),
                            true,
                            check_board,
                        );
                    }
                }
            }
            unsafe {
                swap_particle(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    get_safe_i(
                        height,
                        width,
                        &(
                            i.wrapping_add((orientation_y.signum() * ychange) as usize),
                            j.wrapping_add((orientation_x.signum() * xchange) as usize),
                        ),
                    ),
                    check_board,
                )
            }
            // This marks that the particle's position has been calculated
            unsafe {
                write_updated_field(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    true,
                    check_board,
                );
            }
        }
    }
}
