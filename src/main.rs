use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;
mod animation;
mod character;
mod cursor;
mod movement;

use animation::Animation;
use character::Character;
use cursor::Cursors;
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

    let mouse_normal_texture = rl
        .load_texture(&thread, "assets/cursors/PNG/01.png")
        .unwrap();
    let mouse_hover_texture = rl
        .load_texture(&thread, "assets/cursors/PNG/10.png")
        .unwrap();

    let mouse_click_texture = rl
        .load_texture(&thread, "assets/cursors/PNG/17.png")
        .unwrap();

    let mut animation = Animation {
        texture: human_idle_texture,
        frame_width: 130,
        frame_height: 100,
        frames_per_row: 7,
        first: 0,
        last: 6,
        current: 0,
        speed: 8.0,
        duration_left: 0.1,
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
        move_points: 3,
        hp_points: 100,
        path: Vec::new(),
    };

    // run window --------------------------------------------------------------
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();

        animation.animation_update(delta_time);
        character.update_position(delta_time);
        character.advance_path();

        let (move_range, came_from) = if cursor.is_selected {
            MovementRange::compute_movement_range(
                character.grid_x,
                character.grid_y,
                character.move_points,
                GRID_COLS,
                GRID_ROWS,
            )
        } else {
            (Vec::new(), HashMap::new())
        };

        let mouse_position = rl.get_mouse_position();
        fn mouse_is_clicked(rl: &RaylibHandle) -> bool {
            rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        }

        let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
        let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

        if mouse_is_clicked(&rl) && cursor.is_selected {
            if move_range.contains(&(cursor_grid_x, cursor_grid_y)) {
                let mut path = vec![(cursor_grid_x, cursor_grid_y)];
                let mut current = (cursor_grid_x, cursor_grid_y);
                while let Some(&parent) = came_from.get(&current) {
                    path.push(parent);
                    current = parent;
                }
                path.reverse();
                character.path = path;
            }
        }

        cursor.update_cursor(
            &mouse_normal_texture,
            &mouse_hover_texture,
            &mouse_click_texture,
            cursor_grid_x,
            cursor_grid_y,
            character.grid_x,
            character.grid_y,
            mouse_is_clicked(&rl),
        );

        // drawing --------------------------------------------------------------
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

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
        d.draw_texture_pro(
            &animation.texture,
            animation.animation_frame(),
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

        d.draw_texture_ex(
            cursor.current_cursor_texture,
            Vector2::new(mouse_position.x, mouse_position.y),
            0.0,
            0.7,
            Color::WHITE,
        );
    }
}
