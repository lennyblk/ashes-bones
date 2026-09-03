#[derive(Clone, Copy, PartialEq)]
pub enum EnemyState {
    Idle,
    Dying,
    Dead,
    Hurt,
}

pub struct Enemy {
    pub grid_x: i32,
    pub grid_y: i32,
    pub hp_points: i32,
    pub defense: i32,
    pub state: EnemyState,
    pub facing_left: bool,
}
