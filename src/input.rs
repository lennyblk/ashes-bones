use crate::command::Command;
use crate::unit::{PendingAction, Unit, UnitState};
use raylib::consts::KeyboardKey::*;
use raylib::consts::MouseButton::*;
use raylib::prelude::*;

pub fn handle_movement_normal_click(
    rl: &RaylibHandle,
    idx: usize,
    unit: &Unit,
    move_range: &[(i32, i32)],
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> Option<Command> {
    let cursor = (cursor_grid_x, cursor_grid_y);
    (rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && unit.state == UnitState::Idle
        && move_range.contains(&cursor)
        && cursor != (unit.grid_x, unit.grid_y))
        .then_some(Command::Act {
            unit: idx,
            to: cursor,
            action: None,
        })
}

#[allow(clippy::too_many_arguments)]
pub fn handle_movement_action_click(
    rl: &RaylibHandle,
    idx: usize,
    unit: &mut Unit,
    valid_positions: &[(i32, i32)],
    target_reachable: bool,
    target: &Unit,
    action: PendingAction,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> Option<Command> {
    if !(rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && unit.state == UnitState::Idle
        && target_reachable
        && cursor_grid_x == target.grid_x
        && cursor_grid_y == target.grid_y)
    {
        return None;
    }

    let current_distance =
        (target.grid_x - unit.grid_x).abs() + (target.grid_y - unit.grid_y).abs();
    if current_distance <= unit.attack_range {
        Some(Command::Act {
            unit: idx,
            to: (unit.grid_x, unit.grid_y),
            action: Some(action),
        })
    } else if valid_positions.len() == 1 {
        Some(Command::Act {
            unit: idx,
            to: valid_positions[0],
            action: Some(action),
        })
    } else {
        // plusieurs cases possibles : le joueur choisit. C'est de l'interface locale,
        // rien n'est encore joué donc pas de Command
        unit.pending_action = Some(action);
        unit.state = UnitState::ChoosingPosition;
        None
    }
}

pub fn handle_movement_choosing_position_click(
    rl: &RaylibHandle,
    idx: usize,
    unit: &Unit,
    valid_attack_positions: &[(i32, i32)],
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> Option<Command> {
    let cursor = (cursor_grid_x, cursor_grid_y);
    // pending_action posé par handle_movement_action_click quand le choix a commencé
    (rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        && unit.state == UnitState::ChoosingPosition
        && valid_attack_positions.contains(&cursor))
    .then(|| Command::Act {
        unit: idx,
        to: cursor,
        action: unit.pending_action,
    })
}

pub fn mouse_is_clicked(rl: &RaylibHandle) -> bool {
    rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
}

pub fn speed_toggle_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KEY_F)
}

pub fn cancel_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KEY_B)
}

pub fn pause_pressed(rl: &RaylibHandle) -> bool {
    rl.is_key_pressed(KEY_ESCAPE)
}

pub fn is_button_clicked(
    mouse_position: Vector2,
    mouse_clicked: bool,
    button_rect: Rectangle,
) -> bool {
    mouse_clicked && button_rect.check_collision_point_rec(mouse_position)
}
