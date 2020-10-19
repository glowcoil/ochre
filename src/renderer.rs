use crate::{TILE_SIZE, ATLAS_SIZE, Backend, Picture, Vertex};

pub struct Renderer {
    data: Vec<u64>,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    next_row: u16,
    next_col: u16,
}

impl Renderer {
    pub fn new() -> Renderer {
        let mut data = vec![0; ATLAS_SIZE];
        for row in 0..TILE_SIZE {
            // for col in 0..TILE_SIZE {
                data[row * ATLAS_SIZE / TILE_SIZE] = 0xFFFFFFFF;
            // }
        }

        Renderer {
            data,
            vertices: Vec::new(),
            indices: Vec::new(),
            next_row: 0,
            next_col: 1,
        }
    }

    pub fn draw(&mut self, picture: &Picture) {
        let mut tiles_start = 0;
        let mut spans_start = 0;

        let mut tile_index = 0;

        for layer in picture.layers.iter() {
            let col = [
                (layer.color.r * 256.0).min(255.0) as u8,
                (layer.color.g * 256.0).min(255.0) as u8,
                (layer.color.b * 256.0).min(255.0) as u8,
                (layer.color.a * 256.0).min(255.0) as u8,
            ];

            for tile in picture.tiles[tiles_start..tiles_start + layer.tiles].iter() {
                let base = self.vertices.len() as u16;

                let x1 = tile.x * TILE_SIZE as i16;
                let x2 = (tile.x + 1) * TILE_SIZE as i16;
                let y1 = tile.y * TILE_SIZE as i16;
                let y2 = (tile.y + 1) * TILE_SIZE as i16;

                let u1 = self.next_col * TILE_SIZE as u16;
                let u2 = (self.next_col + 1) * TILE_SIZE as u16;
                let v1 = self.next_row * TILE_SIZE as u16;
                let v2 = (self.next_row + 1) * TILE_SIZE as u16;

                self.vertices.push(Vertex { pos: [x1, y1], col, uv: [u1, v1] });
                self.vertices.push(Vertex { pos: [x2, y1], col, uv: [u2, v1] });
                self.vertices.push(Vertex { pos: [x2, y2], col, uv: [u2, v2] });
                self.vertices.push(Vertex { pos: [x1, y2], col, uv: [u1, v2] });
                self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

                for row in 0..TILE_SIZE {
                    // for col in 0..TILE_SIZE {
                        self.data[self.next_row as usize * ATLAS_SIZE + self.next_col as usize + row * ATLAS_SIZE / TILE_SIZE] = picture.data[tile_index + row];
                    // }
                }
                tile_index += TILE_SIZE;

                self.next_col += 1;
                if self.next_col as usize == ATLAS_SIZE / TILE_SIZE {
                    self.next_col = 0;
                    self.next_row += 1;
                    self.data.resize(self.data.len() + ATLAS_SIZE, 0);
                }
            }

            tiles_start += layer.tiles;

            for span in picture.spans[spans_start..spans_start + layer.spans].iter() {
                let base = self.vertices.len() as u16;

                let x1 = span.x * TILE_SIZE as i16;
                let x2 = (span.x + span.len) * TILE_SIZE as i16;
                let y1 = span.y * TILE_SIZE as i16;
                let y2 = (span.y + 1) * TILE_SIZE as i16;

                self.vertices.push(Vertex { pos: [x1, y1], col, uv: [0, 0] });
                self.vertices.push(Vertex { pos: [x2, y1], col, uv: [0, 0] });
                self.vertices.push(Vertex { pos: [x2, y2], col, uv: [0, 0] });
                self.vertices.push(Vertex { pos: [x1, y2], col, uv: [0, 0] });
                self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }

            spans_start += layer.spans;
        }
    }

    pub fn submit(&mut self, backend: &mut dyn Backend, screen_width: u32, screen_height: u32) {
        let buffer: &[u8] = unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len() * std::mem::size_of::<u64>()) };
        backend.upload(0, 0, ATLAS_SIZE as u32, (self.next_row as u32 + 1) * TILE_SIZE as u32, &buffer);
        backend.draw(&self.vertices, &self.indices, screen_width, screen_height);

        *self = Renderer::new();
    }
}
