use std::{fs, mem::discriminant};

use egui::{ColorImage, TextureHandle, TextureOptions, epaint::Hsva, text::LayoutJob};
use egui_colorgradient::ColorInterpolator;
use egui_dialogs::Dialogs;
use rand::SeedableRng;
use strum::IntoEnumIterator;

use crate::{
    egui_input::BrushTool,
    locale::Locale,
    material::{AIR, Material},
    particle::Particle,
    physics::{BLACK_BODY_RADIATION_COLORS, PhysicalReactions},
    reactions::{
        BurningReaction, ChemicalReactions, MaterialType, MinglingReaction, PhaseTransition,
    },
    world::Board,
};

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

// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // If we add new fields, give them default values when deserializing old state
pub struct EFrameApp<'a> {
    #[serde(skip)]
    pub viewed_particle: Particle,
    #[serde(skip)]
    pub fps_values: Vec<f32>,
    #[serde(skip)]
    pub physical_transitions: PhysicalReactions,
    #[serde(skip)]
    pub chemical_reactions: ChemicalReactions,
    #[serde(skip)]
    pub black_body_gradient: ColorInterpolator,
    #[serde(skip)]
    pub(crate) dialogs: Dialogs<'a>,
    #[serde(skip)]
    pub debug_text_job: LayoutJob,
    #[serde(skip)]
    pub material_texture: TextureHandle,
    #[serde(skip)]
    pub heatmap_texture: TextureHandle,
    #[serde(skip)]
    pub(crate) game_board: Board,
    #[serde(skip)]
    pub(crate) materials: Vec<(String, Material)>,
    #[serde(skip)]
    pub(crate) material_categories: Vec<Vec<(String, Material)>>,
    #[serde(skip)]
    pub(crate) program_options: ApplicationOptions,
    #[serde(skip)]
    pub(crate) selected_tool: BrushTool,
    #[serde(skip)]
    pub(crate) selected_category: MaterialType,
    #[serde(skip)]
    pub framecount: u64,
    #[serde(skip)]
    pub rng: rand::rngs::SmallRng,
    #[serde(skip)]
    pub(crate) dialogopen: bool,
}

impl Default for EFrameApp<'_> {
    fn default() -> Self {
        // Initializes the default values
        let mut game_board: Board = Board::default();
        let ctx: egui::Context = egui::Context::default();
        let mut program_options: ApplicationOptions = ApplicationOptions::default();

        // Generates the game's board
        game_board.create_board();

        // Initializes the texture handles
        let material_texture = ctx.load_texture(
            "Board".to_string(),
            ColorImage::example(),
            TextureOptions::NEAREST,
        );
        let heatmap_texture = ctx.load_texture(
            "Board_heatmap".to_string(),
            ColorImage::example(),
            TextureOptions::LINEAR,
        );
        let mut locales: Vec<Locale> = vec![];
        let mut materials: Vec<(String, Material)> = vec![];

        // Locale
        locales = import_locales(&mut locales);

        // Materials
        materials = import_materials(&mut materials);

        // Physical transitions
        let physical_transitions: PhysicalReactions = import_transitions();

        // Chemical reactions
        let chemical_reactions: ChemicalReactions = import_reactions();

        // This is for the PC platform
        #[cfg(not(any(target_os = "android", target_arch = "wasm32", target_os = "ios")))]
        {
            use std::fs;

            /*// This is for serializing particles/components with new fields and enums - testing purposes

            let mingling_reactions: Vec<PhaseTransition> = vec![PhaseTransition {
                from: 0_usize,
                to: vec![(0_usize, 1.0_f32)],
            }];
            let data = serde_json::to_string(&mingling_reactions).unwrap();
            println!("{:?}", data);
            fs::write("src/new.json", data).unwrap();
            let serialized_data: Vec<PhaseTransition> =
                serde_json::from_reader(fs::read("src/new.json").unwrap().as_slice()).unwrap();
            println!("{:?}", serialized_data);*/

            // Sorts the elements by their Id's and outputs them to a list
            materials.sort_by_key(|elem| elem.1.id);
            let mut material_ids: Vec<(usize, String)> = vec![];
            materials.iter().for_each(|element| {
                if discriminant(&element.1.material_type)
                    == discriminant(&MaterialType::Alloy { metals: vec![] })
                {
                    let mut components = String::new();
                    for metal in element.1.material_type.get_alloy_components().iter() {
                        components += format!(
                            "{component} {percent}% ",
                            component = materials[metal.0].0,
                            percent = (metal.1 * 100_f32)
                        )
                        .as_str();
                    }
                    material_ids.push((
                        element.1.id,
                        format!(
                            "{element_name} - ({components})",
                            element_name = element.0.clone()
                        ),
                    ));
                } else {
                    material_ids.push((element.1.id, element.0.clone()));
                }
            });
            fs::write(
                "src/material_ids.json",
                serde_json::to_string_pretty(&material_ids).unwrap(),
            )
            .unwrap();
        }

        program_options.locale = locales;
        let mut material_categories: Vec<Vec<(String, Material)>> = vec![];
        // Sort material by their ID's
        materials.sort_by_key(|elem| elem.1.id);
        for category in MaterialType::iter() {
            let mut category_vec: Vec<(String, Material)> = vec![];
            for material in materials.iter() {
                if discriminant(&category) == discriminant(&material.1.material_type) {
                    category_vec.push(material.clone());
                }
            }
            material_categories.push(category_vec);
        }
        let selected_tool = BrushTool::MaterialBrush {
            selected_material: 0_usize,
        };
        let selected_category = MaterialType::fuel_default();
        let debug_text_job = LayoutJob::default();
        let stops: Vec<(f32, Hsva)> = BLACK_BODY_RADIATION_COLORS
            .iter()
            .map(|stop| (stop.0, stop.1.into()))
            .collect();
        Self {
            viewed_particle: Particle::default(),
            fps_values: vec![0_f32; 256_usize],
            black_body_gradient: egui_colorgradient::Gradient::new(
                egui_colorgradient::InterpolationMethod::Constant,
                stops,
            )
            .interpolator(),
            physical_transitions,
            chemical_reactions,
            debug_text_job,
            game_board,
            materials,
            material_categories,
            material_texture,
            heatmap_texture,
            selected_tool,
            selected_category,
            program_options,
            framecount: 0,
            rng: rand::rngs::SmallRng::seed_from_u64(0_u64),
            dialogs: Dialogs::default(),
            dialogopen: false,
        }
    }
}

pub fn import_locales(locales: &mut Vec<Locale>) -> Vec<Locale> {
    locales.clear();
    #[cfg(not(any(target_os = "android", target_arch = "wasm32", target_os = "ios")))]
    {
        // Materials - PC version (loads them from the src/materials folder)
        let paths = fs::read_dir("src/locale").unwrap();
        for path in paths {
            if path
                .as_ref()
                .is_ok_and(|path| path.file_name() != "default_locale.json")
            {
                let locale: Result<Vec<u8>, std::io::Error> =
                    fs::read(path.as_ref().unwrap().path().display().to_string().as_str());
                let serialized_locale: Locale = serde_json::from_reader(locale.unwrap().as_slice())
                    .unwrap_or(Locale::default());
                locales.push(serialized_locale);
            }
        }
    }
    #[cfg(any(target_os = "android", target_arch = "wasm32", target_os = "ios"))]
    {
        // Materials - Portable version (includes the files in src/materials in the executable file)

        use serde_json::from_str;

        use crate::included_files::FILES;
        // Locale
        locales.push(from_str(&FILES.locales.locale_en).unwrap());
        locales.push(from_str(&FILES.locales.locale_hu).unwrap());
        locales.push(from_str(&FILES.locales.locale_sk).unwrap());

        locales.append(&mut serialized_materials);
    }
    locales.clone()
}

pub fn import_materials(materials: &mut Vec<(String, Material)>) -> Vec<(String, Material)> {
    materials.clear();
    let mut materials: Vec<(String, Material)> = vec![(String::new(), AIR.clone())];
    #[cfg(not(any(target_os = "android", target_arch = "wasm32", target_os = "ios")))]
    {
        // Materials - PC version (loads them from the src/materials folder)
        let paths = fs::read_dir("src/materials/").unwrap();
        for path in paths {
            let materials_per_phase: Result<Vec<u8>, std::io::Error> =
                fs::read(path.as_ref().unwrap().path().display().to_string().as_str());
            let mut serialized_materials: Vec<(String, Material)> =
                serde_json::from_reader(materials_per_phase.unwrap().as_slice()).unwrap();
            materials.append(&mut serialized_materials);
        }
    }
    #[cfg(any(target_os = "android", target_arch = "wasm32", target_os = "ios"))]
    {
        // Materials - Portable version (includes the files in src/materials in the executable file)

        use serde_json::from_str;

        use crate::included_files::FILES;
        let mut serialized_materials: Vec<(String, Material)> =
            from_str(&FILES.materials.solid_materials).unwrap();

        serialized_materials.append(
            (from_str(&FILES.materials.powder_materials))
                .as_mut()
                .unwrap(),
        );

        serialized_materials.append(
            from_str(&FILES.materials.liquid_materials)
                .as_mut()
                .unwrap(),
        );

        serialized_materials.append(from_str(&FILES.materials.gas_materials).as_mut().unwrap());

        serialized_materials.append(
            from_str(&FILES.materials.plasma_materials)
                .as_mut()
                .unwrap(),
        );

        serialized_materials.append(from_str(&FILES.materials.life_materials).as_mut().unwrap());

        materials.append(&mut serialized_materials);
    }
    materials.clone()
}

pub fn import_transitions() -> PhysicalReactions {
    let serialized_transition_melting: Vec<PhaseTransition>;
    let serialized_transition_boiling: Vec<PhaseTransition>;
    let serialized_transition_sublimation: Vec<PhaseTransition>;

    #[cfg(not(any(target_os = "android", target_arch = "wasm32", target_os = "ios")))]
    {
        // Physical transitions - PC version (loads phase change files from src/physics)
        let transition_path_melting = fs::read("src/physics/phase_transitions_melting.json");
        serialized_transition_melting =
            serde_json::from_reader(transition_path_melting.unwrap().as_slice()).unwrap();
        let transition_path_boiling = fs::read("src/physics/phase_transitions_boiling.json");
        serialized_transition_boiling =
            serde_json::from_reader(transition_path_boiling.unwrap().as_slice()).unwrap();
        let transition_path_sublimation =
            fs::read("src/physics/phase_transitions_sublimation.json");
        serialized_transition_sublimation =
            serde_json::from_slice(transition_path_sublimation.unwrap().as_slice()).unwrap();
    }
    #[cfg(any(target_os = "android", target_arch = "wasm32", target_os = "ios"))]
    {
        // Physical transitions - Portable version
        serialized_transition_melting =
            from_str(&FILES.physics_transition.melting_transitions).unwrap();
        serialized_transition_boiling =
            from_str(&FILES.physics_transition.boiling_transitions).unwrap();
        serialized_transition_sublimation =
            from_str(&FILES.physics_transition.sublimation_transitions).unwrap();
    }
    PhysicalReactions {
        melting: serialized_transition_melting,
        boiling: serialized_transition_boiling,
        sublimation: serialized_transition_sublimation,
    }
}

pub fn import_reactions() -> ChemicalReactions {
    let serialized_reactions_burning: Vec<BurningReaction>;
    let serialized_reactions_mingling: Vec<MinglingReaction>;

    #[cfg(not(any(target_os = "android", target_arch = "wasm32", target_os = "ios")))]
    {
        // Chemical reactions - PC version (loads reaction files from src/chemistry)
        let reaction_path_burning = fs::read("src/chemistry/chemical_reactions_burning.json");
        serialized_reactions_burning =
            serde_json::from_reader(reaction_path_burning.unwrap().as_slice()).unwrap();
        let reaction_path_mingling = fs::read("src/chemistry/chemical_reactions_mingling.json");

        serialized_reactions_mingling =
            serde_json::from_slice(reaction_path_mingling.unwrap().as_slice()).unwrap();
    }
    #[cfg(any(target_os = "android", target_arch = "wasm32", target_os = "ios"))]
    {
        // Chemical reactions
        serialized_reactions_burning =
            from_str(&FILES.chemical_reactions.burning_reactions).unwrap();
        serialized_reactions_mingling =
            from_str(&FILES.chemical_reactions.mingling_reactions).unwrap();
    }
    ChemicalReactions {
        burning: serialized_reactions_burning,
        mingling: serialized_reactions_mingling,
    }
}
