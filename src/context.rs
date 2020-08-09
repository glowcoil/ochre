use std::collections::HashMap;

use crate::{TILE_SIZE, ATLAS_SIZE, Path, Tiles, Backend, Vertex, Vec2, Mat2x2};

const SCALE_TOLERANCE = 0.1;
const POSITION_TOLERANCE = 0.25;

pub struct Context {
    cache: HashMap<PathKey, PathEntry>,
}

struct PathKey {
    id: usize,
    offset: 
}

struct PathEntry {
    tile_index: u32,
    tiles: Tiles,
}

impl Context {
    pub fn new() -> Context {
        Context {
            cache: HashMap::new(),
        }
    }
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

    pub fn fill(&mut self, path: &Path, position: Vec2, transform: Mat2x2, color: Color) {
        let tiles = path.fill(position, transform);

        let base = self.vertices.len() as u16;
        // self.vertices.push(Vertex { pos: [tile.x * TILE_SIZE as i16, tile.y * TILE_SIZE as i16], col, uv: [u * TILE_SIZE as u16, v * TILE_SIZE as u16] });
        // self.vertices.push(Vertex { pos: [(tile.x + 1) * TILE_SIZE as i16, tile.y * TILE_SIZE as i16], col, uv: [(u + 1) * TILE_SIZE as u16, v * TILE_SIZE as u16] });
        // self.vertices.push(Vertex { pos: [(tile.x + 1) * TILE_SIZE as i16, (tile.y + 1) * TILE_SIZE as i16], col, uv: [(u + 1) * TILE_SIZE as u16, (v + 1) * TILE_SIZE as u16] });
        // self.vertices.push(Vertex { pos: [tile.x * TILE_SIZE as i16, (tile.y + 1) * TILE_SIZE as i16], col, uv: [u * TILE_SIZE as u16, (v + 1) * TILE_SIZE as u16] });
        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
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
