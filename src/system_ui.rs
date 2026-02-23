use std::f32::consts::PI;
use std::fmt::{self};

use egui::util::hash;
use egui::{
    Color32, ColorImage, Context, Id, LayerId, NumExt, Rect, Response, Stroke, TextureOptions, Ui,
    Vec2, pos2, vec2,
};

use crate::egui_input::BrushShape;
use crate::physics::Phase;
use crate::reactions::MaterialType;
use crate::world::Board;

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Phase::Void => write!(f, ""),
            Phase::Solid { melting_point: _ } => write!(f, "Solid"),
            Phase::Powder {
                coarseness: _,
                melting_point: _,
            } => write!(f, "Powder"),
            Phase::Liquid {
                viscosity: _,
                melting_point: _,
                boiling_point: _,
            } => write!(f, "Liquid"),
            Phase::Gas { boiling_point: _ } => write!(f, "Gas"),
            Phase::Plasma => write!(f, "Plasma"),
        }
    }
}

impl fmt::Display for MaterialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            MaterialType::Acid => write!(f, "Acid"),
            MaterialType::Alloy => write!(f, "Alloy"),
            MaterialType::Base => write!(f, "Base"),
            MaterialType::Ceramic => write!(f, "Ceramic"),
            MaterialType::CAutomata {
                birth: _,
                survival: _,
                stages: _,
            } => write!(f, "Cellular automaton"),
            MaterialType::Cloner => write!(f, "Cloner"),
            MaterialType::Explosive => write!(f, "Explosive"),
            MaterialType::Fuel => write!(f, "Fuel"),
            MaterialType::Glass => write!(f, "Glass"),
            MaterialType::Oxidizer => write!(f, "Oxidizer"),
            MaterialType::Decor => write!(f, "Decor"),
            MaterialType::Sink => write!(f, "Sink"),
            MaterialType::Solution => write!(f, "Solution"),
            MaterialType::Solvent => write!(f, "Solvent"),
        }
    }
}

impl Board {
    pub fn draw_board(&mut self) -> Vec<Color32> {
        self.contents.iter().map(|px| px.display_color).collect()
    }
}

pub fn draw_brush_outlines(game_board: &Board, board: &Response, ui: &mut Ui, ctx: &Context) {
    let mut pixels: Vec<Color32> = vec![];
    if game_board.brush_size.min_elem() <= 0_f32 {
        ui.painter()
            .clone()
            .with_layer_id(LayerId::new(egui::Order::Foreground, Id::new(hash(0))))
            .with_clip_rect(ctx.content_rect())
            .rect(
                Rect::from_min_size(
                    ((((board
                        .hover_pos()
                        .unwrap_or(pos2(-1024_f32, -1024_f32))
                        .to_vec2()
                        - board.interact_rect.min.to_vec2())
                        / vec2(game_board.cellsize.x, game_board.cellsize.y))
                    .floor())
                        * vec2(game_board.cellsize.x, game_board.cellsize.y))
                    .to_pos2()
                    .floor()
                        + board.interact_rect.min.to_vec2()
                        - vec2(
                            game_board.cellsize.x * game_board.brush_size.x,
                            game_board.cellsize.y * game_board.brush_size.y,
                        ),
                    Vec2::new(
                        game_board.brush_size.x * 2_f32 * game_board.cellsize.x
                            + game_board.cellsize.x,
                        game_board.brush_size.y * 2_f32 * game_board.cellsize.y
                            + game_board.cellsize.y,
                    ),
                ),
                1_f32,
                Color32::from_black_alpha(100),
                Stroke::new(2_f32, Color32::WHITE),
                egui::StrokeKind::Outside,
            );
        return;
    } else if game_board.brush_shape == BrushShape::Rectangle {
        for i in -game_board.brush_size.x as i32..=game_board.brush_size.x as i32 {
            for j in -game_board.brush_size.y as i32..=game_board.brush_size.y as i32 {
                if (game_board.brush_size.y * i as f32 - game_board.brush_size.x * j as f32).abs()
                    + (game_board.brush_size.y * i as f32 + game_board.brush_size.x * j as f32)
                        .abs()
                    < (2_f32
                        * (game_board.brush_size.x
                            + (game_board.brush_size.y / game_board.brush_size.x)
                                .floor()
                                .at_most(1_f32))
                        * (game_board.brush_size.y
                            + (game_board.brush_size.x / game_board.brush_size.y)
                                .floor()
                                .at_most(1_f32)))
                    .abs()
                    && (game_board.brush_size.y * i as f32 - game_board.brush_size.x * j as f32)
                        .abs()
                        + (game_board.brush_size.y * i as f32 + game_board.brush_size.x * j as f32)
                            .abs()
                        >= (2_f32 * game_board.brush_size.x * game_board.brush_size.y).abs()
                {
                    pixels.push(Color32::from_white_alpha(255_u8));
                } else if (game_board.brush_size.y * i as f32 - game_board.brush_size.x * j as f32)
                    .abs()
                    + (game_board.brush_size.y * i as f32 + game_board.brush_size.x * j as f32)
                        .abs()
                    < (2_f32 * game_board.brush_size.x * game_board.brush_size.y).abs()
                {
                    pixels.push(Color32::from_black_alpha(100_u8));
                } else {
                    pixels.push(Color32::TRANSPARENT);
                }
            }
        }
    } else if game_board.brush_shape == BrushShape::Rhombus {
        for i in -game_board.brush_size.x as i32..=game_board.brush_size.x as i32 {
            for j in -game_board.brush_size.y as i32..=game_board.brush_size.y as i32 {
                if (j as f32 * game_board.brush_size.x).abs()
                    + (i as f32 * game_board.brush_size.y).abs()
                    < (game_board.brush_size.x
                        + (0.5_f32 * game_board.brush_size.x / game_board.brush_size.min_elem()))
                        * (game_board.brush_size.y
                            + (0.5_f32 * game_board.brush_size.y
                                / game_board.brush_size.min_elem()))
                        .abs()
                    && ((j as f32 * game_board.brush_size.x).abs()
                        + (i as f32 * game_board.brush_size.y).abs())
                    .ceil()
                        >= (game_board.brush_size.x * game_board.brush_size.y)
                            .abs()
                            .floor()
                {
                    pixels.push(Color32::from_white_alpha(255_u8));
                } else if (j as f32 * game_board.brush_size.x).abs()
                    + (i as f32 * game_board.brush_size.y).abs()
                    < ((game_board.brush_size.x
                        - (game_board.brush_size.y / game_board.brush_size.x).floor())
                        * (game_board.brush_size.y
                            - (game_board.brush_size.x / game_board.brush_size.y).floor()))
                    .abs()
                {
                    pixels.push(Color32::from_black_alpha(100_u8));
                } else {
                    pixels.push(Color32::TRANSPARENT);
                }
            }
        }
    } else if game_board.brush_shape == BrushShape::Ellipse {
        for i in -game_board.brush_size.x as i32..=game_board.brush_size.x as i32 {
            for j in -game_board.brush_size.y as i32..=game_board.brush_size.y as i32 {
                if (i as f32
                    / (game_board.brush_size.x
                        + (1_f32 * (game_board.brush_size.x / game_board.brush_size.min_elem()))))
                .powi(2_i32)
                    + (j as f32
                        / (game_board.brush_size.y
                            + (1_f32
                                * (game_board.brush_size.y / game_board.brush_size.min_elem()))))
                    .powi(2_i32)
                    < 1_f32
                    && ((i as f32 / (game_board.brush_size.x)).powi(2_i32)
                        + (j as f32 / (game_board.brush_size.y)).powi(2_i32))
                        >= 1_f32
                {
                    pixels.push(Color32::from_white_alpha(255_u8));
                } else if (i as f32 / (game_board.brush_size.x)).powi(2_i32)
                    + (j as f32 / (game_board.brush_size.y)).powi(2_i32)
                    < 1_f32
                {
                    pixels.push(Color32::from_black_alpha(100_u8));
                } else {
                    pixels.push(Color32::TRANSPARENT);
                }
            }
        }
    }
    let brush_image = ColorImage::new(
        [
            ((game_board.brush_size.x * 2_f32) + 1_f32) as usize,
            ((game_board.brush_size.y * 2_f32) + 1_f32) as usize,
        ],
        pixels,
    );
    let brush_texture = ctx.load_texture("Brush", brush_image, TextureOptions::NEAREST);
    ui.painter()
        .clone()
        .with_layer_id(LayerId::new(egui::Order::Foreground, Id::new(hash(0))))
        .with_clip_rect(ctx.content_rect())
        .image(
            brush_texture.id(),
            Rect::from_min_size(
                ((((board
                    .hover_pos()
                    .unwrap_or(pos2(-1024_f32, -1024_f32))
                    .to_vec2()
                    - board.interact_rect.min.to_vec2())
                    / vec2(game_board.cellsize.x, game_board.cellsize.y))
                .floor())
                    * vec2(game_board.cellsize.x, game_board.cellsize.y))
                .to_pos2()
                .floor()
                    + board.interact_rect.min.to_vec2()
                    - vec2(
                        game_board.cellsize.x * game_board.brush_size.x,
                        game_board.cellsize.y * game_board.brush_size.y,
                    ),
                Vec2::new(
                    game_board.brush_size.x * 2_f32 * game_board.cellsize.x + game_board.cellsize.x,
                    game_board.brush_size.y * 2_f32 * game_board.cellsize.y + game_board.cellsize.y,
                ),
            ),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );
}
