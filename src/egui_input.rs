use crate::physics::Phase;
use crate::{material::Material, world::*};
use crate::{material::VOID, particle::Particle};
use egui::{Key, Response, lerp, pos2, vec2};
use std::ops::{Not, RangeInclusive};

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
        for i in -(game_board.brush_size.y.floor() as i32 / 2_i32)
            ..=game_board.brush_size.y.floor() as i32 / 2_i32
        {
            for j in -(game_board.brush_size.x.floor() as i32 / 2_i32)
                ..=game_board.brush_size.x.floor() as i32 / 2_i32
            {
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
                    game_board.contents[cellpos] =
                        Particle::new(&materials[material].1, vec2(0_f32, 0_f32), 293.15);
                    game_board.contents[cellpos].display_color = materials[selected_material_id]
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
                    game_board.contents[cellpos].display_color[3] =
                        materials[selected_material_id].1.material_color.color.a();
                    if materials[selected_material_id].1.phase == Phase::Plasma {
                        game_board.contents[cellpos].energy = 70_f32;
                    }
                }
            }
        }
    } else if response.dragged_by(egui::PointerButton::Secondary)
        || response.clicked_by(egui::PointerButton::Secondary)
    {
        for i in -(game_board.brush_size.x.floor() as i32)..=game_board.brush_size.y.floor() as i32
        {
            for j in -game_board.brush_size.y as i32..=game_board.brush_size.x.floor() as i32 {
                let cellpos = ((i + pos.y as i32) as usize, (j + pos.x as i32) as usize);
                if game_board.contents.get(cellpos.0, cellpos.1).is_some() {
                    game_board.contents[cellpos] = Particle::default();
                }
            }
        }
    };
    // Brush resizing with mouse scroll
    let mouse_scroll = response.ctx.input(|input| input.raw_scroll_delta);
    if mouse_scroll.y.abs() >= 0.1_f32
        && ((game_board.brush_size.min_elem() > 1_f32 && mouse_scroll.y.signum() == -1_f32)
            || (game_board.brush_size.max_elem() < 256_f32 && mouse_scroll.y.signum() == 1_f32))
    {
        game_board.brush_size += vec2(
            2_f32 * (mouse_scroll.y.signum()),
            2_f32 * (mouse_scroll.y.signum()),
        );
    }
}

pub fn handle_key_inputs(game_board: &mut Board, is_paused: &mut bool, response: Response) {
    if response.ctx.input(|i| i.key_pressed(Key::R)) {
        game_board.create_board();
    } else if response.ctx.input(|i| i.key_pressed(Key::Space)) {
        *is_paused = is_paused.not();
    }
}

#[derive(Clone, PartialEq, PartialOrd)]
#[repr(u8)]
pub(crate) enum BrushShape {
    Rectangle,
    Rhombus,
    Ellipse,
}
