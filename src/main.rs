mod window;

extern crate glutin;

use window::Window;

fn main() {
    let mut window = Window::new();
    window.run();
}
