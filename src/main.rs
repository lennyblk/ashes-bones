use raylib::consts::MouseButton;
use raylib::prelude::*;

const TILE_SIZE: i32 = 48;
const GRID_COLS: i32 = 25;
const GRID_ROWS: i32 = 16;
const SCREEN_WIDTH: i32 = GRID_COLS * TILE_SIZE;
const SCREEN_HEIGHT: i32 = GRID_ROWS * TILE_SIZE;

struct Animation {
    texture: Texture2D,
    frame_width: i32,
    frame_height: i32,
    frames_per_row: i32,
    first: i32,
    last: i32,
    current: i32,
    speed: f32,
    duration_left: f32,
}

impl Animation {
    fn animation_update(&mut self, delta_time: f32) {
        self.duration_left -= delta_time;

        if self.duration_left <= 0.0 {
            self.current += 1;
            if self.current > self.last {
                self.current = self.first;
            }
            self.duration_left = 1.0 / self.speed;
        }
    }

    fn animation_frame(&self) -> Rectangle {
        let col = self.current % self.frames_per_row;
        let row = self.current / self.frames_per_row;

        Rectangle {
            x: (col * self.frame_width) as f32,
            y: (row * self.frame_height) as f32,
            width: self.frame_width as f32,
            height: self.frame_height as f32,
        }
    }
}

struct Cursors<'a> {
    current_cursor_texture: &'a Texture2D,
    position: Vector2,
}

impl<'a> Cursors<'a> {
    fn update_cursor(
        normal_texture: &'a Texture2D,
        hover_texture: &'a Texture2D,
        cursor_grid_x: i32,
        cursor_grid_y: i32,
        char_grid_x: i32,
        char_grid_y: i32,
    ) -> Self {
        let is_hovering = cursor_grid_x == char_grid_x && cursor_grid_y == char_grid_y;
        let current_cursor_texture = if is_hovering {
            hover_texture
        } else {
            normal_texture
        };

        Cursors {
            current_cursor_texture,
            position: Vector2::new(
                (cursor_grid_x * TILE_SIZE) as f32,
                (cursor_grid_y * TILE_SIZE) as f32,
            ),
        }
    }
}

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

    // run window
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();

        animation.animation_update(delta_time);

        let mouse_position = rl.get_mouse_position();

        let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
        let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

        let cursor = Cursors::update_cursor(
            &mouse_normal_texture,
            &mouse_hover_texture,
            cursor_grid_x,
            cursor_grid_y,
            0, // char_grid_x
            0, // char_grid_y
        );

        // drawing
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        for i in (0..SCREEN_HEIGHT).step_by(TILE_SIZE as usize) {
            d.draw_rectangle_lines(0, i, SCREEN_WIDTH, 1, Color::BLACK);
        }

        for i in (0..SCREEN_WIDTH).step_by(TILE_SIZE as usize) {
            d.draw_rectangle_lines(i, 0, 1, SCREEN_HEIGHT, Color::BLACK);
        }

        d.draw_texture_pro(
            &animation.texture,
            animation.animation_frame(),
            Rectangle {
                x: grid_to_screen_x(0) as f32,
                y: grid_to_screen_y(0) as f32,
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
