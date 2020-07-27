mod geom;
mod path;
mod backend;
mod gl_backend;

pub use geom::*;
pub use path::*;
pub use backend::*;
pub use gl_backend::*;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32, pub g: f32, pub b: f32, pub a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}
