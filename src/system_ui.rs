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
            MaterialType::Corrosive => write!(f, "Corrosive"),
            MaterialType::Alloy => write!(f, "Alloy"),
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
                        (game_board.brush_size.x * 2_f32 * game_board.cellsize.x)
                            + game_board.cellsize.x,
                        (game_board.brush_size.y * 2_f32 * game_board.cellsize.y)
                            + game_board.cellsize.y,
                    ),
                ),
                0_f32,
                Color32::from_black_alpha(100),
                Stroke::new(2_f32, Color32::WHITE),
                egui::StrokeKind::Outside,
            );
        return;
    }
    for y in -game_board.brush_size.y as i64..=game_board.brush_size.y as i64 {
        for x in -game_board.brush_size.x as i64..=game_board.brush_size.x as i64 {
            pixels.push(get_shape(game_board.brush_shape, game_board.brush_size, x, y).0);
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

pub fn get_shape(brush_shape: BrushShape, brush_size: Vec2, x: i64, y: i64) -> (Color32, bool) {
    let outline = Color32::from_white_alpha(255_u8);
    let fill = Color32::from_black_alpha(100_u8);
    let background = Color32::TRANSPARENT;
    if brush_size.min_elem() == 0_f32 {
        return (background, true);
    }

    // Rectangle shape
    // 2a - width of the rectangle
    // 2b - height of rectangle
    // Function of the shape:
    // |b*x-a*y| + |b*x+a*y| = |2*a*b|
    if brush_shape == BrushShape::Rectangle {
        if (brush_size.y * x as f32 - brush_size.x * y as f32).abs()
            + (brush_size.y * x as f32 + brush_size.x * y as f32).abs()
            < (2_f32
                * (brush_size.x + (brush_size.y / brush_size.x).floor().at_most(1_f32))
                * (brush_size.y + (brush_size.x / brush_size.y).floor().at_most(1_f32)))
            .abs()
            && (brush_size.y * x as f32 - brush_size.x * y as f32).abs()
                + (brush_size.y * x as f32 + brush_size.x * y as f32).abs()
                >= (2_f32 * brush_size.x * brush_size.y).abs()
        {
            return (outline, true);
        } else if (brush_size.y * x as f32 - brush_size.x * y as f32).abs()
            + (brush_size.y * x as f32 + brush_size.x * y as f32).abs()
            < (2_f32 * brush_size.x * brush_size.y).abs()
        {
            return (fill, true);
        }
    }
    // Rhombus shape
    // 2a - width of the rhombus
    // 2b - height of rhombus
    // Function of the shape:
    // |b*x| + |a*y| = |a*b|
    else if brush_shape == BrushShape::Rhombus {
        if (brush_size.y * x as f32).abs() + (brush_size.x * y as f32).abs()
            < (brush_size.x * brush_size.y).abs()
            && (brush_size.y * x as f32).abs() + (brush_size.x * y as f32).abs()
                >= ((brush_size.x - 1_f32) * (brush_size.y - 1_f32)).abs()
        {
            return (outline, true);
        } else if (brush_size.y * x as f32).abs() + (brush_size.x * y as f32).abs()
            < ((brush_size.x - 1_f32) * (brush_size.y - 1_f32)).abs()
        {
            return (fill, true);
        }
    }
    // Ellipse Shape
    // 2a - width of the ellipse
    // 2b - height of ellipse
    // Function of the shape:
    // (x/a)^2 + (y/b)^2 = 1^2
    else if brush_shape == BrushShape::Ellipse {
        if (x as f32 / (brush_size.x + (1_f32 * (brush_size.x / brush_size.min_elem()))))
            .powi(2_i32)
            + (y as f32 / (brush_size.y + (1_f32 * (brush_size.y / brush_size.min_elem()))))
                .powi(2_i32)
            < 1_f32
            && ((x as f32 / (brush_size.x)).powi(2_i32) + (y as f32 / (brush_size.y)).powi(2_i32))
                >= 1_f32
        {
            return (outline, true);
        } else if (x as f32 / (brush_size.x)).powi(2_i32) + (y as f32 / (brush_size.y)).powi(2_i32)
            < 1_f32
        {
            return (fill, true);
        }
    }
    (background, false)
}
