fn handle_mouse_input(game_board: &mut Board, selected_material: &mut Material) {
    let row_count: i32 = game_board.height as i32;
    let col_count: i32 = game_board.width as i32;
    let cursor_position = EFrameApp::raw_input_hook(&mut self, ctx, raw_input);
    //let cursor_position = egui::RawInput::mouse_position();
    let x = (cursor_position.0 - 5.0) / game_board.cellsize.x;
    let y = (cursor_position.1 - 25.0) / game_board.cellsize.y;
    if (is_mouse_button_down(MouseButton::Left) || is_mouse_button_down(MouseButton::Right))
        && cursor_position.0 > game_board.cellsize.x - 5.0
        && cursor_position.0 < (game_board.cellsize.x * col_count as f32)
        && cursor_position.1 > game_board.cellsize.y + 25.0
        && cursor_position.1 < (game_board.cellsize.y * row_count as f32) + 60.0
    {
        let material = if is_mouse_button_down(MouseButton::Left) {
            selected_material.clone()
        } else {
            Material {
                name: "Void".to_string(),
                density: 0.0,
                phase: Phase::Void,
                material_type: Material_Type::Atmosphere,
                durability: -1,
                color: color_u8!(0, 0, 0, 100),
            }
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
                            speed: mq::math::vec2(0.0, game_board.gravity.signum() * 1.0),
                            temperature: 20.0,
                            updated: true,
                            seed: rand::gen_range(0.0, 1.0),
                        }
                }
            }
        }
    }
    if (mouse_wheel().1 > -120.0 && mouse_wheel().1 <= -60.0 && game_board.brushsize < row_count)
        || (mouse_wheel().1 < 120.0 && mouse_wheel().1 >= 60.0 && game_board.brushsize > 2)
    {
        game_board.brushsize -= 2 * (mouse_wheel().1 / 60.0) as i32;
    } else if (mouse_wheel().1 <= -120.0 && game_board.brushsize < row_count)
        || (mouse_wheel().1 >= 120.0 && game_board.brushsize > 2)
    {
        game_board.brushsize -= 2 * (mouse_wheel().1 / 120.0) as i32;
    }
}

fn handle_key_inputs(game_board: &mut Board, is_paused: &mut bool) {
    if is_key_pressed(KeyCode::R) {
        game_board.create_board(game_board.width, game_board.height);
    }
    if is_key_pressed(KeyCode::Space) {
        *is_paused = is_paused.not();
    }
}