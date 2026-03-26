//! Gradient editor widget for [egui](https://www.egui.rs/).

use cache::FrameCacheDyn;
use egui::Popup;
use egui::color_picker::{Alpha, color_picker_hsva_2d};
use egui::ecolor::Hsva;
use egui::style::WidgetVisuals;
use egui::{
    Button, Color32, ColorImage, ComboBox, Id, LayerId, Mesh, Order, Painter, PointerButton, Rect,
    Sense, Shape, Stroke, StrokeKind, TextureHandle, TextureOptions, Ui, Vec2, lerp, pos2, vec2,
};
pub use gradient::{ColorInterpolator, Gradient, InterpolationMethod};

mod cache;
mod gradient;
