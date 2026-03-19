use bevy::prelude::*;

#[derive(Component)]
pub struct LifeCell;

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellPosition {
    pub x: i32,
    pub y: i32,
}
