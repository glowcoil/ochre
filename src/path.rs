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
}
