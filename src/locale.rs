use egui::ahash::AHashMap;
use serde::{Deserialize, Serialize};

use crate::reactions::{MachineTypes, MaterialType};

#[derive(Debug, Serialize, Deserialize)]
pub struct Locale {
    options_title: String,
    screen_option_label: String,
    fullscreen_button: String,
    fullscreen_tooltip: String,
    windowed_button: String,
    windowed_tooltip: String,
    gravity_option_label: String,
    brush_size_label: String,
    x_axis_label: String,
    y_axis_label: String,
    reset_button: String,
    reset_tooltip: String,
    eraser_button: String,
    eraser_tooltip: String,
    heat_button: String,
    heat_tooltip: String,
    cool_button: String,
    cool_tooltip: String,
    rectangle_brush_tooltip: String,
    rhombus_brush_tooltip: String,
    ellipse_brush_tooltip: String,
    element_names: AHashMap<String, String>,
    category_names: AHashMap<u8, String>,
}

impl Default for Locale {
    fn default() -> Self {
        Self {
            options_title: String::from("Options"),

            screen_option_label: String::from("Screen:"),
            fullscreen_button: String::from("Fullscreen"),
            fullscreen_tooltip: String::from("Maximizes the applications screen."),
            windowed_button: String::from("Windowed"),
            windowed_tooltip: String::from("Sets the application's screen as a resizeable window."),
            gravity_option_label: String::from("Gravity:"),
            brush_size_label: String::from("Brush size:"),
            x_axis_label: String::from("X axis:"),
            y_axis_label: String::from("Y axis:"),
            reset_button: String::from("Reset"),
            reset_tooltip: String::from("Resets the board."),
            eraser_button: String::from("Eraser"),
            eraser_tooltip: String::from("Selects the eraser tool."),
            heat_button: String::from("Heat"),
            heat_tooltip: String::from("Selects the heating tool."),
            cool_button: String::from("Cool"),
            cool_tooltip: String::from("Selects the cooling tool."),
            rectangle_brush_tooltip: String::from("Rectangle"),
            rhombus_brush_tooltip: String::from("Rhombus"),
            ellipse_brush_tooltip: String::from("Ellipse"),
            element_names: AHashMap::from([
                (String::from("GAS:METHANE"), String::from("Methane")),
                (String::from("LIQUID:WATER"), String::from("Water")),
                (String::from("LIQUID:MILK"), String::from("Milk")),
                (String::from("LIQUID:HONEY"), String::from("Honey")),
                (String::from("PLASMA:FIRE"), String::from("Fire")),
                (String::from("POWDER:SAND"), String::from("Sand")),
                (
                    String::from("POWDER:COLORFUL_SAND"),
                    String::from("Colorful sand"),
                ),
                (String::from("POWDER:SAWDUST"), String::from("Sawdust")),
                (String::from("SOLID:WOOD"), String::from("Wood")),
                (String::from("SOLID:CLONER"), String::from("Cloner")),
                (String::from("SOLID:SINK"), String::from("Sink")),
                (String::from("LIFE:GOL"), String::from("Game of Life")),
                (String::from("LIFE:MAZE"), String::from("Maze")),
                (
                    String::from("LIFE:WALLED_CITIES"),
                    String::from("Walled cities"),
                ),
                (String::from("LIFE:DIAMOEBA"), String::from("Diamoeba")),
                (String::from("LIFE:STAR_WARS"), String::from("Star wars")),
            ]),
            category_names: AHashMap::from([
                (MaterialType::Alloy.discriminant(), String::from("Alloys")),
                (
                    MaterialType::Ceramic.discriminant(),
                    String::from("Ceramic and glass materials"),
                ),
                (
                    (MaterialType::Corrosive {
                        ph_value: f32::default(),
                        blacklist: bool::default(),
                        material_list: vec![],
                    })
                    .discriminant(),
                    String::from("Corrosive materials"),
                ),
                (
                    MaterialType::Decor.discriminant(),
                    String::from("Decorative materials"),
                ),
                (
                    MaterialType::Explosive.discriminant(),
                    String::from("Explosive materials"),
                ),
                (MaterialType::Fuel.discriminant(), String::from("Fuels")),
                (MaterialType::Metal.discriminant(), String::from("Metals")),
                (
                    MaterialType::Oxidizer.discriminant(),
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
                        stages: 0_u8,
                    }
                    .discriminant(),
                    String::from("Cellular automatons"),
                ),
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
