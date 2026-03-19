use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystemSet {
    Input,
    Physics,
    TailUpdate,
    GolTick,
    Combat,
    Collision,
    Cleanup,
}
