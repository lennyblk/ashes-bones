use raylib::prelude::*;

pub struct Animation {
    pub texture: Texture2D,
    pub frame_width: i32,
    pub frame_height: i32,
    pub frames_per_row: i32,
    pub first: i32,
    pub last: i32,
    pub current: i32,
    pub speed: f32,
    pub duration_left: f32,
}

impl Animation {
    pub fn animation_update(&mut self, delta_time: f32) {
        self.duration_left -= delta_time;

        if self.duration_left <= 0.0 {
            self.current += 1;
            if self.current > self.last {
                self.current = self.first;
            }
            self.duration_left = 1.0 / self.speed;
        }
    }

    pub fn animation_frame(&self) -> Rectangle {
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
