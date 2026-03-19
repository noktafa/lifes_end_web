use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Heading(pub f32);

#[derive(Component, Default)]
pub struct Thrusting(pub bool);

#[derive(Component, Default)]
pub struct Boosting(pub bool);

#[derive(Component)]
pub struct BoostFuel {
    pub current: f32,
    pub max: f32,
    pub regen_rate: f32,
    pub burn_rate: f32,
}

impl Default for BoostFuel {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
            regen_rate: 15.0,
            burn_rate: 40.0,
        }
    }
}

#[derive(Component)]
pub struct PlayerStats {
    pub mass: f32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self { mass: 1.0 }
    }
}
