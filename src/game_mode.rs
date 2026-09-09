#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GameMode {
    CombatScreen,
    GridScreen,
}
#[derive(PartialEq)]
pub enum TurnPhase {
    PlayerTurn,
    EnemyTurn,
}
