use crate::locale::Locale;

#[derive(Default, Clone)]
pub(crate) struct ApplicationOptions {
    pub simulation_stopped: bool,
    pub selected_locale: usize,
    pub locale: Vec<Locale>,
    pub fullscreen: bool,
    pub temperature_scale: TemperatureScale,
    pub debug_mode: bool,
}

#[derive(Default, Clone, Copy, PartialEq, PartialOrd)]
pub enum TemperatureScale {
    #[default]
    Kelvin,
    Celsius,
    Fahrenheit,
}

pub fn get_temperature(scale: TemperatureScale, temperature: f32) -> f32 {
    match scale {
        TemperatureScale::Kelvin => temperature,
        TemperatureScale::Celsius => temperature - 273.15_f32,
        TemperatureScale::Fahrenheit => ((temperature - 273.15_f32) * 1.8_f32) + 32_f32,
    }
}

pub fn get_sign(scale: TemperatureScale) -> String {
    match scale {
        TemperatureScale::Kelvin => String::from("K"),
        TemperatureScale::Celsius => String::from("°C"),
        TemperatureScale::Fahrenheit => String::from("°F"),
    }
}
