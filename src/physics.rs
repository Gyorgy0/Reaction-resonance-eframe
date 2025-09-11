use std::mem;

use crate::world::Board;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Phase {
    Void,
    Solid,
    Powder { coarseness: f32 }, // Coarseness is the average diameter of a powder particle (between 0 and 1) (in cm), -> the smaller the diameter, the powder becomes more "clumpier"
    Liquid { viscosity: f32 }, // Viscosity gives the rate, which the liquid spreads, for e.g. water has a viscosity of 1.0, the bigger the viscosity, the thicker the fluid is
    Gas,                       // Not fully implemented
    Plasma { energy: f32 },
}

impl Phase {
    fn get_coarseness(&self) -> f32 {
        let mut returnval: f32 = 0.0;
        if let Phase::Powder { coarseness } = self {
            returnval = *coarseness
        };
        returnval
    }
    fn get_viscosity(&self) -> f32 {
        let mut returnval: f32 = 0.0;
        if let Phase::Liquid { viscosity } = self {
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
    pub(crate) fn solve_particle(&mut self, i: usize, j: usize, framedelta: f32) {
        match self.contents[i][j].material.phase {
            Phase::Void => {}

            Phase::Solid => {}
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            // POWDER PHYSICS
            /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            Phase::Powder { coarseness: _f32 } => {
                // Gravity simulation
                self.contents[i][j].speed.y += self.gravity * framedelta;
                let mut ychange = 0;
                for _k in 0..self.contents[i][j].speed.y.abs() as i32 {
                    // Falling and checking if there is a particle with a larger density
                    if self.contents[i][j].material.density
                        > self
                            .contents
                            .get(i + (self.gravity.signum() as i32 * _k) as usize)
                            .unwrap_or(&self.contents[i])
                            .get(j)
                            .unwrap_or(&self.contents[i][j])
                            .material
                            .density
                        && self.contents[i][j].updated
                    {
                        ychange = _k;
                    }
                    // Checks if the particle falls inside bounds
                    else if self
                        .contents
                        .get(i + (self.gravity.signum() as i32 * _k) as usize)
                        .unwrap_or(&self.contents[i])
                        .get(j)
                        .is_none()
                    {
                        self.contents[i][j].speed.y -= self.gravity * framedelta;
                        break;
                    }
                    // Checks, whether there is another denser particle in the path of the falling particle
                    else if self
                        .contents
                        .get(i + (self.gravity.signum() as i32 * _k) as usize)
                        .unwrap_or(&self.contents[i])
                        .get(j)
                        .unwrap_or(&self.contents[i][j])
                        .material
                        .phase
                        == Phase::Solid
                        || self
                            .contents
                            .get(
                                i + ((self.gravity.signum() as i32 * _k)
                                    + self.gravity.signum() as i32)
                                    as usize,
                            )
                            .unwrap_or(&self.contents[i])
                            .get(j)
                            .unwrap_or(&self.contents[i][j])
                            .material
                            .phase
                            == (Phase::Powder { coarseness: _f32 })
                    {
                        self.contents[i][j].speed.y -= self.gravity * framedelta;
                        break;
                    }
                }
                if ychange != 0 {
                    let swapped_particle = self.contents[i][j].clone();
                    self.contents[i][j] = self.contents
                        [i + (self.gravity.signum() as i32 * ychange) as usize][j]
                        .clone();
                    self.contents[i + (self.gravity.signum() as i32 * ychange) as usize][j] =
                        swapped_particle;

                    self.contents[i + ((self.gravity.signum() as i32 * ychange) as usize)][j]
                        .updated = false;
                }
                // This decides where the particle falls (left or right)
                let rnd = self.rngs[i][j];
                if self.contents[i][j].updated
                    && self
                        .contents
                        .get(i + (self.gravity.signum() as i32) as usize)
                        .is_some()
                    && self
                        .contents
                        .get(i + (self.gravity.signum() as i32) as usize)
                        .unwrap_or(&self.contents[i])
                        .get(j + 1)
                        .unwrap_or(&self.contents[i][j])
                        .material
                        .density
                        < self.contents[i][j].material.density
                    && self
                        .contents
                        .get(i + (self.gravity.signum() as i32) as usize)
                        .unwrap_or(&self.contents[i])
                        .get(j + 1)
                        .unwrap_or(&self.contents[i][j])
                        .material
                        .phase
                        != Phase::Solid
                    && self.contents[i][j].temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - self.contents[i][j]
                                .material
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                    && rnd.signum() == -1.0
                {
                    let swapped_particle = self.contents[i][j].clone();
                    self.contents[i][j] =
                        self.contents[i + (self.gravity.signum() as i32) as usize][j + 1].clone();
                    self.contents[i + (self.gravity.signum() as i32) as usize][j + 1] =
                        swapped_particle;
                    self.contents[i + (self.gravity.signum() as i32) as usize][j + 1].updated =
                        false;
                }
                if self.contents[i][j].updated
                    && self
                        .contents
                        .get(i + (self.gravity.signum() as i32) as usize)
                        .is_some()
                    && self
                        .contents
                        .get(i + (self.gravity.signum() as i32) as usize)
                        .unwrap_or(&self.contents[i])
                        .get(j - 1)
                        .unwrap_or(&self.contents[i][j])
                        .material
                        .density
                        < self.contents[i][j].material.density
                    && self
                        .contents
                        .get(i + (self.gravity.signum() as i32) as usize)
                        .unwrap_or(&self.contents[i])
                        .get(j - 1)
                        .unwrap_or(&self.contents[i][j])
                        .material
                        .phase
                        != Phase::Solid
                    && self.contents[i][j].temperature // SEED needs to be implemented!!!
                        >= ((1_f32
                            - self.contents[i][j]
                                .material
                                .phase
                                .get_coarseness()
                                .sqrt()
                                .sqrt())
                        .sqrt())
                        .powi(8)
                    && rnd.signum() == 1.0
                {
                    let swapped_particle = self.contents[i][j].clone();
                    self.contents[i][j] =
                        self.contents[i + (self.gravity.signum() as i32) as usize][j - 1].clone();
                    self.contents[i + (self.gravity.signum() as i32) as usize][j - 1] =
                        swapped_particle;

                    self.contents[i + (self.gravity.signum() as i32) as usize][j - 1].updated =
                        false;
                }
                // This marks that the particle's position has been calculated
                self.contents[i][j].updated = true;
            }
            _ => {} /*
                    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
                    // LIQUID PHYSICS
                    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
                    Phase::Liquid { viscosity: _f32 } => {
                        // Gravity simulation
                        self.contents[i][j].speed.y += self.gravity * framedelta;
                        let mut ychange = 0;
                        for _k in 0..self.contents[i][j].speed.y.abs() as i32 {
                            // Falling and checking if there is a particle with a larger density
                            if self.contents[i][j].material.density
                                > self
                                    .contents
                                    .get(
                                        ((i + (self.gravity.signum() as i32 * _k)) * col_count + j)
                                            as usize,
                                    )
                                    .unwrap_or(&self.contents[i][j])
                                    .material
                                    .density
                                && self.contents[i][j].updated
                            {
                                ychange = _k;
                            }
                            // Checks if the particle falls inside bounds
                            else if self
                                .contents
                                .get(((i + (self.gravity.signum() as i32 * _k)) * col_count + j) as usize)
                                .is_none()
                            {
                                self.contents[i][j].speed.y -= self.gravity * framedelta;
                                break;
                            }
                            // Checks, whether there is another denser particle in the path of the falling particle
                            else if self
                                .contents
                                .get(((i + (self.gravity.signum() as i32 * _k)) * col_count + j) as usize)
                                .unwrap_or(&self.contents[i][j])
                                .material
                                .phase
                                == Phase::Solid
                                || self
                                    .contents
                                    .get(
                                        ((i + (self.gravity.signum() as i32 * _k)
                                            + self.gravity.signum() as i32)
                                            * col_count
                                            + j) as usize,
                                    )
                                    .unwrap_or(
                                        &self.contents[((i + (self.gravity.signum() as i32 * _k))
                                            * col_count
                                            + j) as usize],
                                    )
                                    .material
                                    .phase
                                    == (Phase::Powder { coarseness: _f32 })
                                || self
                                    .contents
                                    .get(
                                        ((i + (self.gravity.signum() as i32 * _k)
                                            + self.gravity.signum() as i32)
                                            * col_count
                                            + j) as usize,
                                    )
                                    .unwrap_or(
                                        &self.contents[((i + (self.gravity.signum() as i32 * _k))
                                            * col_count
                                            + j) as usize],
                                    )
                                    .material
                                    .phase
                                    == (Phase::Liquid { viscosity: _f32 })
                            {
                                self.contents[i][j].speed.y -= self.gravity * framedelta;
                                break;
                            }
                        }
                        if ychange != 0 {
                            self.contents.swap(
                                cellpos,
                                ((i + (self.gravity.signum() as i32 * ychange)) * col_count + j) as usize,
                            );
                            self.contents[((i + (self.gravity.signum() as i32 * ychange)) * col_count + j) as usize].updated = false;
                        }
                        // Rng determines which side should the particle fall
                        let mut orientation: i32 = 0;
                        if self.contents[i][j].speed.x.abs() > 1.0 {
                            self.contents[i][j].speed.x = 0.0;
                        } else {
                            let rnd = self.rngs[i][j];
                            if rnd.abs()
                                >= (1_f32
                                    - self.contents[i][j]
                                        .material
                                        .phase
                                        .get_viscosity()
                                        .sqrt()
                                        .sqrt()
                                        .sqrt())
                                .powi(16)
                            {
                                self.contents[i][j].speed.x += rnd.signum()
                                    * (rnd.abs() + self.contents[i][j].material.phase.get_viscosity())
                                        .powi(4);
                                orientation = (self.contents[i][j].speed.x.signum()
                                    * (self.contents[i][j].speed.x.abs() + 1.0))
                                    as i32;
                            }
                        }
                        if self
                            .contents
                            .get((i * col_count + j + 1) as usize)
                            .unwrap_or(&self.contents[i][j])
                            .material
                            .density
                            < self.contents[i][j].material.density
                            && self
                                .contents
                                .get((i * col_count + j - 1) as usize)
                                .unwrap_or(&self.contents[i][j])
                                .material
                                .density
                                > self.contents[i][j].material.density
                        {
                            self.contents[i][j].speed.x = self.contents[i][j].speed.x.abs();
                            orientation = (self.contents[i][j].speed.x.abs() + 1.0) as i32;
                        } else if self
                            .contents
                            .get((i * col_count + j + 1) as usize)
                            .unwrap_or(&self.contents[i][j])
                            .material
                            .density
                            > self.contents[i][j].material.density
                            && self
                                .contents
                                .get((i * col_count + j - 1) as usize)
                                .unwrap_or(&self.contents[i][j])
                                .material
                                .density
                                < self.contents[i][j].material.density
                        {
                            self.contents[i][j].speed.x = -self.contents[i][j].speed.x.abs();
                            orientation = -(self.contents[i][j].speed.x.abs() + 1.0) as i32;
                        } else if self
                            .contents
                            .get((i * col_count + j + 1) as usize)
                            .unwrap_or(&self.contents[i][j])
                            .material
                            .density
                            <= self.contents[i][j].material.density
                            && self
                                .contents
                                .get((i * col_count + j - 1) as usize)
                                .unwrap_or(&self.contents[i][j])
                                .material
                                .density
                                <= self.contents[i][j].material.density
                        {
                            orientation = (self.contents[i][j].speed.x.signum()
                                * (self.contents[i][j].speed.x.abs() + 1.0))
                                as i32;
                        }

                        for _k in 0..orientation.abs() {
                            // This condition checks, whether the particle can fall to the determined side
                            if self.is_in_bounds(j, orientation.signum() * _k)
                                && self
                                    .contents
                                    .get((i * col_count + j + orientation.signum() * _k) as usize)
                                    .unwrap_or(&self.contents[i][j])
                                    .material
                                    .density
                                    <= self.contents[i][j].material.density
                                && (self
                                    .contents
                                    .get((i * col_count + j + orientation.signum() * _k) as usize)
                                    .unwrap_or(&self.contents[i][j])
                                    .material
                                    .phase
                                    != Phase::Solid
                                    || self
                                        .contents
                                        .get((i * col_count + j + orientation.signum() * _k) as usize)
                                        .unwrap_or(&self.contents[i][j])
                                        .material
                                        .phase
                                        != (Phase::Powder { coarseness: _f32 }))
                            {
                                self.contents.swap(
                                    cellpos,
                                    ((i * col_count) + (j + orientation.signum() * _k)) as usize,
                                );
                                self.contents
                                    [((i * col_count) + (j + orientation.signum() * _k)) as usize]
                                    .updated = true;
                            } else {
                                self.contents[i][j].speed.x *= -1.0;
                                break;
                            }
                        }
                        // This marks that the particle's position has been calculated
                        self.contents[i][j].updated = true;
                    }
                    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
                    // GAS PHYSICS
                    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
                    Phase::Gas => {
                        // Rng determines which side should the particle fall
                        let mut orientation: i32 = 0;
                        // This calculates the position on the Y axis
                        if self.contents[i][j].speed.y.abs() > 1.0 {
                            self.contents[i][j].speed.y = 0.0;
                        } else {
                            // Rand range: (-1.0..1.0)
                            let rnd = self.rngs[i][j];
                            self.contents[i][j].speed.y += rnd.signum() * (rnd.abs() / 2_f32);
                            orientation = (self.contents[i][j].speed.y.signum()
                                * (self.contents[i][j].speed.y.abs() + 1.0))
                                as i32;
                        }

                        for _k in 0..orientation.abs() {
                            // This condition checks, whether the particle can fall to the determined side
                            if self
                                .contents
                                .get(((i + (orientation.signum() * _k)) * col_count + j) as usize)
                                .is_some()
                                && self
                                    .contents
                                    .get(((i + (orientation.signum() * _k)) * col_count + j) as usize)
                                    .unwrap_or(&self.contents[i][j])
                                    .material
                                    .density
                                    <= self.contents[i][j].material.density
                                && (std::mem::discriminant(
                                    &self
                                        .contents
                                        .get(((i + (orientation.signum() * _k)) * col_count + j) as usize)
                                        .unwrap_or(&self.contents[i][j])
                                        .material
                                        .phase,
                                ) != std::mem::discriminant(&Phase::Solid)
                                    || std::mem::discriminant(
                                        &self
                                            .contents
                                            .get(
                                                ((i + (orientation.signum() * _k)) * col_count + j)
                                                    as usize,
                                            )
                                            .unwrap_or(&self.contents[i][j])
                                            .material
                                            .phase,
                                    ) != std::mem::discriminant(&(Phase::Powder { coarseness: 0_f32 }))
                                    || std::mem::discriminant(
                                        &self
                                            .contents
                                            .get(
                                                ((i + (orientation.signum() * _k)) * col_count + j)
                                                    as usize,
                                            )
                                            .unwrap_or(&self.contents[i][j])
                                            .material
                                            .phase,
                                    ) != std::mem::discriminant(&Phase::Liquid { viscosity: 0_f32 }))
                            {
                                self.contents.swap(
                                    cellpos,
                                    (((i + (orientation.signum() * _k)) * col_count) + j) as usize,
                                );
                                self.contents
                                    [(((i + (orientation.signum() * _k)) * col_count) + j) as usize]
                                    .updated = true;
                            }
                        }
                        orientation = 0;
                        // This calculates the position on the X axis
                        if self.contents[i][j].speed.x.abs() > 1.0 {
                            self.contents[i][j].speed.x = 0.0;
                        } else {
                            // Rand range: (-1.0..1.0)
                            let rnd = self.rngs[i][j];
                            self.contents[i][j].speed.x += rnd.signum() * (rnd.abs());
                            orientation = (self.contents[i][j].speed.x.signum()
                                * (self.contents[i][j].speed.x.abs() + 1.0))
                                as i32;
                        }

                        for _k in 0..orientation.abs() {
                            // This condition checks, whether the particle can fall to the determined side
                            if self.is_in_bounds(j, orientation.signum() * _k)
                                && self
                                    .contents
                                    .get((i * col_count + j + (orientation.signum() * _k)) as usize)
                                    .unwrap_or(&self.contents[i][j])
                                    .material
                                    .density
                                    <= self.contents[i][j].material.density
                                && (std::mem::discriminant(
                                    &self
                                        .contents
                                        .get((i * col_count + j + (orientation.signum() * _k)) as usize)
                                        .unwrap_or(&self.contents[i][j])
                                        .material
                                        .phase,
                                ) != std::mem::discriminant(&Phase::Solid)
                                    || std::mem::discriminant(
                                        &self
                                            .contents
                                            .get((i * col_count + j + (orientation.signum() * _k)) as usize)
                                            .unwrap_or(&self.contents[i][j])
                                            .material
                                            .phase,
                                    ) != std::mem::discriminant(&(Phase::Powder { coarseness: 0_f32 }))
                                    || std::mem::discriminant(
                                        &self
                                            .contents
                                            .get((i * col_count + j + (orientation.signum() * _k)) as usize)
                                            .unwrap_or(&self.contents[i][j])
                                            .material
                                            .phase,
                                    ) != std::mem::discriminant(&(Phase::Liquid { viscosity: 0_f32 })))
                            {
                                self.contents.swap(
                                    cellpos,
                                    ((i * col_count) + j + (orientation.signum() * _k)) as usize,
                                );
                                self.contents
                                    [((i * col_count) + j + (orientation.signum() * _k)) as usize]
                                    .updated = true;
                            }
                        }
                        // This marks that the particle's position has been calculated
                        self.contents[i][j].updated = true;
                    }
                    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
                    // PLASMA PHYSICS
                    /////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
                    Phase::Plasma { energy: _f32 } => {
                        let cellenergy = self.contents[i][j].material.phase.get_energy();
                        if cellenergy > 1.0 {
                            self.contents[i][j].material.phase = Phase::Plasma {
                                energy: cellenergy - 1.0,
                            };
                        } else {
                            self.contents[i][j].material = Material {
                                name: "Void".to_string(),
                                density: 0.0,
                                phase: Phase::Void,
                                material_type: Material_Type::Atmosphere,
                                durability: -1,
                                color: Color32::from_rgba_unmultiplied(0, 0, 0, 100),
                            };
                        }

                        // Rng determines which side should the particle fall
                        let mut orientation: i32 = 0;
                        // This calculates the position on the Y axis
                        if self.contents[i][j].speed.y.abs() > 1.0 {
                            self.contents[i][j].speed.y = 0.0;
                        } else {
                            // Rand range: (-1.0..1.0)
                            let rnd = self.rngs[i][j];
                            self.contents[i][j].speed.y += rnd.signum() * (rnd.abs());
                            orientation = (self.contents[i][j].speed.y.signum()
                                * (self.contents[i][j].speed.y.abs() + 1.0))
                                as i32;
                        }

                        for _k in 0..orientation.abs() {
                            // This condition checks, whether the particle can fall to the determined side
                            if self
                                .contents
                                .get(((i + (orientation.signum() * _k)) * col_count + j) as usize)
                                .is_some()
                                && self
                                    .contents
                                    .get(((i + (orientation.signum() * _k)) * col_count + j) as usize)
                                    .unwrap_or(&self.contents[i][j])
                                    .material
                                    .density
                                    <= self.contents[i][j].material.density
                                && (self
                                    .contents
                                    .get(((i + (orientation.signum() * _k)) * col_count + j) as usize)
                                    .unwrap_or(&self.contents[i][j])
                                    .material
                                    .phase
                                    != Phase::Solid
                                    || std::mem::discriminant(
                                        &self
                                            .contents
                                            .get(
                                                ((i + (orientation.signum() * _k)) * col_count + j)
                                                    as usize,
                                            )
                                            .unwrap_or(&self.contents[i][j])
                                            .material
                                            .phase,
                                    ) != std::mem::discriminant(&(Phase::Powder { coarseness: 0_f32 }))
                                    || self
                                        .contents
                                        .get(((i + (orientation.signum() * _k)) * col_count + j) as usize)
                                        .unwrap_or(&self.contents[i][j])
                                        .material
                                        .phase
                                        != (Phase::Liquid { viscosity: 0_f32 }))
                            {
                                self.contents.swap(
                                    cellpos,
                                    (((i + (orientation.signum() * _k)) * col_count) + j) as usize,
                                );
                                self.contents
                                    [(((i + (orientation.signum() * _k)) * col_count) + j) as usize]
                                    .updated = true;
                            } else {
                                self.contents[i][j].speed.y *= -1.0;
                                break;
                            }
                        }
                        orientation = 0;
                        // This calculates the position on the X axis
                        if self.contents[i][j].speed.x.abs() > 1.0 {
                            self.contents[i][j].speed.x = 0.0;
                        } else {
                            // Rand range: (-1.0..1.0)
                            let rnd = self.rngs[i][j];
                            self.contents[i][j].speed.x += rnd.signum() * (rnd.abs());
                            orientation = (self.contents[i][j].speed.x.signum()
                                * (self.contents[i][j].speed.x.abs() + 1.0))
                                as i32;
                        }

                        for _k in 0..orientation.abs() {
                            // This condition checks, whether the particle can fall to the determined side
                            if self.is_in_bounds(j, orientation.signum() * _k)
                                && self
                                    .contents
                                    .get((i * col_count + j + (orientation.signum() * _k)) as usize)
                                    .unwrap_or(&self.contents[i][j])
                                    .material
                                    .density
                                    <= self.contents[i][j].material.density
                                && (self
                                    .contents
                                    .get((i * col_count + j + (orientation.signum() * _k)) as usize)
                                    .unwrap_or(&self.contents[i][j])
                                    .material
                                    .phase
                                    != Phase::Solid
                                    || std::mem::discriminant(
                                        &self
                                            .contents
                                            .get((i * col_count + j + (orientation.signum() * _k)) as usize)
                                            .unwrap_or(&self.contents[i][j])
                                            .material
                                            .phase,
                                    ) != std::mem::discriminant(&(Phase::Powder { coarseness: 0_f32 }))
                                    || std::mem::discriminant(
                                        &self
                                            .contents
                                            .get((i * col_count + j + (orientation.signum() * _k)) as usize)
                                            .unwrap_or(&self.contents[i][j])
                                            .material
                                            .phase,
                                    ) != std::mem::discriminant(&(Phase::Liquid { viscosity: 0_f32 })))
                            {
                                self.contents.swap(
                                    cellpos,
                                    ((i * col_count) + j + (orientation.signum() * _k)) as usize,
                                );
                                self.contents
                                    [((i * col_count) + j + (orientation.signum() * _k)) as usize]
                                    .updated = true;
                            } else {
                                self.contents[i][j].speed.x *= -1.0;
                                break;
                            }
                        }
                        // This marks that the particle's position has been calculated
                        self.contents[i][j].updated = true;
                    } */
        }
    }
}
