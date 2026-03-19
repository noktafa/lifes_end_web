use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Component)]
pub struct TailSegment {
    pub index: usize,
}

#[derive(Component, Default)]
pub struct TailChain {
    pub segments: Vec<Entity>,
}

#[derive(Component)]
pub struct PositionHistory {
    pub positions: VecDeque<Vec2>,
    pub max_length: usize,
}

impl Default for PositionHistory {
    fn default() -> Self {
        Self {
            positions: VecDeque::new(),
            max_length: 100,
        }
    }
}
