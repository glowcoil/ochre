use ochre::{Color, Path, Vec2, Vertex, Renderer};

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
                            shape.pts.push(Vec2::new(400.0, 250.0) + 4.0 * Vec2::new(x as f32, y as f32));
                            //path.move_to(Vec2::new(400.0, 1000.0) + 3.0 * Vec2::new(x as f32, -y as f32));

                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            shape.cmds.push(Cmd::Line);
                            shape.pts.push(Vec2::new(400.0, 250.0) + 4.0 * Vec2::new(x as f32, y as f32));
                            //path.line_to(Vec2::new(400.0, 1000.0) + 3.0 * Vec2::new(x as f32, -y as f32));
                        }
                        usvg::PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                            shape.cmds.push(Cmd::Curve);
                            shape.pts.push(Vec2::new(400.0, 250.0) + 4.0 * Vec2::new(x1 as f32, y1 as f32));
                            shape.pts.push(Vec2::new(400.0, 250.0) + 4.0 * Vec2::new(x2 as f32, y2 as f32));
                            shape.pts.push(Vec2::new(400.0, 250.0) + 4.0 * Vec2::new(x as f32, y as f32));
                            //path.cubic_to(Vec2::new(400.0, 1000.0) + 2.0 * Vec2::new(x1 as f32, -y1 as f32), Vec2::new(400.0, 1000.0) + 2.0 * Vec2::new(x2 as f32, -y2 as f32), Vec2::new(400.0, 1000.0) + 2.0 * Vec2::new(x as f32, -y as f32));
                        }
                        usvg::PathSegment::ClosePath => {
                            shape.cmds.push(Cmd::Close);
                            //path.close();
                        }
                    }
                }
                if let Some(ref fill) = p.fill {
                    if let usvg::Paint::Color(color) = fill.paint {
                        let color = Color::rgba(color.red as f32 / 255.0, color.green as f32 / 255.0, color.blue as f32 / 255.0, fill.opacity.value() as f32);
                        //paths.push((color, path));
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

    //let mut tiles = Vec::new();

    use ochre::TILE_SIZE;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut data = Vec::new();
    
    for _ in 0..1 {
    use rayon::iter::ParallelIterator;
    use rayon::iter::IntoParallelRefIterator;
    /*let tileses: Vec<(Color, Vec<(i16, i16, [u8; TILE_SIZE * TILE_SIZE], u8)>)> = shapes.iter().map(|(color, shape, stroke)| {
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

        (*color, path.to_spans())
    
        //tiles.extend_from_slice(&path.to_spans());
    }).collect();*/

    let mut rect = Path::new();
    rect.move_to(Vec2::new(0.0, 0.0))
        .line_to(Vec2::new(0.0, 1000.0))
        .line_to(Vec2::new(1900.0, 1000.0))
        .line_to(Vec2::new(1900.0, 0.0))
        .close();
    let tileses = vec![(Color::rgba(1.0, 1.0, 1.0, 1.0), rect.to_spans())];

    data = vec![0u8; 4096 * 4096];
    //let mut vertices = Vec::new();
    //let mut indices = Vec::new();
    for row in 0..TILE_SIZE {
        for col in 0..TILE_SIZE {
            data[row * 4096 + col] = 255;
        }
    }
    vertices = Vec::new();
    indices = Vec::new();
    let mut u = 1;
    let mut v = 0;
    for (color, tiles) in tileses {
        let mut x_prev = tiles.first().unwrap().0;
        let mut y_prev = tiles.first().unwrap().1;
        let mut last_winding = 0;
        for (x, y, tile, winding) in tiles {
            if v as usize == 4096 / TILE_SIZE {
                break;
            }
            let base = vertices.len() as u16;
            let col = [(color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8, (color.a * 255.0) as u8];
            vertices.push(Vertex { pos: [x * TILE_SIZE as i16, y * TILE_SIZE as i16], col, uv: [u, v] });
            vertices.push(Vertex { pos: [(x + 1) * TILE_SIZE as i16, y * TILE_SIZE as i16], col, uv: [u + 1, v] });
            vertices.push(Vertex { pos: [(x + 1) * TILE_SIZE as i16, (y + 1) * TILE_SIZE as i16], col, uv: [u + 1, v + 1] });
            vertices.push(Vertex { pos: [x * TILE_SIZE as i16, (y + 1) * TILE_SIZE as i16], col, uv: [u, v + 1] });
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

            let base = vertices.len() as u16;
            if y == y_prev && x > x_prev + 1 && last_winding == 1 {
                vertices.push(Vertex { pos: [(x_prev + 1) * TILE_SIZE as i16, y * TILE_SIZE as i16], col, uv: [0, 0] });
                vertices.push(Vertex { pos: [x * TILE_SIZE as i16, y * TILE_SIZE as i16], col, uv: [0, 0] });
                vertices.push(Vertex { pos: [x * TILE_SIZE as i16, (y + 1) * TILE_SIZE as i16], col, uv: [0, 0] });
                vertices.push(Vertex { pos: [(x_prev + 1) * TILE_SIZE as i16, (y + 1) * TILE_SIZE as i16], col, uv: [0, 0] });
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }

            x_prev = x;
            y_prev = y;
            last_winding = winding;

            for row in 0..TILE_SIZE {
                for col in 0..TILE_SIZE {
                    data[v as usize * TILE_SIZE * 4096 + row * 4096 + u as usize * TILE_SIZE + col] = tile[row * TILE_SIZE + col];
                }
            }

            u += 1;
            if u as usize == 4096 / TILE_SIZE {
                u = 0;
                v += 1;
            }
        }
    }
    }

    dbg!(time.elapsed().div_f64(1.0));

    let mut renderer = Renderer::new();


    use gl::types::{GLint, GLuint};
    let mut tex: GLuint = 0;
    unsafe {
        gl::GenTextures(1, &mut tex as *mut GLuint);
        gl::BindTexture(gl::TEXTURE_2D, tex);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::R8 as GLint, 4096, 4096, 0, gl::RED, gl::UNSIGNED_BYTE, data.as_ptr() as *const std::ffi::c_void);
    }

    let mut running = true;
    while running {
        renderer.clear([1.0, 1.0, 1.0, 1.0]);

        let mut query: u32 = 0;
        unsafe {
            gl::GenQueries(1, &mut query);
            gl::BeginQuery(gl::TIME_ELAPSED, query);
        }

        renderer.draw(&vertices[..], &indices[..], tex, 1900, 1000);

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
