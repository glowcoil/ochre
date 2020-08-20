use crate::geom::*;

const TOLERANCE: f32 = 0.1;
pub const TILE_SIZE: usize = 8;

#[derive(Clone)]
pub struct Path {
    commands: Vec<PathCommand>,
    points: Vec<Vec2>,
}

#[derive(Copy, Clone)]
enum PathCommand {
    Move,
    Line,
    Quadratic,
    Cubic,
    Close,
}

impl Path {
    pub fn new() -> Path {
        Path {
            commands: Vec::new(),
            points: Vec::new(),
        }
    }

    pub fn move_to(&mut self, point: Vec2) -> &mut Self {
        self.commands.push(PathCommand::Move);
        self.points.push(point);
        self
    }

    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        self.commands.push(PathCommand::Line);
        self.points.push(point);
        self
    }

    pub fn quadratic_to(&mut self, control: Vec2, point: Vec2) -> &mut Self {
        self.commands.push(PathCommand::Quadratic);
        self.points.push(control);
        self.points.push(point);
        self
    }

    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, point: Vec2) -> &mut Self {
        self.commands.push(PathCommand::Cubic);
        self.points.push(control1);
        self.points.push(control2);
        self.points.push(point);
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.commands.push(PathCommand::Close);
        self
    }

    pub fn flatten(&self, transform: Mat2x2) -> Polygon {
        let mut contours = Vec::new();
        let mut points = Vec::new();
        let mut last = Vec2::new(0.0, 0.0);
        let mut i = 0;
        for command in self.commands.iter() {
            match command {
                PathCommand::Move => {
                    contours.push(points.len());
                    let point = transform * self.points[i];
                    points.push(point);
                    last = point;
                    i += 1;
                }
                PathCommand::Line => {
                    let point = transform * self.points[i];
                    points.push(point);
                    last = point;
                    i += 1;
                }
                PathCommand::Quadratic => {
                    let control = transform * self.points[i];
                    let point = transform * self.points[i + 1];
                    let a_x = last.x - 2.0 * control.x + point.x;
                    let a_y = last.y - 2.0 * control.y + point.y;
                    let n = ((a_x * a_x + a_y * a_y) / (8.0 * TOLERANCE * TOLERANCE)).sqrt().sqrt() as usize;
                    let dt = 1.0 / n as f32;
                    let mut t = 0.0;
                    for _ in 0..n.saturating_sub(1) {
                        t += dt;
                        let p01 = Vec2::lerp(t, last, control);
                        let p12 = Vec2::lerp(t, control, point);
                        points.push(Vec2::lerp(t, p01, p12));
                    }
                    points.push(point);
                    last = point;
                    i += 2;
                }
                PathCommand::Cubic => {
                    let control1 = transform * self.points[i];
                    let control2 = transform * self.points[i + 1];
                    let point = transform * self.points[i + 2];
                    let a_x = -last.x + 3.0 * control1.x - 3.0 * control2.x + point.x;
                    let b_x = 3.0 * (last.x - 2.0 * control1.x + control2.x);
                    let a_y = -last.y + 3.0 * control1.y - 3.0 * control2.y + point.y;
                    let b_y = 3.0 * (last.y - 2.0 * control1.y + control2.y);
                    let conc = (b_x * b_x + b_y * b_y).max((a_x + b_x) * (a_x + b_x) + (a_y + b_y) * (a_y + b_y));
                    let n = (conc / (8.0 * TOLERANCE * TOLERANCE)).sqrt().sqrt() as usize;
                    let dt = 1.0 / n as f32;
                    let mut t = 0.0;
                    for _ in 0..n.saturating_sub(1) {
                        t += dt;
                        let p01 = Vec2::lerp(t, last, control1);
                        let p12 = Vec2::lerp(t, control1, control2);
                        let p23 = Vec2::lerp(t, control2, point);
                        let p012 = Vec2::lerp(t, p01, p12);
                        let p123 = Vec2::lerp(t, p12, p23);
                        points.push(Vec2::lerp(t, p012, p123));
                    }
                    points.push(point);
                    last = point;
                    i += 3;
                }
                PathCommand::Close => {
                    let first = points[*contours.last().unwrap()];
                    if last != first {
                        points.push(first);
                    }
                }
            }
        }

        Polygon { contours, points }
    }

    pub fn fill(&self, position: Vec2, transform: Mat2x2) -> Tiles {
        self.flatten(transform).rasterize(position)
    }
}

#[derive(Clone)]
pub struct Polygon {
    contours: Vec<usize>,
    points: Vec<Vec2>,
}

#[derive(Clone)]
pub struct Tiles {
    pub tiles: Vec<Tile>,
    pub spans: Vec<Span>,
    pub data: Vec<u8>,
}

#[derive(Copy, Clone)]
pub struct Tile {
    pub x: i16,
    pub y: i16,
    pub index: usize,
}

#[derive(Copy, Clone)]
pub struct Span {
    pub x: i16,
    pub y: i16,
    pub len: i16,
}

impl Polygon {
    fn rasterize(&self, position: Vec2) -> Tiles {
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

        let mut increments = Vec::new();
        let mut tile_increments = Vec::new();
        for contour in 0..self.contours.len() {
            let start = self.contours[contour];
            let end = *self.contours.get(contour + 1).unwrap_or(&self.points.len());
            let mut last = self.points[start] + position;
            let mut tile_y_prev = (last.y as u16 / TILE_SIZE as u16) as i16;
            for &point in &self.points[start + 1..end] {
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

        let mut tiles = Vec::new();
        let mut spans = Vec::new();
        let mut data = Vec::new();

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
                tiles.push(Tile { x: bin.tile_x, y: bin.tile_y, index: data.len() });
                data.extend_from_slice(&tile);
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
                        spans.push(Span { x: bin.tile_x + 1, y: bin.tile_y, len: bins[i + 1].tile_x - (bin.tile_x + 1) });
                    }
                }
            }
        }

        Tiles { tiles, spans, data }
    }
}
