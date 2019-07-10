use std::collections::HashMap;
use std::ffi::{CStr, CString};
use gl::types::{GLuint, GLint, GLchar, GLenum, GLvoid};

macro_rules! offset {
    ($type:ty, $field:ident) => { &(*(0 as *const $type)).$field as *const _ as usize }
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub col: [f32; 4],
}

#[derive(Copy, Clone, Debug)]
pub struct TexturedVertex {
    pub pos: [f32; 3],
    pub col: [f32; 4],
    pub uv: [f32; 2],
}

#[derive(Copy, Clone, Debug)]
pub struct TrapezoidVertex {
    pub pos: [f32; 2],
    pub from: [f32; 2],
    pub to: [f32; 2],
}

pub enum TextureFormat { RGBA, A }
pub type TextureId = usize;

pub struct RenderOptions {
    pub target: Option<TextureId>,
}

impl Default for RenderOptions {
    fn default() -> RenderOptions {
        RenderOptions { target: None }
    }
}

pub struct Renderer {
    width: u32,
    height: u32,

    prog: Program,
    prog_tex_rgba: Program,
    prog_tex_a: Program,
    prog_trapezoids: Program,

    textures: HashMap<TextureId, Texture>,
    texture_id: TextureId,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Renderer {
        let prog = Program::new(
            &CStr::from_bytes_with_nul(VERT).unwrap(),
            &CStr::from_bytes_with_nul(FRAG).unwrap()).unwrap();
        let prog_tex_rgba = Program::new(
            &CStr::from_bytes_with_nul(VERT_TEX_RGBA).unwrap(),
            &CStr::from_bytes_with_nul(FRAG_TEX_RGBA).unwrap()).unwrap();
        let prog_tex_a = Program::new(
            &CStr::from_bytes_with_nul(VERT_TEX_A).unwrap(),
            &CStr::from_bytes_with_nul(FRAG_TEX_A).unwrap()).unwrap();
        let prog_trapezoids = Program::new(
            &CStr::from_bytes_with_nul(VERT_TRAPEZOIDS).unwrap(),
            &CStr::from_bytes_with_nul(FRAG_TRAPEZOIDS).unwrap()).unwrap();

        unsafe {
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
        }

        Renderer {
            width,
            height,

            prog,
            prog_tex_rgba,
            prog_tex_a,
            prog_trapezoids,

            textures: HashMap::new(),
            texture_id: 0,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn clear(&mut self, col: [f32; 4], options: &RenderOptions) {
        self.apply_options(options);
        unsafe {
            gl::ClearColor(col[0], col[1], col[2], col[3]);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        self.unapply_options(options);
    }

    pub fn draw(&mut self, vertices: &[Vertex], indices: &[u16], options: &RenderOptions) {
        self.apply_options(options);
        let vertex_array = VertexArray::new(vertices, indices);
        unsafe {
            gl::UseProgram(self.prog.id);
            gl::DrawElements(gl::TRIANGLES, vertex_array.count, gl::UNSIGNED_SHORT, 0 as *const GLvoid);
        }
        self.unapply_options(options);
    }

    pub fn draw_textured(&mut self, vertices: &[TexturedVertex], indices: &[u16], texture: TextureId, options: &RenderOptions) {
        self.apply_options(options);
        let texture = self.textures.get(&texture).unwrap();
        let vertex_array = VertexArray::new(vertices, indices);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
            match texture.format {
                TextureFormat::RGBA => { gl::UseProgram(self.prog_tex_rgba.id); }
                TextureFormat::A => { gl::UseProgram(self.prog_tex_a.id); }
            }
            gl::Uniform1i(0, 0);
            gl::DrawElements(gl::TRIANGLES, vertex_array.count, gl::UNSIGNED_SHORT, 0 as *const GLvoid);
        }
        self.unapply_options(options);
    }

    pub fn draw_trapezoids(&mut self, vertices: &[TrapezoidVertex], indices: &[u16], options: &RenderOptions) {
        self.apply_options(options);
        let vertex_array = VertexArray::new(vertices, indices);
        unsafe {
            gl::BlendFunc(gl::ONE, gl::ONE);
            gl::UseProgram(self.prog_trapezoids.id);
            gl::DrawElements(gl::TRIANGLES, vertex_array.count, gl::UNSIGNED_SHORT, 0 as *const GLvoid);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
        }
        self.unapply_options(options);
    }

    pub fn create_texture(&mut self, format: TextureFormat, width: u32, height: u32, pixels: &[u8]) -> TextureId {
        let id = self.texture_id;
        self.textures.insert(id, Texture::new(format, width, height, pixels));
        self.texture_id += 1;
        id
    }

    pub fn update_texture(&mut self, texture: TextureId, x: u32, y: u32, width: u32, height: u32, pixels: &[u8]) {
        self.textures.get_mut(&texture).unwrap().update(x, y, width, height, pixels);
    }

    pub fn delete_texture(&mut self, texture: TextureId) {
        self.textures.remove(&texture);
    }

    fn apply_options(&mut self, options: &RenderOptions) {
        if let Some(target) = options.target {
            let texture = self.textures.get_mut(&target).unwrap();
            if texture.framebuffer.is_none() {
                texture.framebuffer = Some(Framebuffer::new(texture.id));
            }
            unsafe {
                gl::Viewport(0, 0, texture.width as GLint, texture.height as GLint);
                gl::BindFramebuffer(gl::FRAMEBUFFER, texture.framebuffer.as_ref().unwrap().id);
            }
        }
    }

    fn unapply_options(&mut self, options: &RenderOptions) {
        if let Some(target) = options.target {
            unsafe {
                gl::Viewport(0, 0, self.width as GLint, self.height as GLint);
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            }
        }
    }
}

struct Program {
    id: GLuint,
}

impl Program {
    fn new(vert_src: &CStr, frag_src: &CStr) -> Result<Program, String> {
        unsafe {
            let vert = shader(vert_src, gl::VERTEX_SHADER).unwrap();
            let frag = shader(frag_src, gl::FRAGMENT_SHADER).unwrap();
            let prog = gl::CreateProgram();
            gl::AttachShader(prog, vert);
            gl::AttachShader(prog, frag);
            gl::LinkProgram(prog);

            let mut valid: GLint = 1;
            gl::GetProgramiv(prog, gl::COMPILE_STATUS, &mut valid);
            if valid == 0 {
                let mut len: GLint = 0;
                gl::GetProgramiv(prog, gl::INFO_LOG_LENGTH, &mut len);
                let error = CString::new(vec![b' '; len as usize]).unwrap();
                gl::GetProgramInfoLog(prog, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
                return Err(error.into_string().unwrap());
            }

            gl::DetachShader(prog, vert);
            gl::DetachShader(prog, frag);

            gl::DeleteShader(vert);
            gl::DeleteShader(frag);

            Ok(Program { id: prog })
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); }
    }
}

fn shader(shader_src: &CStr, shader_type: GLenum) -> Result<GLuint, String> {
    unsafe {
        let shader: GLuint = gl::CreateShader(shader_type);
        gl::ShaderSource(shader, 1, &shader_src.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);

        let mut valid: GLint = 1;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut valid);
        if valid == 0 {
            let mut len: GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let error = CString::new(vec![b' '; len as usize]).unwrap();
            gl::GetShaderInfoLog(shader, len, std::ptr::null_mut(), error.as_ptr() as *mut GLchar);
            return Err(error.into_string().unwrap());
        }

        Ok(shader)
    }
}

trait VertexAttribs {
    unsafe fn attribs();
}

impl VertexAttribs for Vertex {
    unsafe fn attribs() {
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, pos) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, col) as *const GLvoid);
    }
}

impl VertexAttribs for TexturedVertex {
    unsafe fn attribs() {
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<TexturedVertex>() as GLint, offset!(TexturedVertex, pos) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, std::mem::size_of::<TexturedVertex>() as GLint, offset!(TexturedVertex, col) as *const GLvoid);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<TexturedVertex>() as GLint, offset!(TexturedVertex, uv) as *const GLvoid);
    }
}

impl VertexAttribs for TrapezoidVertex {
    unsafe fn attribs() {
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<TrapezoidVertex>() as GLint, offset!(TrapezoidVertex, pos) as *const GLvoid);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<TrapezoidVertex>() as GLint, offset!(TrapezoidVertex, from) as *const GLvoid);
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, std::mem::size_of::<TrapezoidVertex>() as GLint, offset!(TrapezoidVertex, to) as *const GLvoid);
    }
}

struct VertexArray<V> {
    vao: GLuint,
    vbo: GLuint,
    ibo: GLuint,
    count: i32,
    phantom: std::marker::PhantomData<V>,
}

impl<V: VertexAttribs> VertexArray<V> {
    fn new(vertices: &[V], indices: &[u16]) -> VertexArray<V> {
        let mut vbo: u32 = 0;
        let mut ibo: u32 = 0;
        let mut vao: u32 = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<V>()) as isize, vertices.as_ptr() as *const std::ffi::c_void, gl::DYNAMIC_DRAW);

            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * std::mem::size_of::<u16>()) as isize, indices.as_ptr() as *const std::ffi::c_void, gl::DYNAMIC_DRAW);

            V::attribs();
        }
        VertexArray { vao, vbo, ibo, count: indices.len() as i32, phantom: std::marker::PhantomData }
    }
}

impl<V> Drop for VertexArray<V> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteBuffers(1, &self.ibo);
            gl::DeleteBuffers(1, &self.vbo);
        }
    }
}

struct Texture {
    id: GLuint,
    format: TextureFormat,
    width: u32,
    height: u32,
    framebuffer: Option<Framebuffer>,
}

impl Texture {
    fn new(format: TextureFormat, width: u32, height: u32, pixels: &[u8]) -> Texture {
        let flipped = flip(pixels, width);
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            match format {
                TextureFormat::RGBA => {
                    assert!(flipped.len() as u32 == width * height * 4);
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::SRGB8_ALPHA8 as GLint, width as i32, height as i32, 0, gl::RGBA, gl::UNSIGNED_INT_8_8_8_8, flipped.as_ptr() as *const std::ffi::c_void);
                }
                TextureFormat::A => {
                    assert!(flipped.len() as u32 == width * height);
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R16F as GLint, width as i32, height as i32, 0, gl::RED, gl::UNSIGNED_BYTE, flipped.as_ptr() as *const std::ffi::c_void);
                }
            }
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }
        Texture { id, format, width, height, framebuffer: None }
    }

    fn update(&mut self, x: u32, y: u32, width: u32, height: u32, pixels: &[u8]) {
        let flipped = flip(pixels, width);
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id); }
        match self.format {
            TextureFormat::RGBA => {
                if flipped.len() as u32 != width * height * 4 { panic!() }
                unsafe {
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                    gl::TexSubImage2D(gl::TEXTURE_2D, 0, x as i32, y as i32, width as i32, height as i32, gl::RGBA, gl::UNSIGNED_INT_8_8_8_8, flipped.as_ptr() as *const std::ffi::c_void);
                }
            }
            TextureFormat::A => {
                if flipped.len() as u32 != width * height { panic!() }
                unsafe {
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                    gl::TexSubImage2D(gl::TEXTURE_2D, 0, x as i32, y as i32, width as i32, height as i32, gl::RED, gl::UNSIGNED_BYTE, flipped.as_ptr() as *const std::ffi::c_void);
                }
            }
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { gl::DeleteProgram(self.id); }
    }
}

fn flip(pixels: &[u8], width: u32) -> Vec<u8> {
    let mut flipped: Vec<u8> = Vec::with_capacity(pixels.len());
    for chunk in pixels.rchunks(width as usize) {
        flipped.extend(chunk);
    }
    flipped
}

struct Framebuffer {
    id: GLuint,
}

impl Framebuffer {
    fn new(texture_id: GLuint) -> Framebuffer {
        let mut id: GLuint = 0;
        unsafe {
            gl::GenFramebuffers(1, &mut id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture_id, 0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
        Framebuffer { id }
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe { gl::DeleteFramebuffers(1, &self.id); }
    }
}

const VERT: &[u8] = b"
#version 330

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 col;

out vec4 v_col;

void main() {
    gl_Position = vec4(pos, 1.0);
    v_col = col;
}
\0";
const FRAG: &[u8] = b"
#version 330

in vec4 v_col;

out vec4 f_col;

void main() {
    f_col = v_col;
}
\0";
const VERT_TEX_RGBA: &[u8] = b"
#version 330

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 col;
layout(location = 2) in vec2 uv;

out vec2 v_uv;
out vec4 v_col;

void main() {
    gl_Position = vec4(pos, 1.0);
    v_uv = uv;
    v_col = col;
}
\0";
const FRAG_TEX_RGBA: &[u8] = b"
#version 330

uniform sampler2D tex;

in vec2 v_uv;
in vec4 v_col;

out vec4 f_col;

void main() {
    f_col = v_col * texture(tex, v_uv);
}
\0";
const VERT_TEX_A: &[u8] = b"
#version 330

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 col;
layout(location = 2) in vec2 uv;

out vec4 v_col;
out vec2 v_uv;

void main() {
    gl_Position = vec4(pos, 1.0);
    v_uv = uv;
    v_col = col;
}
\0";
const FRAG_TEX_A: &[u8] = b"
#version 330

uniform sampler2D tex;

in vec4 v_col;
in vec2 v_uv;

out vec4 f_col;

void main() {
    f_col = v_col * texture(tex, v_uv).r;
}
\0";
const VERT_TRAPEZOIDS: &[u8] = b"
#version 330

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 from;
layout(location = 2) in vec2 to;

out vec2 v_from;
out vec2 v_to;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    v_from = from;
    v_to = to;
}
\0";
const FRAG_TRAPEZOIDS: &[u8] = b"
#version 330

in vec2 v_from;
in vec2 v_to;

out float f_alpha;

void main() {
    vec2 origin = gl_FragCoord.xy - vec2(0.5, 0.5);
    vec2 from = v_from - origin;
    vec2 d = v_to - v_from;
    vec2 bottom_left = clamp(-from / d, 0.0, 1.0);
    vec2 top_right = clamp((1.0 - from) / d, 0.0, 1.0);
    vec2 enter = min(bottom_left, top_right);
    vec2 exit = max(bottom_left, top_right);
    float inside_t1 = max(enter.x, enter.y);
    float inside_t2 = max(min(exit.x, exit.y), inside_t1);
    float inside = (from.x + 0.5 * (inside_t1 + inside_t2) * d.x) * (inside_t2 - inside_t1);
    float right = max(d.x < 0.0 ? min(enter.x, exit.y) - enter.y : exit.y - max(exit.x, enter.y), 0.0);
    f_alpha = d.y * (inside + right);
}
\0";
