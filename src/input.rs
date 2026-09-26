use crate::movement::MovementRange;
use crate::unit::{Unit, UnitState};
use raylib::consts::KeyboardKey::*;
use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;

pub fn handle_movement_normal_click(
    rl: &RaylibHandle,
    unit: &mut Unit,
    move_range: &Vec<(i32, i32)>,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) && unit.state == UnitState::Idle {
        if move_range.contains(&(cursor_grid_x, cursor_grid_y))
            && (cursor_grid_x, cursor_grid_y) != (unit.grid_x, unit.grid_y)
        {
            let waypoints = MovementRange::build_waypoints(
                came_from,
                (unit.grid_x, unit.grid_y),
                (cursor_grid_x, cursor_grid_y),
            );
            unit.move_points_remaining -= MovementRange::path_cost(
                came_from,
                (unit.grid_x, unit.grid_y),
                (cursor_grid_x, cursor_grid_y),
            );
            unit.state = UnitState::Walking;
            unit.attack_target = None;
            unit.path = waypoints;
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
pub fn handle_movement_attack_click(
    rl: &RaylibHandle,
    unit: &mut Unit,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    valid_attack_positions: &Vec<(i32, i32)>,
    enemy_attackable: bool,
    enemy: &Unit,
    enemy_idx: usize,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && unit.state == UnitState::Idle
        && enemy_attackable
        && cursor_grid_x == enemy.grid_x
        && cursor_grid_y == enemy.grid_y
    {
        // on retient l'ennemi cliqué maintenant : peu importe combien de cases il faut
        // parcourir ou combien d'autres unités traînent autour, c'est lui qu'on ira frapper
        unit.attack_target = Some(enemy_idx);

        let current_distance =
            (enemy.grid_x - unit.grid_x).abs() + (enemy.grid_y - unit.grid_y).abs();
        if current_distance <= unit.attack_range {
            unit.state = UnitState::Attacking;
        } else if valid_attack_positions.len() == 1 {
            let waypoints = MovementRange::build_waypoints(
                came_from,
                (unit.grid_x, unit.grid_y),
                valid_attack_positions[0],
            );
            unit.move_points_remaining -= MovementRange::path_cost(
                came_from,
                (unit.grid_x, unit.grid_y),
                valid_attack_positions[0],
            );
            unit.state = UnitState::Walking;
            unit.path = waypoints;
        } else {
            unit.state = UnitState::ChoosingPosition;
        }
        return true;
    }
    false
}

pub fn handle_movement_choosing_position_click(
    rl: &RaylibHandle,
    unit: &mut Unit,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    valid_attack_positions: &Vec<(i32, i32)>,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> bool {
    if rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && unit.state == UnitState::ChoosingPosition
        && valid_attack_positions.contains(&(cursor_grid_x, cursor_grid_y))
    {
        let waypoints = MovementRange::build_waypoints(
            came_from,
            (unit.grid_x, unit.grid_y),
            (cursor_grid_x, cursor_grid_y),
        );
        unit.move_points_remaining -= MovementRange::path_cost(
            came_from,
            (unit.grid_x, unit.grid_y),
            (cursor_grid_x, cursor_grid_y),
        );
        // attack_target déjà posé par handle_movement_attack_click quand la ChoosingPosition
        // a commencé, on y touche pas ici
        unit.state = UnitState::Walking;
        unit.path = waypoints;
        return true;
    }
    false
}

pub fn cancel_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KEY_B)
}

pub fn is_button_clicked(
    mouse_position: Vector2,
    mouse_clicked: bool,
    button_rect: Rectangle,
) -> bool {
    mouse_clicked && button_rect.check_collision_point_rec(mouse_position)
}
