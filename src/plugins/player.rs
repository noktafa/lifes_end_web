use bevy::prelude::*;

use crate::components::common::Velocity;
use crate::components::player::*;
use crate::components::tail::*;
use crate::resources::game_config::GameConfig;
use crate::states::GameState;
use crate::systems::GameSystemSet;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    player_input.in_set(GameSystemSet::Input),
                    apply_movement.in_set(GameSystemSet::Physics),
                    apply_velocity.in_set(GameSystemSet::Physics).after(apply_movement),
                    bounce_walls.in_set(GameSystemSet::Physics).after(apply_velocity),
                    sync_player_rotation.in_set(GameSystemSet::Physics),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player,
            Heading(0.0),
            Thrusting(false),
            Boosting(false),
            BoostFuel::default(),
            Velocity::default(),
            PlayerStats::default(),
            TailChain::default(),
            PositionHistory::default(),
            // Body
            Sprite {
                color: Color::srgb(0.15, 0.45, 0.9),
                custom_size: Some(Vec2::new(22.0, 14.0)),
                ..default()
            },
            Transform::from_translation(Vec3::ZERO),
        ))
        .with_children(|parent| {
            // Nose / cockpit — bright triangle-ish front indicator
            parent.spawn((
                Sprite {
                    color: Color::srgb(0.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(10.0, 6.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(12.0, 0.0, 0.1)),
            ));
            // Engine glow — rear indicator (dim red)
            parent.spawn((
                Sprite {
                    color: Color::srgb(0.6, 0.1, 0.1),
                    custom_size: Some(Vec2::new(5.0, 10.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(-13.0, 0.0, 0.1)),
            ));
        });
}

fn player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Heading, &mut Thrusting, &mut Boosting, &mut BoostFuel), With<Player>>,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    let Ok((mut heading, mut thrusting, mut boosting, mut fuel)) = query.get_single_mut() else {
        return;
    };

    // Rotation — instant and snappy
    let mut rotation_delta = 0.0;
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        rotation_delta += config.rotation_speed * time.delta_secs();
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        rotation_delta -= config.rotation_speed * time.delta_secs();
    }
    heading.0 += rotation_delta;

    // Thrust
    thrusting.0 = keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp);

    // Boost — Shift burns fuel for massive acceleration
    let wants_boost = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    if wants_boost && fuel.current > 0.0 && thrusting.0 {
        boosting.0 = true;
        fuel.current = (fuel.current - fuel.burn_rate * time.delta_secs()).max(0.0);
    } else {
        boosting.0 = false;
        // Regen fuel when not boosting
        fuel.current = (fuel.current + fuel.regen_rate * time.delta_secs()).min(fuel.max);
    }
}

fn apply_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<
        (&Heading, &Thrusting, &Boosting, &PlayerStats, &mut Velocity),
        With<Player>,
    >,
    config: Res<GameConfig>,
    time: Res<Time>,
) {
    let Ok((heading, thrusting, boosting, stats, mut velocity)) = query.get_single_mut() else {
        return;
    };

    let forward = Vec2::new(heading.0.cos(), heading.0.sin());
    let dt = time.delta_secs();

    // Thrust / boost
    if thrusting.0 {
        let force = if boosting.0 {
            config.boost_force
        } else {
            config.thrust_force
        };
        let acceleration = forward * (force / stats.mass);
        velocity.0 += acceleration * dt;
    }

    // Brake — S/Down actively slows you down hard
    let braking = keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown);
    if braking {
        let speed = velocity.0.length();
        if speed > 5.0 {
            let brake_amount = config.brake_force / stats.mass * dt;
            let new_speed = (speed - brake_amount).max(0.0);
            velocity.0 = velocity.0.normalize() * new_speed;
        } else {
            velocity.0 = Vec2::ZERO;
        }
    }

    // Drift friction — turning while moving: the sideways component bleeds off faster
    // This gives the "powerslide" feel from Rocket League
    let turning = keyboard.pressed(KeyCode::KeyA)
        || keyboard.pressed(KeyCode::KeyD)
        || keyboard.pressed(KeyCode::ArrowLeft)
        || keyboard.pressed(KeyCode::ArrowRight);

    if turning && velocity.0.length() > 10.0 {
        // Decompose velocity into forward and sideways components
        let speed = velocity.0.length();
        let vel_dir = velocity.0.normalize();
        let forward_component = vel_dir.dot(forward);
        let side = Vec2::new(-forward.y, forward.x);
        let side_component = vel_dir.dot(side);

        // Sideways bleeds off faster = drift feel
        let new_forward = forward_component * config.friction;
        let new_side = side_component * config.drift_friction;
        velocity.0 = (forward * new_forward + side * new_side) * speed;
    } else {
        // Normal friction
        velocity.0 *= config.friction;
    }

    // Velocity dead zone
    if velocity.0.length() < 1.0 {
        velocity.0 = Vec2::ZERO;
    }

    // Speed cap
    let max_speed = if boosting.0 {
        config.max_boost_velocity
    } else {
        config.max_velocity
    };
    let speed = velocity.0.length();
    if speed > max_speed {
        velocity.0 = velocity.0.normalize() * max_speed;
    }
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in &mut query {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

fn bounce_walls(
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>,
    config: Res<GameConfig>,
) {
    let Ok((mut velocity, mut transform)) = query.get_single_mut() else {
        return;
    };

    let hw = config.arena_half_width;
    let hh = config.arena_half_height;

    if transform.translation.x > hw {
        transform.translation.x = hw;
        velocity.x = -velocity.x.abs() * config.bounce_damping;
    } else if transform.translation.x < -hw {
        transform.translation.x = -hw;
        velocity.x = velocity.x.abs() * config.bounce_damping;
    }

    if transform.translation.y > hh {
        transform.translation.y = hh;
        velocity.y = -velocity.y.abs() * config.bounce_damping;
    } else if transform.translation.y < -hh {
        transform.translation.y = -hh;
        velocity.y = velocity.y.abs() * config.bounce_damping;
    }
}

fn sync_player_rotation(mut query: Query<(&Heading, &mut Transform), With<Player>>) {
    let Ok((heading, mut transform)) = query.get_single_mut() else {
        return;
    };
    transform.rotation = Quat::from_rotation_z(heading.0);
}
