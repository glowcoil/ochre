use crate::render::{Renderer, Vertex};

const TOLERANCE: f32 = 0.1;

pub struct Graphics {
    renderer: Renderer,
    width: f32,
    height: f32,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl Graphics {
    pub fn new(width: f32, height: f32) -> Graphics {
        Graphics {
            renderer: Renderer::new(),
            width,
            height,
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn clear(&mut self, color: Color) {
        self.renderer.clear(color.to_linear());
    }

    pub fn begin_frame(&mut self) {
        self.vertices = Vec::new();
        self.indices = Vec::new();
    }

    pub fn end_frame(&mut self) {
        self.renderer.draw(&self.vertices, &self.indices);
    }

    pub fn path(&mut self) -> PathBuilder {
        PathBuilder::new(self)
    }

    pub fn fill_rect(&mut self, pos: Point, size: Point) {
        self.path()
            .move_to(pos)
            .line_to(Point::new(pos.x, pos.y + size.y))
            .line_to(Point::new(pos.x + size.x, pos.y + size.y))
            .line_to(Point::new(pos.x + size.x, pos.y))
            .fill_convex();
    }
}

pub struct PathBuilder<'g> {
    graphics: &'g mut Graphics,
    points: Vec<Point>,
    components: Vec<usize>,
    cursor: Point,
}

impl<'g> PathBuilder<'g> {
    fn new(graphics: &'g mut Graphics) -> PathBuilder<'g> {
        PathBuilder { graphics, points: Vec::new(), components: vec![0], cursor: Point::new(0.0, 0.0) }
    }

    pub fn move_to(&mut self, point: Point) -> &mut Self {
        if *self.components.last().unwrap() != self.points.len() {
            self.points.push(self.cursor);
            self.components.push(self.points.len());
        }
        self.cursor = point;
        self
    }

    pub fn line_to(&mut self, point: Point) -> &mut Self {
        self.points.push(self.cursor);
        self.cursor = point;
        self
    }

    pub fn fill_convex(&mut self) {
        self.points.push(self.cursor);
        let start = self.graphics.vertices.len() as u16;
        for point in self.points.iter() {
            let ndc = point.pixel_to_ndc(self.graphics.width, self.graphics.height);
            self.graphics.vertices.push(Vertex { pos: [ndc.x, ndc.y, 0.0], col: [1.0, 1.0, 1.0, 1.0] });
        }
        for i in (start+1)..(self.graphics.vertices.len() as u16 - 1) {
            self.graphics.indices.extend(&[start, i, i + 1]);
        }
    }
}

#[derive(Copy, Clone)]
pub struct Color {
    pub r: f32, pub g: f32, pub b: f32, pub a: f32,
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    fn to_linear(&self) -> [f32; 4] {
        [srgb_to_linear(self.r), srgb_to_linear(self.g), srgb_to_linear(self.b), self.a]
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
