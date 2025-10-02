use egui::{Key, PointerButton, Response, Vec2, lerp, pos2};
use std::ops::Not;

use crate::{physics::Phase, world::*};

pub fn handle_mouse_input(
    game_board: &mut Board,
    selected_material: &mut Material,
    response: Response,
) {
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
                    .get((i + pos.y as i32) as usize, (j + pos.x as i32) as usize)
                    .is_some()
                    && (game_board
                        .contents
                        .get((i + pos.y as i32) as usize, (j + pos.x as i32) as usize)
                        .unwrap()
                        .material
                        == VOID
                        || selected_material.phase == Phase::Void)
                {
                    game_board.contents
                        [((i + pos.y as i32) as usize, (j + pos.x as i32) as usize)] = Particle {
                        material: material.clone(),
                        speed: Vec2::new(0.0, game_board.gravity.signum() * 1.0),
                        temperature: 20.0,
                        updated: true,
                    };
                    game_board.contents
                        [((i + pos.y as i32) as usize, (j + pos.x as i32) as usize)]
                        .material
                        .material_color
                        .color = game_board.contents
                        [((i + pos.y as i32) as usize, (j + pos.x as i32) as usize)]
                        .material
                        .material_color
                        .color
                        .gamma_multiply(lerp(
                            game_board.contents
                                [((i + pos.y as i32) as usize, (j + pos.x as i32) as usize)]
                                .material
                                .material_color
                                .shinyness
                                .clone(),
                            game_board.rngs
                                [((i + pos.y as i32) as usize, (j + pos.x as i32) as usize)],
                        ));
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
                    .get((i + pos.y as i32) as usize, (j + pos.x as i32) as usize)
                    .is_some()
                {
                    game_board.contents
                        [((i + pos.y as i32) as usize, (j + pos.x as i32) as usize)] = Particle {
                        material: material.clone(),
                        speed: Vec2::new(0.0, game_board.gravity.signum() * 1.0),
                        temperature: 20.0,
                        updated: true,
                    };
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
