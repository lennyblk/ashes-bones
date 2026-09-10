#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GameMode {
    TitleScreen,
    CombatScreen,
    GridScreen,
    Victory,
    Defeat,
}
#[derive(PartialEq)]
pub enum TurnPhase {
    PlayerTurn,
    EnemyTurn,
}
