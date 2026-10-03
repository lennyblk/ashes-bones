use crate::duel::DuelSide;
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use raylib::prelude::*;

use super::{Attempt, TimingResult};

// rayon de l'anneau cible, et d'où part l'anneau qui se resserre
const TARGET_RADIUS: f32 = 50.0;
const START_RADIUS: f32 = TARGET_RADIUS + 160.0;
// vitesse de l'anneau en px/s avant tempo : 160 px en 1.6 s
const SHRINK_SPEED: f32 = 100.0;

/// charge (Cavalry / Ghoul) : un anneau se resserre sur la cible, on clique quand
/// il passe dessus. window = épaisseur de la zone jaune, precision = zone verte,
/// tempo = vitesse de l'anneau. Le temps passé dans la zone vaut window / tempo
/// secondes, comme la barre de timing : même difficulté de base
pub struct ChargeRing {
    radius: f32,
    speed: f32,
    // demi-épaisseurs des zones autour de TARGET_RADIUS
    good_band: f32,
    perfect_band: f32,
}

/// centré en bas de l'écran, le cercle de départ (+ son fond) tient en entier
fn center() -> Vector2 {
    Vector2::new(
        SCREEN_WIDTH as f32 / 2.0,
        SCREEN_HEIGHT as f32 - START_RADIUS - 14.0,
    )
}

impl ChargeRing {
    pub fn new(_rl: &RaylibHandle, side: &DuelSide) -> ChargeRing {
        ChargeRing {
            radius: START_RADIUS,
            speed: SHRINK_SPEED * side.tempo,
            good_band: side.window * SHRINK_SPEED / 2.0,
            perfect_band: side.precision * SHRINK_SPEED / 2.0,
        }
    }

    pub fn area(&self) -> Rectangle {
        let c = center();
        Rectangle {
            x: c.x - START_RADIUS,
            y: c.y - START_RADIUS,
            width: START_RADIUS * 2.0,
            height: START_RADIUS * 2.0,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, delta_time: f32) -> Option<Attempt> {
        self.radius -= self.speed * delta_time;

        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let gap = (self.radius - TARGET_RADIUS).abs();
            // trop tôt : raté, mais l'anneau continue sa course
            return Some(if gap <= self.perfect_band {
                Attempt::Hit(TimingResult::Perfect)
            } else if gap <= self.good_band {
                Attempt::Hit(TimingResult::Good)
            } else {
                Attempt::Miss
            });
        }

        // l'anneau a dépassé la cible sans clic : raté, nouvelle charge
        // (jouée seulement s'il reste une vie, sinon MiniGame finit sur BAD)
        if self.radius < TARGET_RADIUS - self.good_band {
            self.radius = START_RADIUS;
            return Some(Attempt::Miss);
        }
        None
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let c = center();
        d.draw_circle_v(c, START_RADIUS + 6.0, Color::new(20, 20, 20, 150));

        // zones autour de l'anneau cible : jaune = GOOD, vert = PERFECT
        d.draw_ring(
            c,
            TARGET_RADIUS - self.good_band,
            TARGET_RADIUS + self.good_band,
            0.0,
            360.0,
            64,
            Color::YELLOW,
        );
        if self.perfect_band > 0.0 {
            // au moins 1 px de large pour rester visible
            let half = self.perfect_band.max(0.5);
            d.draw_ring(
                c,
                TARGET_RADIUS - half,
                TARGET_RADIUS + half,
                0.0,
                360.0,
                64,
                Color::GREEN,
            );
        }
        d.draw_circle_v(c, 6.0, Color::WHITE);

        // l'anneau qui se resserre
        d.draw_ring(
            c,
            (self.radius - 2.0).max(0.0),
            self.radius + 2.0,
            0.0,
            360.0,
            64,
            Color::WHITE,
        );
    }
}
