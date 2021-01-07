use std::ffi::{CStr, CString};
use gl::types::{GLuint, GLint, GLchar, GLenum, GLvoid, GLsizei};

use ochre::{Color, Path, Picture, Transform, Vertex};

macro_rules! offset {
    ($type:ty, $field:ident) => { &(*(0 as *const $type)).$field as *const _ as usize }
}

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64))
        .with_title("ochre");
    let context = glutin::ContextBuilder::new()
        .with_vsync(false)
        .build_windowed(window_builder, &events_loop)
        .unwrap();
    let context = unsafe { context.make_current() }.unwrap();

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let path = std::env::args().nth(1).expect("provide an svg file");
    let tree = usvg::Tree::from_file(path, &usvg::Options::default()).unwrap();

    let mut picture = Picture::new();
    for child in tree.root().children() {
        match *child.borrow() {
            usvg::NodeKind::Path(ref p) => {
                let mut path = Path::new();
                for segment in p.data.0.iter() {
                    match *segment {
                        usvg::PathSegment::MoveTo { x, y } => {
                            path.move_to(x as f32, y as f32);
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            path.line_to(x as f32, y as f32);
                        }
                        usvg::PathSegment::CurveTo {
                            x1,
                            y1,
                            x2,
                            y2,
                            x,
                            y,
                        } => {
                            path.cubic_to(
                                x1 as f32, y1 as f32,
                                x2 as f32, y2 as f32,
                                x as f32, y as f32,
                            );
                        }
                        usvg::PathSegment::ClosePath => {
                            path.close();
                        }
                    }
                }
                if let Some(ref fill) = p.fill {
                    if let usvg::Paint::Color(color) = fill.paint {
                        let color = Color::rgba(
                            color.red as f32 / 255.0,
                            color.green as f32 / 255.0,
                            color.blue as f32 / 255.0,
                            fill.opacity.value() as f32,
                        );
                        picture.fill(&path, Transform::translate(200.0, 200.0), color);
                    }
                }
                if let Some(ref stroke) = p.stroke {
                    if let usvg::Paint::Color(color) = stroke.paint {
                        let color = Color::rgba(
                            color.red as f32 / 255.0,
                            color.green as f32 / 255.0,
                            color.blue as f32 / 255.0,
                            stroke.opacity.value() as f32,
                        );
                        picture.stroke(
                            &path,
                            1.0 * stroke.width.value() as f32,
                            Transform::translate(200.0, 200.0),
                            color,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    let prog = Program::new(
        &CStr::from_bytes_with_nul(VERT).unwrap(),
        &CStr::from_bytes_with_nul(FRAG).unwrap()).unwrap();

    unsafe {
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        gl::Enable(gl::BLEND);
    }

    let (tex_width, tex_height) = picture.tiles_size();
    let tiles = picture.tiles();

    let mut tex: GLuint = 0;
    unsafe {
        gl::GenTextures(1, &mut tex);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8 as GLint, tex_width as GLsizei, tex_height as GLsizei, 0, gl::RED, gl::UNSIGNED_BYTE, tiles.as_ptr() as *const std::ffi::c_void);
    }

    let vertices = picture.vertices();
    let indices = picture.indices();

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
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices.len() * std::mem::size_of::<u32>()) as isize, indices.as_ptr() as *const std::ffi::c_void, gl::STREAM_DRAW);

        let pos = gl::GetAttribLocation(prog.id, b"pos\0" as *const u8 as *const i8) as GLuint;
        gl::EnableVertexAttribArray(pos);
        gl::VertexAttribPointer(pos, 2, gl::SHORT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, pos) as *const GLvoid);

        let uv = gl::GetAttribLocation(prog.id, b"uv\0" as *const u8 as *const i8) as GLuint;
        gl::EnableVertexAttribArray(uv);
        gl::VertexAttribPointer(uv, 2, gl::UNSIGNED_SHORT, gl::FALSE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, uv) as *const GLvoid);

        let col = gl::GetAttribLocation(prog.id, b"col\0" as *const u8 as *const i8) as GLuint;
        gl::EnableVertexAttribArray(col);
        gl::VertexAttribPointer(col, 4, gl::UNSIGNED_BYTE, gl::TRUE, std::mem::size_of::<Vertex>() as GLint, offset!(Vertex, col) as *const GLvoid);

        gl::UseProgram(prog.id);

        let res = gl::GetUniformLocation(prog.id, b"res\0" as *const u8 as *const i8);
        gl::Uniform2ui(res, SCREEN_WIDTH, SCREEN_HEIGHT);

        let atlas_size = gl::GetUniformLocation(prog.id, b"atlas_size\0" as *const u8 as *const i8);
        gl::Uniform2ui(atlas_size, tex_width, tex_height);

        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        let tex_uniform = gl::GetUniformLocation(prog.id, b"tex\0" as *const u8 as *const i8);
        gl::Uniform1i(tex_uniform, 0);
    }

    let mut running = true;
    while running {
        unsafe {
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::DrawElements(gl::TRIANGLES, indices.len() as GLint, gl::UNSIGNED_INT, 0 as *const std::ffi::c_void);
        }

        context.swap_buffers().unwrap();

        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => {
                use glutin::WindowEvent::*;
                match event {
                    CloseRequested => {
                        running = false;
                    }
                    _ => {}
                }
            }
            _ => {}
        });
    }

    unsafe {
        gl::DeleteTextures(1, &tex);
        gl::DeleteVertexArrays(1, &vao);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteBuffers(1, &ibo);
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
uniform uvec2 atlas_size;

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 uv;
layout(location = 2) in vec4 col;

out vec2 v_uv;
out vec4 v_col;

void main() {
    vec2 scaled = 2.0 * pos / vec2(res);
    gl_Position = vec4(scaled.x - 1.0, 1.0 - scaled.y, 0.0, 1.0);
    v_uv = uv / vec2(atlas_size);
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
