use crate::render::{Renderer, Vertex};

const TOLERANCE: f32 = 0.1;

pub struct Graphics {
    renderer: Renderer,
    width: f32,
    height: f32,
    color: Color,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Graphics {
    pub fn new(width: f32, height: f32) -> Graphics {
        Graphics {
            renderer: Renderer::new(),
            width,
            height,
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn clear(&mut self, color: Color) {
        self.renderer.clear(color.to_linear_premul());
    }

    pub fn begin_frame(&mut self) {
        self.vertices = Vec::new();
        self.indices = Vec::new();
    }

    pub fn end_frame(&mut self) {
        self.renderer.draw(&self.vertices, &self.indices);
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn draw_mesh(&mut self, mesh: &Mesh) {
        let base = self.vertices.len() as u16;
        for point in &mesh.vertices[0..mesh.fringe_vertices] {
            let ndc = point.pixel_to_ndc(self.width, self.height);
            self.vertices.push(Vertex { pos: [ndc.x, ndc.y, 0.0], col: self.color.to_linear_premul() });
        }
        for point in &mesh.vertices[mesh.fringe_vertices..] {
            let ndc = point.pixel_to_ndc(self.width, self.height);
            self.vertices.push(Vertex { pos: [ndc.x, ndc.y, 0.0], col: [0.0, 0.0, 0.0, 0.0] });
        }
        for index in &mesh.indices {
            self.indices.push(base + index);
        }
    }
}

pub struct Path {
    points: Vec<Point>,
    components: Vec<usize>,
}

impl Path {
    pub fn new() -> Path {
        Path {
            points: vec![Point::new(0.0, 0.0)],
            components: vec![0],
        }
    }

    pub fn move_to(&mut self, point: Point) -> &mut Self {
        if *self.components.last().unwrap() == self.points.len() - 1 {
            *self.points.last_mut().unwrap() = point;
        } else {
            self.components.push(self.points.len());
            self.points.push(point);
        }
        self
    }

    pub fn line_to(&mut self, point: Point) -> &mut Self {
        self.points.push(point);
        self
    }

    pub fn quadratic_to(&mut self, control: Point, point: Point) -> &mut Self {
        let current = *self.points.last().unwrap();
        let a_x = current.x - 2.0 * control.x + point.x;
        let a_y = current.y - 2.0 * control.y + point.y;
        let dt = 10.0 * ((4.0 * TOLERANCE) / (a_x * a_x + a_y * a_y)).sqrt();
        let mut t = dt;
        while t < 1.0 {
            let p12 = Point::lerp(t, current, control);
            let p23 = Point::lerp(t, control, point);
            self.points.push(Point::lerp(t, p12, p23));
            t += dt;
        }
        self
    }

    pub fn cubic_to(&mut self, control1: Point, control2: Point, point: Point) -> &mut Self {
        let current = *self.points.last().unwrap();
        let a_x = -current.x + 3.0 * control1.x - 3.0 * control2.x + point.x;
        let b_x = 3.0 * (current.x - 2.0 * control1.x + control2.x);
        let a_y = -current.y + 3.0 * control1.y - 3.0 * control2.y + point.y;
        let b_y = 3.0 * (current.y - 2.0 * control1.y + control2.y);
        let conc = (b_x * b_x + b_y * b_y).max((a_x + b_x) * (a_x + b_x) + (a_y + b_y) * (a_y + b_y));
        let dt = 10.0 * ((4.0 * TOLERANCE) / conc).sqrt();
        let mut t = dt;
        while t < 1.0 {
            let p12 = Point::lerp(t, current, control1);
            let p23 = Point::lerp(t, control1, control2);
            let p34 = Point::lerp(t, control2, point);
            let p123 = Point::lerp(t, p12, p23);
            let p234 = Point::lerp(t, p23, p34);
            self.points.push(Point::lerp(t, p123, p234));
            t += dt;
        }
        self
    }

    pub fn arc_to(&mut self, radius: f32, point: Point) -> &mut Self {
        let current = *self.points.last().unwrap();
        let winding = radius.signum();
        let to_midpoint = 0.5 * (point - current);
        let dist_to_midpoint = to_midpoint.length();
        let radius = radius.abs().max(to_midpoint.length());
        let dist_to_center = (radius * radius - dist_to_midpoint * dist_to_midpoint).sqrt();
        let to_center = winding * dist_to_center * if to_midpoint.length() == 0.0 {
            Point::new(-1.0, 0.0)
        } else {
            Point::new(to_midpoint.y, -to_midpoint.x).normalized()
        };
        let center = current + to_midpoint + to_center;
        let mut angle = current - center;
        let end_angle = point - center;
        let rotor_x = (1.0 - 2.0 * (TOLERANCE / radius)).max(0.0);
        let rotor_y = -winding * (1.0 - rotor_x * rotor_x).sqrt();
        loop {
            let prev_sign = winding * (angle.x * end_angle.y - angle.y * end_angle.x);
            angle = Point::new(rotor_x * angle.x - rotor_y * angle.y, rotor_x * angle.y + rotor_y * angle.x);
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
            let normal = Point::new(-tangent.y, tangent.x).normalized();
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

    pub fn rect(pos: Point, size: Point) -> Path {
        let mut path = Path::new();
        path.move_to(pos)
            .line_to(Point::new(pos.x, pos.y + size.y))
            .line_to(Point::new(pos.x + size.x, pos.y + size.y))
            .line_to(Point::new(pos.x + size.x, pos.y));
        path
    }

    pub fn rect_fill(pos: Point, size: Point) -> Mesh {
        Path::rect(pos, size).fill_convex()
    }

    pub fn round_rect(pos: Point, size: Point, radius: f32) -> Path {
        let radius = radius.min(0.5 * size.x).min(0.5 * size.y);
        let mut path = Path::new();
        path.move_to(Point::new(pos.x, pos.y + radius))
            .line_to(Point::new(pos.x, pos.y + size.y - radius))
            .arc_to(radius, Point::new(pos.x + radius, pos.y + size.y))
            .line_to(Point::new(pos.x + size.x - radius, pos.y + size.y))
            .arc_to(radius, Point::new(pos.x + size.x, pos.y + size.y - radius))
            .line_to(Point::new(pos.x + size.x, pos.y + radius))
            .arc_to(radius, Point::new(pos.x + size.x - radius, pos.y))
            .line_to(Point::new(pos.x + radius, pos.y))
            .arc_to(radius, Point::new(pos.x, pos.y + radius));
        path
    }

    pub fn round_rect_fill(pos: Point, size: Point, radius: f32) -> Mesh {
        Path::round_rect(pos, size, radius).fill_convex()
    }
}

pub struct Mesh {
    vertices: Vec<Point>,
    indices: Vec<u16>,
    fringe_vertices: usize,
    fringe_indices: usize,
}

#[derive(Copy, Clone)]
pub struct Color {
    pub r: f32, pub g: f32, pub b: f32, pub a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    fn to_linear_premul(&self) -> [f32; 4] {
        [
            self.a * srgb_to_linear(self.r),
            self.a * srgb_to_linear(self.g),
            self.a * srgb_to_linear(self.b),
            self.a
        ]
    }
}

fn srgb_to_linear(x: f32) -> f32 {
    if x < 0.04045 { x / 12.92 } else { ((x + 0.055)/1.055).powf(2.4)  }
}

use std::ops;

#[derive(Copy, Clone, Debug)]
pub struct Point { pub x: f32, pub y: f32 }

impl Point {
    #[inline]
    pub fn new(x: f32, y: f32) -> Point {
        Point { x: x, y: y }
    }

    #[inline]
    pub fn distance(self, other: Point) -> f32 {
        ((other.x - self.x) * (other.x - self.x) + (other.y - self.y) * (other.y - self.y)).sqrt()
    }

    #[inline]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline]
    pub fn normalized(self) -> Point {
        let len = self.length();
        Point { x: self.x / len, y: self.y / len }
    }

    #[inline]
    pub fn dot(self, other: Point) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn lerp(t: f32, a: Point, b: Point) -> Point {
        (1.0 - t) * a + t * b
    }

    #[inline]
    fn pixel_to_ndc(self, screen_width: f32, screen_height: f32) -> Point {
        Point {
            x: 2.0 * (self.x / screen_width as f32 - 0.5),
            y: 2.0 * (1.0 - self.y / screen_height as f32 - 0.5),
        }
    }
}

impl ops::Add for Point {
    type Output = Point;
    #[inline]
    fn add(self, rhs: Point) -> Point {
        Point { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl ops::AddAssign for Point {
    #[inline]
    fn add_assign(&mut self, other: Point) {
        *self = *self + other;
    }
}

impl ops::Sub for Point {
    type Output = Point;
    #[inline]
    fn sub(self, rhs: Point) -> Point {
        Point { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl ops::SubAssign for Point {
    #[inline]
    fn sub_assign(&mut self, other: Point) {
        *self = *self - other;
    }
}

impl ops::Mul<f32> for Point {
    type Output = Point;
    #[inline]
    fn mul(self, rhs: f32) -> Point {
        Point { x: self.x * rhs, y: self.y * rhs }
    }
}

impl ops::Mul<Point> for f32 {
    type Output = Point;
    #[inline]
    fn mul(self, rhs: Point) -> Point {
        Point { x: self * rhs.x, y: self * rhs.y }
    }
}

impl ops::MulAssign<f32> for Point {
    #[inline]
    fn mul_assign(&mut self, other: f32) {
        *self = *self * other;
    }
}
