use std::ffi::{CStr, CString};
use gl::types::{GLuint, GLint, GLchar, GLenum, GLvoid, GLsizei};

use crate::{ATLAS_SIZE, Backend, Vertex, Color};

macro_rules! offset {
    ($type:ty, $field:ident) => { &(*(0 as *const $type)).$field as *const _ as usize }
}

pub struct GlBackend {
    prog: Program,
    tex: GLuint,
}

impl GlBackend {
    pub fn new() -> GlBackend {
        let prog = Program::new(
            &CStr::from_bytes_with_nul(VERT).unwrap(),
            &CStr::from_bytes_with_nul(FRAG).unwrap()).unwrap();

        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
        }

        let mut tex: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8 as GLint, ATLAS_SIZE as GLint, ATLAS_SIZE as GLint, 0, gl::RED, gl::UNSIGNED_BYTE, std::ptr::null_mut());
        }

        GlBackend {
            prog,
            tex,
        }
    }
}

impl Backend for GlBackend {
    fn clear(&mut self, col: Color) {
        unsafe {
            gl::ClearColor(col.r, col.g, col.b, col.a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn upload(&mut self, x: u32, y: u32, width: u32, height: u32, data: &[u8]) {
        assert!((width * height) as usize == data.len());
        unsafe {
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, x as GLint, y as GLint, width as GLsizei, height as GLsizei, gl::RED, gl::UNSIGNED_BYTE, data.as_ptr() as *const std::ffi::c_void);
        }
    }

    fn draw(&mut self, vertices: &[Vertex], indices: &[u16], screen_width: u32, screen_height: u32) {
        let mut vbo: GLuint = 0;
        let mut ibo: GLuint = 0;
        let mut vao: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (vertices.len() * std::mem::size_of::<Vertex>()) as isize, vertices.as_ptr() as *const std::ffi::c_void, gl::STREAM_DRAW);

            gl::GenBuffers(1, &mut ibo);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * std::mem::size_of::<u16>()) as isize, indices.as_ptr() as *const std::ffi::c_void, gl::STREAM_DRAW);

            let pos = gl::GetAttribLocation(self.prog.id, b"pos\0" as *const u8 as *const i8) as GLuint;
            gl::EnableVertexAttribArray(pos);
            gl::VertexAttribPointer(pos, 2, gl::SHORT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, pos) as *const GLvoid);

            let uv = gl::GetAttribLocation(self.prog.id, b"uv\0" as *const u8 as *const i8) as GLuint;
            gl::EnableVertexAttribArray(uv);
            gl::VertexAttribPointer(uv, 2, gl::UNSIGNED_SHORT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, uv) as *const GLvoid);

            let col = gl::GetAttribLocation(self.prog.id, b"col\0" as *const u8 as *const i8) as GLuint;
            gl::EnableVertexAttribArray(col);
            gl::VertexAttribPointer(col, 4, gl::UNSIGNED_BYTE, gl::TRUE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, col) as *const GLvoid);

            gl::UseProgram(self.prog.id);

            let res = gl::GetUniformLocation(self.prog.id, b"res\0" as *const u8 as *const i8);
            gl::Uniform2ui(res, screen_width, screen_height);

            let atlas_size = gl::GetUniformLocation(self.prog.id, b"atlas_size\0" as *const u8 as *const i8);
            gl::Uniform1ui(atlas_size, crate::ATLAS_SIZE as u32);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.tex);
            let tex = gl::GetUniformLocation(self.prog.id, b"tex\0" as *const u8 as *const i8);
            gl::Uniform1i(tex, 0);

            gl::DrawElements(gl::TRIANGLES, indices.len() as GLint, gl::UNSIGNED_SHORT, 0 as *const std::ffi::c_void);

            gl::DeleteVertexArrays(1, &vao);
            gl::DeleteBuffers(1, &vbo);
            gl::DeleteBuffers(1, &ibo);
        }
    }
}

impl Drop for GlBackend {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.tex);
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

const VERT: &[u8] = b"
#version 330

uniform uvec2 res;
uniform uint atlas_size;

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 col;

out vec2 v_uv;
out vec4 v_col;

void main() {
    vec2 scaled = 2.0 * pos / vec2(res);
    gl_Position = vec4(scaled.x - 1.0, 1.0 - scaled.y, 0.0, 1.0);
    v_uv = uv / float(atlas_size);
    v_col = col;
}
\0";
const FRAG: &[u8] = b"
#version 330

uniform sampler2D tex;

in vec2 v_uv;
in vec4 v_col;

out vec4 f_col;

void main() {
    f_col = v_col * vec4(1.0, 1.0, 1.0, texture(tex, v_uv).r);
}
\0";
