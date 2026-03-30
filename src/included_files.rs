pub struct MaterialFolder<'a> {
    pub solid_materials: &'a str,
    pub powder_materials: &'a str,
    pub liquid_materials: &'a str,
    pub gas_materials: &'a str,
    pub plasma_materials: &'a str,
    pub life_materials: &'a str,
}

pub struct PhysicsFolder<'a> {
    pub melting_transitions: &'a str,
    pub boiling_transitions: &'a str,
    pub sublimation_transitions: &'a str,
}

pub struct ChemistryFolder<'a> {
    pub alloy_reactions: &'a str,
    pub burning_reactions: &'a str,
    pub metal_reactions: &'a str,
}

pub struct LocaleFolder<'a> {
    pub locale_en: &'a str,
    pub locale_hu: &'a str,
    pub locale_sk: &'a str,
}

pub struct IncludedFiles<'a> {
    pub materials: MaterialFolder<'a>,
    pub physics_transition: PhysicsFolder<'a>,
    pub chemical_reactions: ChemistryFolder<'a>,
    pub locales: LocaleFolder<'a>,
}

pub const FILES: IncludedFiles = IncludedFiles {
    materials: MATERIAL_FILES,
    physics_transition: PHYSICS_FILES,
    chemical_reactions: CHEMISTRY_FILES,
    locales: LOCALE_FILES,
};

const MATERIAL_FILES: MaterialFolder = MaterialFolder {
    solid_materials: include_str!("materials/solid.json"),
    powder_materials: include_str!("materials/powder.json"),
    liquid_materials: include_str!("materials/liquid.json"),
    gas_materials: include_str!("materials/gas.json"),
    plasma_materials: include_str!("materials/plasma.json"),
    life_materials: include_str!("materials/life.json"),
};

const PHYSICS_FILES: PhysicsFolder = PhysicsFolder {
    melting_transitions: include_str!("physics/phase_transitions_melting.json"),
    boiling_transitions: include_str!("physics/phase_transitions_boiling.json"),
    sublimation_transitions: include_str!("physics/phase_transitions_sublimation.json"),
};
const CHEMISTRY_FILES: ChemistryFolder = ChemistryFolder {
    alloy_reactions: include_str!("chemistry/chemical_reactions_alloys.json"),
    burning_reactions: include_str!("chemistry/chemical_reactions_burning.json"),
    metal_reactions: include_str!("chemistry/chemical_reactions_metals.json"),
};
const LOCALE_FILES: LocaleFolder = LocaleFolder {
    locale_en: include_str!("locale/locale_en.json"),
    locale_hu: include_str!("locale/locale_hu.json"),
    locale_sk: include_str!("locale/locale_sk.json"),
};
