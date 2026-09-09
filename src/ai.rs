use crate::movement::MovementRange;
use crate::unit::{Unit, UnitState};
use crate::{GRID_COLS, GRID_ROWS};

pub fn ai_move_toward_target(unit: &mut Unit, target: &Unit, blocked_tiles: &Vec<(i32, i32)>) {
    let (reachable, came_from) = MovementRange::compute_movement_range(
        unit.grid_x,
        unit.grid_y,
        unit.move_points,
        GRID_COLS,
        GRID_ROWS,
        target.grid_x,
        target.grid_y,
        target,
        blocked_tiles,
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

        // attaque automatique à l'arrivée si la case d'arrivée est à portée
        let distance_at_arrival =
            (destination.0 - target.grid_x).abs() + (destination.1 - target.grid_y).abs();
        unit.attack_target = distance_at_arrival <= unit.attack_range;

        unit.path = path;
        unit.state = UnitState::Walking;
    }
}

pub fn find_closest_target<'a>(unit: &Unit, candidates: &'a [Unit]) -> Option<&'a Unit> {
    candidates
        .iter()
        .filter(|c| c.is_alive())
        .min_by_key(|c| (c.grid_x - unit.grid_x).abs() + (c.grid_y - unit.grid_y).abs())
}

pub fn ai_attack_if_in_range(attacker: &mut Unit, defender: &Unit) {
    if !attacker.path.is_empty() || attacker.state != UnitState::Idle {
        return; // encore en train de bouger, ou déjà en train de faire autre chose
    }

    let distance =
        (defender.grid_x - attacker.grid_x).abs() + (defender.grid_y - attacker.grid_y).abs();

    if distance <= attacker.attack_range {
        attacker.state = UnitState::Attacking;
        attacker.attack_target = true;
    }
}

pub fn take_turn(unit: &mut Unit, targets: &[Unit], blocked_tiles: &Vec<(i32, i32)>) {
    unit.start_turn();
    if let Some(target) = find_closest_target(unit, targets) {
        ai_attack_if_in_range(unit, target);
        if unit.state != UnitState::Attacking {
            ai_move_toward_target(unit, target, blocked_tiles);
        }
    }
}
