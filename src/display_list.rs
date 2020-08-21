use crate::geom::*;
use crate::renderer::*;

pub struct DisplayList {
	pub(crate) items: Vec<DisplayItem>,
}

pub struct DisplayItem {
    pub path: PathId,
    pub position: Vec2,
    pub transform: Mat2x2,
    pub color: Color,
}

impl DisplayList {
    pub fn new() -> DisplayList {
        DisplayList {
            items: Vec::new(),
        }
    }

    pub fn fill(&mut self, path: PathId, position: Vec2, transform: Mat2x2, color: Color) {
        self.items.push(DisplayItem {
            path,
            position,
            transform,
            color,
        });
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32, pub g: f32, pub b: f32, pub a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}
