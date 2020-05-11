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
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    pub fn to_linear_premul(&self) -> [f32; 4] {
        fn srgb_to_linear(x: f32) -> f32 {
            if x < 0.04045 { x / 12.92 } else { ((x + 0.055)/1.055).powf(2.4)  }
        }

        [
            self.a * srgb_to_linear(self.r),
            self.a * srgb_to_linear(self.g),
            self.a * srgb_to_linear(self.b),
            self.a
        ]
    }
}
