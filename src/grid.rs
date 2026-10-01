use std::collections::HashMap;

use raylib::prelude::*;

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
            if let Some(ally_idx) = game.units.iter().position(|u| {
                u.faction == healer_faction
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

    GridHighlights {
        move_range,
        choosing_positions,
        attackable_enemy_positions,
        healable_ally_positions,
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
