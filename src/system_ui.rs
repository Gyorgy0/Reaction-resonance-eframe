use std::fmt::{self};

use egui::Color32;

use crate::physics::Phase;
use crate::reactions::MaterialType;
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

impl fmt::Display for MaterialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            MaterialType::Acid => write!(f, "Acid"),
            MaterialType::Alloy => write!(f, "Alloy"),
            MaterialType::Atmosphere => write!(f, "Atmosphere"),
            MaterialType::Base => write!(f, "Base"),
            MaterialType::Ceramic => write!(f, "Ceramic"),
            MaterialType::Explosive => write!(f, "Explosive"),
            MaterialType::Fuel => write!(f, "Fuel"),
            MaterialType::Glass => write!(f, "Glass"),
            MaterialType::Oxidizer => write!(f, "Oxidizer"),
            MaterialType::Decor => write!(f, "Decor"),
            MaterialType::Solution => write!(f, "Solution"),
            MaterialType::Solvent => write!(f, "Solvent"),
        }
    }
}

impl Board {
    pub fn draw_board(&mut self) -> Vec<Color32> {
        let f: Vec<Color32> = self
            .contents
            .iter()
            .map(|px| px.material.material_color.color)
            .collect();
        f
    }
}
