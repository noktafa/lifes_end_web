use bevy::prelude::*;

use crate::components::combat::CellDestroyed;
use crate::components::gol::*;
use crate::components::player::*;
use crate::components::tail::*;
use crate::resources::game_config::GameConfig;
use crate::resources::gol_grid::LifeGrid;
use crate::states::GameState;
use crate::systems::GameSystemSet;

pub struct TailPlugin;

const SAMPLES_PER_SEGMENT: usize = 8;

impl Plugin for TailPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                record_position_history.in_set(GameSystemSet::TailUpdate),
                update_tail_from_history
                    .in_set(GameSystemSet::TailUpdate)
                    .after(record_position_history),
                grow_tail.in_set(GameSystemSet::Cleanup),
                check_tail_cell_collisions.in_set(GameSystemSet::Collision),
                update_player_mass.in_set(GameSystemSet::Cleanup),
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn record_position_history(
    mut query: Query<(&Transform, &mut PositionHistory), With<Player>>,
) {
    let Ok((transform, mut history)) = query.get_single_mut() else {
        return;
    };
    let pos = transform.translation.truncate();
    history.positions.push_front(pos);
    if history.positions.len() > history.max_length {
        history.positions.pop_back();
    }
}

fn update_tail_from_history(
    player_query: Query<&PositionHistory, With<Player>>,
    mut segment_query: Query<(&TailSegment, &mut Transform)>,
) {
    let Ok(history) = player_query.get_single() else {
        return;
    };

    for (segment, mut transform) in &mut segment_query {
        let history_index = (segment.index + 1) * SAMPLES_PER_SEGMENT;
        if let Some(&pos) = history.positions.get(history_index) {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

fn grow_tail(
    mut events: EventReader<CellDestroyed>,
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut TailChain, &mut PositionHistory), With<Player>>,
) {
    let Ok((player_transform, mut chain, mut history)) = player_query.get_single_mut() else {
        return;
    };

    let count = events.read().count();
    for _ in 0..count {
        let index = chain.segments.len();

        // Color gradient: bright blue near head, fading toward end
        let t = (index as f32 / 30.0).min(1.0);
        let r = 0.1 + t * 0.1;
        let g = 0.4 + t * 0.2;
        let b = 1.0 - t * 0.3;

        let segment = commands
            .spawn((
                TailSegment { index },
                Sprite {
                    color: Color::srgb(r, g, b),
                    custom_size: Some(Vec2::splat(10.0)),
                    ..default()
                },
                Transform::from_translation(player_transform.translation),
            ))
            .id();

        chain.segments.push(segment);
        history.max_length = (chain.segments.len() + 1) * SAMPLES_PER_SEGMENT + 20;
    }
}

fn check_tail_cell_collisions(
    segments: Query<&Transform, With<TailSegment>>,
    cells: Query<(Entity, &Transform, &CellPosition), With<LifeCell>>,
    mut commands: Commands,
    mut grid: ResMut<LifeGrid>,
    mut cell_destroyed: EventWriter<CellDestroyed>,
    config: Res<GameConfig>,
) {
    let hit_radius = config.cell_size * 0.6;

    for seg_transform in &segments {
        let seg_pos = seg_transform.translation.truncate();
        for (cell_entity, cell_transform, cell_pos) in &cells {
            let dist = seg_pos.distance(cell_transform.translation.truncate());
            if dist < hit_radius {
                // Tail whip — destroy the cell
                commands.entity(cell_entity).despawn();
                grid.alive_cells.remove(&(cell_pos.x, cell_pos.y));
                cell_destroyed.send(CellDestroyed);
            }
        }
    }
}

fn update_player_mass(
    mut query: Query<(&TailChain, &mut PlayerStats), With<Player>>,
    config: Res<GameConfig>,
) {
    let Ok((chain, mut stats)) = query.get_single_mut() else {
        return;
    };
    stats.mass = config.player_base_mass + (chain.segments.len() as f32 * config.mass_per_segment);
}
