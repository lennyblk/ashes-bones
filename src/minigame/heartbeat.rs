use crate::duel::DuelSide;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use raylib::prelude::*;

use super::{Attempt, TimingResult};

const BEAT_COUNT: usize = 3;
// battements « pour rien » au début, juste pour donner le rythme
const COUNT_IN: usize = 2;
// temps entre deux battements avant tempo
const BEAT_INTERVAL: f32 = 0.7;
// le PERFECT se juge sur le décalage moyen des 3 clics : precision x ce facteur
// (precision 3% -> 45 ms de décalage moyen max)
const PERFECT_SCALE: f32 = 1.5;
// vitesse de défilement des notes vers la ligne de frappe
const SCROLL_SPEED: f32 = 250.0;
const TRACK_WIDTH: f32 = 600.0;
const TRACK_HEIGHT: f32 = 44.0;
const HIT_LINE_OFFSET: f32 = 110.0;

#[derive(Clone, Copy, PartialEq)]
enum Beat {
    Waiting,
    // décalage du clic par rapport au battement, en secondes
    Hit(f32),
    // passé ou mal cliqué, pardonné par une vie
    Skipped,
}

/// pulsation de sang (Blood Knight) : cliquer en rythme sur 3 battements.
/// window = tolérance autour de chaque battement (bande jaune),
/// precision = décalage moyen max pour PERFECT (bande verte), tempo = vitesse du cœur
pub struct Heartbeat {
    time: f32,
    interval: f32,
    // demi-largeurs en secondes
    good_tolerance: f32,
    perfect_tolerance: f32,
    beats: [Beat; BEAT_COUNT],
}

fn track() -> Rectangle {
    Rectangle {
        x: SCREEN_WIDTH as f32 / 2.0 - TRACK_WIDTH / 2.0,
        y: SCREEN_HEIGHT as f32 - 100.0,
        width: TRACK_WIDTH,
        height: TRACK_HEIGHT,
    }
}

fn hit_line_x() -> f32 {
    track().x + HIT_LINE_OFFSET
}

impl Heartbeat {
    pub fn new(_rl: &RaylibHandle, side: &DuelSide) -> Heartbeat {
        Heartbeat {
            time: 0.0,
            interval: BEAT_INTERVAL / side.tempo,
            // même logique que la barre : la bande jaune dure window / tempo secondes
            good_tolerance: side.window / side.tempo / 2.0,
            perfect_tolerance: side.precision * PERFECT_SCALE / side.tempo,
            beats: [Beat::Waiting; BEAT_COUNT],
        }
    }

    /// instant du battement i (après les battements de rythme)
    fn beat_time(&self, i: usize) -> f32 {
        (COUNT_IN + 1 + i) as f32 * self.interval
    }

    pub fn area(&self) -> Rectangle {
        let track = track();
        Rectangle {
            x: track.x,
            y: track.y - 90.0,
            width: track.width,
            height: track.height + 90.0,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, delta_time: f32) -> Option<Attempt> {
        self.time += delta_time;

        // battement passé sans clic : raté (sauté s'il reste une vie)
        for i in 0..BEAT_COUNT {
            if self.beats[i] == Beat::Waiting && self.time > self.beat_time(i) + self.good_tolerance
            {
                self.beats[i] = Beat::Skipped;
                return Some(Attempt::Miss);
            }
        }

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            // le clic vise le prochain battement pas encore joué
            let next = self.beats.iter().position(|b| *b == Beat::Waiting)?;
            let offset = self.time - self.beat_time(next);
            // clic pendant l'intro ou loin avant la note : ignoré, pas un contretemps
            if offset < -self.interval / 2.0 {
                return None;
            }
            if offset.abs() > self.good_tolerance {
                // clic à contretemps : raté, ce battement est perdu
                self.beats[next] = Beat::Skipped;
                return Some(Attempt::Miss);
            }
            self.beats[next] = Beat::Hit(offset);
        }

        if self.beats.contains(&Beat::Waiting) {
            return None;
        }
        // tous les battements joués : PERFECT si aucun sauté et décalage moyen serré
        let offsets: Vec<f32> = self
            .beats
            .iter()
            .filter_map(|b| match b {
                Beat::Hit(offset) => Some(offset.abs()),
                _ => None,
            })
            .collect();
        let all_hit = offsets.len() == BEAT_COUNT;
        let mean = offsets.iter().sum::<f32>() / offsets.len().max(1) as f32;
        Some(Attempt::Hit(
            if all_hit && self.perfect_tolerance > 0.0 && mean <= self.perfect_tolerance {
                TimingResult::Perfect
            } else {
                TimingResult::Good
            },
        ))
    }

    /// force du battement de cœur (1 juste sur un battement, retombe vite)
    fn pulse(&self) -> f32 {
        let since_beat = self.time % self.interval;
        (-since_beat * 8.0).exp()
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let track = track();
        let area = self.area();
        d.draw_rectangle_rec(
            Rectangle::new(
                area.x - 12.0,
                area.y - 12.0,
                area.width + 24.0,
                area.height + 24.0,
            ),
            Color::new(20, 20, 20, 170),
        );

        // cœur au-dessus de la ligne de frappe, il bat au rythme
        let total = (COUNT_IN + 1 + BEAT_COUNT) as f32 * self.interval;
        let beating = self.time >= self.interval * 0.5 && self.time <= total + self.interval;
        let scale = 1.0 + if beating { 0.35 * self.pulse() } else { 0.0 };
        draw_heart(
            d,
            Vector2::new(hit_line_x(), area.y + 42.0),
            24.0 * scale,
            Color::new(200, 20, 40, 255),
        );

        // piste + bandes de tolérance autour de la ligne de frappe
        d.draw_rectangle_rec(track, Color::new(50, 15, 20, 230));
        let line_x = hit_line_x();
        let good_half = self.good_tolerance * SCROLL_SPEED;
        d.draw_rectangle(
            (line_x - good_half) as i32,
            track.y as i32,
            (good_half * 2.0) as i32,
            track.height as i32,
            Color::YELLOW,
        );
        let perfect_half = (self.perfect_tolerance * SCROLL_SPEED).max(1.0);
        if self.perfect_tolerance > 0.0 {
            d.draw_rectangle(
                (line_x - perfect_half) as i32,
                track.y as i32,
                (perfect_half * 2.0) as i32,
                track.height as i32,
                Color::GREEN,
            );
        }
        d.draw_rectangle(
            line_x as i32 - 2,
            track.y as i32 - 6,
            4,
            track.height as i32 + 12,
            Color::WHITE,
        );

        // notes : gouttes de sang qui arrivent de la droite
        let center_y = track.y + track.height / 2.0;
        for (i, beat) in self.beats.iter().enumerate() {
            let x = line_x + (self.beat_time(i) - self.time) * SCROLL_SPEED;
            if x < track.x - 20.0 || x > track.x + track.width + 20.0 {
                continue;
            }
            let position = Vector2::new(x, center_y);
            match beat {
                Beat::Waiting => {
                    d.draw_circle_v(position, 13.0, Color::new(220, 30, 50, 255));
                    d.draw_circle_lines(x as i32, center_y as i32, 13.0, Color::WHITE);
                }
                Beat::Hit(_) => {}
                Beat::Skipped => {
                    let s = 9.0;
                    d.draw_line_ex(
                        position - Vector2::new(s, s),
                        position + Vector2::new(s, s),
                        3.0,
                        Color::RED,
                    );
                    d.draw_line_ex(
                        position - Vector2::new(s, -s),
                        position + Vector2::new(s, -s),
                        3.0,
                        Color::RED,
                    );
                }
            }
        }

        // un point par battement joué : blanc = touché, rouge = raté
        for (i, beat) in self.beats.iter().enumerate() {
            let position = Vector2::new(
                track.x + track.width - 60.0 + i as f32 * 20.0,
                area.y + 12.0,
            );
            match beat {
                Beat::Waiting => {
                    d.draw_circle_lines(position.x as i32, position.y as i32, 6.0, Color::WHITE)
                }
                Beat::Hit(_) => d.draw_circle_v(position, 6.0, Color::WHITE),
                Beat::Skipped => d.draw_circle_v(position, 6.0, Color::RED),
            }
        }
    }
}

/// cœur : deux cercles + un triangle pointe en bas
fn draw_heart(d: &mut RaylibDrawHandle, center: Vector2, size: f32, color: Color) {
    let r = size * 0.55;
    let left = Vector2::new(center.x - r * 0.9, center.y - r * 0.3);
    let right = Vector2::new(center.x + r * 0.9, center.y - r * 0.3);
    d.draw_circle_v(left, r, color);
    d.draw_circle_v(right, r, color);
    let a = Vector2::new(center.x - r * 1.85, center.y - r * 0.05);
    let b = Vector2::new(center.x + r * 1.85, center.y - r * 0.05);
    let tip = Vector2::new(center.x, center.y + size * 1.2);
    // raylib ne dessine un triangle que dans le sens antihoraire : on essaie les deux
    d.draw_triangle(a, tip, b, color);
    d.draw_triangle(a, b, tip, color);
}
