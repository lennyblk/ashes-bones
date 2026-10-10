#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GameMode {
    TitleScreen,
    CombatScreen,
    GridScreen,
    FactionSelectionScreen,
    PauseScreen,
    GuideScreen,
    MultiplayerScreen,
    HostScreen,
    Victory,
    Defeat,
}
#[derive(PartialEq)]
pub enum TurnPhase {
    PlayerTurn,
    EnemyTurn,
}
