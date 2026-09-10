use crate::TILE_SIZE;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Faction {
    Human,
    Undead,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum UnitClass {
    Soldier,
    Wraith,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum UnitState {
    Idle,
    Walking,
    ChoosingPosition,
    CombatEntering,
    Attacking,
    Hurt,
    Dying,
    Dead,
    MiniGame,
}

#[derive(Clone)]
pub struct Unit {
    pub name: String,
    pub faction: Faction,
    pub class: UnitClass,
    pub grid_x: i32,
    pub grid_y: i32,
    pub screen_x: f32,
    pub screen_y: f32,
    pub move_points: i32,
    pub move_points_remaining: i32,
    pub has_attacked: bool,
    pub hp_points: i32,
    pub hp_max_points: i32,
    pub path: Vec<(i32, i32)>,
    pub state: UnitState,
    pub facing_left: bool,
    pub attack_range: i32,
    pub attack_power: i32,
    pub defense: i32,
    pub attack_target: bool,
}

impl Unit {
    pub fn update_position(&mut self, delta_time: f32) {
        let target_x = self.grid_x * TILE_SIZE - 40;
        let target_y = self.grid_y * TILE_SIZE - 40;

        let distance_x = target_x as f32 - self.screen_x;
        let distance_y = target_y as f32 - self.screen_y;

        // abs (recup la valeur absolu de la distance restante) pour recup toujours
        // un valeur positif que j'aille a gauche ou a droite, comme ca je snap pas
        // trop tot si c'est negatif
        if distance_x.abs() > 1.0 {
            self.screen_x += distance_x * delta_time * 3.0; // le chiffre est la vitesse de déplacement
        } else {
            self.screen_x = target_x as f32;
        }

        if distance_y.abs() > 1.0 {
            self.screen_y += distance_y * delta_time * 3.0;
        } else {
            self.screen_y = target_y as f32;
        }
    }

    pub fn advance_path(&mut self) {
        if self.path.is_empty() {
            return;
        }
        let target = self.path[0];
        let target_screen_x = (target.0 * TILE_SIZE - 40) as f32;
        let target_screen_y = (target.1 * TILE_SIZE - 40) as f32;

        if (self.screen_x - target_screen_x).abs() < 1.0
            && (self.screen_y - target_screen_y).abs() < 1.0
        {
            self.path.remove(0);
            if self.path.is_empty() {
                if self.attack_target {
                    self.state = UnitState::Attacking;
                } else {
                    self.state = UnitState::Idle;
                }
            }
            return;
        }
        if self.screen_x < target_screen_x {
            self.facing_left = false;
        } else if self.screen_x > target_screen_x {
            self.facing_left = true;
        };

        self.grid_x = target.0;
        self.grid_y = target.1;
    }

    pub fn is_alive(&self) -> bool {
        self.state != UnitState::Dead
    }

    // tour par tour ---------------------------------------------------------

    pub fn start_turn(&mut self) {
        self.move_points_remaining = self.move_points;
        self.has_attacked = false;
    }

    pub fn has_moved(&self) -> bool {
        self.move_points_remaining < self.move_points
    }

    pub fn can_wait(&self) -> bool {
        self.state == UnitState::Idle && self.has_moved() && self.move_points_remaining > 0
    }

    pub fn wait(&mut self) {
        self.move_points_remaining = 0;
        self.has_attacked = true;
    }

    pub fn can_attack_any(&self, enemies: &[Unit]) -> bool {
        enemies.iter().any(|enemy| {
            let distance = (enemy.grid_x - self.grid_x).abs() + (enemy.grid_y - self.grid_y).abs();
            enemy.is_alive() && distance <= self.attack_range
        })
    }

    pub fn has_finished_turn(&self, enemies: &[Unit]) -> bool {
        !self.is_alive()
            || (self.move_points_remaining == 0
                && (self.has_attacked || !self.can_attack_any(enemies)))
    }
}
