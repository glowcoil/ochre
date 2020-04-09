use ochre::{Color, Graphics, Path, Vec2};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
        .with_title("gouache");
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &events_loop).unwrap();
    let context = unsafe { context.make_current() }.unwrap();

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let mut graphics = Graphics::new(800.0, 600.0);

    let mut running = true;
    while running {
        graphics.clear(Color::rgba(0.1, 0.15, 0.2, 1.0));
        graphics.begin_frame();
        graphics.set_color(Color::rgba(1.0, 1.0, 1.0, 1.0));
        let rect = Path::rect_fill(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        graphics.draw_mesh(&rect);
        graphics.set_color(Color::rgba(0.5, 0.25, 1.0, 0.75));
        let mut path = Path::new();
        path.move_to(Vec2::new(400.0, 300.0))
            .quadratic_to(Vec2::new(500.0, 200.0), Vec2::new(400.0, 100.0))
            .cubic_to(Vec2::new(350.0, 150.0), Vec2::new(100.0, 250.0), Vec2::new(400.0, 300.0));
        graphics.draw_mesh(&path.fill_convex());
        graphics.set_color(Color::rgba(0.0, 0.5, 1.0, 0.5));
        let mut path = Path::new();
        path.move_to(Vec2::new(600.0, 300.0))
            .arc_to(50.0, Vec2::new(600.0, 400.0))
            .arc_to(50.0, Vec2::new(600.0, 300.0));
        graphics.draw_mesh(&path.fill_convex());
        graphics.set_color(Color::rgba(0.8, 0.5, 0.0, 1.0));
        let round_rect = Path::round_rect_fill(Vec2::new(100.0, 10.0), Vec2::new(100.0, 100.0), 20.0);
        graphics.draw_mesh(&round_rect);
        graphics.end_frame();
        graphics.draw_texture_test();
        graphics.draw_trapezoids_test();

        context.swap_buffers().unwrap();

        events_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent { event, .. } => {
                    use glutin::WindowEvent::*;
                    match event {
                        Resized(size) => {}
                        Moved(position) => {}
                        CloseRequested => {
                            running = false;
                        }
                        Destroyed => {}
                        DroppedFile(path) => {}
                        HoveredFile(path) => {}
                        HoveredFileCancelled => {}
                        ReceivedCharacter(c) => {}
                        Focused(focus) => {}
                        KeyboardInput { input, .. } => {}
                        CursorMoved { position, modifiers, .. } => {}
                        CursorEntered { .. } => {}
                        CursorLeft { .. } => {}
                        MouseWheel { delta, modifiers, .. } => {}
                        MouseInput { state, button, modifiers, .. } => {}
                        Refresh => {}
                        HiDpiFactorChanged(factor) => {}
                        _ => {}
                    }
                }
                _ => {}
            }
        });
    }
}
