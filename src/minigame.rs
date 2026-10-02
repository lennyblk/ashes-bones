use crate::animation::Animation;
use crate::duel::{DuelConditions, DuelSide};
use crate::unit::{Faction, Unit, UnitState};
use raylib::prelude::*;

#[derive(Clone, Copy)]
pub enum TimingResult {
    Bad,
    Good,
    Perfect,
}

pub struct TimingBar {
    pub cursor_position: f32,
    pub speed: f32,
    pub zone_start: f32,
    pub zone_end: f32,
    pub perfect_zone_start: f32,
    pub perfect_zone_end: f32,
    pub time_remaining: f32,
    pub result: Option<TimingResult>,
    // ratés pardonnés restants (soutien / garde sur la grille)
    pub lives: u8,
    // > 0 pendant qu'on affiche "SAVED" après un raté pardonné
    pub saved_timer: f32,
}

impl TimingBar {
    /// zone placée aléatoirement (jamais collée aux bords), taille selon les conditions du duel
    pub fn new(rl: &RaylibHandle, side: &DuelSide) -> TimingBar {
        let min_start = 10;
        let max_start = (100.0 * (0.9 - side.zone_width)) as i32;
        let zone_start =
            rl.get_random_value::<i32>(min_start..=max_start.max(min_start)) as f32 / 100.0;
        let zone_end = zone_start + side.zone_width;

        let zone_center = zone_start + side.zone_width / 2.0;
        let perfect_zone_start = zone_center - side.perfect_width / 2.0;
        let perfect_zone_end = zone_center + side.perfect_width / 2.0;

        TimingBar {
            cursor_position: 0.0,
            speed: side.speed,
            zone_start,
            zone_end,
            perfect_zone_start,
            perfect_zone_end,
            time_remaining: 4.0,
            result: None,
            lives: side.lives,
            saved_timer: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) {
        self.cursor_position += self.speed * delta_time;
        if self.cursor_position > 1.0 {
            self.cursor_position = 1.0;
            self.speed = -self.speed;
        } else if self.cursor_position < 0.0 {
            self.cursor_position = 0.0;
            self.speed = -self.speed;
        }

        self.saved_timer = (self.saved_timer - delta_time).max(0.0);

        if self.result.is_none() {
            self.time_remaining -= delta_time;
            if self.time_remaining <= 0.0 {
                self.result = Some(TimingResult::Bad);
            }
        }
    }

    /// un raté avec une vie en stock est pardonné : on perd la vie et la barre continue
    pub fn try_hit(&mut self) -> Option<TimingResult> {
        if self.cursor_position >= self.zone_start && self.cursor_position <= self.zone_end {
            if self.cursor_position >= self.perfect_zone_start
                && self.cursor_position <= self.perfect_zone_end
            {
                self.result = Some(TimingResult::Perfect);
            } else {
                self.result = Some(TimingResult::Good);
            }
        } else if self.lives > 0 {
            self.lives -= 1;
            self.saved_timer = 0.6;
        } else {
            self.result = Some(TimingResult::Bad);
        }
        self.result
    }
}

pub fn handle_minigame(
    attacker: &mut Unit,
    _defender: &mut Unit,
    player_faction: Faction,
    timing_bar: &mut Option<TimingBar>,
    conditions: &DuelConditions,
    damage_multiplier: &mut f32,
    rl: &RaylibHandle,
    delta_time: f32,
    attacker_attack_animation: &mut Animation,
) -> Option<TimingResult> {
    if attacker.state != UnitState::MiniGame {
        return None;
    }

    let player_is_attacker = attacker.faction == player_faction;

    if timing_bar.is_none() {
        *timing_bar = Some(TimingBar::new(rl, conditions.side(player_is_attacker)));
    }

    let bar = timing_bar.as_mut().unwrap();
    bar.update(delta_time);

    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        bar.try_hit();
    }

    if let Some(result) = bar.result {
        *damage_multiplier = result.to_multiplier(player_is_attacker);
        attacker.state = match attacker.pending_action {
            Some(crate::unit::PendingAction::Heal(_)) => UnitState::Healing,
            _ => UnitState::Attacking,
        };
        attacker_attack_animation.current = 0;
        attacker_attack_animation.finished = false;
        *timing_bar = None;
        return Some(result);
    }
    None
}

impl TimingResult {
    pub fn to_multiplier(&self, is_attacking: bool) -> f32 {
        let base = match self {
            TimingResult::Bad => 0.5,
            TimingResult::Good => 1.0,
            TimingResult::Perfect => 1.5,
        };

        if is_attacking { base } else { 2.0 - base }
    }
}
