pub mod geom;
pub mod path;
pub mod render;

pub use geom::*;
pub use path::*;
pub use render::*;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32, pub g: f32, pub b: f32, pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    pub fn premultiply(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r: a * r, g: a * g, b: a * b, a }
    }
}
