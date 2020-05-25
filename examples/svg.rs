use ochre::{Color, Path, Vec2, Quad, Renderer};

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(1900.0, 1000.0))
        .with_title("ochre");
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &events_loop).unwrap();
    let context = unsafe { context.make_current() }.unwrap();

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    let path = std::env::args().nth(1).expect("provide an svg file");
    let tree = usvg::Tree::from_file(path, &usvg::Options::default()).unwrap();
    let mut shapes = Vec::new();

    #[derive(Clone)]
    struct Shape { cmds: Vec<Cmd>, pts: Vec<Vec2> }
    #[derive(Copy, Clone)]
    enum Cmd { Move, Line, Curve, Close }

    for child in tree.root().children() {
        match *child.borrow() {
            usvg::NodeKind::Path(ref p) => {
                let mut shape = Shape { cmds: Vec::new(), pts: Vec::new() };
                for segment in p.data.0.iter() {
                    match *segment {
                        usvg::PathSegment::MoveTo { x, y } => {
                            shape.cmds.push(Cmd::Move);
                            shape.pts.push(Vec2::new(400.0, 250.0) + 2.0 * Vec2::new(x as f32, y as f32));

                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            shape.cmds.push(Cmd::Line);
                            shape.pts.push(Vec2::new(400.0, 250.0) + 2.0 * Vec2::new(x as f32, y as f32));
                        }
                        usvg::PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                            shape.cmds.push(Cmd::Curve);
                            shape.pts.push(Vec2::new(400.0, 250.0) + 2.0 * Vec2::new(x1 as f32, y1 as f32));
                            shape.pts.push(Vec2::new(400.0, 250.0) + 2.0 * Vec2::new(x2 as f32, y2 as f32));
                            shape.pts.push(Vec2::new(400.0, 250.0) + 2.0 * Vec2::new(x as f32, y as f32));
                        }
                        usvg::PathSegment::ClosePath => {
                            shape.cmds.push(Cmd::Close);
                        }
                    }
                }
                if let Some(ref fill) = p.fill {
                    if let usvg::Paint::Color(color) = fill.paint {
                        let color = Color::rgba(color.red as f32 / 255.0, color.green as f32 / 255.0, color.blue as f32 / 255.0, fill.opacity.value() as f32);
                        shapes.push((color, shape.clone(), None));
                    }
                }
                if let Some(ref stroke) = p.stroke {
                    if let usvg::Paint::Color(color) = stroke.paint {
                        let color = Color::rgba(color.red as f32 / 255.0, color.green as f32 / 255.0, color.blue as f32 / 255.0, stroke.opacity.value() as f32);
                        shapes.push((color, shape, Some(2.0 * stroke.width.value() as f32)));
                    }
                }
            }
            _ => {}
        }
    }
    
    let time = std::time::Instant::now();

    let mut quads = Vec::new();
    
    for _ in 0..1 {
        use rayon::iter::ParallelIterator;
        use rayon::iter::IntoParallelRefIterator;
        let _: Vec<()> = shapes.iter().map(|(color, shape, stroke)| {
            let mut path = Path::new();
            let mut pt = 0;
            for cmd in shape.cmds.iter() {
                match *cmd {
                    Cmd::Move => {
                        path.move_to(shape.pts[pt]);
                        pt += 1;
                    }
                    Cmd::Line => {
                        path.line_to(shape.pts[pt]);
                        pt += 1;
                    }
                    Cmd::Curve => {
                        path.cubic_to(shape.pts[pt], shape.pts[pt + 1], shape.pts[pt + 2]);
                        pt += 3;
                    }
                    Cmd::Close => {
                        path.close();
                    }
                }
            }
            if let Some(width) = stroke {
                path = path.stroke(*width);
            }

            for span in path.to_spans() {
                let col = [(255.0 * color.r) as u8, (255.0 * color.g) as u8, (255.0 * color.b) as u8, (255.0 * span.coverage) as u8];
                quads.push(Quad { pos: [span.x, span.y], size: [span.len, 1], col });
            }
        }).collect();

        

        /*let mut rect = Path::new();
        rect.move_to(Vec2::new(0.0, 0.0))
            .line_to(Vec2::new(0.0, 1000.0))
            .line_to(Vec2::new(1900.0, 1000.0))
            .line_to(Vec2::new(1900.0, 0.0))
            .close();*/

    }

    dbg!(time.elapsed().div_f64(1.0));

    let mut renderer = Renderer::new();


    let mut running = true;
    while running {
        renderer.clear([1.0, 1.0, 1.0, 1.0]);

        let mut query: u32 = 0;
        unsafe {
            gl::GenQueries(1, &mut query);
            gl::BeginQuery(gl::TIME_ELAPSED, query);
        }

        renderer.draw_quads(&quads[..], 1900, 1000);

        let mut elapsed: u64 = 0;
        unsafe {
            gl::EndQuery(gl::TIME_ELAPSED);
            let mut available: i32 = 0;
            while available == 0 {
                gl::GetQueryObjectiv(query, gl::QUERY_RESULT_AVAILABLE, &mut available);
            }
            gl::GetQueryObjectui64v(query, gl::QUERY_RESULT, &mut elapsed);
        }
        
        println!("{}", elapsed);

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
