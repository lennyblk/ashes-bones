use std::collections::HashMap;

use crate::duel::{self, DuelSide};
use crate::movement::MovementRange;
use crate::unit::{PendingAction, Unit, UnitState};
use crate::{GRID_COLS, GRID_ROWS};

/// sous ce ratio de PV, un allié est considéré comme à soigner
const AI_HEAL_THRESHOLD: f32 = 0.5;

// poids de la note d'une attaque (cible + case de départ). Plus c'est haut, mieux c'est
const SCORE_DAMAGE: f32 = 100.0; // x (dégâts attendus / PV de la cible), plafonné à 1
const SCORE_KILL_LIKELY: f32 = 60.0; // les dégâts attendus suffisent à tuer
const SCORE_KILL_POSSIBLE: f32 = 25.0; // tue seulement si le joueur rate sa parade
const SCORE_TARGET_THREAT: f32 = 0.2; // x attaque de la cible : on vise les gros frappeurs
const SCORE_GUARDED_TILE: f32 = 8.0; // par allié collé à la case d'arrivée (gardé au tour suivant)
const SCORE_SHOOTER_EXPOSED: f32 = 15.0; // tireur qui finit collé à un ennemi
const SCORE_STEP: f32 = 0.5; // par case parcourue, pour départager à note égale

/// ce que l'unité IA fait ce tour-ci
pub struct Plan {
    // cases de passage (vide = on agit sur place)
    pub path: Vec<(i32, i32)>,
    pub action: Option<PendingAction>,
}

/// applique un plan : marche puis action à l'arrivée, ou action tout de suite
pub fn apply_plan(unit: &mut Unit, plan: Plan) {
    unit.pending_action = plan.action;
    if !plan.path.is_empty() {
        unit.path = plan.path;
        unit.state = UnitState::Walking;
    } else if let Some(action) = plan.action {
        unit.state = action.to_state();
    }
}

fn distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn pos(unit: &Unit) -> (i32, i32) {
    (unit.grid_x, unit.grid_y)
}

/// cases atteignables ce tour (case actuelle comprise), les autres unités vivantes bloquent
fn reachable(
    units: &[Unit],
    idx: usize,
    blocked_tiles: &[(i32, i32)],
) -> (Vec<(i32, i32)>, HashMap<(i32, i32), (i32, i32)>) {
    let mut occupied = blocked_tiles.to_vec();
    for (i, other) in units.iter().enumerate() {
        if i != idx && other.is_alive() {
            occupied.push(pos(other));
        }
    }
    let unit = &units[idx];
    MovementRange::compute_movement_range(
        unit.grid_x,
        unit.grid_y,
        unit.move_points_remaining,
        GRID_COLS,
        GRID_ROWS,
        &occupied,
    )
}

fn plan_to(
    units: &[Unit],
    idx: usize,
    came_from: &HashMap<(i32, i32), (i32, i32)>,
    destination: (i32, i32),
    action: Option<PendingAction>,
) -> Plan {
    Plan {
        path: MovementRange::build_waypoints(came_from, pos(&units[idx]), destination),
        action,
    }
}

/// multiplicateur de dégâts attendu quand le joueur pare avec cette barre
pub fn expected_parry_multiplier(side: &DuelSide) -> f32 {
    let single_try = (side.window / side.tempo * 2.5).clamp(0.0, 0.95);
    let hit = 1.0 - (1.0 - single_try).powi(side.lives as i32 + 1);
    let perfect = hit * (2.0 * side.precision / side.window).min(1.0);
    let good = hit - perfect;
    let bad = 1.0 - hit;
    perfect * 0.5 + good * 1.0 + bad * 1.5
}

/// note de l'attaque de `idx` sur `target_idx` depuis `from`
pub fn attack_score(units: &[Unit], idx: usize, from: (i32, i32), target_idx: usize) -> f32 {
    let me = &units[idx];
    let target = &units[target_idx];
    // c'est le joueur qui pare : seule la barre du défenseur compte pour l'IA
    let conditions = duel::compute(units, idx, from, target_idx);
    let base = conditions.base_damage as f32;
    let expected = base * expected_parry_multiplier(&conditions.defender);
    let hp = target.hp_points.max(1) as f32;

    let mut score = SCORE_DAMAGE * (expected / hp).min(1.0);
    if expected >= hp {
        score += SCORE_KILL_LIKELY;
    } else if base * 1.5 >= hp {
        score += SCORE_KILL_POSSIBLE;
    }
    score += SCORE_TARGET_THREAT * target.attack_power as f32;

    // au tour suivant c'est le joueur qui attaquera cette case : on veut y être gardé
    let neighbors = units
        .iter()
        .enumerate()
        .filter(|(i, u)| *i != idx && u.is_alive() && distance(pos(u), from) == 1);
    let (allies, enemies): (Vec<_>, Vec<_>) = neighbors.partition(|(_, u)| u.faction == me.faction);
    score += SCORE_GUARDED_TILE * allies.len().min(duel::MAX_LIVES as usize) as f32;
    if me.attack_range >= 2 && !enemies.is_empty() {
        score -= SCORE_SHOOTER_EXPOSED;
    }
    score
}

/// meilleure attaque possible ce tour : toutes les cibles x toutes les cases d'où frapper
fn best_attack(
    units: &[Unit],
    idx: usize,
    tiles: &[(i32, i32)],
    came_from: &HashMap<(i32, i32), (i32, i32)>,
) -> Option<Plan> {
    let me = &units[idx];
    let start = pos(me);
    let mut best: Option<(f32, usize, (i32, i32))> = None;
    for (target_idx, target) in units.iter().enumerate() {
        if target.faction == me.faction || !target.is_alive() {
            continue;
        }
        for &tile in tiles {
            if distance(tile, pos(target)) > me.attack_range {
                continue;
            }
            let steps = MovementRange::path_cost(came_from, start, tile) as f32;
            let score = attack_score(units, idx, tile, target_idx) - SCORE_STEP * steps;
            if best.is_none_or(|(best_score, _, _)| score > best_score) {
                best = Some((score, target_idx, tile));
            }
        }
    }
    best.map(|(_, target_idx, tile)| {
        plan_to(
            units,
            idx,
            came_from,
            tile,
            Some(PendingAction::Attack(target_idx)),
        )
    })
}

/// pas d'attaque possible : on se rapproche de l'ennemi le plus proche,
/// en préférant à distance égale une case collée à un allié
fn approach_closest_enemy(
    units: &[Unit],
    idx: usize,
    tiles: &[(i32, i32)],
    came_from: &HashMap<(i32, i32), (i32, i32)>,
) -> Option<Plan> {
    let me = &units[idx];
    let target = units
        .iter()
        .filter(|u| u.faction != me.faction && u.is_alive())
        .min_by_key(|u| distance(pos(u), pos(me)))?;
    let allies_next_to = |tile: (i32, i32)| {
        units
            .iter()
            .enumerate()
            .filter(|(i, u)| {
                *i != idx && u.is_alive() && u.faction == me.faction && distance(pos(u), tile) == 1
            })
            .count() as i32
    };
    let destination = tiles
        .iter()
        .min_by_key(|&&tile| (distance(tile, pos(target)), -allies_next_to(tile)))?;
    Some(plan_to(units, idx, came_from, *destination, None))
}

/// parmi les alliés blessés, prend celui avec le ratio de PV le plus bas (sous le seuil)
fn find_ally_to_heal(units: &[Unit], idx: usize) -> Option<usize> {
    let hp_ratio = |u: &Unit| u.hp_points as f32 / u.hp_max_points as f32;
    let me = &units[idx];
    units
        .iter()
        .enumerate()
        .filter(|(i, a)| {
            *i != idx && a.faction == me.faction && a.is_alive() && hp_ratio(a) < AI_HEAL_THRESHOLD
        })
        .min_by(|(_, a), (_, b)| hp_ratio(a).total_cmp(&hp_ratio(b)))
        .map(|(i, _)| i)
}

/// ordre de priorité : soigner à portée > meilleure attaque (en bougeant si besoin)
/// > marcher vers un blessé > se rapprocher d'un ennemi
pub fn plan_turn(units: &[Unit], idx: usize, blocked_tiles: &[(i32, i32)]) -> Plan {
    let me = &units[idx];
    let (tiles, came_from) = reachable(units, idx, blocked_tiles);
    let heal_target = if me.can_heal {
        find_ally_to_heal(units, idx)
    } else {
        None
    };

    if let Some(ally_idx) = heal_target {
        if distance(pos(me), pos(&units[ally_idx])) <= me.attack_range {
            return Plan {
                path: Vec::new(),
                action: Some(PendingAction::Heal(ally_idx)),
            };
        }
    }
    if let Some(plan) = best_attack(units, idx, &tiles, &came_from) {
        return plan;
    }
    if let Some(ally_idx) = heal_target {
        let ally_pos = pos(&units[ally_idx]);
        if let Some(&destination) = tiles.iter().min_by_key(|&&t| distance(t, ally_pos)) {
            let in_range = distance(destination, ally_pos) <= me.attack_range;
            let action = in_range.then_some(PendingAction::Heal(ally_idx));
            return plan_to(units, idx, &came_from, destination, action);
        }
    }
    approach_closest_enemy(units, idx, &tiles, &came_from).unwrap_or(Plan {
        path: Vec::new(),
        action: None,
    })
}
