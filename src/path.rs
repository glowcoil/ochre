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
                    let dt = ((4.0 * TOLERANCE) / (last - 2.0 * control + point).length()).sqrt();
                    let mut t = 0.0;
                    while t < 1.0 {
                        t = (t + dt).min(1.0);
                        let p01 = Vec2::lerp(t, last, control);
                        let p12 = Vec2::lerp(t, control, point);
                        path.line_to(Vec2::lerp(t, p01, p12));
                    }
                    i += 2;
                }
                PathCommand::Cubic => {
                    let last = *path.points.last().unwrap_or(&Vec2::new(0.0, 0.0));
                    let control1 = transform * self.points[i];
                    let control2 = transform * self.points[i + 1];
                    let point = transform * self.points[i + 2];
                    let a = -1.0 * last + 3.0 * control1 - 3.0 * control2 + point;
                    let b = 3.0 * (last - 2.0 * control1 + control2);
                    let conc = b.length().max((a + b).length());
                    let dt = ((8.0f32.sqrt() * TOLERANCE) / conc).sqrt();
                    let mut t = 0.0;
                    while t < 1.0 {
                        t = (t + dt).min(1.0);
                        let p01 = Vec2::lerp(t, last, control1);
                        let p12 = Vec2::lerp(t, control1, control2);
                        let p23 = Vec2::lerp(t, control2, point);
                        let p012 = Vec2::lerp(t, p01, p12);
                        let p123 = Vec2::lerp(t, p12, p23);
                        path.line_to(Vec2::lerp(t, p012, p123));
                    }
                    i += 3;
                }
                PathCommand::Close => {
                    path.close();
                }
            }
        }

        path
    }

    pub(crate) fn stroke(&self, width: f32) -> Path {
        let mut path = Path::new();

        let mut contour_start = 0;
        let mut contour_end = 0;
        let mut closed = false;
        let mut commands = self.commands.iter();
        loop {
            let command = commands.next();

            if let None | Some(PathCommand::Move) = command {
                if contour_start != contour_end {
                    let contour = &self.points[contour_start..contour_end];

                    let base = path.points.len();

                    let first_point = if closed {
                        *contour.last().unwrap()
                    } else {
                        contour[0]
                    };
                    let mut prev_point = first_point;
                    let mut prev_normal = Vec2::new(0.0, 0.0);
                    let mut points = contour.into_iter();
                    loop {
                        let point = points.next();

                        let next_point = if let Some(&point) = point {
                            point
                        } else {
                            first_point
                        };

                        if next_point != prev_point {
                            let next_tangent = next_point - prev_point;
                            let next_normal = Vec2::new(-next_tangent.y, next_tangent.x).normalized();

                            let offset = 1.0 / (1.0 + prev_normal.dot(next_normal));
                            if offset.abs() > 2.0 {
                                path.points.push(prev_point + 0.5 * width * prev_normal);
                                path.points.push(prev_point + 0.5 * width * next_normal);
                            } else {
                                path.points.push(prev_point + 0.5 * width * offset * (prev_normal + next_normal));
                            }

                            prev_point = next_point;
                            prev_normal = next_normal;
                        }

                        if point.is_none() { break; }
                    }

                    if path.points.len() > base {
                        path.commands.push(PathCommand::Move);
                        for _ in (base + 1)..path.points.len() {
                            path.commands.push(PathCommand::Line);
                        }
                        if closed {
                            path.commands.push(PathCommand::Close);
                        }
                    }

                    let base = path.points.len();

                    let first_point = if closed {
                        contour[0]
                    } else {
                        *contour.last().unwrap()
                    };
                    let mut prev_point = first_point;
                    let mut prev_normal = Vec2::new(0.0, 0.0);
                    let mut points = contour.into_iter().rev();
                    loop {
                        let point = points.next();

                        let next_point = if let Some(&point) = point {
                            point
                        } else {
                            first_point
                        };

                        if next_point != prev_point {
                            let next_tangent = next_point - prev_point;
                            let next_normal = Vec2::new(-next_tangent.y, next_tangent.x).normalized();

                            let offset = 1.0 / (1.0 + prev_normal.dot(next_normal));
                            if offset.abs() > 2.0 {
                                path.points.push(prev_point + 0.5 * width * prev_normal);
                                path.points.push(prev_point + 0.5 * width * next_normal);
                            } else {
                                path.points.push(prev_point + 0.5 * width * offset * (prev_normal + next_normal));
                            }

                            prev_point = next_point;
                            prev_normal = next_normal;
                        }

                        if point.is_none() { break; }
                    }

                    if path.points.len() > base {
                        if closed {
                            path.commands.push(PathCommand::Move);
                        } else {
                            path.commands.push(PathCommand::Line);
                        }
                        for _ in (base + 1)..path.points.len() {
                            path.commands.push(PathCommand::Line);
                        }
                        path.commands.push(PathCommand::Close);
                    }
                }
            }

            if let Some(command) = command {
                match command {
                    PathCommand::Move => {
                        contour_start = contour_end;
                        contour_end += 1;
                    }
                    PathCommand::Line => {
                        contour_end += 1;
                    }
                    PathCommand::Close => {
                        closed = true;
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }

        path
    }
}
