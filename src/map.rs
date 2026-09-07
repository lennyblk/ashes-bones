use crate::TILE_SIZE;
use raylib::prelude::*;
use tiled::Loader;

#[derive(Clone, Copy)]
pub struct PlacedTile {
    pub tileset_index: usize,
    pub id: u32,
    pub flip_h: bool,
    pub flip_v: bool,
}

/// Un calque de tuiles
pub struct TileLayer {
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<Option<PlacedTile>>,
}

/// Texture + infos de découpe d'un tileset.
pub struct TilesetTexture {
    pub texture: Texture2D,
    pub tile_width: i32,
    pub tile_height: i32,
    pub columns: i32,
    pub spacing: i32,
    pub margin: i32,
}

pub struct TileMap {
    pub tilesets: Vec<TilesetTexture>,
    pub layers: Vec<TileLayer>,
    pub blocked_tiles: Vec<(i32, i32)>,
}

impl TileMap {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread, path: &str) -> TileMap {
        let mut loader = Loader::new();
        let map = loader.load_tmx_map(path).unwrap();

        // tilesets -> textures ---------------------------------------------
        let mut tilesets: Vec<TilesetTexture> = Vec::new();
        for tileset in map.tilesets() {
            let image = tileset
                .image
                .as_ref()
                .unwrap_or_else(|| panic!("tileset '{}' sans image", tileset.name));
            let image_path = image.source.to_string_lossy().to_string();
            let texture = rl
                .load_texture(thread, &image_path)
                .unwrap_or_else(|e| panic!("impossible de charger {image_path}: {e}"));
            tilesets.push(TilesetTexture {
                texture,
                tile_width: tileset.tile_width as i32,
                tile_height: tileset.tile_height as i32,
                columns: tileset.columns as i32,
                spacing: tileset.spacing as i32,
                margin: tileset.margin as i32,
            });
        }

        // calques + tuiles bloquées ----------------------------------------
        let mut layers: Vec<TileLayer> = Vec::new();
        let mut blocked_tiles: Vec<(i32, i32)> = Vec::new();

        for layer in map.layers() {
            let Some(tile_layer) = layer.as_tile_layer() else {
                continue;
            };
            let width = tile_layer.width().unwrap() as i32;
            let height = tile_layer.height().unwrap() as i32;

            let is_blocked_layer = layer
                .properties
                .get("blocked")
                .map_or(false, |v| v == &tiled::PropertyValue::BoolValue(true));

            let mut tiles: Vec<Option<PlacedTile>> = Vec::with_capacity((width * height) as usize);
            for y in 0..height {
                for x in 0..width {
                    let placed = tile_layer.get_tile(x, y).map(|tile| PlacedTile {
                        tileset_index: tile.tileset_index(),
                        id: tile.id(),
                        flip_h: tile.flip_h,
                        flip_v: tile.flip_v,
                    });
                    if placed.is_some() && is_blocked_layer {
                        blocked_tiles.push((x, y));
                    }
                    tiles.push(placed);
                }
            }

            layers.push(TileLayer {
                name: layer.name.clone(),
                width,
                height,
                tiles,
            });
        }

        TileMap {
            tilesets,
            layers,
            blocked_tiles,
        }
    }

    /// chaque tuile mise à l'échelle sur TILE_SIZE.
    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        for layer in &self.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let Some(tile) = layer.tiles[(y * layer.width + x) as usize] else {
                        continue;
                    };
                    let tileset = &self.tilesets[tile.tileset_index];

                    let col = tile.id as i32 % tileset.columns;
                    let row = tile.id as i32 / tileset.columns;
                    let src_x = tileset.margin + col * (tileset.tile_width + tileset.spacing);
                    let src_y = tileset.margin + row * (tileset.tile_height + tileset.spacing);

                    let mut source = Rectangle::new(
                        src_x as f32,
                        src_y as f32,
                        tileset.tile_width as f32,
                        tileset.tile_height as f32,
                    );
                    if tile.flip_h {
                        source.width = -source.width;
                    }
                    if tile.flip_v {
                        source.height = -source.height;
                    }

                    let dest = Rectangle::new(
                        (x * TILE_SIZE) as f32,
                        (y * TILE_SIZE) as f32,
                        TILE_SIZE as f32,
                        TILE_SIZE as f32,
                    );

                    d.draw_texture_pro(
                        &tileset.texture,
                        source,
                        dest,
                        Vector2::zero(),
                        0.0,
                        Color::WHITE,
                    );
                }
            }
        }
    }
}
