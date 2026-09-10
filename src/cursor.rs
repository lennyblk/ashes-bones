use crate::TILE_SIZE;
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CursorType {
    Normal,
    Hover,
    Click,
}

pub struct Cursors {
    pub cursor_type: CursorType,
    pub position: Vector2,
    pub is_selected: bool,
}

impl Cursors {
    pub fn update_cursor(
        &mut self,
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

        self.cursor_type = if self.is_selected {
            CursorType::Click
        } else if is_hovering {
            CursorType::Hover
        } else {
            CursorType::Normal
        };

        self.position = Vector2::new(
            (cursor_grid_x * TILE_SIZE) as f32,
            (cursor_grid_y * TILE_SIZE) as f32,
        );
    }
}
