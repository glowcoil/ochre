use std::collections::HashMap;
use std::ffi::{CStr, CString};
use gl::types::{GLuint, GLint, GLchar, GLenum, GLvoid};

macro_rules! offset {
    ($type:ty, $field:ident) => { &(*(0 as *const $type)).$field as *const _ as usize }
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: [i16; 2],
    pub col: [u8; 4],
}

#[derive(Copy, Clone, Debug)]
pub struct Quad {
    pub pos: [i16; 2],
    pub size: [u16; 2],
    pub col: [u8; 4],
}

pub struct Renderer {
    prog: Program,
}

impl Renderer {
    pub fn new() -> Renderer {
        let prog = Program::new(
            &CStr::from_bytes_with_nul(VERT).unwrap(),
            &CStr::from_bytes_with_nul(FRAG).unwrap()).unwrap();

        unsafe {
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
        }

        Renderer {
            prog,
        }
    }

    pub fn clear(&mut self, col: [f32; 4]) {
        unsafe {
            gl::ClearColor(col[0], col[1], col[2], col[3]);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn draw_quads(&mut self, quads: &[Quad], width: u32, height: u32) {
        let mut quad_vbo: u32 = 0;
        let mut quads_vbo: u32 = 0;
        let mut vao: u32 = 0;
        unsafe {
            let quad: [[f32; 2]; 4] = [
                [0.0, 0.0],
                [1.0, 0.0],
                [0.0, 1.0],
                [1.0, 1.0],
            ];

            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::GenBuffers(1, &mut quad_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (quad.len() * std::mem::size_of::<[f32; 2]>()) as isize, quad.as_ptr() as *const std::ffi::c_void, gl::STREAM_DRAW);

            let uv = gl::GetAttribLocation(self.prog.id, b"uv\0" as *const u8 as *const i8) as GLuint;
            gl::EnableVertexAttribArray(uv);
            gl::VertexAttribPointer(uv, 2, gl::FLOAT, gl::FALSE, 0, 0 as *const GLvoid);
            gl::VertexAttribDivisor(uv, 0);

            gl::GenBuffers(1, &mut quads_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, quads_vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (quads.len() * std::mem::size_of::<Quad>()) as isize, quads.as_ptr() as *const std::ffi::c_void, gl::STREAM_DRAW);

            let pos = gl::GetAttribLocation(self.prog.id, b"pos\0" as *const u8 as *const i8) as GLuint;
            gl::EnableVertexAttribArray(pos);
            gl::VertexAttribPointer(pos, 2, gl::SHORT, gl::FALSE, std::mem::size_of::<Quad>() as GLint, offset!(Quad, pos) as *const GLvoid);
            gl::VertexAttribDivisor(pos, 1);

            let size = gl::GetAttribLocation(self.prog.id, b"size\0" as *const u8 as *const i8) as GLuint;
            gl::EnableVertexAttribArray(size);
            gl::VertexAttribPointer(size, 2, gl::UNSIGNED_SHORT, gl::FALSE, std::mem::size_of::<Quad>() as GLint, offset!(Quad, size) as *const GLvoid);
            gl::VertexAttribDivisor(size, 1);

            let col = gl::GetAttribLocation(self.prog.id, b"col\0" as *const u8 as *const i8) as GLuint;
            gl::EnableVertexAttribArray(col);
            gl::VertexAttribPointer(col, 4, gl::UNSIGNED_BYTE, gl::TRUE, std::mem::size_of::<Quad>() as GLint, offset!(Quad, col) as *const GLvoid);
            gl::VertexAttribDivisor(col, 1);

            gl::UseProgram(self.prog.id);

            let res = gl::GetUniformLocation(self.prog.id, b"res\0" as *const u8 as *const i8);
            gl::Uniform2ui(res, width, height);

            gl::DrawArraysInstanced(gl::TRIANGLE_STRIP, 0, quad.len() as i32, quads.len() as i32);

            gl::DeleteVertexArrays(1, &vao);
            gl::DeleteBuffers(1, &quad_vbo);
            gl::DeleteBuffers(1, &quads_vbo);
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

layout(location = 0) in vec2 uv;
layout(location = 1) in vec2 pos;
layout(location = 2) in vec2 size;
layout(location = 3) in vec4 col;

out vec4 v_col;

void main() {
    vec2 scaled = 2.0 * (pos + uv * size) / vec2(res);
    gl_Position = vec4(scaled.x - 1.0, 1.0 - scaled.y, 0.0, 1.0);
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
