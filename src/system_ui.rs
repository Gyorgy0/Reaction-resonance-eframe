use std::fmt::{self};

use egui::epaint::TextShape;
use egui::text::LayoutJob;
use egui::util::hash;
use egui::{
    Color32, ColorImage, Context, FontId, Id, ImageSource, LayerId, NumExt, Pos2, Rect, Response,
    Rgba, Stroke, TextFormat, TextureOptions, Ui, Vec2, include_image, pos2, vec2,
};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::egui_input::BrushShape;
use crate::material::Material;
use crate::particle::Particle;
use crate::physics::Phase;
use crate::reactions::{MachineTypes, MaterialType};
use crate::world::{Board, get_safe_i};

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Phase::Air => write!(f, ""),
            Phase::Solid {
                melting_point: _,
                sublimation_point: _,
            } => write!(f, "Solid"),
            Phase::Powder {
                melting_point: _,
                sublimation_point: _,
            } => write!(f, "Powder"),
            Phase::Liquid {
                viscosity: _,
                melting_point: _,
                boiling_point: _,
            } => write!(f, "Liquid"),
            Phase::Gas {
                boiling_point: _,
                sublimation_point: _,
            } => write!(f, "Gas"),
            Phase::Plasma => write!(f, "Plasma"),
        }
    }
}

impl fmt::Display for MaterialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MaterialType::Corrosive {
                ph_value: _,
                blacklist: _,
                material_list: _,
            } => {
                write!(f, "Corrosive")
            }
            MaterialType::Alloy { metals: _ } => write!(f, "Alloy"),
            MaterialType::Ceramic => write!(f, "Ceramic"),
            MaterialType::CAutomata {
                birth: _,
                survival: _,
                stages: _,
            } => write!(f, "Cellular automaton"),
            MaterialType::Machine { machine: _ } => write!(f, "Machine"),
            MaterialType::Metal { reactivity: _ } => write!(f, "Metal"),
            MaterialType::Explosive {
                burn_time: _,
                ignition_temperature: _,
                explosion_power: _,
            } => write!(f, "Explosive"),
            MaterialType::Fuel {
                burn_time: _,
                ignition_temperature: _,
                flame_temperature: _,
            } => write!(f, "Fuel"),
            MaterialType::Oxidizer {
                combustion_speedup: _,
            } => write!(f, "Oxidizer"),
            MaterialType::Decor => write!(f, "Decor"),
            MaterialType::Solution => write!(f, "Solution"),
        }
    }
}
impl fmt::Display for MachineTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            MachineTypes::Cloner => write!(f, "Cloner"),
            MachineTypes::Sink => write!(f, "Sink"),
        }
    }
}

impl MaterialType {
    pub fn get_icon(&self) -> ImageSource<'_> {
        match self {
            MaterialType::Corrosive {
                ph_value: _,
                blacklist: _,
                material_list: _,
            } => include_image!("assets/corrosives_icon.svg"),
            MaterialType::Alloy { metals: _ } => include_image!("assets/alloys_icon.svg"),
            MaterialType::CAutomata {
                survival: _,
                birth: _,
                stages: _,
            } => include_image!("assets/cautomatas_icon.svg"),
            MaterialType::Ceramic => include_image!("assets/ceramics_icon.svg"),
            MaterialType::Explosive {
                burn_time: _,
                ignition_temperature: _,
                explosion_power: _,
            } => {
                include_image!("assets/explosives_icon.svg")
            }
            MaterialType::Fuel {
                burn_time: _,
                ignition_temperature: _,
                flame_temperature: _,
            } => include_image!("assets/fuels_icon.svg"),
            MaterialType::Machine { machine: _ } => include_image!("assets/machines_icon.svg"),
            MaterialType::Metal { reactivity: _ } => include_image!("assets/metals_icon.svg"),
            MaterialType::Oxidizer {
                combustion_speedup: _,
            } => include_image!("assets/oxidizers_icon.svg"),
            MaterialType::Decor => include_image!("assets/decors_icon.svg"),
            MaterialType::Solution => include_image!("assets/solutions_icon.svg"),
        }
    }
}

impl Board {
    pub fn draw_board(&mut self) -> Vec<Color32> {
        let pixels: Vec<Color32> = vec![Color32::BLACK; self.contents.len()];
        pixels
            .par_iter()
            .enumerate()
            .map(|px| self.contents.get_elem(px.0).display_color)
            .collect()
    }
    pub fn draw_board_temperature(
        &mut self,
        gradient: &egui_colorgradient::ColorInterpolator,
    ) -> Vec<Color32> {
        let pixels: Vec<Color32> = vec![Color32::TRANSPARENT; self.contents.len()];
        pixels
            .par_iter()
            .enumerate()
            .map(|px| {
                gradient
                    .sample_at(self.contents.get_elem(px.0).temperature)
                    .unwrap_or(Rgba::TRANSPARENT)
                    .into()
            })
            .collect()
    }
}

pub fn draw_brush_outlines(game_board: &Board, board: &Response, ui: &mut Ui, ctx: &Context) {
    let mut pixels: Vec<Color32> = vec![];
    if game_board.brush_size.min_elem() <= 0_f32 {
        ui.painter()
            .clone()
            .with_layer_id(LayerId::new(egui::Order::Foreground, Id::new(hash(0_i32))))
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

pub fn debug_text_rendering(
    game_board: &Board,
    materials: &[(String, Material)],
    debug_text_job: &mut LayoutJob,
    fps_values: &mut Vec<f32>,
    board: &Response,
    ctx: &Context,
    ui: &mut Ui,
) {
    let cursor_position = board.hover_pos().unwrap_or(pos2(-1024_f32, -1024_f32));
    let pos = ((cursor_position - board.interact_rect.min) / game_board.cellsize)
        .floor()
        .to_pos2();
    let default_particle = Particle::default();
    let viewed_particle: &Particle = game_board
        .contents
        .get(get_safe_i(
            &(game_board.height as usize),
            &(game_board.width as usize),
            &(pos.y as usize, pos.x as usize),
        ))
        .unwrap_or(&default_particle);

    let fps_interval_len = 100_usize;
    let fps_value = ui.input(|i| 1_f32 / i.stable_dt);
    fps_values.insert(0_usize, fps_value);
    fps_values.remove(fps_interval_len - 1_usize);

    let mean_fps = fps_values.iter().sum::<f32>() / fps_values.len() as f32;
    debug_text_job.append(
        format!(
            "\nFPS: {}\n\nName: {}\nParticle:\n{:?}",
            mean_fps.round(),
            materials[viewed_particle.material_id].0,
            viewed_particle
        )
        .as_str(),
        0_f32,
        TextFormat {
            font_id: FontId::new(18_f32, egui::FontFamily::Monospace),
            color: Color32::WHITE,
            ..Default::default()
        },
    );
    ui.painter()
        .clone()
        .with_clip_rect(ctx.used_rect())
        .with_layer_id(LayerId::new(egui::Order::Foreground, Id::new(hash(0_i32))))
        .add(TextShape::new(
            Pos2::new(5_f32, 45_f32),
            ctx.fonts_mut(|font| font.layout_job(debug_text_job.clone())),
            Color32::WHITE,
        ));
    *debug_text_job = LayoutJob::default();
}

pub fn get_particle(game_board: &Board, board: &Response) -> Particle {
    let cursor_position = board.hover_pos().unwrap_or(pos2(-1024_f32, -1024_f32));
    let pos = ((cursor_position - board.interact_rect.min) / game_board.cellsize)
        .floor()
        .to_pos2();
    let default_particle = Particle::default();
    let viewed_particle: &Particle = game_board
        .contents
        .get(get_safe_i(
            &(game_board.height as usize),
            &(game_board.width as usize),
            &(pos.y as usize, pos.x as usize),
        ))
        .unwrap_or(&default_particle);
    *viewed_particle
}
