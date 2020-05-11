use ochre::{Color, Path, Vec2, Vertex, Span, Renderer, RenderOptions};

const TEXT: &'static str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor
incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis
nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu
fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in
culpa qui officia deserunt mollit anim id est laborum.

Curabitur pretium tincidunt lacus. Nulla gravida orci a odio. Nullam varius,
turpis et commodo pharetra, est eros bibendum elit, nec luctus magna felis
sollicitudin mauris. Integer in mauris eu nibh euismod gravida. Duis ac tellus
et risus vulputate vehicula. Donec lobortis risus a elit. Etiam tempor. Ut
ullamcorper, ligula eu tempor congue, eros est euismod turpis, id tincidunt
sapien risus a quam. Maecenas fermentum consequat mi. Donec fermentum.
Pellentesque malesuada nulla a mi. Duis sapien sem, aliquet nec, commodo eget,
consequat quis, neque. Aliquam faucibus, elit ut dictum aliquet, felis nisl
adipiscing sapien, sed malesuada diam lacus eget erat. Cras mollis scelerisque
nunc. Nullam arcu. Aliquam consequat. Curabitur augue lorem, dapibus quis,
laoreet et, pretium ac, nisi. Aenean magna nisl, mollis quis, molestie eu,
feugiat in, orci. In hac habitasse platea dictumst.";

fn main() {
    let mut events_loop = glutin::EventsLoop::new();
    let window_builder = glutin::WindowBuilder::new()
        .with_dimensions(glutin::dpi::LogicalSize::new(1920.0, 1000.0))
        .with_title("ochre");
    let context = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window_builder, &events_loop).unwrap();
    let context = unsafe { context.make_current() }.unwrap();

    gl::load_with(|symbol| context.get_proc_address(symbol) as *const _);

    // let font = ttf_parser::Font::from_data(include_bytes!("../../gouache/res/SourceSansPro-Regular.otf"), 0).unwrap();
    // use ttf_parser::OutlineBuilder;
    // struct Builder { path: Path, offset: Vec2, scale: f32 }
    // impl OutlineBuilder for Builder {
    //     fn move_to(&mut self, x: f32, y: f32) {
    //         self.path.move_to(self.offset + self.scale * Vec2::new(x, -y));
    //     }
    //     fn line_to(&mut self, x: f32, y: f32) {
    //         self.path.line_to(self.offset + self.scale * Vec2::new(x, -y));
    //     }
    //     fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
    //         self.path.quadratic_to(self.offset + self.scale * Vec2::new(x1, y1), self.offset + self.scale * Vec2::new(x, -y));
    //     }
    //     fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
    //         self.path.cubic_to(self.offset + self.scale * Vec2::new(x1, -y1), self.offset + self.scale * Vec2::new(x2, -y2), self.offset + self.scale * Vec2::new(x, -y));
    //     }
    //     fn close(&mut self) {
    //         self.path.close();
    //     }
    // }
    // let mut paths = Vec::new();
    // let mut pos = Vec2::new(0.0, 0.0);
    // let scale = 14.0 / font.units_per_em().unwrap() as f32;
    // let line_height = scale * (font.height() + font.line_gap()) as f32;
    // for c in TEXT.chars() {
    //     if c == '\n' {
    //         pos.x = 0.0;
    //         pos.y += line_height;
    //     } else {
    //         let glyph_id = font.glyph_index(c).unwrap();
    //         let mut builder = Builder { path: Path::new(), offset: Vec2::new(50.0, 50.0) + pos, scale };
    //         font.outline_glyph(glyph_id, &mut builder).unwrap();
    //         paths.push((Color::rgba(1.0, 1.0, 1.0, 1.0), builder.path));
    //         pos.x += scale * font.glyph_hor_metrics(glyph_id).unwrap().advance as f32;
    //     }
    // }

    let mut paths = Vec::new();

    let tree = usvg::Tree::from_file("../gouache/res/boston.svg", &usvg::Options::default()).unwrap();
    for child in tree.root().children() {
        match *child.borrow() {
            usvg::NodeKind::Path(ref p) => {
                let mut path = Path::new();
                for segment in p.data.0.iter() {
                    match *segment {
                        usvg::PathSegment::MoveTo { x, y } => {
                            path.move_to(Vec2::new(400.0, 900.0) + 2.0 * Vec2::new(x as f32, -y as f32));
                        }
                        usvg::PathSegment::LineTo { x, y } => {
                            path.line_to(Vec2::new(400.0, 900.0) + 2.0 * Vec2::new(x as f32, -y as f32));
                        }
                        usvg::PathSegment::CurveTo { x1, y1, x2, y2, x, y } => {
                            path.cubic_to(Vec2::new(400.0, 900.0) + 2.0 * Vec2::new(x1 as f32, -y1 as f32), Vec2::new(400.0, 900.0) + 2.0 * Vec2::new(x2 as f32, -y2 as f32), Vec2::new(400.0, 900.0) + 2.0 * Vec2::new(x as f32, -y as f32));
                        }
                        usvg::PathSegment::ClosePath => {
                            path.close();
                        }
                    }
                }
                if let Some(ref fill) = p.fill {
                    if let usvg::Paint::Color(color) = fill.paint {
                        let color = Color::rgba(color.red as f32 / 255.0, color.green as f32 / 255.0, color.blue as f32 / 255.0, fill.opacity.value() as f32);
                        paths.push((color, path));
                    }
                }
                // if let Some(ref stroke) = p.stroke {
                //     if let usvg::Paint::Color(color) = stroke.paint {
                //         let color = Color::rgba(color.red as f32 / 255.0, color.green as f32 / 255.0, color.blue as f32 / 255.0, stroke.opacity.value() as f32);
                //         let stroke = path.stroke(stroke.width.value() as f32);
                //         let path_descr = alloc.add_path(&stroke).unwrap();
                //         mesh_builder.add_path(path_descr, Mat4x4::id(), color);
                //         min = min.min(stroke.offset());
                //         max = max.max(stroke.size());
                //     }
                // }
            }
            _ => {}
        }
    }

    // let mut input = String::new();
    // std::io::stdin().read_line(&mut input);

    let time = std::time::Instant::now();
    let mut vertices = Vec::new();
    // let mut increments = Vec::with_capacity(4096);
    // let mut spans = Vec::with_capacity(4096);
    // for _ in 0..100 {
        // vertices.truncate(0);

        use rayon::iter::IntoParallelRefIterator;
        use rayon::iter::ParallelIterator;
        let spans: Vec<(Color, Vec<Span>)> = paths.par_iter().map(|(color, path)| {
                // let mut vertices = Vec::new();
                let mut increments = Vec::new();
                let mut spans = Vec::new();
                path.to_spans(&mut increments, &mut spans);
                (*color, spans)
            })
            .collect();
        for (color, spans) in spans {
            for span in spans {
                let start = Vec2::new(span.x as f32, span.y as f32).pixel_to_ndc(1920.0, 1000.0);
                let end = Vec2::new((span.x + span.len as i16) as f32, span.y as f32).pixel_to_ndc(1920.0, 1000.0);
                let brightness = (color.r + color.g + color.b) / 3.0;
                // let alpha = (1.0 - brightness) * (1.0 - (1.0 - span.coverage) * (1.0 - span.coverage)) + brightness * span.coverage * span.coverage;
                // let col = Color::rgba(color.r, color.g, color.b, color.a * alpha).to_linear_premul();
                let col = [(255.0 * span.coverage * color.a * color.r) as u8, (255.0 * span.coverage * color.a * color.g) as u8, (255.0 * span.coverage * color.a * color.b) as u8, (255.0 * span.coverage * color.a) as u8];
                // let col = Color::rgba(color.r, color.g, color.b, color.a * span.coverage).to_linear_premul();
                vertices.push(Vertex { pos: [span.x as u16, span.y as u16], col });
                vertices.push(Vertex { pos: [span.x as u16 + span.len, span.y as u16], col });
            }
        }

        // for v in vs {
        //     vertices.extend_from_slice(&v[..]);
        // }
        // for (color, path) in paths.iter() {
        //     increments.truncate(0);
        //     spans.truncate(0);
        //     path.to_spans(&mut increments, &mut spans);
        //     for span in spans.iter() {
        //         let start = Vec2::new(span.x as f32, span.y as f32).pixel_to_ndc(1920.0, 1000.0);
        //         let end = Vec2::new((span.x + span.len as i16) as f32, span.y as f32).pixel_to_ndc(1920.0, 1000.0);
        //         let brightness = (color.r + color.g + color.b) / 3.0;
        //         // let alpha = (1.0 - brightness) * (1.0 - (1.0 - span.coverage) * (1.0 - span.coverage)) + brightness * span.coverage * span.coverage;
        //         // let col = Color::rgba(color.r, color.g, color.b, color.a * alpha).to_linear_premul();
        //         let col = [(255.0 * span.coverage * color.a * color.r) as u8, (255.0 * span.coverage * color.a * color.g) as u8, (255.0 * span.coverage * color.a * color.b) as u8, (255.0 * span.coverage * color.a) as u8];
        //         // let col = Color::rgba(color.r, color.g, color.b, color.a * span.coverage).to_linear_premul();
        //         vertices.push(Vertex { pos: [span.x as u16, span.y as u16], col });
        //         vertices.push(Vertex { pos: [span.x as u16 + span.len, span.y as u16], col });
        //     }
        // }
    // }
    dbg!(time.elapsed());


    // let mut path = Path::new();
    // path.move_to(Vec2::new(0.0, 0.0))
    //     .line_to(Vec2::new(0.0, 1000.0))
    //     .line_to(Vec2::new(1920.0, 1000.0))
    //     .line_to(Vec2::new(1920.0, 0.0))
    //     .line_to(Vec2::new(0.0, 0.0))
    //     .close();
    // path.move_to(Vec2::new(400.0, 300.0))
    //     .quadratic_to(Vec2::new(500.0, 200.0), Vec2::new(400.0, 100.0))
    //     .cubic_to(Vec2::new(350.0, 150.0), Vec2::new(100.0, 250.0), Vec2::new(400.0, 300.0));
    // let spans = path.to_spans();
    // dbg!(&path);
    // dbg!(&spans);

    // let mut vertices = Vec::new();
    // for span in spans {
    //     let start = Vec2::new(span.x as f32, span.y as f32).pixel_to_ndc(1920.0, 1000.0);
    //     let end = Vec2::new((span.x + span.len as i16) as f32, span.y as f32).pixel_to_ndc(1920.0, 1000.0);
    //     let col = Color::rgba(1.0, 1.0, 1.0, span.coverage).to_linear_premul();
    //     vertices.push(Vertex { pos: [start.x, start.y], col });
    //     vertices.push(Vertex { pos: [end.x, end.y], col });
    // }

    let mut renderer = Renderer::new(1920, 1000);

    let mut running = true;
    while running {
        renderer.clear([1.0, 1.0, 1.0, 1.0], &RenderOptions::default());

        let mut query: u32 = 0;
        unsafe {
            gl::GenQueries(1, &mut query);
            gl::BeginQuery(gl::TIME_ELAPSED, query);
        }

        // for v in vs.iter() {
        //     renderer.draw_lines(&v[..]);
        // }
        renderer.draw_lines(&vertices[..]);

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

        // graphics.begin_frame();
        // graphics.set_color(Color::rgba(1.0, 1.0, 1.0, 1.0));
        // let rect = Path::rect_fill(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        // graphics.draw_mesh(&rect);
        // graphics.set_color(Color::rgba(0.5, 0.25, 1.0, 0.75));
        // let mut path = Path::new();
        // path.move_to(Vec2::new(400.0, 300.0))
        //     .quadratic_to(Vec2::new(500.0, 200.0), Vec2::new(400.0, 100.0))
        //     .cubic_to(Vec2::new(350.0, 150.0), Vec2::new(100.0, 250.0), Vec2::new(400.0, 300.0));
        // graphics.draw_mesh(&path.fill_convex());
        // graphics.set_color(Color::rgba(0.0, 0.5, 1.0, 0.5));
        // let mut path = Path::new();
        // path.move_to(Vec2::new(1000.0, 300.0))
        //     .arc_to(50.0, Vec2::new(1000.0, 400.0))
        //     .arc_to(50.0, Vec2::new(1000.0, 300.0));
        // graphics.draw_mesh(&path.fill_convex());
        // graphics.set_color(Color::rgba(0.8, 0.5, 0.0, 1.0));
        // let round_rect = Path::round_rect_fill(Vec2::new(100.0, 10.0), Vec2::new(100.0, 100.0), 20.0);
        // graphics.draw_mesh(&round_rect);

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
