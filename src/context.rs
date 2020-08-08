use crate::{Path, Backend, Vertex, Vec2, Mat2x2};

pub struct Context {

}

pub struct Frame<'c, 'b> {
    context: &'c mut Context,
    backend: &'b mut dyn Backend,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl<'c, 'b> Frame<'c, 'b> {
    pub fn new(context: &'c mut Context, backend: &'b mut dyn Backend) -> Frame<'c, 'b> {
        Frame {
            context,
            backend,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn fill(path: &Path, position: Vec2, transform: Mat2x2, color: Color) {
        let tiles = path.fill(position, transform);
    }
}

impl<'c, 'b> Drop for Frame<'c, 'b> {
    fn drop(&mut self) {

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
