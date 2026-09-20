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
}

impl Cursors {
    pub fn update_cursor(
        &mut self,
        cursor_grid_x: i32,
        cursor_grid_y: i32,
        is_hovering_selectable_unit: bool,
        is_unit_selected: bool,
    ) {
        self.cursor_type = if is_unit_selected {
            CursorType::Click
        } else if is_hovering_selectable_unit {
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
