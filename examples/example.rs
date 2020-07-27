use ochre::{Path, Vec2, Vertex, Renderer, TILE_SIZE, ATLAS_SIZE};

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
    let tiles = path.fill();

    let mut data = vec![0; ATLAS_SIZE * ATLAS_SIZE];
    for row in 0..TILE_SIZE {
        for col in 0..TILE_SIZE {
            data[row * ATLAS_SIZE + col] = 255;
        }
    }

    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut u = 1;
    let mut v = 0;
    for tile in tiles.tiles {
        let base = vertices.len() as u32;
        let col = [255, 255, 255, 255];
        vertices.push(Vertex { pos: [tile.x * TILE_SIZE as i16, tile.y * TILE_SIZE as i16], col, uv: [u * TILE_SIZE as u16, v * TILE_SIZE as u16] });
        vertices.push(Vertex { pos: [(tile.x + 1) * TILE_SIZE as i16, tile.y * TILE_SIZE as i16], col, uv: [(u + 1) * TILE_SIZE as u16, v * TILE_SIZE as u16] });
        vertices.push(Vertex { pos: [(tile.x + 1) * TILE_SIZE as i16, (tile.y + 1) * TILE_SIZE as i16], col, uv: [(u + 1) * TILE_SIZE as u16, (v + 1) * TILE_SIZE as u16] });
        vertices.push(Vertex { pos: [tile.x * TILE_SIZE as i16, (tile.y + 1) * TILE_SIZE as i16], col, uv: [u * TILE_SIZE as u16, (v + 1) * TILE_SIZE as u16] });
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        for row in 0..TILE_SIZE {
            for col in 0..TILE_SIZE {
                data[v as usize * TILE_SIZE * ATLAS_SIZE + row * ATLAS_SIZE + u as usize * TILE_SIZE + col] = tiles.data[tile.index + row * TILE_SIZE + col];
            }
        }

        u += 1;
        if u as usize == ATLAS_SIZE / TILE_SIZE {
            u = 0;
            v += 1;
        }
    }

    for span in tiles.spans {
        let base = vertices.len() as u32;
        let col = [255, 255, 255, 255];
        vertices.push(Vertex { pos: [span.x * TILE_SIZE as i16, span.y * TILE_SIZE as i16], col, uv: [0, 0] });
        vertices.push(Vertex { pos: [(span.x + span.len) * TILE_SIZE as i16, span.y * TILE_SIZE as i16], col, uv: [0, 0] });
        vertices.push(Vertex { pos: [(span.x + span.len) * TILE_SIZE as i16, (span.y + 1) * TILE_SIZE as i16], col, uv: [0, 0] });
        vertices.push(Vertex { pos: [span.x * TILE_SIZE as i16, (span.y + 1) * TILE_SIZE as i16], col, uv: [0, 0] });
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    let mut renderer = Renderer::new();

    renderer.upload(0, 0, ochre::ATLAS_SIZE as u32, ochre::ATLAS_SIZE as u32, &data);

    let mut running = true;
    while running {
        renderer.clear([0.0, 0.0, 0.0, 1.0]);

        renderer.draw(&vertices[..], &indices[..], 800, 600);
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
