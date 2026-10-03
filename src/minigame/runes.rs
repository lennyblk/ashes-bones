use crate::duel::{BASE_WINDOW, DuelSide};
use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use raylib::prelude::*;

use super::{Attempt, TimingResult, perfect_time};

const RUNE_COUNT: usize = 4;
const SEQUENCE_LEN: usize = 4;
// démo : chaque rune reste allumée LIT_TIME puis s'éteint GAP_TIME (avant tempo)
const LIT_TIME: f32 = 0.45;
const GAP_TIME: f32 = 0.15;
// temps pour refaire la suite en GOOD avec la window de base (la souris est plus
// lente que le clavier, d'où plus que la séquence de l'assassin)
pub const BASE_GOOD_TIME: f32 = 6.0;
const RUNE_RADIUS: f32 = 38.0;
const RUNE_GAP: f32 = 30.0;
const TIME_BAR_HEIGHT: f32 = 16.0;

// une forme + une couleur par rune, pour les reconnaître même en daltonien
const RUNE_SIDES: [i32; RUNE_COUNT] = [3, 4, 5, 6];
const RUNE_COLORS: [Color; RUNE_COUNT] = [
    Color::new(170, 100, 255, 255), // violet
    Color::new(60, 200, 255, 255),  // cyan
    Color::new(255, 150, 40, 255),  // orange
    Color::new(255, 90, 170, 255),  // rose
];

enum Phase {
    // la suite s'allume rune par rune
    Show { step: usize, timer: f32 },
    // le joueur la refait à la souris
    Recall,
}

/// mémoire de runes (Mage / Banshee) : regarder la suite, puis la recliquer dans l'ordre.
/// window = temps pour la refaire, precision = part de ce temps qui donne PERFECT,
/// tempo = vitesse de la démo et du temps accordé
pub struct RuneMemory {
    sequence: [usize; SEQUENCE_LEN],
    phase: Phase,
    // runes déjà recliquées dans le bon ordre
    done: usize,
    elapsed: f32,
    good_time: f32,
    // 0 = PERFECT impossible
    perfect_time: f32,
    tempo: f32,
    // retour visuel : rune cliquée (index, temps restant) et mauvais clic
    click_flash: Option<(usize, f32)>,
    wrong_flash: f32,
    aim: Vector2,
}

fn random_sequence(rl: &RaylibHandle) -> [usize; SEQUENCE_LEN] {
    std::array::from_fn(|_| rl.get_random_value::<i32>(0..=RUNE_COUNT as i32 - 1) as usize)
}

fn rune_center(i: usize) -> Vector2 {
    let width = RUNE_COUNT as f32 * RUNE_RADIUS * 2.0 + (RUNE_COUNT - 1) as f32 * RUNE_GAP;
    Vector2::new(
        SCREEN_WIDTH as f32 / 2.0 - width / 2.0
            + RUNE_RADIUS
            + i as f32 * (RUNE_RADIUS * 2.0 + RUNE_GAP),
        SCREEN_HEIGHT as f32 - 140.0,
    )
}

impl RuneMemory {
    pub fn new(rl: &RaylibHandle, side: &DuelSide) -> RuneMemory {
        let good_time = BASE_GOOD_TIME * (side.window / BASE_WINDOW) / side.tempo;
        let perfect_time = perfect_time(side, good_time);
        RuneMemory {
            sequence: random_sequence(rl),
            phase: Phase::Show {
                step: 0,
                timer: 0.0,
            },
            done: 0,
            elapsed: 0.0,
            good_time,
            perfect_time,
            tempo: side.tempo,
            click_flash: None,
            wrong_flash: 0.0,
            aim: rl.get_mouse_position(),
        }
    }

    /// pendant la démo la barre de temps ne tourne pas : regarder ne coûte pas de temps
    pub fn is_showing(&self) -> bool {
        matches!(self.phase, Phase::Show { .. })
    }

    pub fn area(&self) -> Rectangle {
        let first = rune_center(0);
        let last = rune_center(RUNE_COUNT - 1);
        Rectangle {
            x: first.x - RUNE_RADIUS,
            y: first.y - RUNE_RADIUS,
            width: last.x - first.x + RUNE_RADIUS * 2.0,
            height: RUNE_RADIUS * 2.0 + 14.0 + TIME_BAR_HEIGHT,
        }
    }

    fn time_bar(&self) -> Rectangle {
        let area = self.area();
        Rectangle::new(
            area.x,
            area.y + RUNE_RADIUS * 2.0 + 14.0,
            area.width,
            TIME_BAR_HEIGHT,
        )
    }

    /// rune allumée pendant la démo (None entre deux runes)
    fn lit_rune(&self) -> Option<usize> {
        match self.phase {
            Phase::Show { step, timer } if timer < LIT_TIME / self.tempo => {
                Some(self.sequence[step])
            }
            _ => None,
        }
    }

    pub fn update(&mut self, rl: &RaylibHandle, delta_time: f32) -> Option<Attempt> {
        self.aim = rl.get_mouse_position();
        self.wrong_flash = (self.wrong_flash - delta_time).max(0.0);
        if let Some((rune, time_left)) = self.click_flash {
            self.click_flash = (time_left > delta_time).then(|| (rune, time_left - delta_time));
        }

        match &mut self.phase {
            Phase::Show { step, timer } => {
                *timer += delta_time;
                if *timer >= (LIT_TIME + GAP_TIME) / self.tempo {
                    *step += 1;
                    *timer = 0.0;
                    if *step >= SEQUENCE_LEN {
                        self.phase = Phase::Recall;
                        self.elapsed = 0.0;
                    }
                }
                None
            }
            Phase::Recall => {
                self.elapsed += delta_time;
                // trop lent : raté, nouvelle suite montrée (s'il reste une vie)
                if self.elapsed > self.good_time {
                    self.sequence = random_sequence(rl);
                    self.phase = Phase::Show {
                        step: 0,
                        timer: 0.0,
                    };
                    self.done = 0;
                    return Some(Attempt::Miss);
                }
                if !rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                    return None;
                }
                // clic à côté des runes : ignoré
                let clicked =
                    (0..RUNE_COUNT).find(|&i| self.aim.distance(rune_center(i)) <= RUNE_RADIUS)?;
                if clicked != self.sequence[self.done] {
                    // mauvaise rune : raté, on reste au même endroit de la suite
                    self.wrong_flash = 0.3;
                    return Some(Attempt::Miss);
                }
                self.click_flash = Some((clicked, 0.15));
                self.done += 1;
                if self.done < SEQUENCE_LEN {
                    return None;
                }
                Some(Attempt::Hit(if self.elapsed <= self.perfect_time {
                    TimingResult::Perfect
                } else {
                    TimingResult::Good
                }))
            }
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        let area = self.area();
        d.draw_rectangle_rec(
            Rectangle::new(
                area.x - 16.0,
                area.y - 16.0,
                area.width + 32.0,
                area.height + 32.0,
            ),
            Color::new(20, 20, 20, 170),
        );

        let lit = self.lit_rune();
        for i in 0..RUNE_COUNT {
            let center = rune_center(i);
            let flashed = self.click_flash.is_some_and(|(rune, _)| rune == i);
            let color = if lit == Some(i) || flashed {
                RUNE_COLORS[i]
            } else {
                // rune éteinte : même couleur, assombrie
                Color::new(
                    RUNE_COLORS[i].r / 3,
                    RUNE_COLORS[i].g / 3,
                    RUNE_COLORS[i].b / 3,
                    255,
                )
            };
            let radius = if lit == Some(i) {
                RUNE_RADIUS * 1.12
            } else {
                RUNE_RADIUS
            };
            d.draw_poly(center, RUNE_SIDES[i], radius, -90.0, color);
            let outline = if self.wrong_flash > 0.0 && !self.is_showing() {
                Color::RED
            } else if flashed {
                Color::WHITE
            } else {
                Color::new(255, 255, 255, 120)
            };
            d.draw_poly_lines_ex(center, RUNE_SIDES[i], radius, -90.0, 3.0, outline);
        }

        // progression de la suite : un point par rune (plein = déjà refait)
        let dots_y = area.y - 10.0;
        for i in 0..SEQUENCE_LEN {
            let x =
                area.x + area.width / 2.0 + (i as f32 - (SEQUENCE_LEN as f32 - 1.0) / 2.0) * 16.0;
            let position = Vector2::new(x, dots_y);
            if i < self.done {
                d.draw_circle_v(position, 5.0, Color::WHITE);
            } else {
                d.draw_circle_lines(x as i32, dots_y as i32, 5.0, Color::WHITE);
            }
        }

        // barre de temps (seulement pendant qu'on refait la suite)
        if !self.is_showing() {
            let bar = self.time_bar();
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

            // pointeur (le curseur système est caché pendant le jeu)
            d.draw_circle_v(self.aim, 7.0, Color::WHITE);
            d.draw_circle_lines(self.aim.x as i32, self.aim.y as i32, 7.0, Color::BLACK);
        }
    }
}
