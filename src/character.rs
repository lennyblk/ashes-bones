use crate::TILE_SIZE;

#[derive(PartialEq)]
pub enum CharacterState {
    Idle,
    Walking,
}

pub struct Character {
    pub grid_x: i32,
    pub grid_y: i32,
    pub screen_x: f32,
    pub screen_y: f32,
    pub move_points: i32,
    pub hp_points: i32,
    pub path: Vec<(i32, i32)>,
    pub state: CharacterState,
    pub facing_left: bool,
    pub attack_range: i32,
}

impl Character {
    pub fn update_position(&mut self, delta_time: f32) {
        let target_x = self.grid_x * TILE_SIZE - 40;
        let target_y = self.grid_y * TILE_SIZE - 40;

        let distance_x = target_x as f32 - self.screen_x;
        let distance_y = target_y as f32 - self.screen_y;

        // abs (recup la valeur absolu de la distance restante) pour recup toujours un valeur positif que j'aille a gauche ou a droite, comme ca je snap pas trop tot si c'est negatif
        if distance_x.abs() > 1.0 {
            self.screen_x += distance_x * delta_time * 2.6; // le chiffre est la vitesse de déplacement
        } else {
            self.screen_x = target_x as f32;
        }

        if distance_y.abs() > 1.0 {
            self.screen_y += distance_y * delta_time * 2.6;
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
                self.state = CharacterState::Idle;
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
}
