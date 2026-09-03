use raylib::consts::MouseButton::*;
use raylib::ffi::CSSPalette;
use raylib::prelude::*;
use std::collections::HashMap;
mod animation;
mod character;
mod combat;
mod cursor;
mod enemy;
mod input;
mod movement;

use animation::Animation;
use character::Character;
use cursor::Cursors;
use enemy::Enemy;
use movement::MovementRange;

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

    //load texture
    let human_idle_texture = rl
        .load_texture(
            &thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Idle.png",
        )
        .unwrap();
    let human_walking_texture = rl
        .load_texture(
            &thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Walk.png",
        )
        .unwrap();

    let human_attack_texture = rl
        .load_texture(
            &thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Attact 1.png",
        )
        .unwrap();

    let enemy_idle_texture = rl
        .load_texture(
            &thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith/Wraith-Idle.png",
        )
        .unwrap();

    let enemy_hurt_texture = rl
        .load_texture(
            &thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith_split shadows/Wraith-Hurt.png",
        )
        .unwrap();

    let enemy_dying_texture = rl
        .load_texture(
            &thread,
            "assets/undeadChar/Undead Wraith 32x32/Undead Wraith_split shadows/Wraith-Die.png",
        )
        .unwrap();

    let mouse_normal_texture = rl
        .load_texture(&thread, "assets/cursors/PNG/01.png")
        .unwrap();
    let mouse_hover_texture = rl
        .load_texture(&thread, "assets/cursors/PNG/10.png")
        .unwrap();

    let mouse_click_texture = rl
        .load_texture(&thread, "assets/cursors/PNG/13.png")
        .unwrap();

    let mouse_select_texture = rl
        .load_texture(&thread, "assets/cursors/selector_frame_v2.png")
        .unwrap();

    let mut human_idle_animation = Animation {
        texture: human_idle_texture,
        frame_width: 130,
        frame_height: 100,
        frames_per_row: 7,
        first: 0,
        last: 6,
        current: 0,
        speed: 8.0,
        duration_left: 0.1,
        finished: false,
        looping: true,
    };

    let mut human_walking_animation = Animation {
        texture: human_walking_texture,
        frame_width: 130,
        frame_height: 100,
        frames_per_row: 8,
        first: 0,
        last: 7,
        current: 0,
        speed: 8.0,
        duration_left: 0.1,
        finished: false,
        looping: true,
    };

    let mut human_attack_animation = Animation {
        texture: human_attack_texture,
        frame_width: 130,
        frame_height: 100,
        frames_per_row: 7,
        first: 0,
        last: 6,
        current: 0,
        speed: 8.0,
        duration_left: 0.1,
        finished: false,
        looping: false,
    };

    let mut enemy_idle_animation = Animation {
        texture: enemy_idle_texture,
        frame_width: 160,
        frame_height: 160,
        frames_per_row: 8,
        first: 0,
        last: 7,
        current: 0,
        speed: 8.0,
        duration_left: 0.1,
        finished: false,
        looping: true,
    };

    let mut enemy_hurt_animation = Animation {
        texture: enemy_hurt_texture,
        frame_width: 160,
        frame_height: 160,
        frames_per_row: 6,
        first: 0,
        last: 5,
        current: 0,
        speed: 8.0,
        duration_left: 0.1,
        finished: false,
        looping: false,
    };

    let mut enemy_dying_animation = Animation {
        texture: enemy_dying_texture,
        frame_width: 160,
        frame_height: 160,
        frames_per_row: 8,
        first: 0,
        last: 7,
        current: 0,
        speed: 8.0,
        duration_left: 0.1,
        finished: false,
        looping: false,
    };

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
        current_cursor_texture: &mouse_normal_texture,
        position: Vector2::new(
            (cursor_grid_x * TILE_SIZE) as f32,
            (cursor_grid_y * TILE_SIZE) as f32,
        ),
        is_selected: false,
    };

    let mut character = Character {
        grid_x: 5,
        grid_y: 5,
        screen_x: grid_to_screen_x(5),
        screen_y: grid_to_screen_y(5),
        move_points: 2,
        hp_points: 100,
        path: Vec::new(),
        state: character::CharacterState::Idle,
        facing_left: false,
        attack_range: 1,
        attack_power: 20,
        defense: 5,
        attack_target: false,
    };

    let mut enemy = Enemy {
        grid_x: 6,
        grid_y: 5,
        hp_points: 100,
        defense: 10,
        state: enemy::EnemyState::Idle,
        facing_left: false,
    };

    let mut attack_animation_started = false;

    // run window --------------------------------------------------------------
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();

        character.update_position(delta_time);
        character.advance_path();

        if character.state == character::CharacterState::Combat && !attack_animation_started {
            human_attack_animation.current = 0;
            human_attack_animation.finished = false;
            attack_animation_started = true;

            if character.grid_x < enemy.grid_x {
                character.facing_left = false;
                enemy.facing_left = true;
            } else {
                character.facing_left = true;
                enemy.facing_left = false;
            }
        }

        // attack vers l'enemy quand je suis en state Combat
        if character.state == character::CharacterState::Combat {
            if human_attack_animation.finished {
                enemy.hp_points -=
                    combat::attack_damage_dealt(character.attack_power, enemy.defense);
                if enemy.hp_points < 0 {
                    enemy.hp_points = 0;
                }
                enemy_hurt_animation.current = 0;
                enemy_hurt_animation.finished = false;
                enemy.state = enemy::EnemyState::Hurt;

                character.state = character::CharacterState::Idle;
                character.attack_target = false;
                attack_animation_started = false;
                println!("Enemy HP: {}", enemy.hp_points);
            }
        }

        if enemy.state == enemy::EnemyState::Hurt {
            enemy_hurt_animation.animation_update(delta_time);
            if enemy_hurt_animation.finished {
                if enemy.hp_points == 0 {
                    enemy_dying_animation.current = 0;
                    enemy_dying_animation.finished = false;
                    enemy.state = enemy::EnemyState::Dying;
                } else {
                    enemy.state = enemy::EnemyState::Idle;
                }
            }
        }

        if enemy.state == enemy::EnemyState::Dying {
            enemy_dying_animation.animation_update(delta_time);
            if enemy_dying_animation.finished {
                enemy.state = enemy::EnemyState::Dead;
            }
        }

        let (move_range, came_from) = if cursor.is_selected {
            MovementRange::compute_movement_range(
                character.grid_x,
                character.grid_y,
                character.move_points,
                GRID_COLS,
                GRID_ROWS,
                enemy.grid_x,
                enemy.grid_y,
                &enemy,
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
            &mut character,
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
            enemy.grid_x,
            enemy.grid_y,
            character.attack_range,
            &enemy,
        );
        let enemy_attackable = !valid_attack_positions.is_empty();

        // if pour attaquer l'ennemi et bouger le personnage si il y a qu'une seule case possible
        // pour attaquer
        if input::handle_movement_attack_click(
            &rl,
            &mut character,
            &mut cursor,
            &came_from,
            &valid_attack_positions,
            enemy_attackable,
            &enemy,
            cursor_grid_x,
            cursor_grid_y,
        ) {
            click_consumed = true;
        }

        // if pour bouger le personnage vers la case choisie pour attaquer l'ennemi
        if input::handle_movement_choosing_position_click(
            &rl,
            &mut character,
            &mut cursor,
            &came_from,
            &valid_attack_positions,
            cursor_grid_x,
            cursor_grid_y,
        ) {
            click_consumed = true;
        }

        if input::cancel_pressed(&rl)
            && character.state == character::CharacterState::ChoosingPosition
        {
            character.state = character::CharacterState::Idle;
        }

        cursor.update_cursor(
            &mouse_normal_texture,
            &mouse_hover_texture,
            &mouse_click_texture,
            cursor_grid_x,
            cursor_grid_y,
            character.grid_x,
            character.grid_y,
            mouse_is_clicked(&rl) && !click_consumed,
        );

        // drawing --------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::DARKGREEN);

        for i in (0..SCREEN_HEIGHT).step_by(TILE_SIZE as usize) {
            d.draw_rectangle_lines(0, i, SCREEN_WIDTH, 1, Color::BLACK);
        }

        for i in (0..SCREEN_WIDTH).step_by(TILE_SIZE as usize) {
            d.draw_rectangle_lines(i, 0, 1, SCREEN_HEIGHT, Color::BLACK);
        }

        for (x, y) in &move_range {
            d.draw_rectangle(
                x * TILE_SIZE,
                y * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
                Color::new(0, 100, 255, 100), // bleu semi-transparent
            );
        }

        if character.state == character::CharacterState::ChoosingPosition {
            for (x, y) in &valid_attack_positions {
                d.draw_rectangle(
                    x * TILE_SIZE,
                    y * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                    Color::new(255, 255, 0, 170), // jaune semi-transparent
                );
            }
        }

        if enemy_attackable {
            d.draw_rectangle(
                enemy.grid_x * TILE_SIZE,
                enemy.grid_y * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
                Color::new(255, 0, 0, 180), // cases rouge
            );
        }

        let current_animation = match character.state {
            character::CharacterState::Idle => &mut human_idle_animation,
            character::CharacterState::Walking => &mut human_walking_animation,
            character::CharacterState::Combat => &mut human_attack_animation,
            character::CharacterState::ChoosingPosition => &mut human_idle_animation,
        };

        let current_enemy_animation = match enemy.state {
            enemy::EnemyState::Idle => &mut enemy_idle_animation,
            enemy::EnemyState::Hurt => &mut enemy_hurt_animation,
            enemy::EnemyState::Dying => &mut enemy_dying_animation,
            enemy::EnemyState::Dead => &mut enemy_idle_animation,
        };

        if move_range.contains(&(cursor_grid_x, cursor_grid_y)) {
            d.draw_texture_ex(
                &mouse_select_texture,
                Vector2::new(
                    (cursor_grid_x * TILE_SIZE) as f32,
                    (cursor_grid_y * TILE_SIZE) as f32,
                ),
                0.0,
                1.0,
                Color::WHITE,
            );
        }
        // l'animation doit tourner ---------------------------------------------------------------------
        current_animation.animation_update(delta_time);
        current_enemy_animation.animation_update(delta_time);

        let mut source_rec_character = current_animation.animation_frame();
        let mut source_rec_enemy = current_enemy_animation.animation_frame();

        if character.facing_left {
            source_rec_character.width = -source_rec_character.width;
        }

        if enemy.facing_left {
            source_rec_enemy.width = -source_rec_enemy.width;
        }

        d.draw_texture_pro(
            &current_animation.texture,
            source_rec_character,
            Rectangle {
                x: character.screen_x,
                y: character.screen_y,
                width: 128.0,
                height: 128.0,
            },
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );

        if enemy.state != enemy::EnemyState::Dead {
            d.draw_texture_pro(
                &current_enemy_animation.texture,
                source_rec_enemy,
                Rectangle {
                    x: grid_to_screen_x(enemy.grid_x),
                    y: grid_to_screen_y(enemy.grid_y),
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
    }
}
