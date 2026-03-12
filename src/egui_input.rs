use crate::system_data::ApplicationOptions;
use crate::system_ui::get_shape;
use crate::{material::Material, world::*};
use crate::{material::VOID, particle::Particle};
use egui::{Key, Response, Vec2, lerp, pos2, vec2};
use std::ops::{AddAssign, Not, RangeInclusive};
use strum_macros::EnumIter;

// Handles mouse/touch controls
pub fn handle_mouse_input(
    game_board: &mut Board,
    materials: &Vec<(String, Material)>,
    selected_material_id: usize,
    response: Response,
) {
    let cursor_position = response.hover_pos().unwrap_or(pos2(-1024_f32, -1024_f32));
    let pos = ((cursor_position - response.interact_rect.min) / game_board.cellsize)
        .floor()
        .to_pos2();
    if response.dragged_by(egui::PointerButton::Primary)
        || response.clicked_by(egui::PointerButton::Primary)
    {
        let material = selected_material_id;
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
                    && (game_board.contents.get(cellpos).unwrap().material_id == VOID.id
                        || selected_material_id == VOID.id)
                {
                    let mut new_particle =
                        Particle::new(&materials[material].1, vec2(0_f32, 0_f32), 293.15);
                    new_particle.display_color = materials[selected_material_id]
                        .1
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            RangeInclusive::new(
                                materials[selected_material_id].1.material_color.shinyness.0,
                                materials[selected_material_id].1.material_color.shinyness.1,
                            ),
                            game_board.rngs[cellpos],
                        ));
                    new_particle.display_color[3] =
                        materials[selected_material_id].1.material_color.color.a();
                    unsafe { write_particle_seq(&game_board.contents, cellpos, new_particle) };
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
    let mouse_scroll = response.ctx.input(|input| input.raw_scroll_delta);
    if mouse_scroll.y.abs() >= 0.1_f32 {
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
