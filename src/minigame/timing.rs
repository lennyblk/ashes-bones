use crate::duel::DuelSide;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use raylib::prelude::*;

use super::{Attempt, TimingResult};

/// barre de timing : un curseur fait des allers-retours, on clique dans la zone.
/// window = largeur de la zone jaune, precision = zone verte, tempo = vitesse du curseur
pub struct TimingBar {
    pub cursor_position: f32,
    pub speed: f32,
    pub zone_start: f32,
    pub zone_end: f32,
    pub perfect_zone_start: f32,
    pub perfect_zone_end: f32,
}

impl TimingBar {
    /// zone placée aléatoirement (jamais collée aux bords)
    pub fn new(rl: &RaylibHandle, side: &DuelSide) -> TimingBar {
        let min_start = 10;
        let max_start = (100.0 * (0.9 - side.window)) as i32;
        let zone_start =
            rl.get_random_value::<i32>(min_start..=max_start.max(min_start)) as f32 / 100.0;
        let zone_end = zone_start + side.window;

        let zone_center = zone_start + side.window / 2.0;
        let perfect_zone_start = zone_center - side.precision / 2.0;
        let perfect_zone_end = zone_center + side.precision / 2.0;

        TimingBar {
            cursor_position: 0.0,
            speed: side.tempo,
            zone_start,
            zone_end,
            perfect_zone_start,
            perfect_zone_end,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, delta_time: f32) -> Option<Attempt> {
        self.cursor_position += self.speed * delta_time;
        if self.cursor_position > 1.0 {
            self.cursor_position = 1.0;
            self.speed = -self.speed;
        } else if self.cursor_position < 0.0 {
            self.cursor_position = 0.0;
            self.speed = -self.speed;
        }

        if !rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            return None;
        }
        let in_zone =
            |start: f32, end: f32| self.cursor_position >= start && self.cursor_position <= end;
        Some(if in_zone(self.perfect_zone_start, self.perfect_zone_end) {
            Attempt::Hit(TimingResult::Perfect)
        } else if in_zone(self.zone_start, self.zone_end) {
            Attempt::Hit(TimingResult::Good)
        } else {
            Attempt::Miss
        })
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let bar_x = SCREEN_WIDTH as f32 / 2.0 - 200.0;
        let bar_y = SCREEN_HEIGHT as f32 - 120.0;
        let bar_width = 400.0;
        let bar_height = 30.0;

        // fond de la barre
        d.draw_rectangle(
            bar_x as i32,
            bar_y as i32,
            bar_width as i32,
            bar_height as i32,
            Color::DARKGRAY,
        );

        // zone good
        let good_x = bar_x + self.zone_start * bar_width;
        let good_width = (self.zone_end - self.zone_start) * bar_width;
        d.draw_rectangle(
            good_x as i32,
            bar_y as i32,
            good_width as i32,
            bar_height as i32,
            Color::YELLOW,
        );

        // zone perfect
        let perfect_x = bar_x + self.perfect_zone_start * bar_width;
        let perfect_width = (self.perfect_zone_end - self.perfect_zone_start) * bar_width;
        d.draw_rectangle(
            perfect_x as i32,
            bar_y as i32,
            perfect_width as i32,
            bar_height as i32,
            Color::GREEN,
        );

        // curseur (ligne blanche qui bouge)
        let cursor_x = bar_x + self.cursor_position * bar_width;
        d.draw_rectangle(
            cursor_x as i32 - 2,
            bar_y as i32 - 5,
            4,
            bar_height as i32 + 10,
            Color::WHITE,
        );
    }
}
