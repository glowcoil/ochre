use ochre::{rasterize, Path, TileBuilder, Transform, TILE_SIZE};

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
    let mut path = Path::new();
    path.move_to(400.0, 300.0)
        .quadratic_to(500.0, 200.0, 400.0, 100.0)
        .cubic_to(350.0, 150.0, 100.0, 250.0, 400.0, 300.0);

    let mut builder = Builder;

    rasterize(&path, Transform::id(), &mut builder);
}
