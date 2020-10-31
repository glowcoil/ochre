use ochre::{Backend, Color, GlBackend, Mat2x2, Path, Picture, Vec2};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(1920.0, 1000.0))
        .with_title("ochre");
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &events_loop)
        .unwrap();
    let context = unsafe { context.make_current() }.unwrap();

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let path = std::env::args().nth(1).expect("provide an svg file");
    let tree = usvg::Tree::from_file(path, &usvg::Options::default()).unwrap();

    let time = std::time::Instant::now();

    let mut picture = Picture::new();

    for child in tree.root().children() {
        match *child.borrow() {
            usvg::NodeKind::Path(ref p) => {
                let mut path = Path::new();
                for segment in p.data.0.iter() {
                    match *segment {
                        usvg::PathSegment::MoveTo { x, y } => {
                            path.move_to(Vec2::new(x as f32, y as f32));
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            path.line_to(Vec2::new(x as f32, y as f32));
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
                                Vec2::new(x1 as f32, y1 as f32),
                                Vec2::new(x2 as f32, y2 as f32),
                                Vec2::new(x as f32, y as f32),
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
                        picture.fill(&path, Vec2::new(0.0, 0.0), Mat2x2::scale(2.0), color);
                    }
                }
                // if let Some(ref stroke) = p.stroke {
                //     if let usvg::Paint::Color(color) = stroke.paint {
                //         let color = Color::rgba(color.red as f32 / 255.0, color.green as f32 / 255.0, color.blue as f32 / 255.0, stroke.opacity.value() as f32);
                //         paths.push((color, shape, Some(1.0 * stroke.width.value() as f32)));
                //     }
                // }
            }
            _ => {}
        }
    }

    dbg!(time.elapsed());

    let mut query: u32 = 0;

    let mut backend = GlBackend::new();

    let mut running = true;
    while running {
        backend.clear(Color::rgba(1.0, 1.0, 1.0, 1.0));

        // unsafe {
        //     gl::GenQueries(1, &mut query);
        //     gl::BeginQuery(gl::TIME_ELAPSED, query);
        // }

        let time = std::time::Instant::now();
        picture.submit(&mut backend, 1920, 1000);
        dbg!(time.elapsed());

        // let mut elapsed: u64 = 0;
        // unsafe {
        //     gl::EndQuery(gl::TIME_ELAPSED);
        //     let mut available: i32 = 0;
        //     while available == 0 {
        //         gl::GetQueryObjectiv(query, gl::QUERY_RESULT_AVAILABLE, &mut available);
        //     }
        //     gl::GetQueryObjectui64v(query, gl::QUERY_RESULT, &mut elapsed);
        // }

        // println!("{}", elapsed);

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
}
