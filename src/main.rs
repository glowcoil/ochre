mod graphics;
mod input;
mod render;
mod window;

extern crate gl;
extern crate glutin;

use window::Window;

fn main() {
    let mut window = Window::new();
    window.run();
}
