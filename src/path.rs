use crate::geom::*;

const TOLERANCE: f32 = 0.1;

#[derive(Clone)]
pub struct Path {
    pub(crate) commands: Vec<PathCommand>,
    pub(crate) points: Vec<Vec2>,
}

#[derive(Copy, Clone)]
pub(crate) enum PathCommand {
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

    pub fn flatten(&self, transform: Mat2x2) -> Path {
        let mut path = Path::new();
        let mut i = 0;
        for command in self.commands.iter() {
            match command {
                PathCommand::Move => {
                    path.move_to(transform * self.points[i]);
                    i += 1;
                }
                PathCommand::Line => {
                    path.line_to(transform * self.points[i]);
                    i += 1;
                }
                PathCommand::Quadratic => {
                    let last = *path.points.last().unwrap_or(&Vec2::new(0.0, 0.0));
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
                        path.line_to(Vec2::lerp(t, p01, p12));
                    }
                    path.line_to(point);
                    i += 2;
                }
                PathCommand::Cubic => {
                    let last = *path.points.last().unwrap_or(&Vec2::new(0.0, 0.0));
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
                        path.line_to(Vec2::lerp(t, p012, p123));
                    }
                    path.line_to(point);
                    i += 3;
                }
                PathCommand::Close => {
                    path.close();
                }
            }
        }

        path
    }

}
