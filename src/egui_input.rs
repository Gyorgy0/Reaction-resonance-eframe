use egui::{pos2, Key, PointerButton, Response, Vec2};
use std::ops::Not;

use crate::world::*;

pub fn handle_mouse_input(
    game_board: &mut Board,
    selected_material: &mut Material,
    response: Response,
    ctx: egui::Context,
) {
    let col_count: i32 = game_board.width as i32;
    let cursor_position = response.hover_pos().unwrap_or(pos2(-1024.0, -1024.0));
    let x = (cursor_position.x - 7.5 + game_board.cellsize.x) / game_board.cellsize.x;
    let y = (cursor_position.y - 45.0 + game_board.cellsize.y) / game_board.cellsize.y;
    if response.dragged_by(PointerButton::Primary) || response.clicked_by(PointerButton::Primary) {
        let material = selected_material.clone();
        for i in -(game_board.brushsize / 2) - 1..game_board.brushsize / 2 {
            for j in -(game_board.brushsize / 2) - 1..game_board.brushsize / 2 {
                if game_board
                    .contents
                    .get(((i + (y as i32)) * col_count + (j + x as i32)) as usize)
                    .is_some()
                    && game_board.is_in_bounds(x as i32, j)
                {
                    game_board.contents[((i + (y as i32)) * col_count + (j + x as i32)) as usize] =
                        Particle {
                            material: material.clone(),
                            speed: Vec2::new(0.0, game_board.gravity.signum() * 1.0),
                            temperature: 20.0,
                            updated: true,
                            seed: rand::random_range(0.0..1.0),
                        }
                }
            }
        }
    } else if response.dragged_by(PointerButton::Secondary)
        || response.clicked_by(PointerButton::Secondary)
    {
        let material = VOID.clone();
        for i in -(game_board.brushsize / 2) - 1..game_board.brushsize / 2 {
            for j in -(game_board.brushsize / 2) - 1..game_board.brushsize / 2 {
                if game_board
                    .contents
                    .get(((i + (y as i32)) * col_count + (j + x as i32)) as usize)
                    .is_some()
                    && game_board.is_in_bounds(x as i32, j)
                {
                    game_board.contents[((i + (y as i32)) * col_count + (j + x as i32)) as usize] =
                        Particle {
                            material: material.clone(),
                            speed: Vec2::new(0.0, game_board.gravity.signum() * 1.0),
                            temperature: 20.0,
                            updated: true,
                            seed: rand::random_range(0.0..1.0),
                        }
                }
            }
        }
    };
    // Brush resizing with mouse scroll
    let mouse_scroll = ctx.input(|input| input.raw_scroll_delta);
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
