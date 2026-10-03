use crate::duel::DuelSide;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use raylib::prelude::*;

use super::{Attempt, TimingResult};

// durée de la canalisation (fixe, le tempo ne change que la vitesse de l'aura)
const CHANNEL_TIME: f32 = 3.0;
const AURA_PER_WINDOW: f32 = 250.0; // window 20% -> aura de 50 px de rayon
const CORE_PER_PRECISION: f32 = 600.0; // precision 3% -> cœur de 18 px de rayon
const CORE_FOCUS: f32 = 1.0;
const AURA_FOCUS: f32 = 0.6;
// seuils de concentration finale (part de CHANNEL_TIME)
const GOOD_FOCUS: f32 = 0.5;
const PERFECT_FOCUS: f32 = 0.85;
// vitesses angulaires de la dérive (rad/s, avant tempo), plus lentes que la cible de l'archer
const FREQ_X: f32 = 0.9;
const FREQ_Y: f32 = 1.4;
const METER_HEIGHT: f32 = 14.0;

/// canalisation (Priest / Necromancer) : garder le curseur dans une aura qui dérive.
/// window = taille de l'aura, precision = taille du cœur (PERFECT), tempo = vitesse de dérive
pub struct Channel {
    aura_radius: f32,
    core_radius: f32,
    tempo: f32,
    time: f32,
    // concentration accumulée, en secondes « pleines »
    focus: f32,
    phase_x: f32,
    phase_y: f32,
    aim: Vector2,
}

fn random_phase(rl: &RaylibHandle) -> f32 {
    rl.get_random_value::<i32>(0..=628) as f32 / 100.0
}

/// zone où l'aura dérive (même place que l'arène de l'archer)
fn arena() -> Rectangle {
    let width = 600.0;
    let height = 200.0;
    Rectangle {
        x: SCREEN_WIDTH as f32 / 2.0 - width / 2.0,
        y: SCREEN_HEIGHT as f32 - height - 50.0,
        width,
        height,
    }
}

fn meter() -> Rectangle {
    let arena = arena();
    Rectangle::new(
        arena.x,
        arena.y + arena.height + 12.0,
        arena.width,
        METER_HEIGHT,
    )
}

impl Channel {
    pub fn new(rl: &RaylibHandle, side: &DuelSide) -> Channel {
        let aura_radius = side.window * AURA_PER_WINDOW;
        Channel {
            aura_radius,
            // le cœur reste plus petit que l'aura même avec beaucoup de precision
            core_radius: (side.precision * CORE_PER_PRECISION).min(aura_radius * 0.8),
            tempo: side.tempo,
            time: 0.0,
            focus: 0.0,
            phase_x: random_phase(rl),
            phase_y: random_phase(rl),
            aim: rl.get_mouse_position(),
        }
    }

    pub fn area(&self) -> Rectangle {
        let arena = arena();
        Rectangle {
            height: arena.height + 12.0 + METER_HEIGHT,
            ..arena
        }
    }

    fn aura_center(&self) -> Vector2 {
        let arena = arena();
        let t = self.time * self.tempo;
        let amplitude_x = arena.width / 2.0 - self.aura_radius;
        let amplitude_y = arena.height / 2.0 - self.aura_radius;
        Vector2::new(
            arena.x + arena.width / 2.0 + amplitude_x * (FREQ_X * t + self.phase_x).sin(),
            arena.y + arena.height / 2.0 + amplitude_y * (FREQ_Y * t + self.phase_y).sin(),
        )
    }

    /// concentration finale si on s'arrêtait maintenant (0..1)
    fn focus_ratio(&self) -> f32 {
        (self.focus / CHANNEL_TIME).min(1.0)
    }

    pub fn update(&mut self, rl: &RaylibHandle, delta_time: f32) -> Option<Attempt> {
        self.aim = rl.get_mouse_position();
        let distance = self.aim.distance(self.aura_center());
        if distance <= self.core_radius {
            self.focus += CORE_FOCUS * delta_time;
        } else if distance <= self.aura_radius {
            self.focus += AURA_FOCUS * delta_time;
        }

        self.time += delta_time;
        if self.time < CHANNEL_TIME {
            return None;
        }
        let ratio = self.focus_ratio();
        if ratio >= PERFECT_FOCUS {
            return Some(Attempt::Hit(TimingResult::Perfect));
        }
        if ratio >= GOOD_FOCUS {
            return Some(Attempt::Hit(TimingResult::Good));
        }
        // pas assez concentré : raté, nouvelle canalisation (jouée s'il reste une vie)
        self.time = 0.0;
        self.focus = 0.0;
        self.phase_x = random_phase(rl);
        self.phase_y = random_phase(rl);
        Some(Attempt::Miss)
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let arena = arena();
        d.draw_rectangle_rec(arena, Color::new(20, 20, 20, 170));
        d.draw_rectangle_lines_ex(arena, 2.0, Color::new(255, 255, 255, 120));

        // aura : plus lumineuse quand le curseur est dedans
        let center = self.aura_center();
        let distance = self.aim.distance(center);
        let inside = distance <= self.aura_radius;
        let glow = if inside { 90 } else { 40 };
        d.draw_circle_v(
            center,
            self.aura_radius + 10.0,
            Color::new(255, 240, 150, glow),
        );
        d.draw_circle_v(center, self.aura_radius, Color::YELLOW);
        if self.core_radius > 0.0 {
            d.draw_circle_v(center, self.core_radius, Color::GREEN);
        }

        // jauge de concentration : sombre = BAD, jaune = GOOD, vert = PERFECT
        let meter = meter();
        d.draw_rectangle_rec(meter, Color::new(60, 60, 60, 230));
        let good_x = meter.x + meter.width * GOOD_FOCUS;
        let perfect_x = meter.x + meter.width * PERFECT_FOCUS;
        d.draw_rectangle(
            good_x as i32,
            meter.y as i32,
            (perfect_x - good_x) as i32,
            meter.height as i32,
            Color::new(255, 255, 0, 110),
        );
        d.draw_rectangle(
            perfect_x as i32,
            meter.y as i32,
            (meter.x + meter.width - perfect_x) as i32,
            meter.height as i32,
            Color::new(0, 228, 48, 110),
        );
        let fill = meter.width * self.focus_ratio();
        d.draw_rectangle(
            meter.x as i32,
            meter.y as i32 + 3,
            fill as i32,
            meter.height as i32 - 6,
            Color::WHITE,
        );
        // avancement de la canalisation
        let progress_x = meter.x + meter.width * (self.time / CHANNEL_TIME).min(1.0);
        d.draw_rectangle(
            progress_x as i32 - 1,
            meter.y as i32 - 4,
            2,
            meter.height as i32 + 8,
            Color::new(255, 255, 255, 140),
        );
        d.draw_rectangle_lines_ex(meter, 1.0, Color::BLACK);

        // pointeur (le curseur système est caché pendant le jeu)
        d.draw_circle_v(self.aim, 6.0, Color::WHITE);
        d.draw_circle_lines(self.aim.x as i32, self.aim.y as i32, 6.0, Color::BLACK);
    }
}
