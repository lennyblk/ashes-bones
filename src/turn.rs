use crate::ai;
use crate::game::Game;
use crate::game_mode::{GameMode, TurnPhase};
use crate::unit::Unit;

pub const ENEMY_TURN_DELAY: f32 = 0.5;

pub fn start_enemy_turn(game: &mut Game) {
    game.current_turn = TurnPhase::EnemyTurn;
    game.enemy_turn_delay = ENEMY_TURN_DELAY;
    game.ai_turn_queue = game
        .units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.faction == game.player_faction.opposite() && u.is_alive())
        .map(|(i, _)| i)
        .collect();
    game.ai_acting_unit = None;
    game.selected_unit = None;
}

pub fn update_enemy_turn(game: &mut Game, delta_time: f32) {
    if game.current_turn == TurnPhase::EnemyTurn && game.game_mode == GameMode::GridScreen {
        if let Some(idx) = game.ai_acting_unit {
            if !game.units[idx].is_busy() {
                game.ai_acting_unit = None;
                game.enemy_turn_delay = ENEMY_TURN_DELAY;
            }
        } else if game.enemy_turn_delay > 0.0 {
            game.enemy_turn_delay -= delta_time;
        } else if let Some(idx) = game.ai_turn_queue.pop() {
            if game.units[idx].is_alive() {
                let targets: Vec<(usize, Unit)> = game
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_, u)| u.faction == game.player_faction && u.is_alive())
                    .map(|(i, u)| (i, u.clone()))
                    .collect();
                // alliés blessés que l'unité pourrait soigner (pas elle-même)
                let ai_faction = game.units[idx].faction;
                let wounded_allies: Vec<(usize, Unit)> = game
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(i, u)| {
                        *i != idx
                            && u.faction == ai_faction
                            && u.is_alive()
                            && u.hp_points < u.hp_max_points
                    })
                    .map(|(i, u)| (i, u.clone()))
                    .collect();
                // toute autre unité vivante (alliée ou ennemie) bloque le passage
                let obstacles: Vec<Unit> = game
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(i, u)| *i != idx && u.is_alive())
                    .map(|(_, u)| u.clone())
                    .collect();
                ai::take_turn(
                    &mut game.units[idx],
                    &targets,
                    &wounded_allies,
                    &obstacles,
                    &game.blocked_tiles,
                );
                game.ai_acting_unit = Some(idx);
            }
        }
    }
}

pub fn end_enemy_turn_if_done(game: &mut Game) {
    let enemy_turn_queue_done = game.ai_turn_queue.is_empty() && game.ai_acting_unit.is_none();
    if game.current_turn == TurnPhase::EnemyTurn
        && game.enemy_turn_delay <= 0.0
        && game.game_mode == GameMode::GridScreen
        && enemy_turn_queue_done
    {
        game.current_turn = TurnPhase::PlayerTurn;
        for u in game
            .units
            .iter_mut()
            .filter(|u| u.faction == game.player_faction)
        {
            u.start_turn();
        }
    }
}
