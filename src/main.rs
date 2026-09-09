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
mod movement;
mod ui;
mod unit;

use cursor::Cursors;
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
        current_cursor_texture: &assets.mouse_normal_texture,
        position: Vector2::new(
            (cursor_grid_x * TILE_SIZE) as f32,
            (cursor_grid_y * TILE_SIZE) as f32,
        ),
        is_selected: false,
    };

    let btn_end_turn = Rectangle {
        x: (SCREEN_WIDTH as f32) - 160.0 - 20.0,
        y: (SCREEN_HEIGHT as f32) - 48.0 - 20.0,
        width: 160.0,
        height: 48.0,
    };

    let btn_wait = Rectangle {
        x: (SCREEN_WIDTH as f32) - 160.0 - 20.0,
        y: btn_end_turn.y - 48.0 - 10.0,
        width: 160.0,
        height: 48.0,
    };

    let banner_your_turn = Rectangle {
        x: (SCREEN_WIDTH as f32) - 384.0 - 20.0,
        y: 10.0,
        width: 384.0,
        height: 64.0,
    };

    let banner_enemy_turn = Rectangle {
        x: (SCREEN_WIDTH as f32) - 384.0 - 20.0,
        y: 10.0,
        width: 384.0,
        height: 64.0,
    };

    let btn_retry = Rectangle {
        x: SCREEN_WIDTH as f32 / 2.0 - 64.0,  // centré, largeur 128
        y: SCREEN_HEIGHT as f32 / 2.0 + 60.0, // sous le texte
        width: 128.0,
        height: 48.0,
    };
    let btn_back = Rectangle {
        x: btn_retry.x,
        y: btn_retry.y + 48.0 + 10.0,
        width: 128.0,
        height: 48.0,
    };
    let btn_exit = Rectangle {
        x: btn_retry.x,
        y: btn_back.y + 48.0 + 10.0,
        width: 128.0,
        height: 48.0,
    };

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
        grid_y: 7,
        screen_x: grid_to_screen_x(5),
        screen_y: grid_to_screen_y(7),
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

    let mut game_mode = game_mode::GameMode::GridScreen;
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

        // écran de fin : retry / back / exit --------------------------------
        let game_over =
            game_mode == game_mode::GameMode::Victory || game_mode == game_mode::GameMode::Defeat;
        if game_over {
            let clicked = mouse_is_clicked(&rl);
            if input::is_button_clicked(mouse_position, clicked, btn_exit) {
                break;
            }
            if input::is_button_clicked(mouse_position, clicked, btn_back) {
                // TODO: retour au menu de titre
            }
            if input::is_button_clicked(mouse_position, clicked, btn_retry) {
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
            && input::is_button_clicked(mouse_position, mouse_is_clicked(&rl), btn_wait)
        {
            soldier.wait();
            cursor.is_selected = false;
            click_consumed = true;
        }

        // fin de tour : bouton end turn, ou auto quand tous les persos ont fini
        let end_turn_clicked =
            input::is_button_clicked(mouse_position, mouse_is_clicked(&rl), btn_end_turn);

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
                    &mut assets.soldier.attack,
                );
                combat::resolve_attack(
                    &mut soldier,
                    &mut wraith,
                    &assets.soldier.attack,
                    &mut assets.wraith.hurt,
                    &mut attack_animation_started,
                );
                combat::update_hurt_state(
                    &mut wraith,
                    delta_time,
                    &mut assets.wraith.hurt,
                    &mut assets.wraith.die,
                );
                combat::update_dying_state(&mut wraith, delta_time, &mut assets.wraith.die);
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
                    &mut assets.wraith.attack,
                );
                combat::resolve_attack(
                    &mut wraith,
                    &mut soldier,
                    &assets.wraith.attack,
                    &mut assets.soldier.hurt,
                    &mut attack_animation_started,
                );
                combat::update_hurt_state(
                    &mut soldier,
                    delta_time,
                    &mut assets.soldier.hurt,
                    &mut assets.soldier.die,
                );
                combat::update_dying_state(&mut soldier, delta_time, &mut assets.soldier.die);
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
            &assets.mouse_normal_texture,
            &assets.mouse_hover_texture,
            &assets.mouse_click_texture,
            cursor_grid_x,
            cursor_grid_y,
            soldier.grid_x,
            soldier.grid_y,
            mouse_is_clicked(&rl) && !click_consumed,
        );

        // animations communes aux deux modes -----------------------------------
        let current_animation = match soldier.state {
            UnitState::Idle => &mut assets.soldier.idle,
            UnitState::Walking => &mut assets.soldier.walk,
            UnitState::Attacking => &mut assets.soldier.attack,
            UnitState::ChoosingPosition => &mut assets.soldier.idle,
            UnitState::CombatEntering => &mut assets.soldier.walk,
            UnitState::Hurt => &mut assets.soldier.hurt,
            UnitState::Dying => &mut assets.soldier.die,
            UnitState::Dead => &mut assets.soldier.idle,
        };

        let current_wraith_animation = match wraith.state {
            UnitState::Idle => &mut assets.wraith.idle,
            UnitState::Walking => &mut assets.wraith.walk,
            UnitState::Attacking => &mut assets.wraith.attack,
            UnitState::ChoosingPosition => &mut assets.wraith.idle,
            UnitState::Hurt => &mut assets.wraith.hurt,
            UnitState::Dying => &mut assets.wraith.die,
            UnitState::Dead => &mut assets.wraith.idle,
            UnitState::CombatEntering => &mut assets.wraith.walk,
        };

        current_animation.animation_update(delta_time);
        if soldier.state == UnitState::Attacking {
            assets.soldier.attack_effect.animation_update(delta_time);
        }

        current_wraith_animation.animation_update(delta_time);
        if wraith.state == UnitState::Attacking {
            assets.wraith.attack_effect.animation_update(delta_time);
        }

        let mut source_rec_soldier = current_animation.animation_frame();
        let mut source_rec_wraith = current_wraith_animation.animation_frame();

        if soldier.facing_left {
            source_rec_soldier.width = -source_rec_soldier.width;
        }

        if wraith.facing_left {
            source_rec_wraith.width = -source_rec_wraith.width;
        }

        // drawing --------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);

        // Grid screen mode ------------------------------------------------------
        if game_mode == game_mode::GameMode::GridScreen
            || game_mode == game_mode::GameMode::Victory
            || game_mode == game_mode::GameMode::Defeat
        {
            d.clear_background(Color::BEIGE);
            tile_map.draw(&mut d);

            // for i in (0..SCREEN_HEIGHT).step_by(TILE_SIZE as usize) {
            //     d.draw_rectangle_lines(0, i, SCREEN_WIDTH, 1, Color::BLACK);
            // }
            //
            // for i in (0..SCREEN_WIDTH).step_by(TILE_SIZE as usize) {
            //     d.draw_rectangle_lines(i, 0, 1, SCREEN_HEIGHT, Color::BLACK);
            // }

            if current_turn == game_mode::TurnPhase::PlayerTurn {
                d.draw_texture_ex(
                    &assets.banner_your_turn,
                    Vector2::new(banner_your_turn.x, banner_your_turn.y),
                    0.0,
                    1.0,
                    Color::WHITE,
                );
                d.draw_texture_ex(
                    &assets.btn_end_turn,
                    Vector2::new(btn_end_turn.x, btn_end_turn.y),
                    0.0,
                    1.0,
                    Color::WHITE,
                );
                if wait_button_visible {
                    d.draw_texture_ex(
                        &assets.btn_wait,
                        Vector2::new(btn_wait.x, btn_wait.y),
                        0.0,
                        1.0,
                        Color::WHITE,
                    );
                }
            } else if current_turn == game_mode::TurnPhase::EnemyTurn {
                d.draw_texture_ex(
                    &assets.banner_enemy_turn,
                    Vector2::new(banner_enemy_turn.x, banner_enemy_turn.y),
                    0.0,
                    1.0,
                    Color::WHITE,
                );
            }

            for (x, y) in &move_range {
                d.draw_rectangle(
                    x * TILE_SIZE,
                    y * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    Color::new(0, 100, 255, 100), // bleu transparent
                );
            }

            if soldier.state == UnitState::ChoosingPosition {
                for (x, y) in &valid_attack_positions {
                    d.draw_rectangle(
                        x * TILE_SIZE,
                        y * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        Color::new(255, 255, 0, 170), // jaune transparent
                    );
                }
            }

            if wraith_attackable {
                d.draw_rectangle(
                    wraith.grid_x * TILE_SIZE,
                    wraith.grid_y * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    Color::new(255, 0, 0, 180), // rouge transparent
                );
            }

            if move_range.contains(&(cursor_grid_x, cursor_grid_y)) {
                d.draw_texture_ex(
                    &assets.mouse_select_texture,
                    Vector2::new(
                        (cursor_grid_x * TILE_SIZE) as f32,
                        (cursor_grid_y * TILE_SIZE) as f32,
                    ),
                    0.0,
                    1.0,
                    Color::WHITE,
                );
            }

            if soldier.is_alive() {
                d.draw_texture_pro(
                    &current_animation.texture,
                    source_rec_soldier,
                    Rectangle {
                        x: soldier.screen_x,
                        y: soldier.screen_y,
                        width: 128.0,
                        height: 128.0,
                    },
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            }

            if wraith.is_alive() {
                d.draw_texture_pro(
                    &current_wraith_animation.texture,
                    source_rec_wraith,
                    Rectangle {
                        x: wraith.screen_x,
                        y: wraith.screen_y,
                        width: 128.0,
                        height: 128.0,
                    },
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            }

            let victory_text = "VICTORY";
            let defeat_text = "DEFEAT";
            let font_size = 65.0;
            let vtext_size = assets.hud_font.measure_text(victory_text, font_size, 1.0);
            let dtext_size = assets.hud_font.measure_text(defeat_text, font_size, 1.0);
            if game_mode == game_mode::GameMode::Victory {
                d.draw_text_ex(
                    &assets.hud_font,
                    victory_text,
                    Vector2::new(
                        SCREEN_WIDTH as f32 / 2.0 - vtext_size.x / 2.0,
                        SCREEN_HEIGHT as f32 / 2.0 - vtext_size.y / 2.0,
                    ),
                    font_size,
                    1.0,
                    Color::GOLD,
                );
            } else if game_mode == game_mode::GameMode::Defeat {
                d.draw_text_ex(
                    &assets.hud_font,
                    defeat_text,
                    Vector2::new(
                        SCREEN_WIDTH as f32 / 2.0 - dtext_size.x / 2.0,
                        SCREEN_HEIGHT as f32 / 2.0 - dtext_size.y / 2.0,
                    ),
                    font_size,
                    1.0,
                    Color::RED,
                );
            }
            if game_over {
                for (texture, rect) in [
                    (&assets.btn_retry, btn_retry),
                    (&assets.btn_back, btn_back),
                    (&assets.btn_exit, btn_exit),
                ] {
                    d.draw_texture_ex(
                        texture,
                        Vector2::new(rect.x, rect.y),
                        0.0,
                        1.0,
                        Color::WHITE,
                    );
                }
            }
            d.draw_texture_ex(
                cursor.current_cursor_texture,
                Vector2::new(mouse_position.x, mouse_position.y),
                0.0,
                0.7,
                Color::WHITE,
            );
        } else {
            // écran de combat --------------------------------------------------
            d.draw_texture_pro(
                &assets.combat_screen_background_texture,
                Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: assets.combat_screen_background_texture.width as f32,
                    height: assets.combat_screen_background_texture.height as f32,
                },
                Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: SCREEN_WIDTH as f32,
                    height: SCREEN_HEIGHT as f32,
                },
                Vector2::new(0.0, 0.0),
                0.0,
                Color::WHITE,
            );

            // HUD combat -------------------------------------------------------
            let bar_width = 300.0;
            let bar_height = 30.0;
            let padding = 50.0;
            let bar_y = padding;

            if soldier.facing_left {
                ui::draw_health_bar(
                    &mut d,
                    SCREEN_WIDTH as f32 - padding - bar_width,
                    bar_y,
                    bar_width,
                    bar_height,
                    soldier.hp_points,
                    soldier.hp_max_points,
                );
                d.draw_text_ex(
                    &assets.hud_font,
                    &soldier.name,
                    Vector2::new(
                        SCREEN_WIDTH as f32 - padding - bar_width,
                        bar_y + bar_height + 10.0,
                    ),
                    24.0,
                    1.0,
                    Color::BLACK,
                );
            } else {
                ui::draw_health_bar(
                    &mut d,
                    padding,
                    bar_y,
                    bar_width,
                    bar_height,
                    soldier.hp_points,
                    soldier.hp_max_points,
                );
                d.draw_text_ex(
                    &assets.hud_font,
                    &soldier.name,
                    Vector2::new(padding, bar_y + bar_height + 10.0),
                    24.0,
                    1.0,
                    Color::BLACK,
                );
            }

            if wraith.facing_left {
                ui::draw_health_bar(
                    &mut d,
                    SCREEN_WIDTH as f32 - padding - bar_width,
                    bar_y,
                    bar_width,
                    bar_height,
                    wraith.hp_points,
                    wraith.hp_max_points,
                );
                d.draw_text_ex(
                    &assets.hud_font,
                    &wraith.name,
                    Vector2::new(
                        SCREEN_WIDTH as f32 - padding - bar_width,
                        bar_y + bar_height + 10.0,
                    ),
                    24.0,
                    1.0,
                    Color::BLACK,
                );
            } else {
                ui::draw_health_bar(
                    &mut d,
                    padding,
                    bar_y,
                    bar_width,
                    bar_height,
                    wraith.hp_points,
                    wraith.hp_max_points,
                );

                d.draw_text_ex(
                    &assets.hud_font,
                    &wraith.name,
                    Vector2::new(padding, bar_y + bar_height + 10.0),
                    24.0,
                    1.0,
                    Color::BLACK,
                );
            }
            // -------------------------------------------------------------------------------
            let sprite_size = 500.0;
            let combat_y = SCREEN_HEIGHT as f32 / 2.0 - sprite_size / 2.0;

            if soldier.is_alive() {
                d.draw_texture_pro(
                    &current_animation.texture,
                    source_rec_soldier,
                    Rectangle {
                        x: soldier_combat_x,
                        y: combat_y,
                        width: sprite_size,
                        height: sprite_size,
                    },
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            }
            if soldier.state == UnitState::Attacking {
                let effect_animation = &assets.soldier.attack_effect;
                let mut source_rec_effect = effect_animation.animation_frame();
                if soldier.facing_left {
                    source_rec_effect.width = -source_rec_effect.width;
                }

                d.draw_texture_pro(
                    &effect_animation.texture,
                    source_rec_effect,
                    Rectangle {
                        x: soldier_combat_x,
                        y: combat_y,
                        width: sprite_size,
                        height: sprite_size,
                    },
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            }
            if wraith.state == UnitState::Attacking {
                let effect_animation = &assets.wraith.attack_effect;
                let mut source_rec_effect = effect_animation.animation_frame();
                if wraith.facing_left {
                    source_rec_effect.width = -source_rec_effect.width;
                }

                d.draw_texture_pro(
                    &effect_animation.texture,
                    source_rec_effect,
                    Rectangle {
                        x: wraith_combat_x,
                        y: combat_y,
                        width: sprite_size,
                        height: sprite_size,
                    },
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            }
            if wraith.is_alive() {
                d.draw_texture_pro(
                    &current_wraith_animation.texture,
                    source_rec_wraith,
                    Rectangle {
                        x: wraith_combat_x,
                        y: combat_y,
                        width: sprite_size,
                        height: sprite_size,
                    },
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
            }
        }
    }
}
