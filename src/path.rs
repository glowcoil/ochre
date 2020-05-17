use crate::geom::*;

const TOLERANCE: f32 = 0.1;

#[derive(Clone, Debug)]
pub struct Path {
    pub contours: Vec<Contour>,
}

#[derive(Clone, Debug)]
pub struct Contour {
    pub points: Vec<Vec2>,
    pub closed: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct Span {
    pub x: i16,
    pub y: i16,
    pub len: u16,
    pub coverage: f32,
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
            let to_midpoint = 0.5 * (point - last);
            let dist_to_midpoint = to_midpoint.length();
            let dist_to_center = (radius * radius - dist_to_midpoint * dist_to_midpoint).sqrt();
            let to_center = dist_to_center * Vec2::new(to_midpoint.y, -to_midpoint.x).normalized();
            let center = last + to_midpoint + if large_arc == sweep { to_center } else { -1.0 * to_center };
            let start_angle = (last.y - center.y).atan2(last.x - center.x);
            let mut end_angle = (point.y - center.y).atan2(point.x - center.x);
            if sweep && end_angle < start_angle { end_angle += 2.0 * std::f32::consts::PI; }
            let n = (std::f32::consts::PI / (1.0 - TOLERANCE / radius).acos()) as usize;
            let dtheta = 2.0 * std::f32::consts::PI / n as f32;
            let mut theta = 0.0;
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

    pub fn to_spans(&self) -> Vec<Span> {
        #[derive(Copy, Clone, Debug)]
        pub struct Increment {
            x: i16,
            y: i16,
            area: f32,
            height: f32,
        }

        let mut len = 0;
        for contour in self.contours.iter() {
            len += contour.points.len();
        }
        let mut increments = Vec::with_capacity(len);

        let (mut y_min, mut y_max) = (0, 0);
        if let Some(contour) = self.contours.first() {
            if let Some(first) = contour.points.first() {
                y_min = first.y as i16;
                y_max = first.y as i16;
            }
        }
        for contour in self.contours.iter() {
            let mut last = *contour.points.last().unwrap();
            for &point in contour.points.iter() {
                if point.y != last.y {
                    let sign = (point.y - last.y).signum();
                    let (p0, p1) = if point.y < last.y { (point, last) } else { (last, point) };
                    let dxdy = (p1.x - p0.x) / (p1.y - p0.y);
                    let dydx = 1.0 / dxdy;
                    y_min = y_min.min(p0.y as i16);
                    y_max = y_max.max(p1.y as i16 + 1);
                    for y in p0.y as i16..p1.y as i16 + 1 {
                        let row_y0 = p0.y.max(y as f32);
                        let row_y1 = p1.y.min((y + 1) as f32);
                        let row_x0 = p0.x + dxdy * (row_y0 - p0.y);
                        let row_x1 = p0.x + dxdy * (row_y1 - p0.y);
                        let row_x_min = row_x0.min(row_x1);
                        let row_x_max = row_x0.max(row_x1);
                        for x in row_x_min as i16..row_x_max as i16 + 1 {
                            let x0 = row_x_min.max(x as f32);
                            let x1 = row_x_max.min((x + 1) as f32);
                            let (y0, y1) = if p0.x == p1.x {
                                (row_y0, row_y1)
                            } else {
                                (p0.y + dydx * (x0 - p0.x), p0.y + dydx * (x1 - p0.x))
                            };
                            let height = sign * (y1 - y0).abs();
                            let area = 0.5 * height * (2.0 * (x + 1) as f32 - x0 - x1);
                            increments.push(Increment { x, y, area, height });
                        }
                    }
                }
                last = point;
            }
        }

        increments.sort_unstable_by_key(|inc| (inc.y, inc.x));

        let mut spans = Vec::new();
        if !increments.is_empty() {
            let mut x = increments[0].x;
            let mut y = increments[0].y;
            let mut coverage: f32 = 0.0;
            let mut accum: f32 = 0.0;
            for increment in increments {
                if increment.x != x || increment.y != y {
                    if coverage != 0.0 {
                        spans.push(Span { x, y, len: 1, coverage: coverage.abs().min(1.0) });
                    }
                    if increment.y == y && increment.x > x + 1 && accum != 0.0 {
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
            if coverage != 0.0 {
                spans.push(Span { x, y, len: 1, coverage: coverage.abs().min(1.0) });
            }
        }

        spans
    }
}