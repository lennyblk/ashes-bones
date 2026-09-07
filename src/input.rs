use crate::cursor::Cursors;
use crate::human::{Human, HumanState};
use crate::movement::MovementRange;
use crate::undead::Undead;
use raylib::consts::KeyboardKey::*;
use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;

pub fn handle_movement_normal_click(
    rl: &RaylibHandle,
    human: &mut Human,
    cursor: &mut Cursors,
    move_range: &Vec<(i32, i32)>,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && cursor.is_selected
        && human.state == HumanState::Idle
    {
        if move_range.contains(&(cursor_grid_x, cursor_grid_y))
            && (cursor_grid_x, cursor_grid_y) != (human.grid_x, human.grid_y)
        {
            let waypoints = MovementRange::build_waypoints(
                came_from,
                (human.grid_x, human.grid_y),
                (cursor_grid_x, cursor_grid_y),
            );
            human.state = HumanState::Walking;
            human.attack_target = false;
            human.path = waypoints;
            cursor.is_selected = false;
            return true;
        }
    }
    false
}

pub fn handle_movement_attack_click(
    rl: &RaylibHandle,
    human: &mut Human,
    cursor: &mut Cursors,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    valid_attack_positions: &Vec<(i32, i32)>,
    enemy_attackable: bool,
    enemy: &Undead,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && cursor.is_selected
        && human.state == HumanState::Idle
        && enemy_attackable
        && cursor_grid_x == enemy.grid_x
        && cursor_grid_y == enemy.grid_y
    {
        let current_distance =
            (enemy.grid_x - human.grid_x).abs() + (enemy.grid_y - human.grid_y).abs();
        if current_distance <= human.attack_range {
            cursor.is_selected = false;
            human.state = HumanState::Combat;
        } else if valid_attack_positions.len() == 1 {
            let waypoints = MovementRange::build_waypoints(
                came_from,
                (human.grid_x, human.grid_y),
                valid_attack_positions[0],
            );
            human.attack_target = true;
            human.state = HumanState::Walking;
            human.path = waypoints;
            cursor.is_selected = false;
        } else {
            human.state = HumanState::ChoosingPosition;
        }
        return true;
    }
    false
}

pub fn handle_movement_choosing_position_click(
    rl: &RaylibHandle,
    human: &mut Human,
    cursor: &mut Cursors,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    valid_attack_positions: &Vec<(i32, i32)>,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && human.state == HumanState::ChoosingPosition
        && valid_attack_positions.contains(&(cursor_grid_x, cursor_grid_y))
    {
        let waypoints = MovementRange::build_waypoints(
            came_from,
            (human.grid_x, human.grid_y),
            (cursor_grid_x, cursor_grid_y),
        );
        human.attack_target = true;
        human.state = HumanState::Walking;
        human.path = waypoints;
        cursor.is_selected = false;
        return true;
    }
    false
}

pub fn cancel_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KEY_B)
}
