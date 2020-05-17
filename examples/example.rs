use ochre::{Color, Path, Vec2, Vertex, Renderer};

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

    let mut path = Path::new();
    path.move_to(Vec2::new(400.0, 300.0))
        .quadratic_to(Vec2::new(500.0, 200.0), Vec2::new(400.0, 100.0))
        .cubic_to(Vec2::new(350.0, 150.0), Vec2::new(100.0, 250.0), Vec2::new(400.0, 300.0));
    let spans = path.to_spans();

    let mut vertices = Vec::new();
    for span in spans {
        let col = [(255.0 * span.coverage) as u8, (255.0 * span.coverage) as u8, (255.0 * span.coverage) as u8, (255.0 * span.coverage) as u8];
        vertices.push(Vertex { pos: [span.x, span.y], col });
        vertices.push(Vertex { pos: [span.x + span.len as i16, span.y], col });
    }

    let mut renderer = Renderer::new();

    let mut running = true;
    while running {
        renderer.clear([0.0, 0.0, 0.0, 1.0]);

        renderer.draw_lines(&vertices[..], 800, 600);
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
