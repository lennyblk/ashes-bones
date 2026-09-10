use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;
mod ai;
mod animation;
mod assets;
mod combat;
mod cursor;
mod game_mode;
mod input;
mod map;
mod minigame;
mod movement;
mod render;
mod ui;
mod unit;

use cursor::{CursorType, Cursors};
use movement::MovementRange;
use unit::{Faction, Unit, UnitClass, UnitState};

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

    let mut assets = assets::load_assets(&mut rl, &thread);

    fn grid_to_screen_x(grid_x: i32) -> f32 {
        let x = (grid_x * TILE_SIZE) as f32 + (-40.0);
        x
    }

    fn grid_to_screen_y(grid_y: i32) -> f32 {
        let y = (grid_y * TILE_SIZE) as f32 + (-40.0);
        y
    }

    let mouse_position = rl.get_mouse_position();

    let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
    let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

    let mut cursor = Cursors {
        cursor_type: CursorType::Normal,
        position: Vector2::new(
            (cursor_grid_x * TILE_SIZE) as f32,
            (cursor_grid_y * TILE_SIZE) as f32,
        ),
        is_selected: false,
    };

    let hud = ui::HudRects::new();

    let mut soldier = Unit {
        name: String::from("Soldier"),
        faction: Faction::Human,
        class: UnitClass::Soldier,
        grid_x: 6,
        grid_y: 10,
        screen_x: grid_to_screen_x(6),
        screen_y: grid_to_screen_y(10),
        move_points: 2,
        move_points_remaining: 2,
        has_attacked: false,
        hp_points: 100,
        hp_max_points: 100,
        path: Vec::new(),
        state: UnitState::Idle,
        facing_left: false,
        attack_range: 1,
        attack_power: 100,
        defense: 5,
        attack_target: false,
    };

    let mut wraith = Unit {
        name: String::from("Wraith"),
        faction: Faction::Undead,
        class: UnitClass::Wraith,
        grid_x: 5,
        grid_y: 10,
        screen_x: grid_to_screen_x(5),
        screen_y: grid_to_screen_y(10),
        move_points: 2,
        move_points_remaining: 2,
        has_attacked: false,
        hp_points: 100,
        hp_max_points: 100,
        path: Vec::new(),
        state: UnitState::Idle,
        facing_left: false,
        attack_range: 1,
        attack_power: 15,
        defense: 10,
        attack_target: false,
    };

    // état de départ, pour le bouton retry
    let soldier_initial = soldier.clone();
    let wraith_initial = wraith.clone();

    let mut game_mode = game_mode::GameMode::TitleScreen;
    let mut soldier_combat_x: f32 = 0.0;
    let mut wraith_combat_x: f32 = 0.0;
    let mut combat_entering_timer: f32 = 0.0;
    let mut combat_ready_timer: f32 = 0.0;
    let mut combat_exit_pause_timer: f32 = 0.0;
    let tile_map = map::TileMap::load(&mut rl, &thread, "assets/maps/ashes-bones-map.tmx");
    let blocked_tiles = tile_map.blocked_tiles.clone();

    let mut attack_animation_started = false;
    let mut active_attacker: Option<Faction> = None;

    let mut current_turn = game_mode::TurnPhase::PlayerTurn;
    let mut enemy_turn_delay: f32 = 0.0;

    let mut timing_bar: Option<minigame::TimingBar> = None;
    let mut damage_multiplier: f32 = 1.0;
    let player_faction = Faction::Human;
    let mut result_display: Option<(minigame::TimingResult, f32, f32)> = None; // alert qui pop au resultat du minigame

    // run window --------------------------------------------------------------
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();

        let mouse_position = rl.get_mouse_position();
        fn mouse_is_clicked(rl: &RaylibHandle) -> bool {
            rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        }
        soldier.update_position(delta_time);
        soldier.advance_path();

        wraith.update_position(delta_time);
        wraith.advance_path();

        let mut click_consumed = false;

        // title screen : play / settings / exit ------------------------------
        if game_mode == game_mode::GameMode::TitleScreen {
            let clicked = mouse_is_clicked(&rl);
            if input::is_button_clicked(mouse_position, clicked, hud.btn_exit_title_screen) {
                break;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_settings_title_screen) {
                // TODO: écran settings
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_play_title_screen) {
                game_mode = game_mode::GameMode::GridScreen;
                click_consumed = true;
            }
        }

        // écran de fin : retry / back / exit --------------------------------
        let game_over =
            game_mode == game_mode::GameMode::Victory || game_mode == game_mode::GameMode::Defeat;
        if game_over {
            let clicked = mouse_is_clicked(&rl);
            if input::is_button_clicked(mouse_position, clicked, hud.btn_exit) {
                break;
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_back) {
                // TODO: retour au menu de titre
            }
            if input::is_button_clicked(mouse_position, clicked, hud.btn_retry) {
                soldier = soldier_initial.clone();
                wraith = wraith_initial.clone();
                game_mode = game_mode::GameMode::GridScreen;
                current_turn = game_mode::TurnPhase::PlayerTurn;
                enemy_turn_delay = 0.0;
                active_attacker = None;
                attack_animation_started = false;
                combat_entering_timer = 0.0;
                combat_ready_timer = 0.0;
                combat_exit_pause_timer = 0.0;
                cursor.is_selected = false;
                click_consumed = true;
            }
        }
        let player_can_act = current_turn == game_mode::TurnPhase::PlayerTurn
            && game_mode == game_mode::GameMode::GridScreen
            && soldier.state == UnitState::Idle;

        // bouton wait : le perso s'arrête là (utile avec plusieurs persos)
        let wait_button_visible = player_can_act && soldier.can_wait();
        if wait_button_visible
            && input::is_button_clicked(mouse_position, mouse_is_clicked(&rl), hud.btn_wait)
        {
            soldier.wait();
            cursor.is_selected = false;
            click_consumed = true;
        }

        // fin de tour : bouton end turn, ou auto quand tous les persos ont fini
        let end_turn_clicked =
            input::is_button_clicked(mouse_position, mouse_is_clicked(&rl), hud.btn_end_turn);

        let all_units_finished = soldier.has_finished_turn(&[wraith.clone()]);

        if player_can_act && (end_turn_clicked || all_units_finished) {
            current_turn = game_mode::TurnPhase::EnemyTurn;
            enemy_turn_delay = ENEMY_TURN_DELAY;
            cursor.is_selected = false;
            click_consumed = true;
        }

        if current_turn == game_mode::TurnPhase::EnemyTurn && enemy_turn_delay > 0.0 {
            enemy_turn_delay -= delta_time;
            if enemy_turn_delay <= 0.0 {
                enemy_turn_delay = 0.0;
                ai::take_turn(&mut wraith, &[soldier.clone()], &blocked_tiles);
            }
        }

        // combat -----------------------------------------------------------
        if game_mode == game_mode::GameMode::GridScreen && active_attacker.is_none() {
            if soldier.state == UnitState::Attacking {
                active_attacker = Some(Faction::Human);
            } else if wraith.state == UnitState::Attacking {
                active_attacker = Some(Faction::Undead);
            }
        }

        let sprite_size = 500.0;
        let overlap = 185.0;
        let center_x = SCREEN_WIDTH as f32 / 2.0;
        let left_x = center_x - sprite_size + overlap;
        let right_x = center_x - overlap;

        let (soldier_target_x, wraith_target_x) = if soldier.facing_left {
            (right_x, left_x)
        } else {
            (left_x, right_x)
        };

        // faire disparaître le message de résultat du minigame après un certain temps
        if let Some((_, time_left, _)) = &mut result_display {
            *time_left -= delta_time;
            if *time_left <= 0.0 {
                result_display = None;
            }
        }
        match active_attacker {
            Some(Faction::Human) => {
                combat::start_attack_if_needed(
                    &mut soldier,
                    &mut wraith,
                    &mut assets.soldier.attack,
                    &mut assets.soldier.attack_effect,
                    &mut attack_animation_started,
                    &mut game_mode,
                    &mut soldier_combat_x,
                    &mut wraith_combat_x,
                );
                combat::enter_combat(
                    &mut soldier,
                    &mut wraith,
                    &mut soldier_combat_x,
                    &mut wraith_combat_x,
                    soldier_target_x,
                    wraith_target_x,
                    &mut combat_entering_timer,
                    delta_time,
                );
                combat::start_attack_after_ready(
                    &mut soldier,
                    &wraith,
                    attack_animation_started,
                    &mut combat_ready_timer,
                    delta_time,
                );
                if let Some(result) = minigame::handle_minigame(
                    &mut soldier,
                    &mut wraith,
                    player_faction,
                    &mut timing_bar,
                    &mut damage_multiplier,
                    &rl,
                    delta_time,
                    &mut assets.soldier.attack,
                ) {
                    result_display = Some((result, 1.0, damage_multiplier));
                }
                combat::resolve_attack(
                    &mut soldier,
                    &mut wraith,
                    &assets.soldier.attack,
                    &mut assets.wraith.hurt,
                    &mut attack_animation_started,
                    damage_multiplier,
                );
                combat::update_hurt_state(&mut wraith, &assets.wraith.hurt, &mut assets.wraith.die);
                combat::update_dying_state(&mut wraith, &assets.wraith.die);
                combat::combat_exit_pause_timer(
                    &mut wraith,
                    attack_animation_started,
                    &mut game_mode,
                    &mut combat_exit_pause_timer,
                    delta_time,
                );
            }
            Some(Faction::Undead) => {
                combat::start_attack_if_needed(
                    &mut wraith,
                    &mut soldier,
                    &mut assets.wraith.attack,
                    &mut assets.wraith.attack_effect,
                    &mut attack_animation_started,
                    &mut game_mode,
                    &mut wraith_combat_x,
                    &mut soldier_combat_x,
                );
                combat::enter_combat(
                    &mut wraith,
                    &mut soldier,
                    &mut wraith_combat_x,
                    &mut soldier_combat_x,
                    wraith_target_x,
                    soldier_target_x,
                    &mut combat_entering_timer,
                    delta_time,
                );
                combat::start_attack_after_ready(
                    &mut wraith,
                    &soldier,
                    attack_animation_started,
                    &mut combat_ready_timer,
                    delta_time,
                );
                if let Some(result) = minigame::handle_minigame(
                    &mut wraith,
                    &mut soldier,
                    player_faction,
                    &mut timing_bar,
                    &mut damage_multiplier,
                    &rl,
                    delta_time,
                    &mut assets.wraith.attack,
                ) {
                    result_display = Some((result, 1.0, damage_multiplier));
                }
                combat::resolve_attack(
                    &mut wraith,
                    &mut soldier,
                    &assets.wraith.attack,
                    &mut assets.soldier.hurt,
                    &mut attack_animation_started,
                    damage_multiplier,
                );
                combat::update_hurt_state(
                    &mut soldier,
                    &assets.soldier.hurt,
                    &mut assets.soldier.die,
                );
                combat::update_dying_state(&mut soldier, &assets.soldier.die);
                combat::combat_exit_pause_timer(
                    &mut soldier,
                    attack_animation_started,
                    &mut game_mode,
                    &mut combat_exit_pause_timer,
                    delta_time,
                );
            }
            None => {}
        }

        if game_mode == game_mode::GameMode::GridScreen {
            if soldier.state == UnitState::Dead {
                game_mode = game_mode::GameMode::Defeat;
            } else if wraith.state == UnitState::Dead {
                game_mode = game_mode::GameMode::Victory;
            }
        }

        if current_turn == game_mode::TurnPhase::EnemyTurn
            && enemy_turn_delay <= 0.0
            && game_mode == game_mode::GameMode::GridScreen
            && wraith.state == UnitState::Idle
        {
            current_turn = game_mode::TurnPhase::PlayerTurn;
            soldier.start_turn();
        }

        if active_attacker.is_some() && game_mode == game_mode::GameMode::GridScreen {
            active_attacker = None;
        }

        let (move_range, came_from) =
            if cursor.is_selected && game_mode == game_mode::GameMode::GridScreen {
                MovementRange::compute_movement_range(
                    soldier.grid_x,
                    soldier.grid_y,
                    soldier.move_points_remaining,
                    GRID_COLS,
                    GRID_ROWS,
                    wraith.grid_x,
                    wraith.grid_y,
                    &wraith,
                    &blocked_tiles,
                )
            } else {
                (Vec::new(), HashMap::new())
            };

        input::cancel_pressed(&rl);

        let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
        let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

        // if pour bouger le personnage
        if input::handle_movement_normal_click(
            &rl,
            &mut soldier,
            &mut cursor,
            &move_range,
            &came_from,
            cursor_grid_x,
            cursor_grid_y,
        ) {
            click_consumed = true;
        }

        // une seule attaque par tour et par unité
        let valid_attack_positions = if soldier.has_attacked {
            Vec::new()
        } else {
            MovementRange::compute_attackable_positions(
                &move_range,
                wraith.grid_x,
                wraith.grid_y,
                soldier.attack_range,
                &wraith,
            )
        };
        let wraith_attackable = !valid_attack_positions.is_empty();

        // if pour attaquer l'ennemi et bouger le personnage si il y a qu'une seule case possible
        // pour attaquer
        if input::handle_movement_attack_click(
            &rl,
            &mut soldier,
            &mut cursor,
            &came_from,
            &valid_attack_positions,
            wraith_attackable,
            &wraith,
            cursor_grid_x,
            cursor_grid_y,
        ) {
            click_consumed = true;
        }

        // if pour bouger le personnage vers la case choisie pour attaquer l'ennemi
        if input::handle_movement_choosing_position_click(
            &rl,
            &mut soldier,
            &mut cursor,
            &came_from,
            &valid_attack_positions,
            cursor_grid_x,
            cursor_grid_y,
        ) {
            click_consumed = true;
        }

        if input::cancel_pressed(&rl) && soldier.state == UnitState::ChoosingPosition {
            soldier.state = UnitState::Idle;
        }

        cursor.update_cursor(
            cursor_grid_x,
            cursor_grid_y,
            soldier.grid_x,
            soldier.grid_y,
            mouse_is_clicked(&rl) && !click_consumed,
        );

        // drawing --------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        if game_mode == game_mode::GameMode::TitleScreen {
            render::draw_title_screen(&mut d, &assets, &hud, mouse_position);
        } else if game_mode == game_mode::GameMode::CombatScreen {
            render::draw_combat_screen(
                &mut d,
                &mut assets,
                delta_time,
                &soldier,
                &wraith,
                soldier_combat_x,
                wraith_combat_x,
                timing_bar.as_ref(),
                result_display.as_ref(),
            );
        } else {
            render::draw_grid_screen(
                &mut d,
                &mut assets,
                delta_time,
                &tile_map,
                &hud,
                &soldier,
                &wraith,
                game_mode,
                &current_turn,
                wait_button_visible,
                &move_range,
                &valid_attack_positions,
                wraith_attackable,
                cursor_grid_x,
                cursor_grid_y,
                cursor.cursor_type,
                mouse_position,
            );
        }
    }
}
