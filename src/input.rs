use crate::human::{Human, HumanState};
use crate::cursor::Cursors;
use crate::undead::Undead;
use crate::movement::MovementRange;
use raylib::consts::KeyboardKey::*;
use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;

pub fn handle_movement_normal_click(
    rl: &RaylibHandle,
    soldier: &mut Human,
    cursor: &mut Cursors,
    move_range: &Vec<(i32, i32)>,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && cursor.is_selected
        && soldier.state == HumanState::Idle
    {
        if move_range.contains(&(cursor_grid_x, cursor_grid_y)) {
            let waypoints = MovementRange::build_waypoints(
                came_from,
                (soldier.grid_x, soldier.grid_y),
                (cursor_grid_x, cursor_grid_y),
            );
            soldier.state = HumanState::Walking;
            soldier.attack_target = false;
            soldier.path = waypoints;
            cursor.is_selected = false;
            return true;
        }
    }
    false
}

pub fn handle_movement_attack_click(
    rl: &RaylibHandle,
    soldier: &mut Human,
    cursor: &mut Cursors,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    valid_attack_positions: &Vec<(i32, i32)>,
    wraith_attackable: bool,
    wraith: &Undead,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && cursor.is_selected
        && soldier.state == HumanState::Idle
        && wraith_attackable
        && cursor_grid_x == wraith.grid_x
        && cursor_grid_y == wraith.grid_y
    {
        let current_distance =
            (wraith.grid_x - soldier.grid_x).abs() + (wraith.grid_y - soldier.grid_y).abs();
        if current_distance <= soldier.attack_range {
            cursor.is_selected = false;
            soldier.state = HumanState::Combat;
        } else if valid_attack_positions.len() == 1 {
            let waypoints = MovementRange::build_waypoints(
                came_from,
                (soldier.grid_x, soldier.grid_y),
                valid_attack_positions[0],
            );
            soldier.attack_target = true;
            soldier.state = HumanState::Walking;
            soldier.path = waypoints;
            cursor.is_selected = false;
        } else {
            soldier.state = HumanState::ChoosingPosition;
        }
        return true;
    }
    false
}

pub fn handle_movement_choosing_position_click(
    rl: &RaylibHandle,
    soldier: &mut Human,
    cursor: &mut Cursors,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    valid_attack_positions: &Vec<(i32, i32)>,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && soldier.state == HumanState::ChoosingPosition
        && valid_attack_positions.contains(&(cursor_grid_x, cursor_grid_y))
    {
        let waypoints = MovementRange::build_waypoints(
            came_from,
            (soldier.grid_x, soldier.grid_y),
            (cursor_grid_x, cursor_grid_y),
        );
        soldier.attack_target = true;
        soldier.state = HumanState::Walking;
        soldier.path = waypoints;
        cursor.is_selected = false;
        return true;
    }
    false
}

pub fn cancel_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KEY_B)
}
