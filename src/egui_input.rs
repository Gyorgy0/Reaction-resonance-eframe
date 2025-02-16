use egui::{Pos2, Response};
use rand::Rng;

use crate::world::*;

pub fn handle_mouse_input(game_board: &mut Board, selected_material: &mut Material, response: Response) {
    let row_count: i32 = game_board.height as i32;
    let col_count: i32 = game_board.width as i32;
    let cursor_position = response.hover_pos().unwrap_or_default();
    let x = (cursor_position.x - 5.0) / game_board.cellsize.x;
    let y = (cursor_position.y - 25.0) / game_board.cellsize.y;
    /*if (response.clicked() || response.secondary_clicked())
        && cursor_position.x > game_board.cellsize.x - 5.0
        && cursor_position.x < (game_board.cellsize.x * col_count as f32)
        && cursor_position.y > game_board.cellsize.y + 25.0
        && cursor_position.y < (game_board.cellsize.y * row_count as f32) + 60.0
    {*/
        let material = if response.dragged() {
            selected_material.clone()
        } else {
            VOID.clone()
        };
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
                            speed: vec2_f32::new(0.0, game_board.gravity.signum() * 1.0),
                            temperature: 20.0,
                            updated: true,
                            seed: rand::thread_rng().gen_range(0.0, 1.0),
                        }
                }
            }
        }
    //}
    /*if (mouse_wheel().1 > -120.0 && mouse_wheel().1 <= -60.0 && game_board.brushsize < row_count)
        || (mouse_wheel().1 < 120.0 && mouse_wheel().1 >= 60.0 && game_board.brushsize > 2)
    {
        game_board.brushsize -= 2 * (mouse_wheel().1 / 60.0) as i32;
    } else if (mouse_wheel().1 <= -120.0 && game_board.brushsize < row_count)
        || (mouse_wheel().1 >= 120.0 && game_board.brushsize > 2)
    {
        game_board.brushsize -= 2 * (mouse_wheel().1 / 120.0) as i32;
    }*/
}

/*fn handle_key_inputs(game_board: &mut Board, is_paused: &mut bool) {
    if is_key_pressed(KeyCode::R) {
        game_board.create_board(game_board.width, game_board.height);
    }
    if is_key_pressed(KeyCode::Space) {
        *is_paused = is_paused.not();
    }
}*/