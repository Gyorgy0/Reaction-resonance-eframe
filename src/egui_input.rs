use egui::{Key, Response, Vec2, lerp, pos2};
use std::{cell, ops::Not};

use crate::world::*;

pub fn handle_mouse_input(
    game_board: &mut Board,
    materials: &Vec<Material>,
    selected_material_id: u64,
    response: Response,
) {
    let cursor_position = response.hover_pos().unwrap_or(pos2(-1024_f32, -1024_f32));
    let pos = ((cursor_position - response.interact_rect.min) / game_board.cellsize)
        .floor()
        .to_pos2();
    if response.dragged_by(egui::PointerButton::Primary)
        || response.clicked_by(egui::PointerButton::Primary)
    {
        let material = selected_material_id.clone();
        for i in -(game_board.brushsize / 2)..=game_board.brushsize / 2 {
            for j in -(game_board.brushsize / 2)..=game_board.brushsize / 2 {
                let cellpos = ((i + pos.y as i32) as usize, (j + pos.x as i32) as usize);
                if game_board.contents.get(cellpos.0, cellpos.1).is_some()
                    && (game_board
                        .contents
                        .get(cellpos.0, cellpos.1)
                        .unwrap()
                        .material_id
                        == VOID.id
                        || selected_material_id == VOID.id)
                {
                    game_board.contents[cellpos] = Particle {
                        material_id: selected_material_id,
                        speed: Vec2::new(0_f32, game_board.gravity.signum() * 1_f32),
                        temperature: 20_f32,
                        display_color: materials[selected_material_id as usize]
                            .material_color
                            .color,
                    };
                    game_board.contents[cellpos].display_color = materials
                        [selected_material_id as usize]
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            materials[selected_material_id as usize]
                                .material_color
                                .shinyness
                                .clone(),
                            game_board.rngs[cellpos],
                        ));
                    game_board.contents[cellpos].display_color[3] = materials
                        [selected_material_id as usize]
                        .material_color
                        .color
                        .a();
                }
            }
        }
    } else if response.dragged_by(egui::PointerButton::Secondary)
        || response.clicked_by(egui::PointerButton::Secondary)
    {
        let material = VOID.clone();
        for i in -(game_board.brushsize / 2)..=game_board.brushsize / 2 {
            for j in -(game_board.brushsize / 2)..=game_board.brushsize / 2 {
                let cellpos = ((i + pos.y as i32) as usize, (j + pos.x as i32) as usize);
                if game_board.contents.get(cellpos.0, cellpos.1).is_some() {
                    game_board.contents[cellpos] = Particle {
                        material_id: 0,
                        speed: Vec2::new(0_f32, game_board.gravity.signum() * 1_f32),
                        temperature: 20_f32,
                        display_color: material.material_color.color,
                    };
                }
            }
        }
    };
    // Brush resizing with mouse scroll
    let mouse_scroll = response.ctx.input(|input| input.raw_scroll_delta);
    if mouse_scroll.y.abs() >= 0.1
        && ((game_board.brushsize > 1 && mouse_scroll.y.signum() == -1_f32)
            || (game_board.brushsize < 256 && mouse_scroll.y.signum() == 1_f32))
    {
        game_board.brushsize += 2 * (mouse_scroll.y.signum()) as i32;
    }
}

pub fn handle_key_inputs(game_board: &mut Board, is_paused: &mut bool, response: Response) {
    if response.ctx.input(|i| i.key_pressed(Key::R)) {
        game_board.create_board();
    } else if response.ctx.input(|i| i.key_pressed(Key::Space)) {
        *is_paused = is_paused.not();
    }
}
