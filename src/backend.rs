use crate::Color;

pub const ATLAS_SIZE: usize = 4096;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [i16; 2],
    pub uv: [u16; 2],
    pub col: [u8; 4],
}

pub trait Backend {
    fn clear(&mut self, color: Color);
    fn upload(&mut self, x: u32, y: u32, width: u32, height: u32, data: &[u8]);
    fn draw(&mut self, vertices: &[Vertex], indices: &[u16], screen_width: u32, screen_height: u32);
}
