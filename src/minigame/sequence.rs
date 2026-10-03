use crate::duel::{BASE_WINDOW, DuelSide};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use raylib::consts::KeyboardKey::{self, *};
use raylib::prelude::*;

use super::{Attempt, TimingResult, perfect_time};

const KEY_COUNT: usize = 4;
// temps pour finir la séquence en GOOD avec la window de base
pub const BASE_GOOD_TIME: f32 = 4.0;
const BOX_SIZE: f32 = 70.0;
const BOX_GAP: f32 = 14.0;
const TIME_BAR_HEIGHT: f32 = 16.0;

#[derive(Clone, Copy, PartialEq)]
enum Arrow {
    Up,
    Down,
    Left,
    Right,
}

impl Arrow {
    fn random(rl: &RaylibHandle) -> Arrow {
        match rl.get_random_value::<i32>(0..=3) {
            0 => Arrow::Up,
            1 => Arrow::Down,
            2 => Arrow::Left,
            _ => Arrow::Right,
        }
    }

    /// flèche du clavier + WASD. raylib lit la position physique des touches :
    /// sur un clavier AZERTY, WASD correspond à ZQSD
    fn keys(self) -> [KeyboardKey; 2] {
        match self {
            Arrow::Up => [KEY_UP, KEY_W],
            Arrow::Down => [KEY_DOWN, KEY_S],
            Arrow::Left => [KEY_LEFT, KEY_A],
            Arrow::Right => [KEY_RIGHT, KEY_D],
        }
    }

    /// direction de la pointe, en coordonnées écran (y vers le bas)
    fn direction(self) -> Vector2 {
        match self {
            Arrow::Up => Vector2::new(0.0, -1.0),
            Arrow::Down => Vector2::new(0.0, 1.0),
            Arrow::Left => Vector2::new(-1.0, 0.0),
            Arrow::Right => Vector2::new(1.0, 0.0),
        }
    }

    const ALL: [Arrow; 4] = [Arrow::Up, Arrow::Down, Arrow::Left, Arrow::Right];
}

/// séquence de touches (Assassin / Wraith) : taper les flèches dans l'ordre, vite.
/// window = temps accordé pour GOOD, precision = part de ce temps qui donne PERFECT,
/// tempo = tout va plus vite
pub struct KeySequence {
    arrows: [Arrow; KEY_COUNT],
    // nombre de flèches déjà réussies
    done: usize,
    elapsed: f32,
    good_time: f32,
    // 0 = PERFECT impossible (ex : défenseur pris à revers)
    perfect_time: f32,
    // flash rouge sur la flèche après une mauvaise touche
    wrong_flash: f32,
}

impl KeySequence {
    pub fn new(rl: &RaylibHandle, side: &DuelSide) -> KeySequence {
        let good_time = BASE_GOOD_TIME * (side.window / BASE_WINDOW) / side.tempo;
        let perfect_time = perfect_time(side, good_time);
        KeySequence {
            arrows: std::array::from_fn(|_| Arrow::random(rl)),
            done: 0,
            elapsed: 0.0,
            good_time,
            perfect_time,
            wrong_flash: 0.0,
        }
    }

    pub fn area(&self) -> Rectangle {
        let width = KEY_COUNT as f32 * BOX_SIZE + (KEY_COUNT - 1) as f32 * BOX_GAP;
        Rectangle {
            x: SCREEN_WIDTH as f32 / 2.0 - width / 2.0,
            y: SCREEN_HEIGHT as f32 - 150.0,
            width,
            height: BOX_SIZE + 12.0 + TIME_BAR_HEIGHT,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, delta_time: f32) -> Option<Attempt> {
        self.elapsed += delta_time;
        self.wrong_flash = (self.wrong_flash - delta_time).max(0.0);

        // trop lent : raté, nouvelle séquence (jouée seulement s'il reste une vie)
        if self.elapsed > self.good_time {
            self.arrows = std::array::from_fn(|_| Arrow::random(rl));
            self.done = 0;
            self.elapsed = 0.0;
            return Some(Attempt::Miss);
        }

        let pressed = Arrow::ALL
            .into_iter()
            .find(|arrow| arrow.keys().iter().any(|&key| rl.is_key_pressed(key)))?;
        if pressed != self.arrows[self.done] {
            // mauvaise touche : raté, on reste sur la même flèche
            self.wrong_flash = 0.3;
            return Some(Attempt::Miss);
        }
        self.done += 1;
        if self.done < KEY_COUNT {
            return None;
        }
        Some(Attempt::Hit(if self.elapsed <= self.perfect_time {
            TimingResult::Perfect
        } else {
            TimingResult::Good
        }))
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let area = self.area();

        for (i, arrow) in self.arrows.iter().enumerate() {
            let rect = Rectangle::new(
                area.x + i as f32 * (BOX_SIZE + BOX_GAP),
                area.y,
                BOX_SIZE,
                BOX_SIZE,
            );
            let (background, border) = if i < self.done {
                (Color::new(40, 120, 40, 230), Color::GREEN)
            } else if i == self.done && self.wrong_flash > 0.0 {
                (Color::new(140, 30, 30, 230), Color::RED)
            } else if i == self.done {
                (Color::new(30, 30, 30, 230), Color::WHITE)
            } else {
                (Color::new(30, 30, 30, 160), Color::new(255, 255, 255, 90))
            };
            d.draw_rectangle_rec(rect, background);
            d.draw_rectangle_lines_ex(rect, 3.0, border);
            let center = Vector2::new(rect.x + BOX_SIZE / 2.0, rect.y + BOX_SIZE / 2.0);
            draw_arrow(d, center, arrow.direction(), BOX_SIZE * 0.32, Color::WHITE);
        }

        // barre de temps : vert = finir ici donne PERFECT, jaune = GOOD
        let bar = Rectangle::new(
            area.x,
            area.y + BOX_SIZE + 12.0,
            area.width,
            TIME_BAR_HEIGHT,
        );
        d.draw_rectangle_rec(bar, Color::YELLOW);
        let perfect_width = bar.width * self.perfect_time / self.good_time;
        d.draw_rectangle(
            bar.x as i32,
            bar.y as i32,
            perfect_width as i32,
            bar.height as i32,
            Color::GREEN,
        );
        let cursor_x = bar.x + bar.width * (self.elapsed / self.good_time).min(1.0);
        d.draw_rectangle(
            cursor_x as i32 - 2,
            bar.y as i32 - 4,
            4,
            bar.height as i32 + 8,
            Color::WHITE,
        );
        d.draw_rectangle_lines_ex(bar, 1.0, Color::BLACK);
    }
}

/// flèche : un trait + une pointe triangulaire dans la direction donnée
fn draw_arrow(d: &mut RaylibDrawHandle, center: Vector2, dir: Vector2, size: f32, color: Color) {
    let side = Vector2::new(-dir.y, dir.x);
    let tip = center + dir * size;
    let head_base = center + dir * (size * 0.1);
    let tail = center - dir * size;
    d.draw_line_ex(tail, head_base, size * 0.35, color);
    let left = head_base + side * (size * 0.7);
    let right = head_base - side * (size * 0.7);
    // raylib ne dessine un triangle que dans le sens antihoraire : on essaie les deux
    d.draw_triangle(tip, left, right, color);
    d.draw_triangle(tip, right, left, color);
}
