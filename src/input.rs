use crate::character::{Character, CharacterState};
use crate::cursor::Cursors;
use crate::enemy::Enemy;
use crate::movement::MovementRange;
use raylib::consts::KeyboardKey::*;
use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;

pub fn handle_movement_normal_click(
    rl: &RaylibHandle,
    character: &mut Character,
    cursor: &mut Cursors,
    move_range: &Vec<(i32, i32)>,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && cursor.is_selected
        && character.state == CharacterState::Idle
    {
        if move_range.contains(&(cursor_grid_x, cursor_grid_y)) {
            let waypoints = MovementRange::build_waypoints(
                came_from,
                (character.grid_x, character.grid_y),
                (cursor_grid_x, cursor_grid_y),
            );
            character.state = CharacterState::Walking;
            character.attack_target = false;
            character.path = waypoints;
            cursor.is_selected = false;
            return true;
        }
    }
    false
}

pub fn handle_movement_attack_click(
    rl: &RaylibHandle,
    character: &mut Character,
    cursor: &mut Cursors,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    valid_attack_positions: &Vec<(i32, i32)>,
    enemy_attackable: bool,
    enemy: &Enemy,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && cursor.is_selected
        && character.state == CharacterState::Idle
        && enemy_attackable
        && cursor_grid_x == enemy.grid_x
        && cursor_grid_y == enemy.grid_y
    {
        let current_distance =
            (enemy.grid_x - character.grid_x).abs() + (enemy.grid_y - character.grid_y).abs();
        if current_distance <= character.attack_range {
            cursor.is_selected = false;
            character.state = CharacterState::Combat;
        } else if valid_attack_positions.len() == 1 {
            let waypoints = MovementRange::build_waypoints(
                came_from,
                (character.grid_x, character.grid_y),
                valid_attack_positions[0],
            );
            character.attack_target = true;
            character.state = CharacterState::Walking;
            character.path = waypoints;
        } else {
            character.state = CharacterState::ChoosingPosition;
        }
        return true;
    }
    false
}

pub fn handle_movement_choosing_position_click(
    rl: &RaylibHandle,
    character: &mut Character,
    cursor: &mut Cursors,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    valid_attack_positions: &Vec<(i32, i32)>,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && character.state == CharacterState::ChoosingPosition
        && valid_attack_positions.contains(&(cursor_grid_x, cursor_grid_y))
    {
        let waypoints = MovementRange::build_waypoints(
            came_from,
            (character.grid_x, character.grid_y),
            (cursor_grid_x, cursor_grid_y),
        );
        character.attack_target = true;
        character.state = CharacterState::Walking;
        character.path = waypoints;
        cursor.is_selected = false;
        return true;
    }
    false
}

pub fn cancel_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KEY_B)
}
