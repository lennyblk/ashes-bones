#[derive(Clone, Copy, PartialEq)]
pub enum UndeadState {
    Idle,
    Dying,
    Dead,
    Hurt,
    CombatEntering,
}

pub struct Undead {
    pub name: String,
    pub grid_x: i32,
    pub grid_y: i32,
    pub hp_points: i32,
    pub defense: i32,
    pub state: UndeadState,
    pub facing_left: bool,
    pub max_hp_points: i32,
}
