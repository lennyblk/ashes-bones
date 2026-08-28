use raylib::consts::MouseButton::*;
use raylib::prelude::*;
use std::collections::HashMap;

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

struct Character {
    grid_x: i32,
    grid_y: i32,
    screen_x: f32,
    screen_y: f32,
    move_points: i32,
    hp_points: i32,
    path: Vec<(i32, i32)>,
}

impl Character {
    fn update_position(&mut self, delta_time: f32) {
        let target_x = self.grid_x * TILE_SIZE - 40;
        let target_y = self.grid_y * TILE_SIZE - 40;

        let distance_x = target_x as f32 - self.screen_x;
        let distance_y = target_y as f32 - self.screen_y;

        // abs (recup la valeur absolu de la distance restante) pour recup toujours un valeur positif que j'aille a gauche ou a droite, comme ca je snap pas trop tot si c'est negatif
        if distance_x.abs() > 1.0 {
            self.screen_x += distance_x * delta_time * 5.0;
        } else {
            self.screen_x = target_x as f32;
        }

        if distance_y.abs() > 1.0 {
            self.screen_y += distance_y * delta_time * 5.0;
        } else {
            self.screen_y = target_y as f32;
        }
    }
    fn advance_path(&mut self) {
        if self.path.is_empty() {
            return;
        }
        let target = self.path[0];
        let target_screen_x = (target.0 * TILE_SIZE - 40) as f32;
        let target_screen_y = (target.1 * TILE_SIZE - 40) as f32;

        if (self.screen_x - target_screen_x).abs() < 1.0
            && (self.screen_y - target_screen_y).abs() < 1.0
        {
            self.path.remove(0);
            return;
        }
        self.grid_x = target.0;
        self.grid_y = target.1;
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
    ) -> (Vec<(i32, i32)>, HashMap<(i32, i32), (i32, i32)>) {
        let mut visited: Vec<(i32, i32)> = vec![(start_x, start_y)];
        let mut queue: Vec<(i32, i32, i32)> = vec![(start_x, start_y, 0)]; // x, y, coût actuel
        let mut index = 0;
        let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();

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
                    came_from.insert((nx, ny), (x, y));
                }
            }
        }
        (visited, came_from)
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
