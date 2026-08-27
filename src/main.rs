use raylib::prelude::*;

const SCREEN_WIDTH: i32 = 800;
const SCREEN_HEIGHT: i32 = 600;

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

fn main() {
    // init window
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Ashes&Bones")
        .build();

    //load texture
    let human_idle_texture = rl
        .load_texture(
            &thread,
            "assets/humanChar/Human soldier/Human soldier/human_soldier-Idle.png",
        )
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

    // run window
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();

        animation.animation_update(delta_time);

        // drawing
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        for i in (40..SCREEN_HEIGHT).step_by(40) {
            d.draw_rectangle_lines(0, i, SCREEN_WIDTH, 1, Color::BLACK);
        }

        for i in (40..SCREEN_WIDTH).step_by(40) {
            d.draw_rectangle_lines(i, 0, 1, SCREEN_HEIGHT, Color::BLACK);
        }

        d.draw_texture_pro(
            &animation.texture,
            animation.animation_frame(),
            Rectangle {
                x: -40.0 as f32,
                y: -40.0 as f32,
                width: 128.0,
                height: 128.0,
            },
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
    }
}
