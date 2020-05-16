use ochre::{Color, Path, Vec2, Renderer};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
        .with_title("ochre");
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &events_loop).unwrap();
    let context = unsafe { context.make_current() }.unwrap();

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let mut renderer = Renderer::new();

    let mut running = true;
    while running {
        renderer.clear([1.0, 1.0, 1.0, 1.0]);

        context.swap_buffers().unwrap();

        events_loop.poll_events(|event| {
            match event {
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
            }
        });
    }
}
