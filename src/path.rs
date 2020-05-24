use crate::geom::*;

const TOLERANCE: f32 = 0.1;
pub const TILE_SIZE: usize = 8;

#[derive(Clone, Debug)]
pub struct Path {
    pub contours: Vec<Contour>,
}

#[derive(Clone, Debug)]
pub struct Contour {
    pub points: Vec<Vec2>,
    pub closed: bool,
}

#[derive(Clone, Debug)]
pub struct Tiles {
    pub map: Vec<Tile>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Tile {
    x: i16,
    y: i16,
    index: Option<usize>,
}

impl Path {
    pub fn new() -> Path {
        Path {
            contours: Vec::new(),
        }
    }

    pub fn move_to(&mut self, point: Vec2) -> &mut Self {
        self.contours.push(Contour {
            points: vec![point],
            closed: false,
        });
        self
    }

    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        if let Some(contour) = self.contours.last_mut() {
            contour.points.push(point);
        }
        self
    }

    pub fn quadratic_to(&mut self, control: Vec2, point: Vec2) -> &mut Self {
        if let Some(contour) = self.contours.last_mut() {
            let last = *contour.points.last().unwrap();
            let a_x = last.x - 2.0 * control.x + point.x;
            let a_y = last.y - 2.0 * control.y + point.y;
            let n = ((a_x * a_x + a_y * a_y) / (8.0 * TOLERANCE * TOLERANCE)).sqrt().sqrt() as usize;
            let dt = 1.0 / n as f32;
            let mut t = 0.0;
            for _ in 0..n.saturating_sub(1) {
                t += dt;
                let p01 = Vec2::lerp(t, last, control);
                let p12 = Vec2::lerp(t, control, point);
                contour.points.push(Vec2::lerp(t, p01, p12));
            }
            contour.points.push(point);
        }
        self
    }

    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, point: Vec2) -> &mut Self {
        if let Some(contour) = self.contours.last_mut() {
            let last = *contour.points.last().unwrap();
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
                contour.points.push(Vec2::lerp(t, p012, p123));
            }
            contour.points.push(point);
        }
        self
    }

    pub fn arc_to(&mut self, radius: f32, large_arc: bool, sweep: bool, point: Vec2) -> &mut Self {
        if let Some(contour) = self.contours.last_mut() {
            let last = *contour.points.last().unwrap();
            let radius = radius.max(0.5 * last.distance(point));
            let to_midpoint = 0.5 * (point - last);
            let dist_to_midpoint = to_midpoint.length();
            let dist_to_center = (radius * radius - dist_to_midpoint * dist_to_midpoint).sqrt();
            let to_center = dist_to_center * Vec2::new(to_midpoint.y, -to_midpoint.x).normalized();
            let center = last + to_midpoint + if large_arc == sweep { to_center } else { -1.0 * to_center };
            let start_angle = (last.y - center.y).atan2(last.x - center.x);
            let mut end_angle = (point.y - center.y).atan2(point.x - center.x);
            if sweep && end_angle < start_angle { end_angle += 2.0 * std::f32::consts::PI; }
            let n = (0.5 * (end_angle - start_angle).abs() / (1.0 - TOLERANCE / radius).acos()) as usize;
            let dtheta = (end_angle - start_angle) / n as f32;
            let mut theta = start_angle;
            for _ in 0..n.saturating_sub(1) {
                theta += dtheta;
                contour.points.push(center + radius * Vec2::new(theta.cos(), theta.sin()));
            }
            contour.points.push(point);
        }
        self
    }

    pub fn close(&mut self) -> &mut Self {
        if let Some(contour) = self.contours.last_mut() {
            contour.closed = true;
        }
        self
    }

    pub fn stroke(&self, width: f32) -> Path {
        let mut path = Path::new();

        for contour in self.contours.iter() {
            let mut points = Vec::new();
            points.extend_from_slice(&contour.points);
            points.extend(contour.points.iter().rev());
            for i in 0..contour.points.len() {
                let prev = points[(i + points.len() - 1) % contour.points.len()];
                let curr = points[i];
                let next = points[(i + 1) % contour.points.len()];
                let prev_tan = (curr - prev).normalized();
                let next_tan = (next - curr).normalized();
                let prev_nor = Vec2::new(-prev_tan.y, prev_tan.x);
                let next_nor = Vec2::new(-next_tan.y, next_tan.x);
                if curr != prev && curr != next {
                    points[i] += 0.5 * width * (prev_nor + next_nor) * (1.0 / (1.0 + prev_nor.dot(next_nor))).min(2.0);
                }
            }
            for i in 0..contour.points.len() {
                let prev = points[contour.points.len() + (i + contour.points.len() - 1) % contour.points.len()];
                let curr = points[contour.points.len() + i];
                let next = points[contour.points.len() + (i + 1) % contour.points.len()];
                let prev_tan = (curr - prev).normalized();
                let next_tan = (next - curr).normalized();
                let prev_nor = Vec2::new(-prev_tan.y, prev_tan.x);
                let next_nor = Vec2::new(-next_tan.y, next_tan.x);
                if curr != prev && curr != next {
                    points[contour.points.len() + i] += 0.5 * width * (prev_nor + next_nor) * (1.0 / (1.0 + prev_nor.dot(next_nor))).min(2.0);
                }
            }
            path.contours.push(Contour { points, closed: true });
        }

        path
    }

    pub fn to_spans(&self) -> Vec<(i16, i16, [u8; TILE_SIZE * TILE_SIZE], u8)> {
        #[derive(Copy, Clone, Debug)]
        pub struct Increment {
            x: i16,
            y: i16,
            area: f32,
            height: f32,
        }

        let mut increments = Vec::new();
        for contour in self.contours.iter() {
            let mut last = *contour.points.last().unwrap();
            for &point in contour.points.iter() {
                if point.y != last.y {
                    let x_dir = (point.x - last.x).signum() as i16;
                    let y_dir = (point.y - last.y).signum() as i16;
                    let dtdx = 1.0 / (point.x - last.x);
                    let dtdy = 1.0 / (point.y - last.y);
                    let end_x = point.x as i16;
                    let end_y = point.y as i16;
                    let mut x = last.x as i16;
                    let mut y = last.y as i16;
                    let mut row_t0: f32 = 0.0;
                    let mut col_t0 = 0.0;
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
        struct Block {
            tile_x: i16,
            tile_y: i16,
            start: usize,
            end: usize,
        }
        let mut blocks = Vec::new();
        let mut block = Block { tile_x: 0, tile_y: 0, start: 0, end: 0 };
        if let Some(first) = increments.first() {
            block.tile_x = ((first.x as u16) / TILE_SIZE as u16) as i16;
            block.tile_y = ((first.y as u16) / TILE_SIZE as u16) as i16;
        }
        for (i, increment) in increments.iter().enumerate() {
            let tile_x = ((increment.x as u16) / TILE_SIZE as u16) as i16;
            let tile_y = ((increment.y as u16) / TILE_SIZE as u16) as i16;
            if tile_x != block.tile_x || tile_y != block.tile_y {
                blocks.push(block);
                block = Block { tile_x, tile_y, start: i, end: i };
            }
            block.end += 1;
        }
        blocks.push(block);
        blocks.sort_unstable_by_key(|block| (block.tile_y, block.tile_x));

        let mut tiles = Vec::new();

        let mut areas = [0.0; TILE_SIZE * TILE_SIZE];
        let mut heights = [0.0; TILE_SIZE * TILE_SIZE];
        let mut prev = [0.0; TILE_SIZE];
        let mut next = [0.0; TILE_SIZE];
        for i in 0..blocks.len() {
            let block = blocks[i];
            for increment in &increments[block.start..block.end] {
                let x = increment.x as usize % TILE_SIZE;
                let y = increment.y as usize % TILE_SIZE;
                areas[(y * TILE_SIZE + x) as usize] += increment.area;
                heights[(y * TILE_SIZE + x) as usize] += increment.height;
            }
            if i + 1 == blocks.len() || blocks[i + 1].tile_x != block.tile_x || blocks[i + 1].tile_y != block.tile_y {
                let mut tile = [0; TILE_SIZE * TILE_SIZE];
                for y in 0..TILE_SIZE {
                    let mut accum = prev[y];
                    for x in 0..TILE_SIZE {
                        tile[y * TILE_SIZE + x] = (((accum + areas[y * TILE_SIZE + x]).abs().min(1.0) * 255.0) as u8);
                        accum += heights[y * TILE_SIZE + x];
                    }
                    next[y] = accum;
                }
                let mut winding = 0;
                for next in next.iter() { if (next.abs() * 255.0) as u8 != 0 { winding = 1; } }
                tiles.push((block.tile_x, block.tile_y, tile, winding));
                areas = [0.0; TILE_SIZE * TILE_SIZE];
                heights = [0.0; TILE_SIZE * TILE_SIZE];
                if i + 1 < blocks.len() && blocks[i + 1].tile_y == block.tile_y {
                    prev = next;
                } else {
                    prev = [0.0; TILE_SIZE];
                }
                next = [0.0; TILE_SIZE];
            }
        }

        tiles

        /*
        let mut counts = vec![0; (y_max + 1 - y_min) as usize];
        for increment in increments.iter() {
            counts[(increment.y - y_min) as usize] += 1;
        }
        let mut starts = Vec::with_capacity((y_max + 1 - y_min) as usize);
        let mut total = 0;
        for count in counts.iter() {
            let new_total = total + *count;
            starts.push(total);
            total = new_total;
        }
        let mut sorted_increments = vec![Increment { x: 0, y: 0, area: 0.0, height: 0.0 }; increments.len()];
        for increment in increments {
            let pos = &mut starts[(increment.y - y_min) as usize];
            sorted_increments[*pos] = increment;
            *pos += 1;
        }
        for (end, count) in starts.iter().zip(counts.iter()) {
            sorted_increments[end - count..*end].sort_unstable_by_key(|inc| inc.x);
        }

        let mut spans = Vec::new();
        if !increments.is_empty() {
            let mut x = increments[0].x;
            let mut y = increments[0].y;
            let mut coverage: f32 = 0.0;
            let mut accum: f32 = 0.0;
            for increment in increments {
                if increment.x != x || increment.y != y {
                    if (coverage.abs() * 255.0) as u8 != 0 {
                        spans.push(Span { x, y, len: 1, coverage: coverage.abs().min(1.0) });
                    }
                    if increment.y == y && increment.x > x + 1 && (accum.abs() * 255.0) as u8 != 0 {
                        spans.push(Span {
                            x: x + 1,
                            y: y,
                            len: (increment.x - x - 1) as u16,
                            coverage: accum.abs().min(1.0),
                        });
                    }
                    if increment.y != y {
                        accum = 0.0;
                    }
                    x = increment.x;
                    y = increment.y;
                    coverage = accum;
                }
                coverage += increment.area;
                accum += increment.height;
            }
            if (coverage.abs() * 255.0) as u8 != 0 {
                spans.push(Span { x, y, len: 1, coverage: coverage.abs().min(1.0) });
            }
        }

        spans*/
    }
}
