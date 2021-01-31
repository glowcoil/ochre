use ochre::{PathCmd, Rasterizer, TileBuilder, Transform, Vec2, TILE_SIZE};

struct Builder;

impl TileBuilder for Builder {
    fn tile(&mut self, x: i16, y: i16, data: [u8; TILE_SIZE * TILE_SIZE]) {
        println!("tile at ({}, {}):", x, y);
        for row in 0..TILE_SIZE {
            print!("  ");
            for col in 0..TILE_SIZE {
                print!("{:3} ", data[row * TILE_SIZE + col]);
            }
            print!("\n");
        }
    }

    fn span(&mut self, x: i16, y: i16, width: u16) {
        println!("span at ({}, {}), width {}", x, y, width);
    }
}

fn main() {
    let mut builder = Builder;

    let mut rasterizer = Rasterizer::new();
    rasterizer.fill(&[
        PathCmd::Move(Vec2::new(400.0, 300.0)),
        PathCmd::Quadratic(Vec2::new(500.0, 200.0), Vec2::new(400.0, 100.0)),
        PathCmd::Cubic(Vec2::new(350.0, 150.0), Vec2::new(100.0, 250.0), Vec2::new(400.0, 300.0)),
        PathCmd::Close,
    ], Transform::id());
    rasterizer.finish(&mut builder);
}
