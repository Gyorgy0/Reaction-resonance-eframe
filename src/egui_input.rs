use crate::particle::Particle;
use crate::physics::PhysicalReactions;
use crate::reactions::ChemicalReactions;
use crate::system_data::ApplicationOptions;
use crate::system_ui::get_shape;
use crate::{material::Material, world::*};
use egui::{Key, Response, Vec2, lerp, pos2, vec2};
use std::ops::{AddAssign, Not, RangeInclusive};
use strum_macros::EnumIter;

// Handles mouse/touch controls
pub fn handle_mouse_input(
    game_board: &mut Board,
    materials: &Vec<(String, Material)>,
    selected_tool: &BrushTool,
    response: Response,
) {
    let cursor_position = response.hover_pos().unwrap_or(pos2(-1024_f32, -1024_f32));
    let pos = ((cursor_position - response.interact_rect.min) / game_board.cellsize)
        .floor()
        .to_pos2();
    if response.dragged_by(egui::PointerButton::Primary)
        || response.clicked_by(egui::PointerButton::Primary)
    {
        for y in -game_board.brush_size.y as i64..=game_board.brush_size.y as i64 {
            for x in -game_board.brush_size.x as i64..=game_board.brush_size.x as i64 {
                let cellpos = get_safe_i(
                    &(game_board.height as usize),
                    &(game_board.width as usize),
                    &((y + pos.y as i64) as usize, (x + pos.x as i64) as usize),
                );

                if get_shape(game_board.brush_shape, game_board.brush_size, x, y).1
                    && game_board.contents.get(cellpos).is_some()
                {
                    get_tool_action(materials, selected_tool, cellpos, game_board);
                }
            }
        }
    } else if response.dragged_by(egui::PointerButton::Secondary)
        || response.clicked_by(egui::PointerButton::Secondary)
    {
        for y in -game_board.brush_size.y as i64..=game_board.brush_size.y as i64 {
            for x in -game_board.brush_size.x as i64..=game_board.brush_size.x as i64 {
                let cellpos = get_safe_i(
                    &(game_board.height as usize),
                    &(game_board.width as usize),
                    &(
                        (y.saturating_add(pos.y as i64)) as usize,
                        (x.saturating_add(pos.x as i64)) as usize,
                    ),
                );
                if get_shape(game_board.brush_shape, game_board.brush_size, x, y).1
                    && game_board.contents.get(cellpos).is_some()
                {
                    unsafe {
                        write_particle_seq(&game_board.contents, cellpos, Particle::default())
                    };
                }
            }
        }
    };
    // Brush resizing with mouse scroll
    let mouse_scroll = response.ctx.input(|input| input.smooth_scroll_delta);
    let time_since_last_scroll = response.ctx.input(|input| input.time_since_last_scroll());
    if mouse_scroll.y.abs() > 1_f32 && time_since_last_scroll < 0.001_f32 {
        resize_brush(
            &mut game_board.brush_size,
            Vec2::splat(1_f32 * (mouse_scroll.y.signum())),
        );
    }
}

// Handles keyboard inputs
pub fn handle_key_inputs(
    game_board: &mut Board,
    materials: &Vec<(String, Material)>,
    physical_transitions: &PhysicalReactions,
    chemical_reactions: &ChemicalReactions,
    program_options: &mut ApplicationOptions,
    framecount: &mut u64,
    framedelta: f32,
    response: Response,
) {
    // R key - Resets the board
    if response.ctx.input(|i| i.key_pressed(Key::R)) {
        game_board.create_board();
    }
    // Space key - Pauses the simulation
    if response.ctx.input(|i| i.key_pressed(Key::Space)) {
        program_options.simulation_stopped = program_options.simulation_stopped.not();
    }
    // A key - Advances the simulation by one step
    if response.ctx.input(|i| i.key_pressed(Key::A)) {
        program_options.simulation_stopped = false;
        update_board(
            game_board,
            materials,
            physical_transitions,
            chemical_reactions,
            program_options.simulation_stopped,
            framecount,
            framedelta,
        );
        program_options.simulation_stopped = true;
    }
    // F3 key - Toggles debug mode
    if response.ctx.input(|i| i.key_pressed(Key::F3)) {
        program_options.debug_mode = program_options.debug_mode.not();
    }
}

#[derive(PartialEq, Copy, Clone, Debug, EnumIter)]
#[repr(u8)]
pub(crate) enum BrushShape {
    Rectangle,
    Rhombus,
    Ellipse,
}

pub fn resize_brush(brush_size: &mut Vec2, change: Vec2) {
    brush_size.add_assign(change);
    *brush_size = brush_size.clamp(vec2(0_f32, 0_f32), vec2(256_f32, 256_f32));
}

#[derive(Debug, PartialEq)]
pub(crate) enum BrushTool {
    MaterialBrush { selected_material: usize },
    ThermalBrush { temp_delta: f32, default_temp: bool },
    MixBrush,
    EraseBrush,
}
impl BrushTool {
    pub fn get_selected_material(&self) -> usize {
        let mut returnval: usize = 0_usize;
        if let BrushTool::MaterialBrush { selected_material } = self {
            returnval = *selected_material;
        };
        returnval
    }

    pub fn get_temp_delta(&self) -> f32 {
        let mut returnval: f32 = 0_f32;
        if let BrushTool::ThermalBrush {
            temp_delta,
            default_temp: _,
        } = self
        {
            returnval = *temp_delta;
        };
        returnval
    }

    pub fn get_default_temp(&self) -> bool {
        let mut returnval: bool = false;
        if let BrushTool::ThermalBrush {
            temp_delta: _,
            default_temp,
        } = self
        {
            returnval = *default_temp;
        };
        returnval
    }
}

pub fn get_tool_action(
    materials: &Vec<(String, Material)>,
    selected_tool: &BrushTool,
    cellpos: usize,
    game_board: &mut Board,
) {
    match selected_tool {
        BrushTool::MaterialBrush {
            selected_material: _,
        } => {
            let mut new_particle = Particle::new(
                &materials[selected_tool.get_selected_material()].1,
                vec2(0_f32, 0_f32),
                293.15,
            );
            new_particle.particle_health = materials[selected_tool.get_selected_material()]
                .1
                .material_type
                .get_max_stage();
            new_particle.temperature = materials[new_particle.material_id].1.initial_temperature;
            new_particle.display_color = materials[new_particle.material_id]
                .1
                .material_color
                .color
                .gamma_multiply(lerp(
                    RangeInclusive::new(
                        materials[new_particle.material_id]
                            .1
                            .material_color
                            .shinyness
                            .0,
                        materials[new_particle.material_id]
                            .1
                            .material_color
                            .shinyness
                            .1,
                    ),
                    game_board.rngs[cellpos],
                ));
            new_particle.display_color[3] = materials[new_particle.material_id]
                .1
                .material_color
                .color
                .a();
            unsafe { write_particle_seq(&game_board.contents, cellpos, new_particle) };
        }

        BrushTool::ThermalBrush {
            temp_delta: _,
            default_temp,
        } => {
            let mut new_particle = *game_board.contents.get_elem(cellpos);
            if !*default_temp {
                new_particle.temperature += selected_tool.get_temp_delta();
                new_particle.temperature = new_particle.temperature.clamp(0_f32, 99_999_f32);
            } else {
                new_particle.temperature = 273.15_f32;
            }
            unsafe { write_particle_seq(&game_board.contents, cellpos, new_particle) };
        }

        BrushTool::MixBrush => todo!(),

        BrushTool::EraseBrush => {
            let new_particle = Particle::default();
            unsafe { write_particle_seq(&game_board.contents, cellpos, new_particle) };
        }
    }
}
