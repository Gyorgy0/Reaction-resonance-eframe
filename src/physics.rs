use crate::{
    material::{Material, tuple_to_rangeinclusive},
    particle::{AtomicParticle, Particle},
    world::{
        AtomicComparedSlice, get_safe_i, swap_particle, write_particle, write_x_speed_field,
        write_y_speed_field,
    },
};
use egui::{Color32, lerp};
use egui::{ahash::AHashMap, vec2};
use serde::{Deserialize, Serialize};
use std::{mem::discriminant, sync::Arc};

pub struct PhysicalReactions {
    pub(crate) melting: AHashMap<usize, usize>,
    pub(crate) boiling: AHashMap<usize, usize>,
    pub(crate) sublimation: AHashMap<usize, usize>,
    pub(crate) ionization: AHashMap<usize, usize>,
}

// Black body radiation gradient:
// ({temperature}, {color of the radiation})
// Color gradient tries to model how the electromagnetic radiation shifts,
// depending on the material's temperature
pub const BLACK_BODY_RADIATION_COLORS: [(f32, Color32); 17] = [
    (
        0_f32,
        Color32::from_rgba_unmultiplied_const(127_u8, 0_u8, 0_u8, 0_u8),
    ),
    (
        700_f32,
        Color32::from_rgba_unmultiplied_const(127_u8, 0_u8, 0_u8, 31_u8),
    ),
    (
        900_f32,
        Color32::from_rgba_unmultiplied_const(255_u8, 9_u8, 0_u8, 127_u8),
    ),
    (
        1000_f32,
        Color32::from_rgba_unmultiplied_const(255_u8, 9_u8, 0_u8, 155_u8),
    ),
    (
        2000_f32,
        Color32::from_rgba_unmultiplied_const(255_u8, 68_u8, 1_u8, 185_u8),
    ),
    (
        3000_f32,
        Color32::from_rgba_unmultiplied_const(255_u8, 125_u8, 36_u8, 215_u8),
    ),
    (
        4000_f32,
        Color32::from_rgba_unmultiplied_const(255_u8, 170_u8, 92_u8, 220_u8),
    ),
    (
        5000_f32,
        Color32::from_rgba_unmultiplied_const(255_u8, 205_u8, 155_u8, 225_u8),
    ),
    (
        6000_f32,
        Color32::from_rgba_unmultiplied_const(255_u8, 231_u8, 217_u8, 230_u8),
    ),
    (
        6500_f32,
        Color32::from_rgba_unmultiplied_const(255_u8, 255_u8, 255_u8, 240_u8),
    ),
    (
        7000_f32,
        Color32::from_rgba_unmultiplied_const(239_u8, 234_u8, 255_u8, 255_u8),
    ),
    (
        8000_f32,
        Color32::from_rgba_unmultiplied_const(202_u8, 220_u8, 255_u8, 240_u8),
    ),
    (
        9000_f32,
        Color32::from_rgba_unmultiplied_const(178_u8, 193_u8, 255_u8, 230_u8),
    ),
    (
        10000_f32,
        Color32::from_rgba_unmultiplied_const(160_u8, 180_u8, 255_u8, 200_u8),
    ),
    (
        15000_f32,
        Color32::from_rgba_unmultiplied_const(122_u8, 149_u8, 255_u8, 200_u8),
    ),
    (
        30000_f32,
        Color32::from_rgba_unmultiplied_const(96_u8, 140_u8, 255_u8, 190_u8),
    ),
    (
        40000_f32,
        Color32::from_rgba_unmultiplied_const(84_u8, 135_u8, 255_u8, 190_u8),
    ),
];

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
    Air,
    Solid { melting_point: f32, sublimation_point: f32 },
    Powder { melting_point: f32, sublimation_point: f32 },                         // Coarseness is the average diameter of a powder particle (between 0 and 1) (in cm), -> the smaller the diameter, the powder becomes more "clumpier"
    Liquid { viscosity: f32, melting_point: f32, boiling_point: f32 },      // Viscosity gives the rate, which the liquid spreads, for e.g. water has a viscosity of 1_f32, the bigger the viscosity, the thicker the fluid is
    Gas {boiling_point: f32, sublimation_point: f32},                                               // Not fully implemented
    Plasma,
}

impl Phase {
    pub fn solid_default() -> Self {
        Self::Solid {
            melting_point: f32::default(),
            sublimation_point: f32::default(),
        }
    }
    pub fn powder_default() -> Self {
        Self::Powder {
            melting_point: f32::default(),
            sublimation_point: f32::default(),
        }
    }
    pub fn liquid_default() -> Self {
        Self::Liquid {
            viscosity: f32::default(),
            melting_point: f32::default(),
            boiling_point: f32::default(),
        }
    }
    pub fn gas_default() -> Self {
        Self::Gas {
            boiling_point: f32::default(),
            sublimation_point: f32::default(),
        }
    }
    pub fn plasma_default() -> Self {
        Self::Plasma
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
    let mut current_particle;

    match &materials[slice_board
        .get_elem(get_safe_i(height, width, &(i, j)))
        .material_id]
        .1
        .phase
    {
        Phase::Air => {}

        Phase::Solid {
            melting_point,
            sublimation_point,
        } => {
            current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
            let mut new_particle = *current_particle;
            if *melting_point < current_particle.temperature
                && physical_transitions
                    .melting
                    .contains_key(&current_particle.material_id)
                && *sublimation_point < 0_f32
            {
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
                return;
            } else if *sublimation_point < current_particle.temperature
                && physical_transitions
                    .sublimation
                    .contains_key(&current_particle.material_id)
                && *melting_point < 0_f32
            {
                new_particle.material_id = *physical_transitions
                    .sublimation
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
                return;
            }
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // POWDER PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Powder {
            melting_point,
            sublimation_point,
        } => {
            current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
            let mut new_particle = *current_particle;
            if *melting_point < current_particle.temperature
                && physical_transitions
                    .melting
                    .contains_key(&current_particle.material_id)
            {
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
                return;
            } else if *sublimation_point < current_particle.temperature
                && physical_transitions
                    .sublimation
                    .contains_key(&current_particle.material_id)
                && *melting_point < 0_f32
            {
                new_particle.material_id = *physical_transitions
                    .sublimation
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
                return;
            }
            // Gravity simulation
            let mut speed_y = current_particle.speed.y;
            let next_particle = slice_board
                .get(get_safe_i(
                    height,
                    width,
                    &(i.wrapping_add(gravity.signum() as usize), j),
                ))
                .unwrap_or(current_particle);
            // Terminal velocity (sqrt((2*m*g)/p*A*Cd)
            // m - mass of the falling particle the particles are represented as a 1 cm^3 cube
            // g - gravitational acceleration
            // p - density of the medium the object is falling through (powder, liquid, gas, plasma)
            // Cd - drag coefficient (in the case of the simulated particles they are squares and the coefficient for cubes is 1.05)
            // A - projected area of object (in the case of the simulation it is 1 cm^2)
            let terminal_velocity =
                ((2_f32 * materials[current_particle.material_id].1.density * gravity)
                    / (materials[next_particle.material_id].1.density * 1_f32 * 1.05_f32))
                    .sqrt();
            if speed_y < terminal_velocity {
                speed_y += gravity * framedelta;
            }
            if discriminant(&materials[next_particle.material_id].1.phase)
                == discriminant(&Phase::solid_default())
            {
                speed_y = 0_f32;
            }
            unsafe {
                write_y_speed_field(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    speed_y,
                    check_board,
                )
            };

            // Change on the Y axis
            let mut ychange = 0;
            for _k in 0..slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .speed
                .y
                .abs() as i32
            {
                // Current particle - the currently evaluated particle
                // Future particle - the current particle's future position - if the particle
                // is out of bounds then it returns the current particle
                let current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
                let future_particle = slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i + (gravity.signum() as i32 * _k) as usize, j),
                    ))
                    .unwrap_or(current_particle);

                // Falling and checking if there is a particle with a larger density
                if materials[current_particle.material_id].1.density
                    > materials[future_particle.material_id].1.density
                    && std::mem::discriminant(&materials[future_particle.material_id].1.phase)
                        != std::mem::discriminant(&Phase::solid_default())
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
                    || std::mem::discriminant(&materials[future_particle.material_id].1.phase)
                        == std::mem::discriminant(&Phase::solid_default())
                    || std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(
                                    (i as f32
                                        + (gravity.signum() as i32 * _k) as f32
                                        + gravity.signum())
                                        as usize,
                                    j,
                                ),
                            ))
                            .unwrap_or(current_particle)
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::powder_default())
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
                }
            }

            // This decides where the particle falls (left or right)
            let rnd = rngs[get_safe_i(height, width, &(i, j))];
            if slice_board
                .get(get_safe_i(
                    height,
                    width,
                    &(
                        i + (gravity.signum() as i32) as usize,
                        (j as i64).wrapping_add(rnd.signum() as i64) as usize,
                    ),
                ))
                .is_some()
                && materials[slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(
                            i + (gravity.signum() as i32) as usize,
                            (j as i64).wrapping_add(rnd.signum() as i64) as usize,
                        ),
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
                            &(
                                i + (gravity.signum() as i32) as usize,
                                (j as i64).wrapping_add(rnd.signum() as i64) as usize,
                            ),
                        ))
                        .unwrap_or(slice_board.get_elem(get_safe_i(height, width, &(i, j))))
                        .material_id]
                        .1
                        .phase,
                ) != std::mem::discriminant(&Phase::solid_default())
            {
                unsafe {
                    swap_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        get_safe_i(
                            height,
                            width,
                            &(
                                i + (gravity.signum() as i32) as usize,
                                (j as i64).wrapping_add(rnd.signum() as i64) as usize,
                            ),
                        ),
                        check_board,
                    );
                }
            }
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // LIQUID PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Liquid {
            viscosity,
            melting_point,
            boiling_point,
        } => {
            // Phase change from liquid to solid and liquid to gas
            current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
            let mut new_particle = *current_particle;
            // Melting (solid -> liquid)
            if *melting_point > current_particle.temperature
                && physical_transitions
                    .melting
                    .contains_key(&current_particle.material_id)
            {
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
                new_particle.speed = vec2(0_f32, 0_f32);
                unsafe {
                    write_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        new_particle,
                        check_board,
                    )
                };
                return;
            // Boiling/evaporation (liquid -> gas)
            } else if *boiling_point < current_particle.temperature
                && physical_transitions
                    .boiling
                    .contains_key(&current_particle.material_id)
            {
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
                new_particle.speed = vec2(0_f32, 0_f32);
                unsafe {
                    write_particle(
                        slice_board,
                        get_safe_i(height, width, &(i, j)),
                        new_particle,
                        check_board,
                    )
                };
                return;
            }

            // Gravity simulation
            let mut speed_y = current_particle.speed.y;
            let next_particle = slice_board
                .get(get_safe_i(
                    height,
                    width,
                    &(i.wrapping_add(gravity.signum() as usize), j),
                ))
                .unwrap_or(current_particle);
            // Terminal velocity (sqrt((2*m*g)/p*A*Cd)
            // m - mass of the falling particle the particles are represented as a 1 cm^3 cube
            // g - gravitational acceleration
            // p - density of the medium the object is falling through (powder, liquid, gas, plasma)
            // Cd - drag coefficient (in the case of the simulated particles they are squares and the coefficient for cubes is 1.05)
            // A - projected area of object (in the case of the simulation it is 1 cm^2)
            let terminal_velocity =
                ((2_f32 * materials[current_particle.material_id].1.density * gravity)
                    / (materials[next_particle.material_id].1.density * 1_f32 * 1.05_f32))
                    .sqrt();
            if speed_y < terminal_velocity {
                speed_y += gravity * framedelta;
            }
            unsafe {
                write_y_speed_field(
                    slice_board,
                    get_safe_i(height, width, &(i, j)),
                    speed_y,
                    check_board,
                )
            };

            // Change on the Y axis
            let mut ychange = 0;
            for k in 0..slice_board
                .get_elem(get_safe_i(height, width, &(i, j)))
                .speed
                .y
                .abs() as i32
            {
                // Current particle - the currently evaluated particle
                // Future particle - the current particle's future position - if the particle
                // is out of bounds then it returns the current particle
                let current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
                let future_particle = slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i + (gravity.signum() as i32 * k) as usize, j),
                    ))
                    .unwrap_or(current_particle);

                // Falling and checking if there is a particle with a larger density
                if materials[current_particle.material_id].1.density
                    > materials[future_particle.material_id].1.density
                    && std::mem::discriminant(&materials[future_particle.material_id].1.phase)
                        != std::mem::discriminant(&Phase::solid_default())
                {
                    ychange = k;
                }
                // Checks if the particle falls inside bounds
                // Checks, whether there is another denser particle in the path of the falling particle
                else if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i + (gravity.signum() as i32 * k) as usize, j),
                    ))
                    .is_none()
                    || std::mem::discriminant(&materials[future_particle.material_id].1.phase)
                        == std::mem::discriminant(&Phase::solid_default())
                    || std::mem::discriminant(
                        &materials[slice_board
                            .get(get_safe_i(
                                height,
                                width,
                                &(
                                    (i as f32
                                        + (gravity.signum() as i32 * k) as f32
                                        + gravity.signum())
                                        as usize,
                                    j,
                                ),
                            ))
                            .unwrap_or(current_particle)
                            .material_id]
                            .1
                            .phase,
                    ) == std::mem::discriminant(&Phase::powder_default())
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
                }
            }

            current_particle =
                slice_board.get_elem(get_safe_i(height, width, &(i + ychange as usize, j)));
            // Viscosity simulation
            let mut speed_x = 0_f32;
            let mut rnd = rngs[get_safe_i(height, width, &(i, j))];
            if rnd.abs() > (1_f32 - (1_f32 / viscosity)).powi(16) {
                if slice_board
                    .get_elem(get_safe_i(
                        height,
                        width,
                        &(i, (j as f32 + rnd.signum()) as usize),
                    ))
                    .material_id
                    == current_particle.material_id
                {
                    rnd *= -1_f32;
                }
                speed_x = rnd.signum() * (rnd.abs() + (1_f32 / viscosity).sqrt());
            }
            // Change on the X axis
            let mut xchange = 0;
            for k in 0_i32..=speed_x.abs() as i32 {
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i, (j as i32 + (rnd.signum() as i32 * k)) as usize),
                    ))
                    .is_some()
                    && (materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i, (j as i32 + (rnd.signum() as i32 * k)) as usize),
                        ))
                        .unwrap_or(current_particle)
                        .material_id]
                        .1
                        .density
                        > materials[current_particle.material_id].1.density
                        || discriminant(
                            &materials[slice_board
                                .get(get_safe_i(
                                    height,
                                    width,
                                    &(i, (j as i32 + (rnd.signum() as i32 * k)) as usize),
                                ))
                                .unwrap_or(current_particle)
                                .material_id]
                                .1
                                .phase,
                        ) != discriminant(&Phase::solid_default()))
                {
                    xchange = k;
                } else {
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
                        &(i, j + (rnd.signum() as i32 * xchange) as usize),
                    ),
                    check_board,
                )
            };
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // GAS PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Gas {
            boiling_point,
            sublimation_point,
        } => {
            // Phase transition fromg as to liquid
            current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
            let mut new_particle = *current_particle;
            if *boiling_point > current_particle.temperature
                && physical_transitions
                    .boiling
                    .contains_key(&current_particle.material_id)
            {
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
                return;
            } else if *sublimation_point > current_particle.temperature
                && physical_transitions
                    .sublimation
                    .contains_key(&current_particle.material_id)
                && *boiling_point < 0_f32
            {
                new_particle.material_id = *physical_transitions
                    .sublimation
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
                return;
            }
            // This calculates the position on the Y axis
            let mut orientation_y: i32 = 0_i32;
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
                // Calculates buoyancy using gravity and material density
                // We limit the velocity so on phase change there are no "teleporting" particles
                let next_particle = slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i.wrapping_add(gravity.signum() as usize), j),
                    ))
                    .unwrap_or(current_particle);
                orientation_y = (((current_particle.speed.y.signum()
                    * (current_particle.speed.y.abs() + 1_f32))
                    - ((materials[next_particle.material_id].1.density
                        / materials[current_particle.material_id].1.density)
                        * gravity
                        * framedelta)) as i32)
                    .clamp(-gravity.abs() as i32, gravity.abs() as i32);
            }
            let mut ychange = 0_i32;
            for k in 0_i32..orientation_y.abs() {
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &((i as i32 + (orientation_y.signum() * k)) as usize, j),
                    ))
                    .is_some()
                    && (materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &((i as i32 + (orientation_y.signum() * k)) as usize, j),
                        ))
                        .unwrap_or(current_particle)
                        .material_id]
                        .1
                        .density
                        < materials[current_particle.material_id].1.density
                        || discriminant(
                            &materials[slice_board
                                .get(get_safe_i(
                                    height,
                                    width,
                                    &((i as i32 + (orientation_y.signum() * k)) as usize, j),
                                ))
                                .unwrap_or(current_particle)
                                .material_id]
                                .1
                                .phase,
                        ) != discriminant(&Phase::solid_default()))
                {
                    ychange = k;
                } else {
                    break;
                }
            }

            // This calculates the position on the X axis
            let mut orientation_x: i32 = 0_i32;
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
                // Rand range: (-1_f32..1_f32)
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
                            + (rnd.signum() * (rnd.abs())),
                        check_board,
                    );
                }
                orientation_x = (current_particle.speed.x.signum()
                    * (current_particle.speed.x.abs() + 1_f32))
                    as i32;
            }

            let mut xchange = 0_i32;
            for k in 0_i32..orientation_x.abs() {
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(
                            i.wrapping_add((orientation_y.signum() * ychange) as usize),
                            (j as i32 + (orientation_x.signum() * k)) as usize,
                        ),
                    ))
                    .is_some()
                    && (materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(
                                i.wrapping_add((orientation_y.signum() * ychange) as usize),
                                (j as i32 + (orientation_x.signum() * k)) as usize,
                            ),
                        ))
                        .unwrap_or(current_particle)
                        .material_id]
                        .1
                        .density
                        < materials[current_particle.material_id].1.density
                        || discriminant(
                            &materials[slice_board
                                .get(get_safe_i(
                                    height,
                                    width,
                                    &(
                                        i.wrapping_add((orientation_y.signum() * ychange) as usize),
                                        (j as i32 + (orientation_x.signum() * k)) as usize,
                                    ),
                                ))
                                .unwrap_or(current_particle)
                                .material_id]
                                .1
                                .phase,
                        ) != discriminant(&Phase::solid_default()))
                {
                    xchange = k;
                } else {
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
                            i.wrapping_add((orientation_y.signum() * ychange) as usize),
                            j.wrapping_add((orientation_x.signum() * xchange) as usize),
                        ),
                    ),
                    check_board,
                )
            }
        }
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        // PLASMA PHYSICS
        /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
        Phase::Plasma => {
            current_particle = slice_board.get_elem(get_safe_i(height, width, &(i, j)));
            // This calculates the position on the Y axis
            let mut orientation_y: i32 = 0_i32;
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
                // Calculates buoyancy using gravity and material density
                // We limit the velocity so on phase change there are no "teleporting" particles
                let next_particle = slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i.wrapping_add(gravity.signum() as usize), j),
                    ))
                    .unwrap_or(current_particle);
                orientation_y = (((current_particle.speed.y.signum()
                    * (current_particle.speed.y.abs() + 1_f32))
                    - ((materials[next_particle.material_id].1.density
                        / materials[current_particle.material_id].1.density)
                        * gravity
                        * framedelta)) as i32)
                    .clamp(-gravity.abs() as i32, gravity.abs() as i32);
            }
            let mut ychange = 0_i32;
            for k in 0_i32..orientation_y.abs() {
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &((i as i32 + (orientation_y.signum() * k)) as usize, j),
                    ))
                    .is_some()
                    && (materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &((i as i32 + (orientation_y.signum() * k)) as usize, j),
                        ))
                        .unwrap_or(current_particle)
                        .material_id]
                        .1
                        .density
                        < materials[current_particle.material_id].1.density
                        || discriminant(
                            &materials[slice_board
                                .get(get_safe_i(
                                    height,
                                    width,
                                    &((i as i32 + (orientation_y.signum() * k)) as usize, j),
                                ))
                                .unwrap_or(current_particle)
                                .material_id]
                                .1
                                .phase,
                        ) != discriminant(&Phase::solid_default()))
                {
                    ychange = k;
                } else {
                    break;
                }
            }

            // This calculates the position on the X axis
            let mut orientation_x: i32 = 0_i32;
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
                // Rand range: (-1_f32..1_f32)
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
                            + (rnd.signum() * (rnd.abs())),
                        check_board,
                    );
                }
                orientation_x = (current_particle.speed.x.signum()
                    * (current_particle.speed.x.abs() + 1_f32))
                    as i32;
            }

            let mut xchange = 0_i32;
            for k in 0_i32..orientation_x.abs() {
                if slice_board
                    .get(get_safe_i(
                        height,
                        width,
                        &(i, (j as i32 + (orientation_x.signum() * k)) as usize),
                    ))
                    .is_some()
                    && (materials[slice_board
                        .get(get_safe_i(
                            height,
                            width,
                            &(i, (j as i32 + (orientation_x.signum() * k)) as usize),
                        ))
                        .unwrap_or(current_particle)
                        .material_id]
                        .1
                        .density
                        < materials[current_particle.material_id].1.density
                        || discriminant(
                            &materials[slice_board
                                .get(get_safe_i(
                                    height,
                                    width,
                                    &(i, (j as i32 + (orientation_x.signum() * k)) as usize),
                                ))
                                .unwrap_or(current_particle)
                                .material_id]
                                .1
                                .phase,
                        ) != discriminant(&Phase::solid_default()))
                {
                    xchange = k;
                } else {
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
                            i.wrapping_add((orientation_y.signum() * ychange) as usize),
                            j.wrapping_add((orientation_x.signum() * xchange) as usize),
                        ),
                    ),
                    check_board,
                )
            }
        }
    }
}
