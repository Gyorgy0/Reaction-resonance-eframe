use crate::{physics::Phase, world::Board};
use serde::{Deserialize, Serialize};

#[derive(Copy, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub(crate) enum Material_Type {
    Acid, // Corrosive material - everything with a pH value lower than 7.0 - for simplicity we use a common amphoter - water H2O
    Alloy, // Mixture of metals
    Atmosphere, // Mixture of materials that are always present in the simulation*/
    Base, // Corrosive material - everything with a pH value higher than 7.0 - for simplicity we use a common amphoter - water H2O
    Ceramic, // Hard, brittle, heat-resistant, and corrosion-resistant material
    Explosive, // A material that generates a lot of energy and lot of gases
    Fuel, // Flammable material under normal circumstances
    Glass, // Amorphous material formed from a molten material and it's cooled without proper crystalization
    Oxidizer, // This material can enhance the explosive power of explosives or the burning of fuels by aiding their combustion
    Solution, // Material that contains other materials e.g. salts, on heat it leaves the dissolved materials behind
    Solvent,  // Dissolves certain materials
}

impl Board {
    #[inline(always)]
    pub(crate) fn solve_reactions(&mut self, i: usize, j: usize, framedelta: f32) {
        if self.contents[(i, j)].material.material_type == Material_Type::Fuel {
            let rnd = rand::random_range(0_u8..4_u8);
            if std::mem::discriminant(
                &self
                    .contents
                    .get(i, j + 1)
                    .unwrap_or(&self.contents[(i, j)])
                    .material
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma { energy: 0_f32 }))
                && self.contents.get(i, j + 1).is_some()
                && rnd == 0
            {
                self.contents[(i, j)] = self.contents[(i, j + 1)].clone();
                self.contents[(i, j)].material.phase = Phase::Plasma { energy: 70.0 };
            } else if std::mem::discriminant(
                &self
                    .contents
                    .get(i, j - 1)
                    .unwrap_or(&self.contents[(i, j)])
                    .material
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma { energy: 0_f32 }))
                && self.contents.get(i, j - 1).is_some()
                && rnd == 1
            {
                self.contents[(i, j)] = self.contents[(i, j - 1)].clone();
                self.contents[(i, j)].material.phase = Phase::Plasma { energy: 70.0 };
            } else if std::mem::discriminant(
                &self
                    .contents
                    .get(i + 1, j)
                    .unwrap_or(&self.contents[(i, j)])
                    .material
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma { energy: 0_f32 }))
                && rnd == 2
            {
                self.contents[(i, j)] = self.contents[(i + 1, j)].clone();
                self.contents[(i, j)].material.phase = Phase::Plasma { energy: 70.0 };
            } else if std::mem::discriminant(
                &self
                    .contents
                    .get(i - 1, j)
                    .unwrap_or(&self.contents[(i, j)])
                    .material
                    .phase,
            ) == std::mem::discriminant(&(Phase::Plasma { energy: 0_f32 }))
                && rnd == 3
            {
                self.contents[(i, j)] = self.contents[(i - 1, j)].clone();
                self.contents[(i, j)].material.phase = Phase::Plasma { energy: 70.0 };
            }
        }
    }
}
