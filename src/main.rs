use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;
mod ai;
mod animation;
mod assets;
mod combat;
mod cursor;
mod game;
mod game_mode;
mod input;
mod map;
mod minigame;
mod movement;
mod render;
mod ui;
mod unit;

use cursor::{CursorType, Cursors};
use game::Game;
use movement::MovementRange;
use unit::{Faction, PendingAction, Unit, UnitState, two_mut};

const TILE_SIZE: i32 = 48;
const GRID_COLS: i32 = 25;
const GRID_ROWS: i32 = 16;
const SCREEN_WIDTH: i32 = GRID_COLS * TILE_SIZE;
const SCREEN_HEIGHT: i32 = GRID_ROWS * TILE_SIZE;
const ENEMY_TURN_DELAY: f32 = 0.5;

fn main() {
    // init window
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Ashes&Bones")
        .build();
    rl.hide_cursor();
    rl.set_exit_key(None); // Échap ouvre le menu pause, ferme plus le jeu

    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    rl.set_random_seed(seed);

    let mut assets = assets::load_assets(&mut rl, &thread);

    let mouse_position = rl.get_mouse_position();

    let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
    let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

    let mut cursor = Cursors {
        cursor_type: CursorType::Normal,
        position: Vector2::new(
            (cursor_grid_x * TILE_SIZE) as f32,
            (cursor_grid_y * TILE_SIZE) as f32,
        ),
    };

    let hud = ui::HudRects::new();

    let tile_map = map::TileMap::load(&mut rl, &thread, "assets/maps/ashes-bones-map.tmx");

    let mut game = Game::new(&rl, tile_map.blocked_tiles.clone());

    // run window --------------------------------------------------------------
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();

        let mouse_position = rl.get_mouse_position();
        // en pause ou dans le guide, la partie est gelée : ni mouvement ni animation
        if !matches!(
            game.game_mode,
            game_mode::GameMode::PauseScreen | game_mode::GameMode::GuideScreen
        ) {
            for unit in game.units.iter_mut() {
                unit.update_position(delta_time);
                unit.advance_path();
            }
        }

        let mut click_consumed = false;

        // Échap : pause pendant une partie, ou retour depuis pause/guide
        if input::pause_pressed(&rl) {
            match game.game_mode {
                game_mode::GameMode::GridScreen | game_mode::GameMode::CombatScreen => {
                    game.paused_from = game.game_mode;
                    game.game_mode = game_mode::GameMode::PauseScreen;
                }
                game_mode::GameMode::PauseScreen => game.game_mode = game.paused_from,
                game_mode::GameMode::GuideScreen => game.game_mode = game.guide_return_to,
                _ => {}
            }
        }

        // title screen : play / settings / learn / exit -----------------------
        if game.game_mode == game_mode::GameMode::TitleScreen {
            let clicked = mouse_is_clicked(&rl);
            if input::is_button_clicked(mouse_position, clicked, hud.btn_exit_title_screen) {
                break;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_settings_title_screen) {
                // TODO: écran settings
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_learn_title_screen) {
                game.guide_return_to = game_mode::GameMode::TitleScreen;
                game.game_mode = game_mode::GameMode::GuideScreen;
                click_consumed = true;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_play_title_screen) {
                game.game_mode = game_mode::GameMode::FactionSelectionScreen;
                click_consumed = true;
            }
        }

        // menu pause : play / guide / settings / back / exit -------------------
        if game.game_mode == game_mode::GameMode::PauseScreen {
            let clicked = mouse_is_clicked(&rl);
            if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_exit) {
                break;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_settings) {
                // TODO: écran settings
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_guide) {
                game.guide_return_to = game_mode::GameMode::PauseScreen;
                game.game_mode = game_mode::GameMode::GuideScreen;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_play) {
                game.game_mode = game.paused_from;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_pause_back) {
                game.reset(&rl, game_mode::GameMode::TitleScreen);
            }
        }

        // guide : bouton back (Échap marche aussi, voir plus haut) --------------
        if game.game_mode == game_mode::GameMode::GuideScreen
            && input::is_button_clicked(mouse_position, mouse_is_clicked(&rl), hud.btn_guide_back)
        {
            game.game_mode = game.guide_return_to;
        }

        // Faction selection screen : Human / Undead --------------------------------
        if game.game_mode == game_mode::GameMode::FactionSelectionScreen {
            if input::cancel_pressed(&rl) {
                game.game_mode = game_mode::GameMode::TitleScreen;
            } else if !click_consumed && mouse_is_clicked(&rl) {
                game.player_faction = if mouse_position.x < SCREEN_WIDTH as f32 / 2.0 {
                    Faction::Human
                } else {
                    Faction::Undead
                };
                game.game_mode = game_mode::GameMode::GridScreen;
            }
        }

        // écran de fin : retry / back / exit --------------------------------
        let game_over = game.game_mode == game_mode::GameMode::Victory
            || game.game_mode == game_mode::GameMode::Defeat;
        if game_over {
            let clicked = mouse_is_clicked(&rl);
            if input::is_button_clicked(mouse_position, clicked, hud.btn_exit) {
                break;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_back) {
                game.reset(&rl, game_mode::GameMode::TitleScreen);
                click_consumed = true;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_retry) {
                game.reset(&rl, game_mode::GameMode::GridScreen);
                click_consumed = true;
            }
        }
        let player_units_busy = game
            .units
            .iter()
            .any(|u| u.faction == game.player_faction && u.is_busy());
        let player_can_act = game.current_turn == game_mode::TurnPhase::PlayerTurn
            && game.game_mode == game_mode::GameMode::GridScreen
            && !player_units_busy;

        // bouton wait
        let wait_button_visible =
            player_can_act && game.selected_unit.is_some_and(|i| game.units[i].can_wait());
        if wait_button_visible
            && input::is_button_clicked(mouse_position, mouse_is_clicked(&rl), hud.btn_wait)
        {
            if let Some(sel) = game.selected_unit {
                game.units[sel].wait();
            }
            game.selected_unit = None;
            click_consumed = true;
        }

        // fin de tour : bouton end turn, ou auto quand tous les persos ont fini
        let end_turn_clicked =
            input::is_button_clicked(mouse_position, mouse_is_clicked(&rl), hud.btn_end_turn);

        let enemy_snapshot: Vec<Unit> = game
            .units
            .iter()
            .filter(|u| u.faction == game.player_faction.opposite())
            .cloned()
            .collect();
        let ally_snapshot: Vec<Unit> = game
            .units
            .iter()
            .filter(|u| u.faction == game.player_faction)
            .cloned()
            .collect();
        let all_units_finished = game
            .units
            .iter()
            .filter(|u| u.faction == game.player_faction && u.is_alive())
            .all(|u| u.has_finished_turn(&enemy_snapshot, &ally_snapshot));

        if player_can_act && (end_turn_clicked || all_units_finished) {
            game.current_turn = game_mode::TurnPhase::EnemyTurn;
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
            click_consumed = true;
        }

        // une unité ennemie à la fois, on attend qu'elle finisse avant la suivante
        if game.current_turn == game_mode::TurnPhase::EnemyTurn
            && game.game_mode == game_mode::GameMode::GridScreen
        {
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

        // combat -----------------------------------------------------------
        if game.game_mode == game_mode::GameMode::GridScreen && game.active_combat.is_none() {
            if let Some(attacker_idx) = game
                .units
                .iter()
                .position(|u| u.state == UnitState::Attacking || u.state == UnitState::Healing)
            {
                if let Some(action) = game.units[attacker_idx].pending_action {
                    game.active_combat = Some((attacker_idx, action.target_idx()));
                }
            }
        }

        let sprite_size = 500.0;
        let overlap = 185.0;
        let center_x = SCREEN_WIDTH as f32 / 2.0;
        let left_x = center_x - sprite_size + overlap;
        let right_x = center_x - overlap;

        // faire disparaître le message de résultat du minigame après un certain temps
        if let Some((_, time_left, _)) = &mut game.result_display {
            *time_left -= delta_time;
            if *time_left <= 0.0 {
                game.result_display = None;
            }
        }

        if let Some((attacker_idx, defender_idx)) = game.active_combat {
            let (attacker_target_x, defender_target_x) = if game.units[attacker_idx].facing_left {
                (right_x, left_x)
            } else {
                (left_x, right_x)
            };

            let is_heal = matches!(
                game.units[attacker_idx].pending_action,
                Some(PendingAction::Heal(_))
            );
            let attacker_class = game.units[attacker_idx].class;
            let defender_class = game.units[defender_idx].class;
            let (attacker, defender) = two_mut(&mut game.units, attacker_idx, defender_idx);

            {
                let (cast_anim, cast_effect) = if is_heal {
                    assets.heal_animation_mut(attacker_class)
                } else {
                    let set = assets.animation_set_mut(attacker_class);
                    (&mut set.attack, &mut set.attack_effect)
                };
                combat::start_attack_if_needed(
                    attacker,
                    defender,
                    cast_anim,
                    cast_effect,
                    &mut game.attack_animation_started,
                    &mut game.game_mode,
                    &mut game.attacker_combat_x,
                    &mut game.defender_combat_x,
                );
            }
            combat::enter_combat(
                attacker,
                defender,
                &mut game.attacker_combat_x,
                &mut game.defender_combat_x,
                attacker_target_x,
                defender_target_x,
                &mut game.combat_entering_timer,
                delta_time,
            );
            combat::start_attack_after_ready(
                attacker,
                defender,
                game.attack_animation_started,
                &mut game.combat_ready_timer,
                delta_time,
            );
            {
                let (cast_anim, _) = if is_heal {
                    assets.heal_animation_mut(attacker_class)
                } else {
                    let set = assets.animation_set_mut(attacker_class);
                    (&mut set.attack, &mut set.attack_effect)
                };
                if let Some(result) = minigame::handle_minigame(
                    attacker,
                    defender,
                    game.player_faction,
                    &mut game.timing_bar,
                    &mut game.damage_multiplier,
                    &rl,
                    delta_time,
                    cast_anim,
                ) {
                    game.result_display = Some((result, 1.0, game.damage_multiplier));
                }
            }
            if is_heal {
                let cast_finished = assets.heal_animation_mut(attacker_class).0.finished;
                combat::resolve_heal(
                    attacker,
                    defender,
                    cast_finished,
                    &mut game.attack_animation_started,
                    game.damage_multiplier,
                );
            } else {
                let cast_finished = assets.animation_set_mut(attacker_class).attack.finished;
                let defender_hurt = &mut assets.animation_set_mut(defender_class).hurt;
                combat::resolve_attack(
                    attacker,
                    defender,
                    cast_finished,
                    defender_hurt,
                    &mut game.attack_animation_started,
                    game.damage_multiplier,
                );
            }
            {
                let defender_set = assets.animation_set_mut(defender_class);
                combat::update_hurt_state(defender, &defender_set.hurt, &mut defender_set.die);
                combat::update_dying_state(defender, &defender_set.die);
            }
            combat::combat_exit_pause_timer(
                defender,
                game.attack_animation_started,
                &mut game.game_mode,
                &mut game.combat_exit_pause_timer,
                delta_time,
            );
        }

        if game.game_mode == game_mode::GameMode::GridScreen {
            let player_alive = game
                .units
                .iter()
                .any(|u| u.faction == game.player_faction && u.is_alive());
            let enemy_alive = game
                .units
                .iter()
                .any(|u| u.faction != game.player_faction && u.is_alive());
            if !player_alive {
                game.game_mode = game_mode::GameMode::Defeat;
            } else if !enemy_alive {
                game.game_mode = game_mode::GameMode::Victory;
            }
        }

        let enemy_turn_queue_done = game.ai_turn_queue.is_empty() && game.ai_acting_unit.is_none();
        if game.current_turn == game_mode::TurnPhase::EnemyTurn
            && game.enemy_turn_delay <= 0.0
            && game.game_mode == game_mode::GameMode::GridScreen
            && enemy_turn_queue_done
        {
            game.current_turn = game_mode::TurnPhase::PlayerTurn;
            for u in game
                .units
                .iter_mut()
                .filter(|u| u.faction == game.player_faction)
            {
                u.start_turn();
            }
        }

        if game.active_combat.is_some() && game.game_mode == game_mode::GameMode::GridScreen {
            game.active_combat = None;
        }

        let (move_range, came_from) = if let Some(sel) = game.selected_unit {
            if game.game_mode == game_mode::GameMode::GridScreen {
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

        input::cancel_pressed(&rl);

        let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
        let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

        let clicked = mouse_is_clicked(&rl);

        // sélection / désélection d'une unité du joueur au clic sur sa case
        if game.game_mode == game_mode::GameMode::GridScreen && clicked && !click_consumed {
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
                &rl,
                &mut game.units[sel],
                &move_range,
                &came_from,
                cursor_grid_x,
                cursor_grid_y,
            ) {
                game.selected_unit = None;
            }
        }

        let (valid_attack_positions, attackable_enemy_positions): (
            Vec<(i32, i32)>,
            Vec<(i32, i32)>,
        ) = match game.selected_unit {
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
                    &rl,
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
                        &rl,
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

        let choosing_positions = match game
            .selected_unit
            .and_then(|sel| game.units[sel].pending_action)
        {
            Some(PendingAction::Heal(_)) => &valid_heal_positions,
            _ => &valid_attack_positions,
        };

        if let Some(sel) = game.selected_unit {
            if input::handle_movement_choosing_position_click(
                &rl,
                &mut game.units[sel],
                &came_from,
                choosing_positions,
                cursor_grid_x,
                cursor_grid_y,
            ) {
                game.selected_unit = None;
            }
        }

        if let Some(sel) = game.selected_unit {
            if input::cancel_pressed(&rl) && game.units[sel].state == UnitState::ChoosingPosition {
                game.units[sel].state = UnitState::Idle;
                game.selected_unit = None;
            }
        }

        let hovering_selectable_unit = game.selected_unit.is_none()
            && game.units.iter().any(|u| {
                u.faction == game.player_faction
                    && u.is_alive()
                    && u.state == UnitState::Idle
                    && u.grid_x == cursor_grid_x
                    && u.grid_y == cursor_grid_y
            });

        cursor.update_cursor(
            cursor_grid_x,
            cursor_grid_y,
            hovering_selectable_unit,
            game.selected_unit.is_some(),
        );

        // drawing --------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        if game.game_mode == game_mode::GameMode::TitleScreen {
            render::draw_title_screen(&mut d, &assets, &hud, mouse_position);
        } else if game.game_mode == game_mode::GameMode::FactionSelectionScreen {
            render::draw_faction_selection_screen(&mut d, &mut assets, delta_time, mouse_position);
        } else if game.game_mode == game_mode::GameMode::PauseScreen {
            render::draw_pause_screen(&mut d, &assets, &hud, mouse_position);
        } else if game.game_mode == game_mode::GameMode::GuideScreen {
            render::draw_guide_screen(&mut d, &mut assets, delta_time, &hud, mouse_position);
        } else if game.game_mode == game_mode::GameMode::CombatScreen {
            if let Some((attacker_idx, defender_idx)) = game.active_combat {
                render::draw_combat_screen(
                    &mut d,
                    &mut assets,
                    delta_time,
                    &game.units[attacker_idx],
                    &game.units[defender_idx],
                    game.attacker_combat_x,
                    game.defender_combat_x,
                    game.timing_bar.as_ref(),
                    game.result_display.as_ref(),
                );
            }
        } else {
            render::draw_grid_screen(
                &mut d,
                &mut assets,
                delta_time,
                &tile_map,
                &hud,
                &game.units,
                game.game_mode,
                &game.current_turn,
                wait_button_visible,
                &move_range,
                choosing_positions,
                &attackable_enemy_positions,
                &healable_ally_positions,
                cursor_grid_x,
                cursor_grid_y,
                cursor.cursor_type,
                mouse_position,
            );
        }
    }
}

fn mouse_is_clicked(rl: &RaylibHandle) -> bool {
    rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
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
