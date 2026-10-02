use std::collections::HashMap;

use raylib::prelude::*;

use crate::duel::{self, DuelConditions};
use crate::game::Game;
use crate::game_mode::GameMode;
use crate::input;
use crate::movement::MovementRange;
use crate::unit::{PendingAction, Unit, UnitState, two_mut};
use crate::{GRID_COLS, GRID_ROWS};

/// cases à surligner sur la grille, calculées pendant les inputs et utilisées au dessin
pub struct GridHighlights {
    pub move_range: Vec<(i32, i32)>,
    // cases d'où l'unité sélectionnée peut attaquer/soigner (mode ChoosingPosition)
    pub choosing_positions: Vec<(i32, i32)>,
    pub attackable_enemy_positions: Vec<(i32, i32)>,
    pub healable_ally_positions: Vec<(i32, i32)>,
    // conditions du duel si on attaque l'ennemi survolé (ou depuis la case survolée)
    pub duel_preview: Option<DuelPreview>,
    // avantage net de chaque case d'attaque possible (mode ChoosingPosition)
    pub position_scores: Vec<((i32, i32), i32)>,
}

pub struct DuelPreview {
    pub attacker_idx: usize,
    pub defender_idx: usize,
    // case d'où partirait l'attaque
    pub from: (i32, i32),
    // plusieurs cases possibles : on montre la meilleure
    pub from_best_tile: bool,
    pub conditions: DuelConditions,
}

/// inputs du joueur sur la grille : sélection, déplacement, attaque, soin,
/// choix de la case d'arrivée, annulation
pub fn update(
    game: &mut Game,
    rl: &RaylibHandle,
    click_consumed: bool,
    cursor_grid_x: i32,
    cursor_grid_y: i32,
) -> GridHighlights {
    let (move_range, came_from) = if let Some(sel) = game.selected_unit {
        if game.game_mode == GameMode::GridScreen {
            // toute autre unité vivante (alliée ou ennemie) bloque le passage
            let mut occupied = game.blocked_tiles.clone();
            for (i, u) in game.units.iter().enumerate() {
                if i != sel && u.is_alive() {
                    occupied.push((u.grid_x, u.grid_y));
                }
            }
            MovementRange::compute_movement_range(
                game.units[sel].grid_x,
                game.units[sel].grid_y,
                game.units[sel].move_points_remaining,
                GRID_COLS,
                GRID_ROWS,
                &occupied,
            )
        } else {
            (Vec::new(), HashMap::new())
        }
    } else {
        (Vec::new(), HashMap::new())
    };

    let clicked = input::mouse_is_clicked(rl);

    // sélection / désélection d'une unité du joueur au clic sur sa case
    if game.game_mode == GameMode::GridScreen && clicked && !click_consumed {
        // clic sur un ennemi -> on affiche ses stats, clic ailleurs -> on les cache
        game.inspected_enemy = game.units.iter().position(|u| {
            u.faction != game.player_faction
                && u.is_alive()
                && u.grid_x == cursor_grid_x
                && u.grid_y == cursor_grid_y
        });

        match game.selected_unit {
            None => {
                game.selected_unit = game.units.iter().position(|u| {
                    u.faction == game.player_faction
                        && u.is_alive()
                        && u.state == UnitState::Idle
                        && u.grid_x == cursor_grid_x
                        && u.grid_y == cursor_grid_y
                });
            }
            Some(sel)
                if game.units[sel].grid_x == cursor_grid_x
                    && game.units[sel].grid_y == cursor_grid_y =>
            {
                game.selected_unit = None;
            }
            _ => {}
        }
    }

    // if pour bouger le personnage sélectionné
    if let Some(sel) = game.selected_unit {
        if input::handle_movement_normal_click(
            rl,
            &mut game.units[sel],
            &move_range,
            &came_from,
            cursor_grid_x,
            cursor_grid_y,
        ) {
            game.selected_unit = None;
        }
    }

    let (valid_attack_positions, attackable_enemy_positions): (Vec<(i32, i32)>, Vec<(i32, i32)>) =
        match game.selected_unit {
            Some(sel) if !game.units[sel].has_attacked => {
                reachable_targets(&game.units, &move_range, sel, |me, u| {
                    u.faction != me.faction && u.is_alive()
                })
            }
            _ => (Vec::new(), Vec::new()),
        };

    let (valid_heal_positions, healable_ally_positions): (Vec<(i32, i32)>, Vec<(i32, i32)>) =
        match game.selected_unit {
            Some(sel) if !game.units[sel].has_attacked && game.units[sel].can_heal => {
                reachable_targets(&game.units, &move_range, sel, |me, u| {
                    u.faction == me.faction
                        && u.is_alive()
                        && u.hp_points < u.hp_max_points
                        && (u.grid_x, u.grid_y) != (me.grid_x, me.grid_y)
                })
            }
            _ => (Vec::new(), Vec::new()),
        };

    // if pour attaquer l'ennemi et bouger le personnage sélectionné si il y a qu'une
    // seule case possible pour attaquer
    if let Some(sel) = game.selected_unit {
        let attacker_faction = game.units[sel].faction;
        if let Some(enemy_idx) = game.units.iter().position(|u| {
            u.faction != attacker_faction
                && u.is_alive()
                && u.grid_x == cursor_grid_x
                && u.grid_y == cursor_grid_y
        }) {
            let (mover, enemy) = two_mut(&mut game.units, sel, enemy_idx);
            if input::handle_movement_action_click(
                rl,
                mover,
                &came_from,
                &valid_attack_positions,
                !valid_attack_positions.is_empty(),
                enemy,
                PendingAction::Attack(enemy_idx),
                cursor_grid_x,
                cursor_grid_y,
            ) {
                if mover.state != UnitState::ChoosingPosition {
                    game.selected_unit = None;
                }
            }
        }
    }

    // pareil, mais pour soigner un allié blessé (Priest / Necromancer)
    if let Some(sel) = game.selected_unit {
        if game.units[sel].can_heal {
            let healer_faction = game.units[sel].faction;
            // pas soi-même : un soigneur blessé survolé se trouverait sinon comme cible,
            // et two_mut(sel, sel) panique
            if let Some(ally_idx) = game.units.iter().enumerate().position(|(i, u)| {
                i != sel
                    && u.faction == healer_faction
                    && u.is_alive()
                    && u.hp_points < u.hp_max_points
                    && u.grid_x == cursor_grid_x
                    && u.grid_y == cursor_grid_y
            }) {
                let (mover, ally) = two_mut(&mut game.units, sel, ally_idx);
                if input::handle_movement_action_click(
                    rl,
                    mover,
                    &came_from,
                    &valid_heal_positions,
                    !valid_heal_positions.is_empty(),
                    ally,
                    PendingAction::Heal(ally_idx),
                    cursor_grid_x,
                    cursor_grid_y,
                ) && mover.state != UnitState::ChoosingPosition
                {
                    game.selected_unit = None;
                }
            }
        }
    }

    // case d'arrivée à choisir : soin ou attaque selon l'action en attente
    let choosing_heal = matches!(
        game.selected_unit
            .and_then(|sel| game.units[sel].pending_action),
        Some(PendingAction::Heal(_))
    );
    let choosing_positions = if choosing_heal {
        valid_heal_positions
    } else {
        valid_attack_positions
    };

    if let Some(sel) = game.selected_unit {
        if input::handle_movement_choosing_position_click(
            rl,
            &mut game.units[sel],
            &came_from,
            &choosing_positions,
            cursor_grid_x,
            cursor_grid_y,
        ) {
            game.selected_unit = None;
        }
    }

    if let Some(sel) = game.selected_unit {
        if input::cancel_pressed(rl) && game.units[sel].state == UnitState::ChoosingPosition {
            game.units[sel].state = UnitState::Idle;
            game.selected_unit = None;
        }
    }

    let (duel_preview, position_scores) = build_duel_preview(
        game,
        &move_range,
        &choosing_positions,
        (cursor_grid_x, cursor_grid_y),
    );

    GridHighlights {
        move_range,
        choosing_positions,
        attackable_enemy_positions,
        healable_ally_positions,
        duel_preview,
        position_scores,
    }
}

fn distance(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

/// preview du duel pour l'unité sélectionnée :
/// - Idle + survol d'un ennemi attaquable -> conditions depuis la case actuelle (ou la meilleure)
/// - ChoosingPosition -> note chaque case possible, preview depuis la case survolée
fn build_duel_preview(
    game: &Game,
    move_range: &Vec<(i32, i32)>,
    choosing_positions: &[(i32, i32)],
    cursor: (i32, i32),
) -> (Option<DuelPreview>, Vec<((i32, i32), i32)>) {
    let none = (None, Vec::new());
    let Some(sel) = game.selected_unit else {
        return none;
    };
    let me = &game.units[sel];
    if game.game_mode != GameMode::GridScreen || me.has_attacked {
        return none;
    }
    let preview_from = |defender_idx: usize, from: (i32, i32), from_best_tile: bool| DuelPreview {
        attacker_idx: sel,
        defender_idx,
        from,
        from_best_tile,
        conditions: duel::compute(&game.units, sel, from, defender_idx),
    };

    match (me.state, me.pending_action) {
        (UnitState::ChoosingPosition, Some(PendingAction::Attack(target))) => {
            let target_pos = (game.units[target].grid_x, game.units[target].grid_y);
            let scores: Vec<((i32, i32), i32)> = choosing_positions
                .iter()
                .filter(|&&p| distance(p, target_pos) <= me.attack_range)
                .map(|&p| (p, duel::compute(&game.units, sel, p, target).score()))
                .collect();
            let preview = scores
                .iter()
                .any(|(p, _)| *p == cursor)
                .then(|| preview_from(target, cursor, false));
            (preview, scores)
        }
        (UnitState::Idle, _) => {
            let Some(target) = game.units.iter().position(|u| {
                u.faction != me.faction && u.is_alive() && (u.grid_x, u.grid_y) == cursor
            }) else {
                return none;
            };
            let here = (me.grid_x, me.grid_y);
            if distance(here, cursor) <= me.attack_range {
                return (Some(preview_from(target, here, false)), Vec::new());
            }
            let positions = MovementRange::compute_attackable_positions(
                move_range,
                cursor.0,
                cursor.1,
                me.attack_range,
                &game.units[target],
            );
            let best = positions
                .iter()
                .max_by_key(|&&p| duel::compute(&game.units, sel, p, target).score());
            (
                best.map(|&p| preview_from(target, p, positions.len() > 1)),
                Vec::new(),
            )
        }
        _ => none,
    }
}

fn reachable_targets(
    units: &[Unit],
    move_range: &Vec<(i32, i32)>,
    sel: usize,
    wants: impl Fn(&Unit, &Unit) -> bool,
) -> (Vec<(i32, i32)>, Vec<(i32, i32)>) {
    let attacker_range = units[sel].attack_range;
    let mut positions = Vec::new();
    let mut target_tiles = Vec::new();
    for target in units.iter().filter(|u| wants(&units[sel], u)) {
        let reachable = MovementRange::compute_attackable_positions(
            move_range,
            target.grid_x,
            target.grid_y,
            attacker_range,
            target,
        );
        if !reachable.is_empty() {
            target_tiles.push((target.grid_x, target.grid_y));
            for pos in reachable {
                if !positions.contains(&pos) {
                    positions.push(pos);
                }
            }
        }
    }
    (positions, target_tiles)
}
