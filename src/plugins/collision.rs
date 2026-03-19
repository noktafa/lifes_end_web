use bevy::prelude::*;

use crate::components::combat::*;
use crate::components::gol::*;
use crate::components::player::Player;
use crate::resources::game_config::GameConfig;
use crate::resources::gol_grid::LifeGrid;
use crate::states::GameState;
use crate::systems::GameSystemSet;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                check_projectile_cell_collisions,
                check_player_cell_collisions,
            )
                .in_set(GameSystemSet::Collision)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn check_projectile_cell_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform), With<Projectile>>,
    cells: Query<(Entity, &Transform, &CellPosition), With<LifeCell>>,
    mut grid: ResMut<LifeGrid>,
    mut cell_destroyed: EventWriter<CellDestroyed>,
    config: Res<GameConfig>,
) {
    let hit_radius = config.cell_size * 0.7;

    for (proj_entity, proj_transform) in &projectiles {
        let proj_pos = proj_transform.translation.truncate();
        for (cell_entity, cell_transform, cell_pos) in &cells {
            let cell_world_pos = cell_transform.translation.truncate();
            let dist = proj_pos.distance(cell_world_pos);
            if dist < hit_radius {
                commands.entity(proj_entity).despawn();
                commands.entity(cell_entity).despawn();
                grid.alive_cells.remove(&(cell_pos.x, cell_pos.y));
                cell_destroyed.send(CellDestroyed);
                break;
            }
        }
    }
}

fn check_player_cell_collisions(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    cells: Query<(Entity, &Transform, &CellPosition), With<LifeCell>>,
    mut grid: ResMut<LifeGrid>,
    mut cell_destroyed: EventWriter<CellDestroyed>,
    config: Res<GameConfig>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let hit_radius = config.cell_size * 0.6;

    for (cell_entity, cell_transform, cell_pos) in &cells {
        let cell_world = cell_transform.translation.truncate();
        if player_pos.distance(cell_world) < hit_radius {
            // Ram = destroy the cell, not the player
            commands.entity(cell_entity).despawn();
            grid.alive_cells.remove(&(cell_pos.x, cell_pos.y));
            cell_destroyed.send(CellDestroyed);
            return;
        }
    }
}
