use egui::{pos2, Key, PointerButton, Response, Vec2};
use std::ops::Not;

use crate::world::*;

pub fn handle_mouse_input(
    game_board: &mut Board,
    selected_material: &mut Material,
    response: Response,
) {
    let col_count: i32 = game_board.width as i32;
    let cursor_position = response.hover_pos().unwrap_or(pos2(-1024.0, -1024.0));
    let pos = ((cursor_position - response.interact_rect.min) / game_board.cellsize)
        .floor()
        .to_pos2();
    if response.dragged_by(PointerButton::Primary) || response.clicked_by(PointerButton::Primary) {
        let material = selected_material.clone();
        for i in -(game_board.brushsize / 2)..=game_board.brushsize / 2 {
            for j in -(game_board.brushsize / 2)..=game_board.brushsize / 2 {
                if game_board
                    .contents
                    .get(((i + (pos.y as i32)) * col_count + (j + pos.x as i32)) as usize)
                    .is_some()
                    && game_board.is_in_bounds(pos.x as i32, j)
                {
                    game_board.contents
                        [((i + (pos.y as i32)) * col_count + (j + pos.x as i32)) as usize] =
                        Particle {
                            material: material.clone(),
                            speed: Vec2::new(0.0, game_board.gravity.signum() * 1.0),
                            temperature: 20.0,
                            updated: true,
                        }
                }
            }
        }
    } else if response.dragged_by(PointerButton::Secondary)
        || response.clicked_by(PointerButton::Secondary)
    {
        let material = VOID.clone();
        for i in -(game_board.brushsize / 2)..=game_board.brushsize / 2 {
            for j in -(game_board.brushsize / 2)..=game_board.brushsize / 2 {
                if game_board
                    .contents
                    .get(((i + (pos.y as i32)) * col_count + (j + pos.x as i32)) as usize)
                    .is_some()
                    && game_board.is_in_bounds(pos.x as i32, j)
                {
                    game_board.contents
                        [((i + (pos.y as i32)) * col_count + (j + pos.x as i32)) as usize] =
                        Particle {
                            material: material.clone(),
                            speed: Vec2::new(0.0, game_board.gravity.signum() * 1.0),
                            temperature: 20.0,
                            updated: true,
                        }
                }
            }
        }
    };
    // Brush resizing with mouse scroll
    let mouse_scroll = response.ctx.input(|input| input.raw_scroll_delta);
    if mouse_scroll.y.abs() >= 0.1
        && ((game_board.brushsize > 1 && mouse_scroll.y.signum() == -1.0)
            || (game_board.brushsize < 256 && mouse_scroll.y.signum() == 1.0))
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
