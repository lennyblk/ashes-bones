#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GameMode {
    TitleScreen,
    CombatScreen,
    GridScreen,
    FactionSelectionScreen,
    Victory,
    Defeat,
}
#[derive(PartialEq)]
pub enum TurnPhase {
    PlayerTurn,
    EnemyTurn,
}
