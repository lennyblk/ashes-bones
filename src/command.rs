use crate::game::Game;
use crate::game_mode::TurnPhase;
use crate::movement::{self, MovementRange};
use crate::turn;
use crate::unit::{PendingAction, UnitState};

#[derive(Clone, Copy)]
pub enum Command {
    Act {
        unit: usize,
        to: (i32, i32), // emplacement pour faire l'action
        action: Option<PendingAction>,
    },
    Wait {
        unit: usize,
    },
    EndTurn,
}

pub fn apply(game: &mut Game, command: Command) {
    match command {
        Command::Act { unit, to, action } => {
            let (tiles, came_from) = movement::reachable(&game.units, unit, &game.blocked_tiles);
            // case hors de portée (plus tard : message réseau faux) -> on ignore
            if !tiles.contains(&to) {
                return;
            }
            let u = &mut game.units[unit];
            let from = (u.grid_x, u.grid_y);
            u.move_points_remaining -= MovementRange::path_cost(&came_from, from, to);
            u.path = MovementRange::build_waypoints(&came_from, from, to);
            u.pending_action = action;
            // l'action se déclenche à l'arrivée (advance_path), ou tout de suite sur place
            u.state = if !u.path.is_empty() {
                UnitState::Walking
            } else {
                action.map_or(UnitState::Idle, PendingAction::to_state)
            };
        }
        Command::Wait { unit } => game.units[unit].wait(),
        Command::EndTurn => match game.current_turn {
            TurnPhase::PlayerTurn => turn::start_enemy_turn(game),
            TurnPhase::EnemyTurn => turn::start_player_turn(game),
        },
    }
}
