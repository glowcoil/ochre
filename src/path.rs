use crate::geom::*;

const TOLERANCE: f32 = 0.1;

pub struct Path {
    pub(crate) points: Vec<Vec2>,
    pub(crate) components: Vec<usize>,
}

impl Path {
    pub fn new() -> Path {
        Path {
            points: vec![Vec2::new(0.0, 0.0)],
            components: vec![0],
        }
    }

    pub fn move_to(&mut self, point: Vec2) -> &mut Self {
        if *self.components.last().unwrap() == self.points.len() - 1 {
            *self.points.last_mut().unwrap() = point;
        } else {
            self.components.push(self.points.len());
            self.points.push(point);
        }
        self
    }

    pub fn line_to(&mut self, point: Vec2) -> &mut Self {
        self.points.push(point);
        self
    }

    pub fn quadratic_to(&mut self, control: Vec2, point: Vec2) -> &mut Self {
        let current = *self.points.last().unwrap();
        let a_x = current.x - 2.0 * control.x + point.x;
        let a_y = current.y - 2.0 * control.y + point.y;
        let dt = ((8.0 * TOLERANCE * TOLERANCE) / (a_x * a_x + a_y * a_y)).sqrt().sqrt();
        let mut t = dt;
        while t < 1.0 {
            let p12 = Vec2::lerp(t, current, control);
            let p23 = Vec2::lerp(t, control, point);
            self.points.push(Vec2::lerp(t, p12, p23));
            t += dt;
        }
        self
    }

    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, point: Vec2) -> &mut Self {
        let current = *self.points.last().unwrap();
        let a_x = -current.x + 3.0 * control1.x - 3.0 * control2.x + point.x;
        let b_x = 3.0 * (current.x - 2.0 * control1.x + control2.x);
        let a_y = -current.y + 3.0 * control1.y - 3.0 * control2.y + point.y;
        let b_y = 3.0 * (current.y - 2.0 * control1.y + control2.y);
        let conc = (b_x * b_x + b_y * b_y).max((a_x + b_x) * (a_x + b_x) + (a_y + b_y) * (a_y + b_y));
        let dt = ((8.0 * TOLERANCE * TOLERANCE) / conc).sqrt().sqrt();
        let mut t = dt;
        while t < 1.0 {
            let p12 = Vec2::lerp(t, current, control1);
            let p23 = Vec2::lerp(t, control1, control2);
            let p34 = Vec2::lerp(t, control2, point);
            let p123 = Vec2::lerp(t, p12, p23);
            let p234 = Vec2::lerp(t, p23, p34);
            self.points.push(Vec2::lerp(t, p123, p234));
            t += dt;
        }
        self
    }

    pub fn arc_to(&mut self, radius: f32, point: Vec2) -> &mut Self {
        let current = *self.points.last().unwrap();
        let winding = radius.signum();
        let to_midpoint = 0.5 * (point - current);
        let dist_to_midpoint = to_midpoint.length();
        let radius = radius.abs().max(to_midpoint.length());
        let dist_to_center = (radius * radius - dist_to_midpoint * dist_to_midpoint).sqrt();
        let to_center = winding * dist_to_center * if to_midpoint.length() == 0.0 {
            Vec2::new(-1.0, 0.0)
        } else {
            Vec2::new(to_midpoint.y, -to_midpoint.x).normalized()
        };
        let center = current + to_midpoint + to_center;
        let mut angle = current - center;
        let end_angle = point - center;
        let rotor_x = (1.0 - 2.0 * (TOLERANCE / radius)).max(0.0);
        let rotor_y = -winding * (1.0 - rotor_x * rotor_x).sqrt();
        loop {
            let prev_sign = winding * (angle.x * end_angle.y - angle.y * end_angle.x);
            angle = Vec2::new(rotor_x * angle.x - rotor_y * angle.y, rotor_x * angle.y + rotor_y * angle.x);
            let sign = winding * (angle.x * end_angle.y - angle.y * end_angle.x);
            if prev_sign <= 0.0 && sign >= 0.0 {
                break;
            }
            self.points.push(center + angle);
        }
        self.points.push(point);
        self
    }

    pub fn fill_convex(mut self) -> Mesh {
        if self.points.len() < 3 {
            return Mesh {
                vertices: Vec::new(),
                indices: Vec::new(),
                fringe_vertices: 0,
                fringe_indices: 0
            };
        }
        let num_points = self.points.len() as u16;
        for i in 0..self.points.len() {
            let prev = self.points[(i + self.points.len() - 1) % self.points.len()];
            let curr = self.points[i];
            let next = self.points[(i + 1) % self.points.len()];
            let prev_tangent = curr - prev;
            let next_tangent = next - curr;
            let tangent = prev_tangent + next_tangent;
            let normal = Vec2::new(-tangent.y, tangent.x).normalized();
            self.points[i] = curr - 0.5 * normal;
            self.points.push(curr + 0.5 * normal);
        }
        let mut indices = Vec::new();
        for i in 1..(num_points.saturating_sub(1) as u16) {
            indices.extend_from_slice(&[0, i, i + 1]);
        }
        let fringe_indices = indices.len();
        for i in 0..(num_points as u16) {
            indices.extend_from_slice(&[
                i, num_points + i, num_points + ((i + 1) % num_points),
                i, num_points + ((i + 1) % num_points), ((i + 1) % num_points),
            ]);
        }
        Mesh {
            vertices: self.points,
            indices,
            fringe_vertices: num_points as usize,
            fringe_indices,
        }
    }

    pub fn rect(pos: Vec2, size: Vec2) -> Path {
        let mut path = Path::new();
        path.move_to(pos)
            .line_to(Vec2::new(pos.x, pos.y + size.y))
            .line_to(Vec2::new(pos.x + size.x, pos.y + size.y))
            .line_to(Vec2::new(pos.x + size.x, pos.y));
        path
    }

    pub fn rect_fill(pos: Vec2, size: Vec2) -> Mesh {
        Path::rect(pos, size).fill_convex()
    }

    pub fn round_rect(pos: Vec2, size: Vec2, radius: f32) -> Path {
        let radius = radius.min(0.5 * size.x).min(0.5 * size.y);
        let mut path = Path::new();
        path.move_to(Vec2::new(pos.x, pos.y + radius))
            .line_to(Vec2::new(pos.x, pos.y + size.y - radius))
            .arc_to(radius, Vec2::new(pos.x + radius, pos.y + size.y))
            .line_to(Vec2::new(pos.x + size.x - radius, pos.y + size.y))
            .arc_to(radius, Vec2::new(pos.x + size.x, pos.y + size.y - radius))
            .line_to(Vec2::new(pos.x + size.x, pos.y + radius))
            .arc_to(radius, Vec2::new(pos.x + size.x - radius, pos.y))
            .line_to(Vec2::new(pos.x + radius, pos.y))
            .arc_to(radius, Vec2::new(pos.x, pos.y + radius));
        path
    }

    pub fn round_rect_fill(pos: Vec2, size: Vec2, radius: f32) -> Mesh {
        Path::round_rect(pos, size, radius).fill_convex()
    }
}

pub struct Mesh {
    pub(crate) vertices: Vec<Vec2>,
    pub(crate) indices: Vec<u16>,
    pub(crate) fringe_vertices: usize,
    pub(crate) fringe_indices: usize,
}
