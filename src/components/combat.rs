use bevy::prelude::*;

#[derive(Component)]
pub struct Projectile {
    pub distance_traveled: f32,
    pub max_distance: f32,
    pub bounces_left: u8,
}

#[derive(Event)]
pub struct CellDestroyed;

#[derive(Event)]
pub struct NukeActivated;
