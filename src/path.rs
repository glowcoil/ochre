use crate::geom::*;

const TOLERANCE: f32 = 0.1;

#[derive(Clone)]
pub struct Path {
    pub(crate) commands: Vec<PathCommand>,
    pub(crate) data: Vec<f32>,
}

#[derive(Copy, Clone)]
pub(crate) enum PathCommand {
    Move,
    Line,
    Quadratic,
    Cubic,
    Conic,
    Close,
}

impl Path {
    pub fn new() -> Path {
        Path {
            commands: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Path {
        let mut path = Path::new();
        path.move_to(x, y);
        path.line_to(x, y + height);
        path.line_to(x + width, y + height);
        path.line_to(x + width, y);
        path.line_to(x, y);
        path.close();
        path
    }

    pub fn round_rect(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Path {
        let radius = radius.min(0.5 * width).min(0.5 * height);
        let weight = 0.5 * 2.0f32.sqrt();

        let mut path = Path::new();
        path.move_to(x + radius, y);
        path.conic_to(x, y, x, y + radius, weight);
        path.line_to(x, y + height - radius);
        path.conic_to(x, y + height, x + radius, y + height, weight);
        path.line_to(x + width - radius, y + height);
        path.conic_to(x + width, y + height, x + width, y + height - radius, weight);
        path.line_to(x + width, y + radius);
        path.conic_to(x + width, y, x + width - radius, y, weight);
        path.line_to(x + radius, y);
        path.close();
        path
    }

    pub fn move_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.commands.push(PathCommand::Move);
        self.data.extend_from_slice(&[x, y]);
        self
    }

    pub fn line_to(&mut self, x: f32, y: f32) -> &mut Self {
        self.commands.push(PathCommand::Line);
        self.data.extend_from_slice(&[x, y]);
        self
    }

    pub fn quadratic_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) -> &mut Self {
        self.commands.push(PathCommand::Quadratic);
        self.data.extend_from_slice(&[x1, y1, x, y]);
        self
    }

    pub fn cubic_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) -> &mut Self {
        self.commands.push(PathCommand::Cubic);
        self.data.extend_from_slice(&[x1, y1, x2, y2, x, y]);
        self
    }

    pub fn conic_to(&mut self, x1: f32, y1: f32, x: f32, y: f32, weight: f32) -> &mut Self {
        self.commands.push(PathCommand::Conic);
        self.data.extend_from_slice(&[x1, y1, x, y, weight]);
        self
    }

    pub fn close(&mut self) -> &mut Self {
        self.commands.push(PathCommand::Close);
        self
    }

    pub fn flatten(&self, transform: Transform) -> Path {
        let mut path = Path::new();
        let mut i = 0;
        let mut last = Vec2::new(0.0, 0.0);
        for command in self.commands.iter() {
            match command {
                PathCommand::Move => {
                    let point = transform.apply(Vec2::new(self.data[i], self.data[i + 1]));
                    path.move_to(point.x, point.y);
                    last = point;
                    i += 2;
                }
                PathCommand::Line => {
                    let point = transform.apply(Vec2::new(self.data[i], self.data[i + 1]));
                    path.line_to(point.x, point.y);
                    last = point;
                    i += 2;
                }
                PathCommand::Quadratic => {
                    let control = transform.apply(Vec2::new(self.data[i], self.data[i + 1]));
                    let point = transform.apply(Vec2::new(self.data[i + 2], self.data[i + 3]));
                    let dt = ((4.0 * TOLERANCE) / (last - 2.0 * control + point).length()).sqrt();
                    let mut t = 0.0;
                    while t < 1.0 {
                        t = (t + dt).min(1.0);
                        let p01 = Vec2::lerp(t, last, control);
                        let p12 = Vec2::lerp(t, control, point);
                        let p = Vec2::lerp(t, p01, p12);
                        path.line_to(p.x, p.y);
                    }
                    last = point;
                    i += 4;
                }
                PathCommand::Cubic => {
                    let control1 = transform.apply(Vec2::new(self.data[i], self.data[i + 1]));
                    let control2 = transform.apply(Vec2::new(self.data[i + 2], self.data[i + 3]));
                    let point = transform.apply(Vec2::new(self.data[i + 4], self.data[i + 5]));
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
                        let p = Vec2::lerp(t, p012, p123);
                        path.line_to(p.x, p.y);
                    }
                    last = point;
                    i += 6;
                }
                PathCommand::Conic => {
                    let control = transform.apply(Vec2::new(self.data[i], self.data[i + 1]));
                    let point = transform.apply(Vec2::new(self.data[i + 2], self.data[i + 3]));
                    let weight = self.data[i + 4];

                    fn flatten_conic(
                        last: Vec2,
                        control: Vec2,
                        point: Vec2,
                        weight: f32,
                        t0: f32,
                        t1: f32,
                        p0: Vec2,
                        p1: Vec2,
                        path: &mut Path,
                    ) {
                        let t = 0.5 * (t0 + t1);
                        let p01 = Vec2::lerp(t, last, weight * control);
                        let p12 = Vec2::lerp(t, weight * control, point);
                        let denom = (1.0 - t) * (1.0 - t) + 2.0 * t * (1.0 - t) * weight + t * t;
                        let midpoint = (1.0 / denom) * Vec2::lerp(t, p01, p12);
                        let err = (midpoint - 0.5 * (p0 + p1)).length();
                        if err > TOLERANCE {
                            flatten_conic(last, control, point, weight, t0, t, p0, midpoint, path);
                            flatten_conic(last, control, point, weight, t, t1, midpoint, p1, path);
                        } else {
                            path.line_to(midpoint.x, midpoint.y);
                            path.line_to(p1.x, p1.y);
                        }
                    };
                    flatten_conic(last, control, point, weight, 0.0, 1.0, last, point, &mut path);

                    last = point;
                    i += 5;
                }
                PathCommand::Close => {
                    path.close();
                }
            }
        }

        path
    }

    pub fn stroke(&self, width: f32) -> Path {
        let mut path = Path::new();

        let flattened = self.flatten(Transform::id());

        let mut contour_start = 0;
        let mut contour_end = 0;
        let mut closed = false;
        let mut commands = flattened.commands.iter();
        loop {
            let command = commands.next();

            if let None | Some(PathCommand::Move) = command {
                if contour_start != contour_end {
                    let contour = &flattened.data[contour_start..contour_end];

                    let base = path.data.len();

                    let first_point = if closed {
                        Vec2::new(contour[contour.len() - 2], contour[contour.len() - 1])
                    } else {
                        Vec2::new(contour[0], contour[1])
                    };
                    let mut prev_point = first_point;
                    let mut prev_normal = Vec2::new(0.0, 0.0);
                    let mut i = 0;
                    loop {
                        let next_point = if i + 2 <= contour.len() {
                            Vec2::new(contour[i], contour[i + 1])
                        } else {
                            first_point
                        };

                        if next_point != prev_point {
                            let next_tangent = next_point - prev_point;
                            let next_normal = Vec2::new(-next_tangent.y, next_tangent.x).normalized();

                            let offset = 1.0 / (1.0 + prev_normal.dot(next_normal));
                            if offset.abs() > 2.0 {
                                let point1 = prev_point + 0.5 * width * prev_normal;
                                let point2 = prev_point + 0.5 * width * next_normal;
                                path.data.extend_from_slice(&[point1.x, point1.y, point2.x, point2.y]);
                            } else {
                                let point = prev_point + 0.5 * width * offset * (prev_normal + next_normal);
                                path.data.extend_from_slice(&[point.x, point.y]);
                            }

                            prev_point = next_point;
                            prev_normal = next_normal;
                        }

                        i += 2;
                        if i >= contour.len() {
                            break;
                        }
                    }

                    if path.data.len() > base {
                        path.commands.push(PathCommand::Move);
                        for _ in 0..((path.data.len() - base) / 2 - 1) {
                            path.commands.push(PathCommand::Line);
                        }
                        if closed {
                            path.commands.push(PathCommand::Close);
                        }
                    }

                    let base = path.data.len();

                    let first_point = if closed {
                        Vec2::new(contour[0], contour[1])
                    } else {
                        Vec2::new(contour[contour.len() - 2], contour[contour.len() - 1])
                    };
                    let mut prev_point = first_point;
                    let mut prev_normal = Vec2::new(0.0, 0.0);
                    let mut i = 0;
                    loop {
                        let next_point = if i + 2 <= contour.len() {
                            Vec2::new(contour[contour.len() - i - 2], contour[contour.len() - i - 1])
                        } else {
                            first_point
                        };

                        if next_point != prev_point {
                            let next_tangent = next_point - prev_point;
                            let next_normal = Vec2::new(-next_tangent.y, next_tangent.x).normalized();

                            let offset = 1.0 / (1.0 + prev_normal.dot(next_normal));
                            if offset.abs() > 2.0 {
                                let point1 = prev_point + 0.5 * width * prev_normal;
                                let point2 = prev_point + 0.5 * width * next_normal;
                                path.data.extend_from_slice(&[point1.x, point1.y, point2.x, point2.y]);
                            } else {
                                let point = prev_point + 0.5 * width * offset * (prev_normal + next_normal);
                                path.data.extend_from_slice(&[point.x, point.y]);
                            }

                            prev_point = next_point;
                            prev_normal = next_normal;
                        }

                        i += 2;
                        if i >= contour.len() {
                            break;
                        }
                    }

                    if path.data.len() > base {
                        if closed {
                            path.commands.push(PathCommand::Move);
                        } else {
                            path.commands.push(PathCommand::Line);
                        }
                        for _ in 0..((path.data.len() - base) / 2 - 1) {
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
                        contour_end += 2;
                    }
                    PathCommand::Line => {
                        contour_end += 2;
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
