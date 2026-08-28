use raylib::consts::MouseButton::*;
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

struct MovementRange {
    reachable_tiles: Vec<(i32, i32)>,
}

impl MovementRange {
    fn compute_movement_range(
        start_x: i32,
        start_y: i32,
        move_points: i32,
        grid_cols: i32,
        grid_rows: i32,
    ) -> Vec<(i32, i32)> {
        let mut visited: Vec<(i32, i32)> = vec![(start_x, start_y)];
        let mut queue: Vec<(i32, i32, i32)> = vec![(start_x, start_y, 0)]; // x, y, coût actuel
        let mut index = 0;

        while index < queue.len() {
            let (x, y, cost) = queue[index];
            index += 1;

            if cost >= move_points {
                continue;
            }

            let neighbors = [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)];

            for (nx, ny) in neighbors {
                let in_bounds = nx >= 0 && nx < grid_cols && ny >= 0 && ny < grid_rows;
                let already_visited = visited.contains(&(nx, ny));

                if in_bounds && !already_visited {
                    visited.push((nx, ny));
                    queue.push((nx, ny, cost + 1));
                }
            }
        }
        visited
    }

    fn can_move_to(&mut self, x: i32, y: i32) -> bool {
        self.reachable_tiles.contains(&(x, y))
    }
}
struct Cursors<'a> {
    current_cursor_texture: &'a Texture2D,
    position: Vector2,
    is_selected: bool,
}

impl<'a> Cursors<'a> {
    fn update_cursor(
        &mut self,
        normal_texture: &'a Texture2D,
        hover_texture: &'a Texture2D,
        click_texture: &'a Texture2D,
        cursor_grid_x: i32,
        cursor_grid_y: i32,
        char_grid_x: i32,
        char_grid_y: i32,
        mouse_just_clicked: bool,
    ) {
        let is_hovering = cursor_grid_x == char_grid_x && cursor_grid_y == char_grid_y;

        if mouse_just_clicked {
            if is_hovering {
                self.is_selected = !self.is_selected;
            } else {
                self.is_selected = false;
            }
        }

        self.current_cursor_texture = if self.is_selected {
            click_texture
        } else if is_hovering {
            hover_texture
        } else {
            normal_texture
        };

        self.position = Vector2::new(
            (cursor_grid_x * TILE_SIZE) as f32,
            (cursor_grid_y * TILE_SIZE) as f32,
        );
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

    let mut char_grid_x = 16;
    let mut char_grid_y = 10;

    // run window --------------------------------------------------------------
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();

        animation.animation_update(delta_time);

        let move_range = if cursor.is_selected {
            MovementRange::compute_movement_range(char_grid_x, char_grid_y, 3, GRID_COLS, GRID_ROWS) // 3 = move points
        } else {
            vec![]
        };

        let mouse_position = rl.get_mouse_position();
        fn mouse_is_clicked(rl: &RaylibHandle) -> bool {
            rl.is_mouse_button_pressed(MOUSE_BUTTON_LEFT)
        }

        let cursor_grid_x = mouse_position.x as i32 / TILE_SIZE;
        let cursor_grid_y = mouse_position.y as i32 / TILE_SIZE;

        if mouse_is_clicked(&rl) && cursor.is_selected {
            if move_range.contains(&(cursor_grid_x, cursor_grid_y)) {
                char_grid_x = cursor_grid_x;
                char_grid_y = cursor_grid_y;
            }
        }

        cursor.update_cursor(
            &mouse_normal_texture,
            &mouse_hover_texture,
            &mouse_click_texture,
            cursor_grid_x,
            cursor_grid_y,
            char_grid_x,
            char_grid_y,
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
                x: grid_to_screen_x(char_grid_x) as f32,
                y: grid_to_screen_y(char_grid_y) as f32,
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
