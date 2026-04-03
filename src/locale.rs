use egui::ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::{
    material::Material,
    reactions::{MachineTypes, MaterialType},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Locale {
    pub language_id: String,
    pub language_name: String,
    pub options_title: String,
    pub screen_option_label: String,
    pub reload_button: String,
    pub reload_tooltip: String,
    pub fullscreen_button: String,
    pub fullscreen_tooltip: String,
    pub windowed_button: String,
    pub windowed_tooltip: String,
    pub temperature_scale_label: String,
    pub temperature_kelvin_option: String,
    pub temperature_celsius_option: String,
    pub temperature_fahrenheit_option: String,
    pub gravity_option_label: String,
    pub sun_button: String,
    pub mercury_button: String,
    pub venus_button: String,
    pub moon_button: String,
    pub earth_button: String,
    pub mars_button: String,
    pub jupiter_button: String,
    pub saturn_button: String,
    pub uranus_button: String,
    pub neptune_button: String,
    pub pluto_button: String,
    pub language_label: String,
    pub generate_default_locale_button: String,
    pub board_size_label: String,
    pub options_ok_button: String,
    pub options_cancel_button: String,
    pub brush_size_label: String,
    pub x_axis_label: String,
    pub y_axis_label: String,
    pub reset_button: String,
    pub reset_tooltip: String,
    pub eraser_button: String,
    pub eraser_tooltip: String,
    pub mix_button: String,
    pub mix_tooltip: String,
    pub heat_button: String,
    pub heat_tooltip: String,
    pub default_heat_button: String,
    pub default_heat_tooltip: String,
    pub cool_button: String,
    pub cool_tooltip: String,
    pub rectangle_brush_tooltip: String,
    pub rhombus_brush_tooltip: String,
    pub ellipse_brush_tooltip: String,
    pub element_names: AHashMap<String, String>,
    pub category_names: AHashMap<u8, String>,
}

impl Locale {
    pub fn get_language_name(&self) -> String {
        String::from(&self.language_name)
    }
    pub fn fill_elements(&mut self, materials: Vec<(String, Material)>) {
        for material in materials {
            if !self.element_names.contains_key(&material.0) {
                self.element_names.insert(material.0, String::new());
            }
        }
    }
}

pub fn get_text(locale: &Vec<Locale>, selected_locale: usize) -> &Locale {
    &locale[selected_locale]
}

impl Default for Locale {
    fn default() -> Self {
        Self {
            language_id: String::from("EN"),
            language_name: String::from("English"),
            options_title: String::from("Options"),
            reload_button: String::from("Reload config"),
            reload_tooltip: String::from("Reloads the imported files"),
            gravity_option_label: String::from("Gravity:"),
            sun_button: String::from("Sun"),
            mercury_button: String::from("Mercury"),
            venus_button: String::from("Venus"),
            moon_button: String::from("Moon"),
            earth_button: String::from("Earth"),
            mars_button: String::from("Mars"),
            jupiter_button: String::from("Jupiter"),
            saturn_button: String::from("Saturn"),
            uranus_button: String::from("Uranus"),
            neptune_button: String::from("Neptune"),
            pluto_button: String::from("Pluto"),
            language_label: String::from("Language:"),
            generate_default_locale_button: String::from("Generate default locale"),
            board_size_label: String::from("Board size:"),
            options_ok_button: String::from("Ok"),
            options_cancel_button: String::from("Cancel"),
            screen_option_label: String::from("Screen:"),
            fullscreen_button: String::from("Fullscreen"),
            fullscreen_tooltip: String::from("Maximizes the applications screen."),
            windowed_button: String::from("Windowed"),
            windowed_tooltip: String::from("Sets the application's screen as a resizeable window."),
            temperature_scale_label: String::from("Temperature scale:"),
            temperature_kelvin_option: String::from("Kelvin (K)"),
            temperature_celsius_option: String::from("Celsius (°C)"),
            temperature_fahrenheit_option: String::from("Fahrenheit (°F)"),
            brush_size_label: String::from("Brush size:"),
            x_axis_label: String::from("X axis:"),
            y_axis_label: String::from("Y axis:"),
            reset_button: String::from("Reset"),
            reset_tooltip: String::from("Resets the board."),
            eraser_button: String::from("Eraser"),
            eraser_tooltip: String::from("Selects the eraser tool."),
            mix_button: String::from("Mix"),
            mix_tooltip: String::from("Selects the mixer tool."),
            heat_button: String::from("Heat"),
            heat_tooltip: String::from("Selects the heating tool."),
            default_heat_button: String::from("Normal"),
            default_heat_tooltip: String::from("Selects the normal temperature tool."),
            cool_button: String::from("Cool"),
            cool_tooltip: String::from("Selects the cooling tool."),
            rectangle_brush_tooltip: String::from("Rectangle"),
            rhombus_brush_tooltip: String::from("Rhombus"),
            ellipse_brush_tooltip: String::from("Ellipse"),
            element_names: AHashMap::from([]),
            category_names: AHashMap::from([
                (
                    MaterialType::alloy_default().discriminant(),
                    String::from("Alloys"),
                ),
                (
                    MaterialType::Ceramic.discriminant(),
                    String::from("Ceramic and glass materials"),
                ),
                (
                    MaterialType::Corrosive.discriminant(),
                    String::from("Corrosive materials"),
                ),
                (
                    MaterialType::Decor.discriminant(),
                    String::from("Decorative materials"),
                ),
                (
                    MaterialType::explosive_default().discriminant(),
                    String::from("Explosive materials"),
                ),
                (
                    MaterialType::fuel_default().discriminant(),
                    String::from("Fuels"),
                ),
                (
                    MaterialType::metal_default().discriminant(),
                    String::from("Metals"),
                ),
                (
                    MaterialType::oxidizer_default().discriminant(),
                    String::from("Oxidizers"),
                ),
                (
                    MaterialType::Solution.discriminant(),
                    String::from("Solutions"),
                ),
                (
                    MaterialType::CAutomata {
                        survival: 0_u8,
                        birth: 0_u8,
                        stages: 0_u16,
                    }
                    .discriminant(),
                    String::from("Cellular automatons"),
                ),
                ((MaterialType::Salt).discriminant(), String::from("Salts")),
                (
                    MaterialType::Machine {
                        machine: MachineTypes::default(),
                    }
                    .discriminant(),
                    String::from("Machines"),
                ),
            ]),
        }
    }
}
