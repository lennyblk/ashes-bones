use crate::game::Game;
use crate::game_mode::TurnPhase;
use crate::movement::{self, MovementRange};
use crate::turn;
use crate::unit::{Faction, PendingAction, UnitState};

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

/// une commande reçue de l'adversaire est-elle jouable telle quelle ? Le réseau peut
/// apporter n'importe quoi : on vérifie tout avant apply, qui suppose une commande légale
pub fn is_legal(game: &Game, command: Command, faction: Faction) -> bool {
    // une unité de `faction`, vivante, prête à agir
    let own_unit = |idx: usize| {
        game.units
            .get(idx)
            .is_some_and(|u| u.faction == faction && u.is_alive() && u.state == UnitState::Idle)
    };
    match command {
        Command::Act { unit, to, action } => {
            if !own_unit(unit) {
                return false;
            }
            let (tiles, _) = movement::reachable(&game.units, unit, &game.blocked_tiles);
            if !tiles.contains(&to) {
                return false;
            }
            let Some(action) = action else {
                return true; // simple déplacement
            };
            let me = &game.units[unit];
            let Some(target) = game.units.get(action.target_idx()) else {
                return false;
            };
            let right_target = match action {
                PendingAction::Attack(_) => target.faction != faction,
                PendingAction::Heal(t) => {
                    me.can_heal
                        && t != unit
                        && target.faction == faction
                        && target.hp_points < target.hp_max_points
                }
            };
            !me.has_attacked
                && target.is_alive()
                && right_target
                && distance(to, (target.grid_x, target.grid_y)) <= me.attack_range
        }
        Command::Wait { unit } => own_unit(unit),
        Command::EndTurn => true,
    }
}

fn distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

pub fn apply(game: &mut Game, command: Command) {
    match command {
        Command::Act { unit, to, action } => {
            let (_, came_from) = movement::reachable(&game.units, unit, &game.blocked_tiles);
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
