use raylib::prelude::*;
mod ai;
mod animation;
mod assets;
mod combat;
mod cursor;
mod duel;
mod game;
mod game_mode;
mod grid;
mod input;
mod map;
mod minigame;
mod movement;
mod render;
mod screens;
mod turn;
mod ui;
mod unit;

use cursor::{CursorType, Cursors};
use game::Game;
use unit::{Unit, UnitState};

const TILE_SIZE: i32 = 48;
const GRID_COLS: i32 = 25;
const GRID_ROWS: i32 = 16;
const SCREEN_WIDTH: i32 = GRID_COLS * TILE_SIZE;
const SCREEN_HEIGHT: i32 = GRID_ROWS * TILE_SIZE;

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

        screens::pause::handle_escape(&mut game, &rl);

        // écrans de menu, dans le même ordre qu'avant : un clic peut changer d'écran
        // et l'écran suivant le voit dans la même frame (d'où click_consumed)
        if screens::title::update(&mut game, &rl, &hud, mouse_position, &mut click_consumed) {
            break;
        }
        if screens::pause::update(&mut game, &rl, &hud, mouse_position) {
            break;
        }
        screens::guide::update(&mut game, &rl, &hud, mouse_position);
        screens::faction_selection::update(&mut game, &rl, mouse_position, click_consumed);
        if screens::game_over::update(&mut game, &rl, &hud, mouse_position, &mut click_consumed) {
            break;
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
            && input::is_button_clicked(mouse_position, input::mouse_is_clicked(&rl), hud.btn_wait)
        {
            if let Some(sel) = game.selected_unit {
                game.units[sel].wait();
            }
            game.selected_unit = None;
            click_consumed = true;
        }

        // fin de tour : bouton end turn, ou auto quand tous les persos ont fini
        let end_turn_clicked = input::is_button_clicked(
            mouse_position,
            input::mouse_is_clicked(&rl),
            hud.btn_end_turn,
        );

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
            turn::start_enemy_turn(&mut game);
            click_consumed = true;
        }

        turn::update_enemy_turn(&mut game, delta_time);

        combat::update(&mut game, &mut assets, &rl, delta_time);

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

        turn::end_enemy_turn_if_done(&mut game);

        if game.active_combat.is_some() && game.game_mode == game_mode::GameMode::GridScreen {
            game.active_combat = None;
        }

        let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
        let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

        let highlights = grid::update(&mut game, &rl, click_consumed, cursor_grid_x, cursor_grid_y);

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
            render::draw_guide_screen(
                &mut d,
                &mut assets,
                delta_time,
                &hud,
                mouse_position,
                game.guide_scroll,
            );
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
                    game.minigame.as_ref(),
                    game.result_display.as_ref(),
                    &game.duel_conditions,
                    game.units[attacker_idx].faction == game.player_faction,
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
                &highlights.move_range,
                &highlights.choosing_positions,
                &highlights.attackable_enemy_positions,
                &highlights.healable_ally_positions,
                cursor_grid_x,
                cursor_grid_y,
                cursor.cursor_type,
                mouse_position,
                game.selected_unit.map(|i| &game.units[i]),
                game.inspected_enemy.map(|i| &game.units[i]),
                highlights.duel_preview.as_ref(),
                &highlights.position_scores,
            );
        }
    }
}
