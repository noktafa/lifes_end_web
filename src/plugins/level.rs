use bevy::prelude::*;

use crate::components::combat::Projectile;
use crate::components::gol::LifeCell;
use crate::components::player::Player;
use crate::components::tail::TailSegment;
use crate::resources::game_config::GameConfig;
use crate::resources::gol_grid::LifeGrid;
use crate::resources::level_config::CurrentLevel;
use crate::states::GameState;
use crate::systems::GameSystemSet;

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_win_condition,
                check_swarm_limit,
            )
                .in_set(GameSystemSet::Cleanup)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_game_entities)
        .add_systems(OnEnter(GameState::LevelComplete), advance_level)
        .add_systems(OnEnter(GameState::Menu), reset_level);
    }
}

fn check_win_condition(
    grid: Option<Res<LifeGrid>>,
    level: Option<Res<CurrentLevel>>,
    cells: Query<&LifeCell>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(grid) = grid else { return };
    let Some(level) = level else { return };
    if grid.alive_cells.is_empty()
        && cells.iter().count() == 0
        && level.waves_remaining.is_empty()
    {
        next_state.set(GameState::LevelComplete);
    }
}

fn check_swarm_limit(
    grid: Option<Res<LifeGrid>>,
    config: Res<GameConfig>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(grid) = grid else { return };
    if grid.alive_cells.len() >= config.swarm_limit {
        next_state.set(GameState::GameOver);
    }
}

fn cleanup_game_entities(
    mut commands: Commands,
    players: Query<Entity, With<Player>>,
    segments: Query<Entity, With<TailSegment>>,
    cells: Query<Entity, With<LifeCell>>,
    projectiles: Query<Entity, With<Projectile>>,
) {
    for entity in players
        .iter()
        .chain(segments.iter())
        .chain(cells.iter())
        .chain(projectiles.iter())
    {
        commands.entity(entity).despawn_recursive();
    }
}

fn advance_level(mut commands: Commands, level: Option<Res<CurrentLevel>>) {
    let next = level.map(|l| l.level_number + 1).unwrap_or(1);
    commands.insert_resource(CurrentLevel {
        level_number: next,
        waves_remaining: vec![],
    });
}

fn reset_level(mut commands: Commands) {
    commands.insert_resource(CurrentLevel {
        level_number: 1,
        waves_remaining: vec![],
    });
}
