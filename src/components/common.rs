use bevy::prelude::*;

#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);
