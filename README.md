# ochre

[![Cargo](https://img.shields.io/crates/v/ochre.svg)](https://crates.io/crates/ochre)
[![Documentation](https://docs.rs/ochre/badge.svg)](https://docs.rs/ochre)

High-quality anti-aliased vector graphics rendering on the GPU.

`ochre` rasterizes a path to a set of 8×8-pixel alpha-mask tiles at the path's boundary and n×8-pixel solid spans for the path's interior, which can then be uploaded to the GPU and rendered. Paths are rasterized using a high-quality analytic anti-aliasing method suitable for both text and general vector graphics.

## Example

```rust
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
```

## Run Examples

```bash
cargo run --example basic
cargo run --example svg examples/res/calabi-yau.svg
cargo run --example svg examples/res/Ghostscript_Tiger.svg
cargo run --example svg examples/res/lorem-ipsum.svg
```

## License

`ochre` is distributed under the terms of both the [MIT license](LICENSE-MIT) and the [Apache license, version 2.0](LICENSE-APACHE). Contributions are accepted under the same terms.
