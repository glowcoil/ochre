use crate::{Mat2x2, Path, PathCommand, Vec2};

pub const TILE_SIZE: usize = 8;
pub const ATLAS_SIZE: usize = 4096;

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32, pub g: f32, pub b: f32, pub a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: [i16; 2],
    pub uv: [u16; 2],
    pub col: [u8; 4],
}

#[derive(Clone)]
pub struct Picture {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub tiles: Vec<u8>,
    next_tile_row: u16,
    next_tile_col: u16,
}

impl Picture {
    pub fn new() -> Picture {
        let mut tiles = vec![0; ATLAS_SIZE * TILE_SIZE];
        for row in 0..TILE_SIZE {
            for col in 0..TILE_SIZE {
                tiles[row * ATLAS_SIZE + col] = 255;
            }
        }

        Picture {
            vertices: Vec::new(),
            indices: Vec::new(),
            tiles,
            next_tile_row: 0,
            next_tile_col: 1,
        }
    }

    pub fn fill(&mut self, path: &Path, position: Vec2, transform: Mat2x2, color: Color) {
        #[derive(Copy, Clone)]
        pub struct Increment {
            x: i16,
            y: i16,
            area: f32,
            height: f32,
        }

        #[derive(Copy, Clone)]
        pub struct TileIncrement {
            tile_x: i16,
            tile_y: i16,
            sign: i8,
        }

        let flattened = path.flatten(transform);

        let mut increments = Vec::new();
        let mut tile_increments = Vec::new();
        let mut first = Vec2::new(0.0, 0.0);
        let mut last = Vec2::new(0.0, 0.0);
        for (&command, &point) in flattened.commands.iter().zip(flattened.points.iter()) {
            let p1;
            let p2;
            match command {
                PathCommand::Move => {
                    p1 = last + position;
                    p2 = first + position;
                    first = point;
                    last = point;
                }
                PathCommand::Line => {
                    p1 = last + position;
                    p2 = point + position;
                    last = point;
                }
                _ => {
                    unreachable!();
                }
            }

            if p1 != p2 {
                let x_dir = (p2.x - p1.x).signum() as i16;
                let y_dir = (p2.y - p1.y).signum() as i16;
                let dtdx = 1.0 / (p2.x - p1.x);
                let dtdy = 1.0 / (p2.y - p1.y);
                let mut x = p1.x as u16 as i16;
                let mut y = p1.y as u16 as i16;
                let mut tile_y_prev = (y as u16 / TILE_SIZE as u16) as i16;
                let mut row_t0: f32 = 0.0;
                let mut col_t0: f32 = 0.0;
                let mut row_t1 = if p1.y == p2.y {
                    std::f32::INFINITY
                } else {
                    let next_y = if p2.y > p1.y { (y + 1) as f32 } else { y as f32 };
                    (dtdy * (next_y - p1.y)).min(1.0)
                };
                let mut col_t1 = if p1.x == p2.x {
                    std::f32::INFINITY
                } else {
                    let next_x = if p2.x > p1.x { (x + 1) as f32 } else { x as f32 };
                    (dtdx * (next_x - p1.x)).min(1.0)
                };
                let x_step = dtdx.abs();
                let y_step = dtdy.abs();

                loop {
                    let t0 = row_t0.max(col_t0);
                    let t1 = row_t1.min(col_t1);
                    let p0 = (1.0 - t0) * p1 + t0 * p2;
                    let p1 = (1.0 - t1) * p1 + t1 * p2;
                    let height = p1.y - p0.y;
                    let right = (x + 1) as f32;
                    let area = 0.5 * height * ((right - p0.x) + (right - p1.x));

                    increments.push(Increment { x, y, area, height });

                    let tile_y = (y as u16 / TILE_SIZE as u16) as i16;
                    if tile_y != tile_y_prev {
                        tile_increments.push(TileIncrement {
                            tile_x: (x as u16 / TILE_SIZE as u16) as i16,
                            tile_y: tile_y_prev.min(tile_y),
                            sign: (tile_y - tile_y_prev) as i8,
                        });
                    }
                    tile_y_prev = tile_y;

                    if row_t1 < col_t1 {
                        row_t0 = row_t1;
                        row_t1 = (row_t1 + y_step).min(1.0);
                        if row_t0 == 1.0 {
                            break;
                        } else {
                            y += y_dir;
                        }
                    } else {
                        col_t0 = col_t1;
                        col_t1 = (col_t1 + x_step).min(1.0);
                        if col_t0 == 1.0 {
                            break;
                        } else {
                            x += x_dir;
                        }
                    }
                }
            }
        }

        #[derive(Copy, Clone)]
        struct Bin {
            tile_x: i16,
            tile_y: i16,
            start: usize,
            end: usize,
        }
        let mut bins = Vec::new();
        let mut bin = Bin { tile_x: 0, tile_y: 0, start: 0, end: 0 };
        if let Some(first) = increments.first() {
            bin.tile_x = ((first.x as u16) / TILE_SIZE as u16) as i16;
            bin.tile_y = ((first.y as u16) / TILE_SIZE as u16) as i16;
        }
        for (i, increment) in increments.iter().enumerate() {
            let tile_x = ((increment.x as u16) / TILE_SIZE as u16) as i16;
            let tile_y = ((increment.y as u16) / TILE_SIZE as u16) as i16;
            if tile_x != bin.tile_x || tile_y != bin.tile_y {
                bins.push(bin);
                bin = Bin { tile_x, tile_y, start: i, end: i };
            }
            bin.end += 1;
        }
        bins.push(bin);
        bins.sort_unstable_by_key(|bin| (bin.tile_y, bin.tile_x));

        tile_increments.sort_unstable_by_key(|tile_inc| (tile_inc.tile_y, tile_inc.tile_x));

        let col = [
            (color.r * 256.0).min(255.0) as u8,
            (color.g * 256.0).min(255.0) as u8,
            (color.b * 256.0).min(255.0) as u8,
            (color.a * 256.0).min(255.0) as u8,
        ];

        let mut areas = [0.0; TILE_SIZE * TILE_SIZE];
        let mut heights = [0.0; TILE_SIZE * TILE_SIZE];
        let mut prev = [0.0; TILE_SIZE];
        let mut next = [0.0; TILE_SIZE];

        let mut tile_increments_i = 0;
        let mut winding = 0;

        for i in 0..bins.len() {
            let bin = bins[i];
            for increment in &increments[bin.start..bin.end] {
                let x = increment.x as usize % TILE_SIZE;
                let y = increment.y as usize % TILE_SIZE;
                areas[(y * TILE_SIZE + x) as usize] += increment.area;
                heights[(y * TILE_SIZE + x) as usize] += increment.height;
            }

            if i + 1 == bins.len() || bins[i + 1].tile_x != bin.tile_x || bins[i + 1].tile_y != bin.tile_y {
                let mut tile = [0; TILE_SIZE * TILE_SIZE];
                for y in 0..TILE_SIZE {
                    let mut accum = prev[y];
                    for x in 0..TILE_SIZE {
                        tile[y * TILE_SIZE + x] = ((accum + areas[y * TILE_SIZE + x]).abs() * 256.0).min(255.0) as u8;
                        accum += heights[y * TILE_SIZE + x];
                    }
                    next[y] = accum;
                }

                let base = self.vertices.len() as u32;

                let x1 = bin.tile_x * TILE_SIZE as i16;
                let x2 = (bin.tile_x + 1) * TILE_SIZE as i16;
                let y1 = bin.tile_y * TILE_SIZE as i16;
                let y2 = (bin.tile_y + 1) * TILE_SIZE as i16;

                let u1 = self.next_tile_col * TILE_SIZE as u16;
                let u2 = (self.next_tile_col + 1) * TILE_SIZE as u16;
                let v1 = self.next_tile_row * TILE_SIZE as u16;
                let v2 = (self.next_tile_row + 1) * TILE_SIZE as u16;

                self.vertices.push(Vertex { pos: [x1, y1], col, uv: [u1, v1] });
                self.vertices.push(Vertex { pos: [x2, y1], col, uv: [u2, v1] });
                self.vertices.push(Vertex { pos: [x2, y2], col, uv: [u2, v2] });
                self.vertices.push(Vertex { pos: [x1, y2], col, uv: [u1, v2] });
                self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

                for row in 0..TILE_SIZE {
                    for col in 0..TILE_SIZE {
                        self.tiles[self.next_tile_row as usize * TILE_SIZE * ATLAS_SIZE + row * ATLAS_SIZE + self.next_tile_col as usize * TILE_SIZE + col] = tile[row * TILE_SIZE + col];
                    }
                }

                self.next_tile_col += 1;
                if self.next_tile_col as usize == ATLAS_SIZE / TILE_SIZE {
                    self.next_tile_col = 0;
                    self.next_tile_row += 1;
                    self.tiles.resize(self.tiles.len() + ATLAS_SIZE * TILE_SIZE, 0);
                }

                areas = [0.0; TILE_SIZE * TILE_SIZE];
                heights = [0.0; TILE_SIZE * TILE_SIZE];
                if i + 1 < bins.len() && bins[i + 1].tile_y == bin.tile_y {
                    prev = next;
                } else {
                    prev = [0.0; TILE_SIZE];
                }
                next = [0.0; TILE_SIZE];

                if i + 1 < bins.len() && bins[i + 1].tile_y == bin.tile_y && bins[i + 1].tile_x > bin.tile_x + 1 {
                    while tile_increments_i < tile_increments.len() {
                        let tile_increment = tile_increments[tile_increments_i];
                        if (tile_increment.tile_y, tile_increment.tile_x) > (bin.tile_y, bin.tile_x) {
                            break;
                        }
                        winding += tile_increment.sign as isize;
                        tile_increments_i += 1;
                    }
                    if winding != 0 {
                        let base = self.vertices.len() as u32;

                        let x1 = (bin.tile_x + 1) * TILE_SIZE as i16;
                        let x2 = (bins[i + 1].tile_x) * TILE_SIZE as i16;
                        let y1 = bin.tile_y * TILE_SIZE as i16;
                        let y2 = (bin.tile_y + 1) * TILE_SIZE as i16;

                        self.vertices.push(Vertex { pos: [x1, y1], col, uv: [0, 0] });
                        self.vertices.push(Vertex { pos: [x2, y1], col, uv: [0, 0] });
                        self.vertices.push(Vertex { pos: [x2, y2], col, uv: [0, 0] });
                        self.vertices.push(Vertex { pos: [x1, y2], col, uv: [0, 0] });
                        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
                    }
                }
            }
        }
    }

    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn tiles(&self) -> &[u8] {
        &self.tiles
    }

    pub fn tiles_size(&self) -> (u32, u32) {
        (ATLAS_SIZE as u32, (self.next_tile_row as u32 + 1) * TILE_SIZE as u32)
    }
}
