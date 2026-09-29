use crate::movement::MovementRange;
use crate::unit::{PendingAction, Unit, UnitState};
use crate::{GRID_COLS, GRID_ROWS};

/// sous ce ratio de PV, un allié est considéré comme à soigner
const AI_HEAL_THRESHOLD: f32 = 0.5;

pub fn ai_move_toward_target(
    unit: &mut Unit,
    target: &Unit,
    action: PendingAction,
    obstacles: &[Unit],
    blocked_tiles: &Vec<(i32, i32)>,
) {
    let mut occupied = blocked_tiles.clone();
    for other in obstacles {
        if other.is_alive() {
            occupied.push((other.grid_x, other.grid_y));
        }
    }

    let (reachable, came_from) = MovementRange::compute_movement_range(
        unit.grid_x,
        unit.grid_y,
        unit.move_points,
        GRID_COLS,
        GRID_ROWS,
        &occupied,
    );

    // min_by_key renvoie Option<&(i32,i32)> : Some(case) si reachable n'est pas vide, None sinon
    let closest = reachable
        .iter()
        .min_by_key(|(x, y)| (x - target.grid_x).abs() + (y - target.grid_y).abs());

    if let Some(&destination) = closest {
        let path =
            MovementRange::build_waypoints(&came_from, (unit.grid_x, unit.grid_y), destination);
        if path.is_empty() {
            return; // déjà sur la meilleure case, rien à faire
        }

        // attaque/soin automatique à l'arrivée si la case d'arrivée est à portée
        let distance_at_arrival =
            (destination.0 - target.grid_x).abs() + (destination.1 - target.grid_y).abs();
        unit.pending_action = (distance_at_arrival <= unit.attack_range).then_some(action);

        unit.path = path;
        unit.state = UnitState::Walking;
    }
}

/// cherche la cible la plus proche parmi `candidates` (index dans `units`, unité)
pub fn find_closest_target<'a>(
    unit: &Unit,
    candidates: &'a [(usize, Unit)],
) -> Option<&'a (usize, Unit)> {
    candidates
        .iter()
        .filter(|(_, c)| c.is_alive())
        .min_by_key(|(_, c)| (c.grid_x - unit.grid_x).abs() + (c.grid_y - unit.grid_y).abs())
}

/// parmi les alliés blessés, prend celui avec le ratio de PV le plus bas (sous le seuil)
pub fn find_ally_to_heal(wounded_allies: &[(usize, Unit)]) -> Option<&(usize, Unit)> {
    let hp_ratio = |u: &Unit| u.hp_points as f32 / u.hp_max_points as f32;
    wounded_allies
        .iter()
        .filter(|(_, a)| a.is_alive() && hp_ratio(a) < AI_HEAL_THRESHOLD)
        .min_by(|(_, a), (_, b)| hp_ratio(a).total_cmp(&hp_ratio(b)))
}

pub fn ai_act_if_in_range(unit: &mut Unit, target: &Unit, action: PendingAction) {
    if !unit.path.is_empty() || unit.state != UnitState::Idle {
        return; // encore en train de bouger, ou déjà en train de faire autre chose
    }

    let distance = (target.grid_x - unit.grid_x).abs() + (target.grid_y - unit.grid_y).abs();

    if distance <= unit.attack_range {
        unit.state = action.to_state();
        unit.pending_action = Some(action);
    }
}

/// ordre de priorité : soigner à portée > attaquer à portée > marcher vers un blessé > marcher vers un ennemi
pub fn take_turn(
    unit: &mut Unit,
    targets: &[(usize, Unit)],
    wounded_allies: &[(usize, Unit)],
    obstacles: &[Unit],
    blocked_tiles: &Vec<(i32, i32)>,
) {
    unit.start_turn();

    let enemy = find_closest_target(unit, targets);
    let ally = if unit.can_heal {
        find_ally_to_heal(wounded_allies)
    } else {
        None
    };

    if let Some((ally_idx, ally)) = ally {
        ai_act_if_in_range(unit, ally, PendingAction::Heal(*ally_idx));
    }
    if let Some((enemy_idx, enemy)) = enemy {
        ai_act_if_in_range(unit, enemy, PendingAction::Attack(*enemy_idx));
    }
    if unit.state == UnitState::Idle {
        if let Some((ally_idx, ally)) = ally {
            ai_move_toward_target(
                unit,
                ally,
                PendingAction::Heal(*ally_idx),
                obstacles,
                blocked_tiles,
            );
        }
    }
    if unit.state == UnitState::Idle {
        if let Some((enemy_idx, enemy)) = enemy {
            ai_move_toward_target(
                unit,
                enemy,
                PendingAction::Attack(*enemy_idx),
                obstacles,
                blocked_tiles,
            );
        }
    }
}
