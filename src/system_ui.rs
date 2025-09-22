use std::fmt::{self};

use egui::Color32;

use crate::physics::Phase;
use crate::reactions::Material_Type;
use crate::world::Board;

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Phase::Void => write!(f, ""),
            Phase::Solid { melting_point: _ } => write!(f, "Solid"),
            Phase::Powder {
                coarseness: _,
                melting_point: _,
            } => write!(f, "Powder"),
            Phase::Liquid {
                viscosity: _,
                boiling_point: _,
            } => write!(f, "Liquid"),
            Phase::Gas {} => write!(f, "Gas"),
            Phase::Plasma { energy: _ } => write!(f, "Plasma"),
        }
    }
}

impl fmt::Display for Material_Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Material_Type::Acid => write!(f, "Acid"),
            Material_Type::Alloy => write!(f, "Alloy"),
            Material_Type::Atmosphere => write!(f, "Atmosphere"),
            Material_Type::Base => write!(f, "Base"),
            Material_Type::Ceramic => write!(f, "Ceramic"),
            Material_Type::Explosive => write!(f, "Explosive"),
            Material_Type::Fuel => write!(f, "Fuel"),
            Material_Type::Glass => write!(f, "Glass"),
            Material_Type::Oxidizer => write!(f, "Oxidizer"),
            Material_Type::Solution => write!(f, "Solution"),
            Material_Type::Solvent => write!(f, "Solvent"),
        }
    }
}

impl Board {
    pub fn draw_board(&mut self) -> Vec<Color32> {
        let f: Vec<Color32> = self.contents.iter().map(|px| px.material.color).collect();
        f
    }
}
