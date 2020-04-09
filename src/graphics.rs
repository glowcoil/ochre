use crate::geom::*;
use crate::path::*;
use crate::render::*;

pub struct Graphics {
    renderer: Renderer,
    width: f32,
    height: f32,
    color: Color,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    atlas_texture: TextureId,
    tex: TextureId,
}

impl Graphics {
    pub fn new(width: f32, height: f32) -> Graphics {
        let mut renderer = Renderer::new(width as u32, height as u32);
        let atlas_texture = renderer.create_texture(TextureFormat::A, 400, 300, &vec![0; 400 * 300]);
        let tex = renderer.create_texture(TextureFormat::RGBA, 1024, 1024, &vec![0; 1024 * 1024 * 4]);
        Graphics {
            renderer,
            width,
            height,
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            vertices: Vec::new(),
            indices: Vec::new(),
            atlas_texture,
            tex,
        }
    }

    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;

        self.renderer.resize(width as u32, height as u32);
    }

    pub fn clear(&mut self, color: Color) {
        self.renderer.clear(color.to_linear_premul(), &RenderOptions::default());
    }

    pub fn begin_frame(&mut self) {
        self.vertices = Vec::new();
        self.indices = Vec::new();
    }

    pub fn end_frame(&mut self) {
        self.renderer.draw(&self.vertices, &self.indices, &RenderOptions::default());
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

    pub fn draw_texture_test(&mut self) {
        let tex = self.renderer.create_texture(TextureFormat::A, 4, 4, &[
            127, 0, 127, 0,
            0, 127, 0, 127,
            127, 0, 127, 0,
            0, 127, 0, 127,
        ]);
        self.renderer.draw_textured(&[
            TexturedVertex { pos: [0.0, 0.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 0.0] },
            TexturedVertex { pos: [1.0, 0.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 0.0] },
            TexturedVertex { pos: [1.0, 1.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 1.0] },
            TexturedVertex { pos: [0.0, 1.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 1.0] },
        ], &[0, 1, 2, 0, 2, 3], tex, &RenderOptions { target: Some(self.tex), ..RenderOptions::default() });
        self.renderer.draw_textured(&[
            TexturedVertex { pos: [0.0, 0.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 0.0] },
            TexturedVertex { pos: [1.0, 0.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 0.0] },
            TexturedVertex { pos: [1.0, 1.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 1.0] },
            TexturedVertex { pos: [0.0, 1.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 1.0] },
        ], &[0, 1, 2, 0, 2, 3], self.tex, &RenderOptions::default());
    }

    pub fn draw_trapezoids_test(&mut self) {
        self.renderer.clear([0.0, 0.0, 0.0, 0.0], &RenderOptions { target: Some(self.atlas_texture) });
        self.renderer.draw_trapezoids(&[
            TrapezoidVertex { pos: [-1.0, -1.0], from: [150.5, 100.5], to: [100.5, 130.5] },
            TrapezoidVertex { pos: [1.0, -1.0], from: [150.5, 100.5], to: [100.5, 130.5] },
            TrapezoidVertex { pos: [1.0, 1.0], from: [150.5, 100.5], to: [100.5, 130.5] },
            TrapezoidVertex { pos: [-1.0, 1.0], from: [150.5, 100.5], to: [100.5, 130.5] },
            TrapezoidVertex { pos: [-1.0, -1.0], from: [100.5, 130.5], to: [50.5, 100.5] },
            TrapezoidVertex { pos: [1.0, -1.0], from: [100.5, 130.5], to: [50.5, 100.5] },
            TrapezoidVertex { pos: [1.0, 1.0], from: [100.5, 130.5], to: [50.5, 100.5] },
            TrapezoidVertex { pos: [-1.0, 1.0], from: [100.5, 130.5], to: [50.5, 100.5] },
            TrapezoidVertex { pos: [-1.0, -1.0], from: [50.5, 100.5], to: [100.5, 50.5] },
            TrapezoidVertex { pos: [1.0, -1.0], from: [50.5, 100.5], to: [100.5, 50.5] },
            TrapezoidVertex { pos: [1.0, 1.0], from: [50.5, 100.5], to: [100.5, 50.5] },
            TrapezoidVertex { pos: [-1.0, 1.0], from: [50.5, 100.5], to: [100.5, 50.5] },
            TrapezoidVertex { pos: [-1.0, -1.0], from: [100.5, 50.5], to: [150.5, 100.5] },
            TrapezoidVertex { pos: [1.0, -1.0], from: [100.5, 50.5], to: [150.5, 100.5] },
            TrapezoidVertex { pos: [1.0, 1.0], from: [100.5, 50.5], to: [150.5, 100.5] },
            TrapezoidVertex { pos: [-1.0, 1.0], from: [100.5, 50.5], to: [150.5, 100.5] },
        ], &[0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7, 8, 9, 10, 8, 10, 11, 12, 13, 14, 12, 14, 15], &RenderOptions { target: Some(self.atlas_texture) });
        self.renderer.draw_textured(&[
            TexturedVertex { pos: [-1.0, -1.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 0.0] },
            TexturedVertex { pos: [0.0, -1.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 0.0] },
            TexturedVertex { pos: [0.0, 0.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [1.0, 1.0] },
            TexturedVertex { pos: [-1.0, 0.0, 0.0], col: [1.0, 1.0, 1.0, 1.0], uv: [0.0, 1.0] },
        ], &[0, 1, 2, 0, 2, 3], self.atlas_texture, &RenderOptions::default());
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
