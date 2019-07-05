use std::collections::HashMap;
use std::ffi::{CStr, CString};
use gl::types::{GLuint, GLint, GLchar, GLenum};

macro_rules! offset {
    ($type:ty, $field:ident) => { &(*(0 as *const $type)).$field as *const _ as usize }
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

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub col: [f32; 4],
}

pub enum TextureFormat { RGBA, A }
pub type TextureId = usize;

pub struct Renderer {
    prog: Program,
    textures: HashMap<TextureId, Texture>,
    texture_id: TextureId,
}

impl Renderer {
    pub fn new() -> Renderer {
        let prog = Program::new(
            &CStr::from_bytes_with_nul(VERT).unwrap(),
            &CStr::from_bytes_with_nul(FRAG).unwrap()).unwrap();

        unsafe {
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
        }

        Renderer {
            prog,
            textures: HashMap::new(),
            texture_id: 0,
        }
    }

    pub fn clear(&mut self, col: [f32; 4]) {
        unsafe {
            gl::ClearColor(col[0], col[1], col[2], col[3]);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn draw(&mut self, vertices: &[Vertex], indices: &[u16]) {
        unsafe {
            let mut vbo: u32 = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<Vertex>()) as isize, vertices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

            let mut ibo: u32 = 0;
            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * std::mem::size_of::<u16>()) as isize, indices.as_ptr() as *const std::ffi::c_void, gl::STATIC_DRAW);

            let mut vao: u32 = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);

            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, pos) as *const gl::types::GLvoid);
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, col) as *const gl::types::GLvoid);

            gl::UseProgram(self.prog.id);

            gl::DrawElements(gl::TRIANGLES, indices.len() as i32, gl::UNSIGNED_SHORT, 0 as *const gl::types::GLvoid);

            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);

            gl::DeleteVertexArrays(1, &vao);
            gl::DeleteBuffers(1, &ibo);
            gl::DeleteBuffers(1, &vbo);
        }
    }

    pub fn create_texture(&mut self, format: TextureFormat, width: usize, height: usize, pixels: &[u8]) -> TextureId {
        let id = self.texture_id;
        self.textures.insert(id, Texture::new(format, width, height, pixels));
        self.texture_id += 1;
        id
    }

    pub fn update_texture(&mut self, texture: TextureId, x: usize, y: usize, width: usize, height: usize, pixels: &[u8]) {
        self.textures.get_mut(&texture).unwrap().update(x, y, width, height, pixels);
    }

    pub fn delete_texture(&mut self, texture: TextureId) {
        self.textures.remove(&texture);
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

struct Texture {
    format: TextureFormat,
    id: GLuint,
}

impl Texture {
    fn new(format: TextureFormat, width: usize, height: usize, pixels: &[u8]) -> Texture {
        let flipped = flip(pixels, width);
        let mut id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            match format {
                TextureFormat::RGBA => {
                    assert!(flipped.len() == width * height * 4);
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA32UI as GLint, width as i32, height as i32, 0, gl::RGBA, gl::UNSIGNED_INT_8_8_8_8, flipped.as_ptr() as *const std::ffi::c_void);
                }
                TextureFormat::A => {
                    assert!(flipped.len() == width * height);
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8 as GLint, width as i32, height as i32, 0, gl::RED, gl::UNSIGNED_BYTE, flipped.as_ptr() as *const std::ffi::c_void);
                }
            }
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }
        Texture { format, id }
    }

    fn update(&mut self, x: usize, y: usize, width: usize, height: usize, pixels: &[u8]) {
        let flipped = flip(pixels, width);
        unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id); }
        match self.format {
            TextureFormat::RGBA => {
                if flipped.len() != width * height * 4 { panic!() }
                unsafe {
                    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                    gl::TexSubImage2D(gl::TEXTURE_2D, 0, x as i32, y as i32, width as i32, height as i32, gl::RGBA, gl::UNSIGNED_INT_8_8_8_8, flipped.as_ptr() as *const std::ffi::c_void);
                }
            }
            TextureFormat::A => {
                if flipped.len() != width * height { panic!() }
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

fn flip(pixels: &[u8], width: usize) -> Vec<u8> {
    let mut flipped: Vec<u8> = Vec::with_capacity(pixels.len());
    for chunk in pixels.rchunks(width) {
        flipped.extend(chunk);
    }
    flipped
}
