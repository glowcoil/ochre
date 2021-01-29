use crate::{Path, PathCommand, Transform, Vec2};

pub const TILE_SIZE: usize = 8;

pub trait TileBuilder {
    fn tile(&mut self, x: i16, y: i16, data: [u8; TILE_SIZE * TILE_SIZE]);
    fn span(&mut self, x: i16, y: i16, width: u16);
}

pub fn rasterize<B: TileBuilder>(path: &Path, transform: Transform, builder: &mut B) {
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
    let mut first = transform.offset;
    let mut last = transform.offset;
    let mut tile_y_prev = (first.y.floor() as i16).wrapping_div_euclid(TILE_SIZE as i16);
    let mut commands = flattened.commands.iter();
    let mut i = 0;
    loop {
        let command = commands.next();

        let p1;
        let p2;
        if let Some(command) = command {
            match command {
                PathCommand::Move => {
                    let point = Vec2::new(flattened.data[i], flattened.data[i + 1]);
                    p1 = last;
                    p2 = first;
                    first = point;
                    last = point;
                    i += 2;
                }
                PathCommand::Line => {
                    let point = Vec2::new(flattened.data[i], flattened.data[i + 1]);
                    p1 = last;
                    p2 = point;
                    last = point;
                    i += 2;
                }
                _ => {
                    continue;
                }
            }
        } else {
            p1 = last;
            p2 = first;
        }

        if p1 != p2 {
            let x_dir = (p2.x - p1.x).signum() as i16;
            let y_dir = (p2.y - p1.y).signum() as i16;
            let dtdx = 1.0 / (p2.x - p1.x);
            let dtdy = 1.0 / (p2.y - p1.y);
            let mut x = p1.x.floor() as i16;
            let mut y = p1.y.floor() as i16;
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

                let tile_y = y.wrapping_div_euclid(TILE_SIZE as i16);
                if tile_y != tile_y_prev {
                    tile_increments.push(TileIncrement {
                        tile_x: x.wrapping_div_euclid(TILE_SIZE as i16),
                        tile_y: tile_y_prev.min(tile_y),
                        sign: (tile_y - tile_y_prev) as i8,
                    });
                    tile_y_prev = tile_y;
                }

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

        if let Some(PathCommand::Move) = command {
            tile_y_prev = (first.y.floor() as i16).wrapping_div_euclid(TILE_SIZE as i16);
        }

        if command.is_none() { break; }
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
        bin.tile_x = (first.x as i16).wrapping_div_euclid(TILE_SIZE as i16);
        bin.tile_y = (first.y as i16).wrapping_div_euclid(TILE_SIZE as i16);
    }
    for (i, increment) in increments.iter().enumerate() {
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

    tile_increments.sort_unstable_by_key(|tile_inc| (tile_inc.tile_y, tile_inc.tile_x));

    let mut areas = [0.0; TILE_SIZE * TILE_SIZE];
    let mut heights = [0.0; TILE_SIZE * TILE_SIZE];
    let mut prev = [0.0; TILE_SIZE];
    let mut next = [0.0; TILE_SIZE];

    let mut tile_increments_i = 0;
    let mut winding = 0;

    for i in 0..bins.len() {
        let bin = bins[i];
        for increment in &increments[bin.start..bin.end] {
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
                while tile_increments_i < tile_increments.len() {
                    let tile_increment = tile_increments[tile_increments_i];
                    if (tile_increment.tile_y, tile_increment.tile_x) > (bin.tile_y, bin.tile_x) {
                        break;
                    }
                    winding += tile_increment.sign as isize;
                    tile_increments_i += 1;
                }
                if winding != 0 {
                    let width = bins[i + 1].tile_x - bin.tile_x - 1;
                    builder.span((bin.tile_x + 1) * TILE_SIZE as i16, bin.tile_y * TILE_SIZE as i16, width as u16);
                }
            }
        }
    }
}
