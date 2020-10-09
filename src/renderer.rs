use std::collections::HashMap;

use crate::{TILE_SIZE, ATLAS_SIZE, DisplayList, Path, Tiles, Backend, Vertex, Vec2, Mat2x2};

const SUBPIXEL_STEPS: usize = 8;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct PathId(usize);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct PathKey {
    id: PathId,
    offset: [u8; 2],
    transform: [u8; 16],
}

impl PathKey {
    fn new(id: PathId, position: Vec2, transform: Mat2x2) -> PathKey {
        let mut transform_bytes: [u8; 16] = [0; 16];
        transform_bytes[0..4].copy_from_slice(&transform.0[0].to_le_bytes());
        transform_bytes[4..8].copy_from_slice(&transform.0[1].to_le_bytes());
        transform_bytes[8..12].copy_from_slice(&transform.0[2].to_le_bytes());
        transform_bytes[12..16].copy_from_slice(&transform.0[3].to_le_bytes());

        PathKey {
            id,
            offset: [
                (position.x.fract() * SUBPIXEL_STEPS as f32) as u8,
                (position.y.fract() * SUBPIXEL_STEPS as f32) as u8,
            ],
            transform: transform_bytes,
        }
    }
}

struct PathEntry {
    tiles: Tiles,
    // location: u32,
}

pub struct Renderer {
    paths: Slab<Path>,
    cache: HashMap<PathKey, PathEntry>,
    atlas: Vec<AtlasEntry>,
    data: Vec<u8>,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            paths: Slab::new(),
            cache: HashMap::new(),
            atlas: vec![];
            data: vec![0; ATLAS_SIZE * ATLAS_SIZE],
        }
    }

    pub fn add_path(&mut self, path: Path) -> PathId {
        PathId(self.paths.add(path))
    }

    pub fn remove_path(&mut self, id: PathId) -> Option<Path> {
        self.paths.remove(id.0)
    }

    pub fn submit(&mut self, display_list: &DisplayList, backend: &mut dyn Backend) {
        let mut data = vec![0; ATLAS_SIZE * ATLAS_SIZE];
        for row in 0..TILE_SIZE {
            for col in 0..TILE_SIZE {
                data[row * ATLAS_SIZE + col] = 255;
            }
        }

        let mut u = 1;
        let mut v = 0;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for item in display_list.items.iter() {
            let key = PathKey::new(item.path, item.position, item.transform);
            let entry = if let Some(entry) = self.cache.get(&key) {
                entry
            } else {
                let path = self.paths.get(item.path.0).unwrap();
                let tiles = path.fill(item.position, item.transform);
                self.cache.insert(key, PathEntry { tiles });
                self.cache.get(&key).unwrap()
            };

            let col = [
                (item.color.r * 256.0).min(255.0) as u8,
                (item.color.g * 256.0).min(255.0) as u8,
                (item.color.b * 256.0).min(255.0) as u8,
                (item.color.a * 256.0).min(255.0) as u8,
            ];

            for tile in entry.tiles.tiles.iter() {
                let base = vertices.len() as u16;
                vertices.push(Vertex { pos: [tile.x * TILE_SIZE as i16, tile.y * TILE_SIZE as i16], col, uv: [u * TILE_SIZE as u16, v * TILE_SIZE as u16] });
                vertices.push(Vertex { pos: [(tile.x + 1) * TILE_SIZE as i16, tile.y * TILE_SIZE as i16], col, uv: [(u + 1) * TILE_SIZE as u16, v * TILE_SIZE as u16] });
                vertices.push(Vertex { pos: [(tile.x + 1) * TILE_SIZE as i16, (tile.y + 1) * TILE_SIZE as i16], col, uv: [(u + 1) * TILE_SIZE as u16, (v + 1) * TILE_SIZE as u16] });
                vertices.push(Vertex { pos: [tile.x * TILE_SIZE as i16, (tile.y + 1) * TILE_SIZE as i16], col, uv: [u * TILE_SIZE as u16, (v + 1) * TILE_SIZE as u16] });
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

                for row in 0..TILE_SIZE {
                    for col in 0..TILE_SIZE {
                        data[v as usize * TILE_SIZE * ATLAS_SIZE + row * ATLAS_SIZE + u as usize * TILE_SIZE + col] = entry.tiles.data[tile.index + row * TILE_SIZE + col];
                    }
                }

                u += 1;
                if u as usize == ATLAS_SIZE / TILE_SIZE {
                    u = 0;
                    v += 1;
                }
            }

            for span in entry.tiles.spans.iter() {
                let base = vertices.len() as u16;
                vertices.push(Vertex { pos: [span.x * TILE_SIZE as i16, span.y * TILE_SIZE as i16], col, uv: [0, 0] });
                vertices.push(Vertex { pos: [(span.x + span.len) * TILE_SIZE as i16, span.y * TILE_SIZE as i16], col, uv: [0, 0] });
                vertices.push(Vertex { pos: [(span.x + span.len) * TILE_SIZE as i16, (span.y + 1) * TILE_SIZE as i16], col, uv: [0, 0] });
                vertices.push(Vertex { pos: [span.x * TILE_SIZE as i16, (span.y + 1) * TILE_SIZE as i16], col, uv: [0, 0] });
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }

        backend.upload(0, 0, ATLAS_SIZE as u32, ATLAS_SIZE as u32, &data);
        backend.draw(&vertices[..], &indices[..], 800, 600);
    }
}

struct Slab<T> {
    next: usize,
    buffer: Vec<Entry<T>>,
}

enum Entry<T> {
    Full(T),
    Empty(usize),
}

impl<T> Slab<T> {
    pub fn new() -> Slab<T> {
        Slab {
            next: 0,
            buffer: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if let Some(Entry::Full(value)) = self.buffer.get(index) {
            return Some(value);
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if let Some(Entry::Full(value)) = self.buffer.get_mut(index) {
            Some(value)
        } else {
            None
        }
    }

    pub fn add(&mut self, value: T) -> usize {
        let index = self.next;
        if let Some(entry) = self.buffer.get_mut(self.next) {
            if let Entry::Empty(next) = std::mem::replace(entry, Entry::Full(value)) {
                self.next = next;
            } else {
                unreachable!()
            }
        } else {
            self.buffer.push(Entry::Full(value));
            self.next = self.buffer.len();
        }
        index
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if let Some(entry) = self.buffer.get_mut(index) {
            if let Entry::Full(value) = std::mem::replace(entry, Entry::Empty(self.next)) {
                self.next = index;
                Some(value)
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }
}
