use std::collections::HashMap;
use std::ffi::{CStr, CString};
use gl::types::{GLuint, GLint, GLchar, GLenum, GLvoid};

macro_rules! offset {
    ($type:ty, $field:ident) => { &(*(0 as *const $type)).$field as *const _ as usize }
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub col: [f32; 4],
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
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::BLEND);
            gl::Enable(gl::FRAMEBUFFER_SRGB);
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

    pub fn draw(&mut self, vertices: &[Vertex], indices: &[u16]) {
        let vertex_array = VertexArray::new(vertices, indices);
        unsafe {
            gl::UseProgram(self.prog.id);
            gl::DrawElements(gl::TRIANGLES, vertex_array.count, gl::UNSIGNED_SHORT, 0 as *const GLvoid);
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
