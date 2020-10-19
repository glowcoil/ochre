use crate::{Color, Mat2x2, Path, Vec2, TILE_SIZE};

pub struct Picture {
    pub(crate) layers: Vec<Layer>,
    pub(crate) tiles: Vec<Tile>,
    pub(crate) spans: Vec<Span>,
    pub(crate) data: Vec<u64>,
}

pub(crate) struct Layer {
    pub(crate) color: Color,
    pub(crate) tiles: usize,
    pub(crate) spans: usize,
}

pub(crate) struct Tile {
    pub(crate) x: i16,
    pub(crate) y: i16,
}

pub(crate) struct Span {
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) len: i16,
}

impl Picture {
    pub fn new() -> Picture {
        Picture {
            layers: Vec::new(),
            tiles: Vec::new(),
            spans: Vec::new(),
            data: Vec::new(),
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

        let polygon = path.flatten(transform);

        let mut increments = Vec::new();
        let mut tile_increments = Vec::new();
        for contour in 0..polygon.contours.len() {
            let start = polygon.contours[contour];
            let end = *polygon.contours.get(contour + 1).unwrap_or(&polygon.points.len());
            let mut last = polygon.points[start] + position;
            let mut tile_y_prev = (last.y as u16 / TILE_SIZE as u16) as i16;
            for &point in &polygon.points[start + 1..end] {
                let point = point + position;
                if point != last {
                    let x_dir = (point.x - last.x).signum() as i16;
                    let y_dir = (point.y - last.y).signum() as i16;
                    let dtdx = 1.0 / (point.x - last.x);
                    let dtdy = 1.0 / (point.y - last.y);
                    let mut x = last.x as u16 as i16;
                    let mut y = last.y as u16 as i16;
                    let mut row_t0: f32 = 0.0;
                    let mut col_t0: f32 = 0.0;
                    let mut row_t1 = if last.y == point.y {
                        std::f32::INFINITY
                    } else {
                        let next_y = if point.y > last.y { (y + 1) as f32 } else { y as f32 };
                        (dtdy * (next_y - last.y)).min(1.0)
                    };
                    let mut col_t1 = if last.x == point.x {
                        std::f32::INFINITY
                    } else {
                        let next_x = if point.x > last.x { (x + 1) as f32 } else { x as f32 };
                        (dtdx * (next_x - last.x)).min(1.0)
                    };
                    let x_step = dtdx.abs();
                    let y_step = dtdy.abs();

                    loop {
                        let t0 = row_t0.max(col_t0);
                        let t1 = row_t1.min(col_t1);
                        let p0 = (1.0 - t0) * last + t0 * point;
                        let p1 = (1.0 - t1) * last + t1 * point;
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
                last = point;
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

        let mut areas = [0.0; TILE_SIZE * TILE_SIZE];
        let mut heights = [0.0; TILE_SIZE * TILE_SIZE];
        let mut prev = [0.0; TILE_SIZE];
        let mut next = [0.0; TILE_SIZE];

        let mut tile_increments_i = 0;
        let mut winding = 0;

        let tiles_start = self.tiles.len();
        let spans_start = self.spans.len();

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
                self.tiles.push(Tile { x: bin.tile_x, y: bin.tile_y });
                for row in 0..TILE_SIZE {
                    use std::convert::TryInto;
                    self.data.push(u64::from_le_bytes(tile[row * TILE_SIZE..(row + 1) * TILE_SIZE].try_into().unwrap()));
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
                        self.spans.push(Span { x: bin.tile_x + 1, y: bin.tile_y, len: bins[i + 1].tile_x - (bin.tile_x + 1) });
                    }
                }
            }
        }

        self.layers.push(Layer {
            color, 
            tiles: self.tiles.len() - tiles_start,
            spans: self.spans.len() - spans_start,
        });
    }
}
