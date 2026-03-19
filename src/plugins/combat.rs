use bevy::prelude::*;

use crate::components::combat::*;
use crate::components::common::Velocity;
use crate::components::gol::LifeCell;
use crate::components::player::*;
use crate::components::tail::*;
use crate::resources::game_config::GameConfig;
use crate::resources::gol_grid::LifeGrid;
use crate::resources::random::GameRng;
use crate::states::GameState;
use crate::systems::GameSystemSet;

const AIM_ASSIST_STRENGTH: f32 = 0.10;
const AIM_ASSIST_CONE: f32 = 0.5;
const AIM_ASSIST_RANGE: f32 = 400.0;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CellDestroyed>()
            .add_event::<NukeActivated>()
            .add_systems(
                Update,
                (
                    player_shoot.in_set(GameSystemSet::Combat),
                    nuke_input.in_set(GameSystemSet::Combat),
                    apply_nuke.in_set(GameSystemSet::Combat).after(nuke_input),
                    move_projectiles.in_set(GameSystemSet::Combat),
                    bounce_projectiles.in_set(GameSystemSet::Combat).after(move_projectiles),
                    expire_projectiles.in_set(GameSystemSet::Cleanup),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn player_shoot(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    player: Query<(&Transform, &Heading), With<Player>>,
    cells: Query<&Transform, With<LifeCell>>,
    config: Res<GameConfig>,
) {
    if !keyboard.just_pressed(KeyCode::Space) && !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok((transform, heading)) = player.get_single() else {
        return;
    };

    let aim_dir = Vec2::new(heading.0.cos(), heading.0.sin());
    let player_pos = transform.translation.truncate();

    let mut best_target: Option<Vec2> = None;
    let mut best_dist = f32::MAX;

    for cell_transform in &cells {
        let cell_pos = cell_transform.translation.truncate();
        let to_cell = cell_pos - player_pos;
        let dist = to_cell.length();

        if dist > AIM_ASSIST_RANGE || dist < 1.0 {
            continue;
        }

        let to_cell_norm = to_cell.normalize();
        let angle = aim_dir.dot(to_cell_norm).acos();

        if angle < AIM_ASSIST_CONE && dist < best_dist {
            best_dist = dist;
            best_target = Some(to_cell_norm);
        }
    }

    let final_dir = if let Some(target_dir) = best_target {
        let blended = aim_dir * (1.0 - AIM_ASSIST_STRENGTH) + target_dir * AIM_ASSIST_STRENGTH;
        blended.normalize()
    } else {
        aim_dir
    };

    let spawn_pos = transform.translation + (final_dir * 15.0).extend(0.0);

    // Max distance = arena diagonal
    let diagonal = (config.arena_half_width * 2.0).hypot(config.arena_half_height * 2.0);

    commands.spawn((
        Projectile {
            distance_traveled: 0.0,
            max_distance: diagonal,
            bounces_left: 1,
        },
        Velocity(final_dir * config.projectile_speed),
        Sprite {
            color: Color::srgb(1.0, 1.0, 0.4),
            custom_size: Some(Vec2::splat(6.0)),
            ..default()
        },
        Transform::from_translation(spawn_pos),
    ));
}

fn move_projectiles(
    mut query: Query<(&Velocity, &mut Transform, &mut Projectile)>,
    time: Res<Time>,
) {
    for (velocity, mut transform, mut projectile) in &mut query {
        let delta = Vec2::new(velocity.x, velocity.y) * time.delta_secs();
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
        projectile.distance_traveled += delta.length();
    }
}

fn bounce_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Velocity, &mut Transform, &mut Projectile)>,
    config: Res<GameConfig>,
) {
    let hw = config.arena_half_width;
    let hh = config.arena_half_height;

    for (entity, mut velocity, mut transform, mut projectile) in &mut query {
        let mut hit_wall = false;

        if transform.translation.x > hw {
            transform.translation.x = hw;
            velocity.x = -velocity.x.abs();
            hit_wall = true;
        } else if transform.translation.x < -hw {
            transform.translation.x = -hw;
            velocity.x = velocity.x.abs();
            hit_wall = true;
        }

        if transform.translation.y > hh {
            transform.translation.y = hh;
            velocity.y = -velocity.y.abs();
            hit_wall = true;
        } else if transform.translation.y < -hh {
            transform.translation.y = -hh;
            velocity.y = velocity.y.abs();
            hit_wall = true;
        }

        if hit_wall {
            if projectile.bounces_left == 0 {
                commands.entity(entity).despawn();
            } else {
                projectile.bounces_left -= 1;
            }
        }
    }
}

fn expire_projectiles(
    mut commands: Commands,
    query: Query<(Entity, &Projectile)>,
) {
    for (entity, projectile) in &query {
        if projectile.distance_traveled >= projectile.max_distance {
            commands.entity(entity).despawn();
        }
    }
}

const NUKE_COST: usize = 10;

fn nuke_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    player: Query<&TailChain, With<Player>>,
    mut nuke_events: EventWriter<NukeActivated>,
) {
    if !keyboard.just_pressed(KeyCode::KeyQ) {
        return;
    }
    let Ok(chain) = player.get_single() else {
        return;
    };
    if chain.segments.len() >= NUKE_COST {
        nuke_events.send(NukeActivated);
    }
}

fn apply_nuke(
    mut commands: Commands,
    mut nuke_events: EventReader<NukeActivated>,
    mut player: Query<(&mut TailChain, &mut PositionHistory), With<Player>>,
    segments: Query<Entity, With<TailSegment>>,
    mut grid: ResMut<LifeGrid>,
    mut rng: ResMut<GameRng>,
) {
    if nuke_events.read().next().is_none() {
        return;
    }

    // Sacrifice the entire tail
    let Ok((mut chain, mut history)) = player.get_single_mut() else {
        return;
    };
    for entity in segments.iter() {
        commands.entity(entity).despawn();
    }
    chain.segments.clear();
    history.positions.clear();
    history.max_length = 100;

    // Nuke all enemy groups
    grid.nuke(&mut rng.rng);
}
