use crate::{TILE_SIZE, ATLAS_SIZE, Backend, Color, Mat2x2, Path, Vec2, Vertex};

pub struct Renderer {
    data: Vec<u8>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    next_row: u16,
    next_col: u16,
}

impl Renderer {
    pub fn new() -> Renderer {
        let mut data = vec![0; ATLAS_SIZE * TILE_SIZE];
        for row in 0..TILE_SIZE {
            for col in 0..TILE_SIZE {
                data[row * ATLAS_SIZE + col] = 255;
            }
        }

        Renderer {
            data,
            vertices: Vec::new(),
            indices: Vec::new(),
            next_row: 0,
            next_col: 1,
        }
    }

    pub fn fill(&mut self, path: &Path, position: Vec2, transform: Mat2x2, color: Color) {
        let col = [
            (color.r * 256.0).min(255.0) as u8,
            (color.g * 256.0).min(255.0) as u8,
            (color.b * 256.0).min(255.0) as u8,
            (color.a * 256.0).min(255.0) as u8,
        ];

        let tiles = path.fill(position, transform);
        for tile in tiles.tiles {
            let base = self.vertices.len() as u16;
            self.vertices.push(Vertex { pos: [tile.x * TILE_SIZE as i16, tile.y * TILE_SIZE as i16], col, uv: [self.next_col * TILE_SIZE as u16, self.next_row * TILE_SIZE as u16] });
            self.vertices.push(Vertex { pos: [(tile.x + 1) * TILE_SIZE as i16, tile.y * TILE_SIZE as i16], col, uv: [(self.next_col + 1) * TILE_SIZE as u16, self.next_row * TILE_SIZE as u16] });
            self.vertices.push(Vertex { pos: [(tile.x + 1) * TILE_SIZE as i16, (tile.y + 1) * TILE_SIZE as i16], col, uv: [(self.next_col + 1) * TILE_SIZE as u16, (self.next_row + 1) * TILE_SIZE as u16] });
            self.vertices.push(Vertex { pos: [tile.x * TILE_SIZE as i16, (tile.y + 1) * TILE_SIZE as i16], col, uv: [self.next_col * TILE_SIZE as u16, (self.next_row + 1) * TILE_SIZE as u16] });
            self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

            for row in 0..TILE_SIZE {
                for col in 0..TILE_SIZE {
                    self.data[self.next_row as usize * TILE_SIZE * ATLAS_SIZE + row * ATLAS_SIZE + self.next_col as usize * TILE_SIZE + col] = tiles.data[tile.index + row * TILE_SIZE + col];
                }
            }

            self.next_col += 1;
            if self.next_col as usize == ATLAS_SIZE / TILE_SIZE {
                self.next_col = 0;
                self.next_row += 1;
                self.data.resize(self.data.len() + ATLAS_SIZE * TILE_SIZE, 0);
            }
        }

        for span in tiles.spans {
            let base = self.vertices.len() as u16;
            self.vertices.push(Vertex { pos: [span.x * TILE_SIZE as i16, span.y * TILE_SIZE as i16], col, uv: [0, 0] });
            self.vertices.push(Vertex { pos: [(span.x + span.len) * TILE_SIZE as i16, span.y * TILE_SIZE as i16], col, uv: [0, 0] });
            self.vertices.push(Vertex { pos: [(span.x + span.len) * TILE_SIZE as i16, (span.y + 1) * TILE_SIZE as i16], col, uv: [0, 0] });
            self.vertices.push(Vertex { pos: [span.x * TILE_SIZE as i16, (span.y + 1) * TILE_SIZE as i16], col, uv: [0, 0] });
            self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    pub fn submit(&mut self, backend: &mut dyn Backend, screen_width: u32, screen_height: u32) {
        backend.upload(0, 0, ATLAS_SIZE as u32, (self.next_row as u32 + 1) * TILE_SIZE as u32, &self.data);
        backend.draw(&self.vertices, &self.indices, screen_width, screen_height);

        *self = Renderer::new();
    }
}
