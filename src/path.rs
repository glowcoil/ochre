use crate::geom::*;

/// A single command in a 2-dimensional vector path.
#[derive(Copy, Clone)]
pub enum PathCmd {
    Move(Vec2),
    Line(Vec2),
    Quadratic(Vec2, Vec2),
    Cubic(Vec2, Vec2, Vec2),
    Conic(Vec2, Vec2, f32),
    Close,
}

impl PathCmd {
    /// Applies the given transform to the path command.
    pub fn transform(&self, transform: Transform) -> PathCmd {
        match *self {
            PathCmd::Move(point) => {
                PathCmd::Move(transform.apply(point))
            }
            PathCmd::Line(point) => {
                PathCmd::Line(transform.apply(point))
            }
            PathCmd::Quadratic(control, point) => {
                PathCmd::Quadratic(transform.apply(control), transform.apply(point))
            }
            PathCmd::Cubic(control1, control2, point) => {
                PathCmd::Cubic(transform.apply(control1), transform.apply(control2), transform.apply(point))
            }
            PathCmd::Conic(control, point, weight) => {
                PathCmd::Conic(transform.apply(control), transform.apply(point), weight)
            }
            PathCmd::Close => {
                PathCmd::Close
            }
        }
    }

    /// Computes a piecewise-linear approximation of the given path command to
    /// within the supplied parametric error tolerance.
    pub fn flatten(&self, last: Vec2, tolerance: f32, mut callback: impl FnMut(PathCmd)) {
        match *self {
            PathCmd::Move(point) => {
                (callback)(PathCmd::Move(point));
            }
            PathCmd::Line(point) => {
                (callback)(PathCmd::Line(point));
            }
            PathCmd::Quadratic(control, point) => {
                let dt = ((4.0 * tolerance) / (last - 2.0 * control + point).length()).sqrt();
                let mut t = 0.0;
                while t < 1.0 {
                    t = (t + dt).min(1.0);
                    let p01 = Vec2::lerp(t, last, control);
                    let p12 = Vec2::lerp(t, control, point);
                    (callback)(PathCmd::Line(Vec2::lerp(t, p01, p12)));
                }
            }
            PathCmd::Cubic(control1, control2, point) => {
                let a = -1.0 * last + 3.0 * control1 - 3.0 * control2 + point;
                let b = 3.0 * (last - 2.0 * control1 + control2);
                let conc = b.length().max((a + b).length());
                let dt = ((8.0f32.sqrt() * tolerance) / conc).sqrt();
                let mut t = 0.0;
                while t < 1.0 {
                    t = (t + dt).min(1.0);
                    let p01 = Vec2::lerp(t, last, control1);
                    let p12 = Vec2::lerp(t, control1, control2);
                    let p23 = Vec2::lerp(t, control2, point);
                    let p012 = Vec2::lerp(t, p01, p12);
                    let p123 = Vec2::lerp(t, p12, p23);
                    (callback)(PathCmd::Line(Vec2::lerp(t, p012, p123)));
                }
            }
            PathCmd::Conic(control, point, weight) => {
                fn flatten_conic(
                    last: Vec2,
                    control: Vec2,
                    point: Vec2,
                    weight: f32,
                    t0: f32,
                    t1: f32,
                    p0: Vec2,
                    p1: Vec2,
                    tolerance: f32,
                    callback: &mut impl FnMut(PathCmd),
                ) {
                    let t = 0.5 * (t0 + t1);
                    let p01 = Vec2::lerp(t, last, weight * control);
                    let p12 = Vec2::lerp(t, weight * control, point);
                    let denom = (1.0 - t) * (1.0 - t) + 2.0 * t * (1.0 - t) * weight + t * t;
                    let midpoint = (1.0 / denom) * Vec2::lerp(t, p01, p12);
                    let err = (midpoint - 0.5 * (p0 + p1)).length();
                    if err > tolerance {
                        flatten_conic(last, control, point, weight, t0, t, p0, midpoint, tolerance, callback);
                        flatten_conic(last, control, point, weight, t, t1, midpoint, p1, tolerance, callback);
                    } else {
                        (callback)(PathCmd::Line(midpoint));
                        (callback)(PathCmd::Line(p1));
                    }
                }

                flatten_conic(last, control, point, weight, 0.0, 1.0, last, point, tolerance, &mut callback);
            }
            PathCmd::Close => {
                (callback)(PathCmd::Close);
            }
        }
    }
}

/// Computes a piecewise-linear approximation of the given path to within the
/// supplied parametric error tolerance.
pub fn flatten(path: &[PathCmd], tolerance: f32) -> Vec<PathCmd> {
    let mut last = Vec2::new(0.0, 0.0);
    let mut output = Vec::new();

    for command in path {
        command.flatten(last, tolerance, |cmd| {
            output.push(cmd);
        });

        match *command {
            PathCmd::Move(point) => {
                last = point;
            }
            PathCmd::Line(point) => {
                last = point;
            }
            PathCmd::Quadratic(_, point) => {
                last = point;
            }
            PathCmd::Cubic(_, _, point) => {
                last = point;
            }
            PathCmd::Conic(_, point, _) => {
                last = point;
            }
            PathCmd::Close => {}
        }
    }

    output
}

/// Converts the given path to a stroked path with the given width.
///
/// This function will panic if the given path is not piecewise-linear (i.e. if
/// it contains [`PathCmd`]s other than `Move`, `Line`, or `Close`.
///
/// The line-cap style is "butt" and the line-join style is "miter."
pub fn stroke(polygon: &[PathCmd], width: f32) -> Vec<PathCmd> {
    #[inline]
    fn get_point(command: PathCmd) -> Vec2 {
        match command {
            PathCmd::Move(point) => point,
            PathCmd::Line(point) => point,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn join(path: &mut Vec<PathCmd>, width: f32, prev_normal: Vec2, next_normal: Vec2, point: Vec2) {
        let offset = 1.0 / (1.0 + prev_normal.dot(next_normal));
        if offset.abs() > 2.0 {
            path.push(PathCmd::Line(point + 0.5 * width * prev_normal));
            path.push(PathCmd::Line(point + 0.5 * width * next_normal));
        } else {
            path.push(PathCmd::Line(point + 0.5 * width * offset * (prev_normal + next_normal)));
        }
    }

    #[inline]
    fn offset(path: &mut Vec<PathCmd>, width: f32, contour: &[PathCmd], closed: bool, reverse: bool) {
        let first_point = if closed == reverse {
            get_point(contour[0])
        } else {
            get_point(*contour.last().unwrap())
        };
        let mut prev_point = first_point;
        let mut prev_normal = Vec2::new(0.0, 0.0);
        let mut i = 0;
        loop {
            let next_point = if i < contour.len() {
                if reverse {
                    get_point(contour[contour.len() - i - 1])
                } else {
                    get_point(contour[i])
                }
            } else {
                first_point
            };

            if next_point != prev_point || i == contour.len() {
                let next_tangent = next_point - prev_point;
                let next_normal = Vec2::new(-next_tangent.y, next_tangent.x);
                let next_normal_len = next_normal.length();
                let next_normal = if next_normal_len == 0.0 {
                    Vec2::new(0.0, 0.0)
                } else {
                    next_normal * (1.0 / next_normal_len)
                };

                join(path, width, prev_normal, next_normal, prev_point);

                prev_point = next_point;
                prev_normal = next_normal;
            }

            i += 1;
            if i > contour.len() {
                break;
            }
        }
    }

    let mut output = Vec::new();

    let mut contour_start = 0;
    let mut contour_end = 0;
    let mut closed = false;
    let mut commands = polygon.iter();
    loop {
        let command = commands.next();

        if let Some(PathCmd::Close) = command {
            closed = true;
        }

        if let None | Some(PathCmd::Move(_)) | Some(PathCmd::Close) = command {
            if contour_start != contour_end {
                let contour = &polygon[contour_start..contour_end];

                let base = output.len();
                offset(&mut output, width, contour, closed, false);
                output[base] = PathCmd::Move(get_point(output[base]));
                if closed {
                    output.push(PathCmd::Close);
                }

                let base = output.len();
                offset(&mut output, width, contour, closed, true);
                if closed {
                    output[base] = PathCmd::Move(get_point(output[base]));
                }
                output.push(PathCmd::Close);
            }
        }

        if let Some(command) = command {
            match command {
                PathCmd::Move(_) => {
                    contour_start = contour_end;
                    contour_end = contour_start + 1;
                }
                PathCmd::Line(_) => {
                    contour_end += 1;
                }
                PathCmd::Close => {
                    contour_start = contour_end + 1;
                    contour_end = contour_start;
                    closed = true;
                }
                _ => {
                    panic!();
                }
            }
        } else {
            break;
        }
    }

    output
}
