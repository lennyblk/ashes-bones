use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;
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

    let mut soldier = Unit {
        name: String::from("Soldier"),
        faction: Faction::Human,
        class: UnitClass::Soldier,
        grid_x: 5,
        grid_y: 5,
        screen_x: grid_to_screen_x(5),
        screen_y: grid_to_screen_y(5),
        move_points: 2,
        hp_points: 100,
        hp_max_points: 100,
        path: Vec::new(),
        state: UnitState::Idle,
        facing_left: false,
        attack_range: 1,
        attack_power: 20,
        defense: 5,
        attack_target: false,
    };

    let mut wraith = Unit {
        name: String::from("Wraith"),
        faction: Faction::Undead,
        class: UnitClass::Wraith,
        grid_x: 6,
        grid_y: 5,
        screen_x: grid_to_screen_x(6),
        screen_y: grid_to_screen_y(5),
        move_points: 4,
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

    let mut attack_animation_started = false;
    let mut game_mode = game_mode::GameMode::GridScreen;
    let mut soldier_combat_x: f32 = 0.0;
    let mut wraith_combat_x: f32 = 0.0;
    let mut combat_entering_timer: f32 = 0.0;
    let mut combat_exit_pause_timer: f32 = 0.0;
    let tile_map = map::TileMap::load(&mut rl, &thread, "assets/maps/ashes-bones-map.tmx");
    let blocked_tiles = tile_map.blocked_tiles.clone();

    // run window --------------------------------------------------------------
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();

        soldier.update_position(delta_time);
        soldier.advance_path();

        wraith.update_position(delta_time);
        wraith.advance_path();

        // combat -----------------------------------------------------------
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
            &mut game_mode,
            &mut combat_exit_pause_timer,
            delta_time,
        );

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

        combat::enter_combat(
            &mut soldier,
            &mut wraith,
            &mut soldier_combat_x,
            &mut wraith_combat_x,
            soldier_target_x,
            wraith_target_x,
            &mut combat_entering_timer,
            delta_time,
            &mut assets.soldier.attack,
        );

        let (move_range, came_from) = if cursor.is_selected {
            MovementRange::compute_movement_range(
                soldier.grid_x,
                soldier.grid_y,
                soldier.move_points,
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

        let mouse_position = rl.get_mouse_position();
        fn mouse_is_clicked(rl: &RaylibHandle) -> bool {
            rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        }

        input::cancel_pressed(&rl);

        let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
        let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

        let mut click_consumed = false;

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

        let valid_attack_positions = MovementRange::compute_attackable_positions(
            &move_range,
            wraith.grid_x,
            wraith.grid_y,
            soldier.attack_range,
            &wraith,
        );
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
        if game_mode == game_mode::GameMode::GridScreen {
            d.clear_background(Color::BEIGE);
            tile_map.draw(&mut d);

            // for i in (0..SCREEN_HEIGHT).step_by(TILE_SIZE as usize) {
            //     d.draw_rectangle_lines(0, i, SCREEN_WIDTH, 1, Color::BLACK);
            // }
            //
            // for i in (0..SCREEN_WIDTH).step_by(TILE_SIZE as usize) {
            //     d.draw_rectangle_lines(i, 0, 1, SCREEN_HEIGHT, Color::BLACK);
            // }

            for (x, y) in &move_range {
                d.draw_rectangle(
                    x * TILE_SIZE,
                    y * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    Color::new(0, 100, 255, 100),
                );
            }

            if soldier.state == UnitState::ChoosingPosition {
                for (x, y) in &valid_attack_positions {
                    d.draw_rectangle(
                        x * TILE_SIZE,
                        y * TILE_SIZE,
                        TILE_SIZE,
                        TILE_SIZE,
                        Color::new(255, 255, 0, 170),
                    );
                }
            }

            if wraith_attackable {
                d.draw_rectangle(
                    wraith.grid_x * TILE_SIZE,
                    wraith.grid_y * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    Color::new(255, 0, 0, 180),
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

            if wraith.is_alive() {
                d.draw_texture_pro(
                    &current_wraith_animation.texture,
                    source_rec_wraith,
                    Rectangle {
                        x: grid_to_screen_x(wraith.grid_x),
                        y: grid_to_screen_y(wraith.grid_y),
                        width: 128.0,
                        height: 128.0,
                    },
                    Vector2::new(0.0, 0.0),
                    0.0,
                    Color::WHITE,
                );
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

            ui::draw_health_bar(
                &mut d,
                padding,
                bar_y,
                bar_width,
                bar_height,
                soldier.hp_points,
                soldier.hp_max_points,
            );

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
                &soldier.name,
                Vector2::new(padding, bar_y + bar_height + 10.0),
                24.0,
                1.0,
                Color::BLACK,
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
            // -------------------------------------------------------------------------------
            let sprite_size = 500.0;
            let combat_y = SCREEN_HEIGHT as f32 / 2.0 - sprite_size / 2.0;

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
