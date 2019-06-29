const FRAME: std::time::Duration = std::time::Duration::from_micros(1_000_000 / 60);

pub struct Window {
    events_loop: glutin::EventsLoop,
    context: glutin::ContextWrapper<glutin::PossiblyCurrent, glutin::Window>,
}

impl Window {
    pub fn new() -> Window {
        let mut events_loop = glutin::EventsLoop::new();
        let window_builder = glutin::WindowBuilder::new()
            .with_dimensions(glutin::dpi::LogicalSize::new(800.0, 600.0))
            .with_title("gouache");
        let context = glutin::ContextBuilder::new()
            .build_windowed(window_builder, &events_loop).unwrap();
        let context = unsafe { context.make_current() }.unwrap();

        Window { events_loop, context }
    }

    pub fn run(&mut self) {
        let mut running = true;
        let mut now = std::time::Instant::now();
        while running {
            let elapsed = now.elapsed();
            now = std::time::Instant::now();

            self.context.swap_buffers().unwrap();

            self.events_loop.poll_events(|event| {
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

            let elapsed = now.elapsed();
            if elapsed < FRAME {
                std::thread::sleep(FRAME - elapsed);
            }
        }
    }
}
