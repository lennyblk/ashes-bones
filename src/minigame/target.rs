use crate::duel::DuelSide;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use raylib::prelude::*;

use super::{Attempt, TimingResult};

// conversion des réglages génériques du duel en pixels
const RADIUS_PER_WINDOW: f32 = 200.0; // window 20% -> cible de 40 px de rayon
const BULLSEYE_PER_PRECISION: f32 = 300.0; // precision 3% -> centre de 9 px de rayon
// vitesses angulaires du mouvement (rad/s, avant tempo) : deux fréquences
// différentes donnent une trajectoire en 8 difficile à anticiper
const FREQ_X: f32 = 1.6;
const FREQ_Y: f32 = 2.3;

/// tir sur cible (Longbowman) : une cible se balade dans l'arène, on clique dessus.
/// window = rayon de la cible, precision = rayon du centre, tempo = vitesse de la cible
pub struct TargetShot {
    radius: f32,
    bullseye: f32,
    tempo: f32,
    time: f32,
    phase_x: f32,
    phase_y: f32,
    aim: Vector2,
    // tirs ratés pardonnés par une vie, affichés comme impacts
    misses: Vec<Vector2>,
}

/// zone où la cible se déplace, en bas de l'écran de combat
pub fn arena() -> Rectangle {
    let width = 600.0;
    let height = 220.0;
    Rectangle {
        x: SCREEN_WIDTH as f32 / 2.0 - width / 2.0,
        y: SCREEN_HEIGHT as f32 - height - 30.0,
        width,
        height,
    }
}

impl TargetShot {
    /// départ aléatoire sur la trajectoire pour que chaque tir soit différent
    pub fn new(rl: &RaylibHandle, side: &DuelSide) -> TargetShot {
        let random_phase = || rl.get_random_value::<i32>(0..=628) as f32 / 100.0;
        TargetShot {
            radius: side.window * RADIUS_PER_WINDOW,
            bullseye: side.precision * BULLSEYE_PER_PRECISION,
            tempo: side.tempo,
            time: 0.0,
            phase_x: random_phase(),
            phase_y: random_phase(),
            aim: rl.get_mouse_position(),
            misses: Vec::new(),
        }
    }

    pub fn area(&self) -> Rectangle {
        arena()
    }

    /// centre de la cible : un 8 (Lissajous) qui reste dans l'arène
    fn target_center(&self) -> Vector2 {
        let arena = arena();
        let amplitude_x = arena.width / 2.0 - self.radius;
        let amplitude_y = arena.height / 2.0 - self.radius;
        Vector2::new(
            arena.x + arena.width / 2.0 + amplitude_x * (FREQ_X * self.time + self.phase_x).sin(),
            arena.y + arena.height / 2.0 + amplitude_y * (FREQ_Y * self.time + self.phase_y).sin(),
        )
    }

    pub fn update(&mut self, rl: &RaylibHandle, delta_time: f32) -> Option<Attempt> {
        self.time += delta_time * self.tempo;
        self.aim = rl.get_mouse_position();

        if !rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            return None;
        }
        let distance = self.aim.distance(self.target_center());
        Some(if distance <= self.bullseye {
            Attempt::Hit(TimingResult::Perfect)
        } else if distance <= self.radius {
            Attempt::Hit(TimingResult::Good)
        } else {
            self.misses.push(self.aim);
            Attempt::Miss
        })
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let arena = arena();
        d.draw_rectangle_rec(arena, Color::new(20, 20, 20, 170));
        d.draw_rectangle_lines_ex(arena, 2.0, Color::new(255, 255, 255, 120));

        // cible : jaune = GOOD, vert = PERFECT (mêmes couleurs que la barre)
        let center = self.target_center();
        d.draw_circle_v(center, self.radius, Color::YELLOW);
        d.draw_ring(
            center,
            self.radius - 2.0,
            self.radius,
            0.0,
            360.0,
            32,
            Color::BLACK,
        );
        if self.bullseye > 0.0 {
            d.draw_circle_v(center, self.bullseye, Color::GREEN);
        }

        for miss in &self.misses {
            let s = 6.0;
            let miss = *miss;
            d.draw_line_ex(
                miss - Vector2::new(s, s),
                miss + Vector2::new(s, s),
                2.0,
                Color::RED,
            );
            d.draw_line_ex(
                miss - Vector2::new(s, -s),
                miss + Vector2::new(s, -s),
                2.0,
                Color::RED,
            );
        }

        // viseur (le curseur système est caché pendant le jeu)
        let aim = self.aim;
        d.draw_ring(aim, 10.0, 12.0, 0.0, 360.0, 24, Color::WHITE);
        d.draw_line_ex(
            aim - Vector2::new(18.0, 0.0),
            aim - Vector2::new(5.0, 0.0),
            2.0,
            Color::WHITE,
        );
        d.draw_line_ex(
            aim + Vector2::new(5.0, 0.0),
            aim + Vector2::new(18.0, 0.0),
            2.0,
            Color::WHITE,
        );
        d.draw_line_ex(
            aim - Vector2::new(0.0, 18.0),
            aim - Vector2::new(0.0, 5.0),
            2.0,
            Color::WHITE,
        );
        d.draw_line_ex(
            aim + Vector2::new(0.0, 5.0),
            aim + Vector2::new(0.0, 18.0),
            2.0,
            Color::WHITE,
        );
    }
}
