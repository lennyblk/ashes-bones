use crate::TILE_SIZE;
use raylib::prelude::*;

pub struct Cursors<'a> {
    pub current_cursor_texture: &'a Texture2D,
    pub position: Vector2,
    pub is_selected: bool,
}

impl<'a> Cursors<'a> {
    pub fn update_cursor(
        &mut self,
        normal_texture: &'a Texture2D,
        hover_texture: &'a Texture2D,
        click_texture: &'a Texture2D,
        cursor_grid_x: i32,
        cursor_grid_y: i32,
        char_grid_x: i32,
        char_grid_y: i32,
        mouse_just_clicked: bool,
    ) {
        let is_hovering = cursor_grid_x == char_grid_x && cursor_grid_y == char_grid_y;

        if mouse_just_clicked {
            if is_hovering {
                self.is_selected = !self.is_selected;
            } else {
                self.is_selected = false;
            }
        }

        self.current_cursor_texture = if self.is_selected {
            click_texture
        } else if is_hovering {
            hover_texture
        } else {
            normal_texture
        };

        self.position = Vector2::new(
            (cursor_grid_x * TILE_SIZE) as f32,
            (cursor_grid_y * TILE_SIZE) as f32,
        );
    }
}
