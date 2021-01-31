use crate::{flatten, stroke, PathCmd, Transform, Vec2};

/// The tile size used by the rasterizer (not configurable).
pub const TILE_SIZE: usize = 8;

const TOLERANCE: f32 = 0.1;

/// A trait to implement for consuming the tile data produced by
/// [`rasterize()`].
///
/// [`rasterize`]: crate::rasterize()
pub trait TileBuilder {
    /// Called with the position and data of an alpha mask tile.
    fn tile(&mut self, x: i16, y: i16, data: [u8; TILE_SIZE * TILE_SIZE]);

    /// Called with the position and width of a solid interior span.
    ///
    /// The height of a span is always [`TILE_SIZE`] pixels.
    ///
    /// [`TILE_SIZE`]: crate::TILE_SIZE
    fn span(&mut self, x: i16, y: i16, width: u16);
}

#[derive(Copy, Clone)]
struct Increment {
    x: i16,
    y: i16,
    area: f32,
    height: f32,
}

#[derive(Copy, Clone)]
struct TileIncrement {
    tile_x: i16,
    tile_y: i16,
    sign: i8,
}

/// Rasterizes paths.
pub struct Rasterizer {
    increments: Vec<Increment>,
    tile_increments: Vec<TileIncrement>,
    first: Vec2,
    last: Vec2,
    tile_y_prev: i16,
}

impl Rasterizer {
    /// Initializes a new rasterizer.
    pub fn new() -> Rasterizer {
        Rasterizer {
            increments: Vec::new(),
            tile_increments: Vec::new(),
            first: Vec2::new(0.0, 0.0),
            last: Vec2::new(0.0, 0.0),
            tile_y_prev: 0,
        }
    }

    /// Begins a new path component starting at the given location.
    pub fn move_to(&mut self, point: Vec2) {
        if self.last != self.first {
            self.line_to(self.first);
        }

        self.first = point;
        self.last = point;
        self.tile_y_prev = (point.y.floor() as i16).wrapping_div_euclid(TILE_SIZE as i16);
    }

    /// Adds a line segment to be rasterized.
    pub fn line_to(&mut self, point: Vec2) {
        if point != self.last {
            let x_dir = (point.x - self.last.x).signum() as i16;
            let y_dir = (point.y - self.last.y).signum() as i16;
            let dtdx = 1.0 / (point.x - self.last.x);
            let dtdy = 1.0 / (point.y - self.last.y);
            let mut x = self.last.x.floor() as i16;
            let mut y = self.last.y.floor() as i16;
            let mut row_t0: f32 = 0.0;
            let mut col_t0: f32 = 0.0;
            let mut row_t1 = if self.last.y == point.y {
                std::f32::INFINITY
            } else {
                let next_y = if point.y > self.last.y { (y + 1) as f32 } else { y as f32 };
                (dtdy * (next_y - self.last.y)).min(1.0)
            };
            let mut col_t1 = if self.last.x == point.x {
                std::f32::INFINITY
            } else {
                let next_x = if point.x > self.last.x { (x + 1) as f32 } else { x as f32 };
                (dtdx * (next_x - self.last.x)).min(1.0)
            };
            let x_step = dtdx.abs();
            let y_step = dtdy.abs();

            loop {
                let t0 = row_t0.max(col_t0);
                let t1 = row_t1.min(col_t1);
                let p0 = (1.0 - t0) * self.last + t0 * point;
                let p1 = (1.0 - t1) * self.last + t1 * point;
                let height = p1.y - p0.y;
                let right = (x + 1) as f32;
                let area = 0.5 * height * ((right - p0.x) + (right - p1.x));

                self.increments.push(Increment { x, y, area, height });

                if row_t1 < col_t1 {
                    row_t0 = row_t1;
                    row_t1 = (row_t1 + y_step).min(1.0);
                    y += y_dir;
                } else {
                    col_t0 = col_t1;
                    col_t1 = (col_t1 + x_step).min(1.0);
                    x += x_dir;
                }

                if row_t0 == 1.0 || col_t0 == 1.0 {
                    x = point.x.floor() as i16;
                    y = point.y.floor() as i16;
                }

                let tile_y = y.wrapping_div_euclid(TILE_SIZE as i16);
                if tile_y != self.tile_y_prev {
                    self.tile_increments.push(TileIncrement {
                        tile_x: x.wrapping_div_euclid(TILE_SIZE as i16),
                        tile_y: self.tile_y_prev.min(tile_y),
                        sign: (tile_y - self.tile_y_prev) as i8,
                    });
                    self.tile_y_prev = tile_y;
                }

                if row_t0 == 1.0 || col_t0 == 1.0 {
                    break;
                }
            }
        }

        self.last = point;
    }

    /// Adds a [`PathCmd`] to be rasterized.
    ///
    /// [`PathCmd`]: crate::PathCmd 
    pub fn command(&mut self, command: PathCmd) {
        command.flatten(self.last, TOLERANCE, |cmd| {
            match cmd {
                PathCmd::Move(point) => {
                    self.move_to(point);
                }
                PathCmd::Line(point) => {
                    self.line_to(point);
                }
                _ => {}
            }
        });
    }

    /// Adds a path to be rasterized as a filled region, applying the given
    /// transform.
    pub fn fill(&mut self, path: &[PathCmd], transform: Transform) {
        for command in path {
            self.command(command.transform(transform));
        }
    }

    /// Adds a path to be rasterized as a stroke with the given width, applying
    /// the given transform.
    pub fn stroke(&mut self, path: &[PathCmd], width: f32, transform: Transform) {
        self.fill(&stroke(&flatten(path, TOLERANCE), width), transform);
    }

    /// Rasterizes the accumulated path data, passing the results to the given
    /// [`TileBuilder`]. Consumes the rasterizer.
    ///
    /// The path is rasterized to a set of 8×8 alpha mask tiles and n×8 solid
    /// interior spans.
    ///
    /// [`TileBuilder`]: crate::TileBuilder
    pub fn finish<B: TileBuilder>(mut self, builder: &mut B) {
        if self.last != self.first {
            self.line_to(self.first);
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
        if let Some(first) = self.increments.first() {
            bin.tile_x = (first.x as i16).wrapping_div_euclid(TILE_SIZE as i16);
            bin.tile_y = (first.y as i16).wrapping_div_euclid(TILE_SIZE as i16);
        }
        for (i, increment) in self.increments.iter().enumerate() {
            let tile_x = increment.x.wrapping_div_euclid(TILE_SIZE as i16);
            let tile_y = increment.y.wrapping_div_euclid(TILE_SIZE as i16);
            if tile_x != bin.tile_x || tile_y != bin.tile_y {
                bins.push(bin);
                bin = Bin { tile_x, tile_y, start: i, end: i };
            }
            bin.end += 1;
        }
        bins.push(bin);
        bins.sort_unstable_by_key(|bin| (bin.tile_y, bin.tile_x));

        self.tile_increments.sort_unstable_by_key(|tile_inc| (tile_inc.tile_y, tile_inc.tile_x));

        let mut areas = [0.0; TILE_SIZE * TILE_SIZE];
        let mut heights = [0.0; TILE_SIZE * TILE_SIZE];
        let mut prev = [0.0; TILE_SIZE];
        let mut next = [0.0; TILE_SIZE];

        let mut tile_increments_i = 0;
        let mut winding = 0;

        for i in 0..bins.len() {
            let bin = bins[i];
            for increment in &self.increments[bin.start..bin.end] {
                let x = (increment.x as usize).wrapping_rem_euclid(TILE_SIZE);
                let y = (increment.y as usize).wrapping_rem_euclid(TILE_SIZE);
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

                builder.tile(bin.tile_x * TILE_SIZE as i16, bin.tile_y * TILE_SIZE as i16, tile);

                areas = [0.0; TILE_SIZE * TILE_SIZE];
                heights = [0.0; TILE_SIZE * TILE_SIZE];
                if i + 1 < bins.len() && bins[i + 1].tile_y == bin.tile_y {
                    prev = next;
                } else {
                    prev = [0.0; TILE_SIZE];
                }
                next = [0.0; TILE_SIZE];

                if i + 1 < bins.len() && bins[i + 1].tile_y == bin.tile_y && bins[i + 1].tile_x > bin.tile_x + 1 {
                    while tile_increments_i < self.tile_increments.len() {
                        let tile_increment = self.tile_increments[tile_increments_i];
                        if (tile_increment.tile_y, tile_increment.tile_x) > (bin.tile_y, bin.tile_x) {
                            break;
                        }
                        winding += tile_increment.sign as isize;
                        tile_increments_i += 1;
                    }
                    if winding != 0 {
                        let width = bins[i + 1].tile_x - bin.tile_x - 1;
                        builder.span((bin.tile_x + 1) * TILE_SIZE as i16, bin.tile_y * TILE_SIZE as i16, width as u16 * TILE_SIZE as u16);
                    }
                }
            }
        }
    }
}
