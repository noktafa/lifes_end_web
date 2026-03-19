use bevy::prelude::*;

use crate::components::player::Player;
use crate::resources::game_config::GameConfig;
use crate::states::GameState;

pub struct CameraPlugin;

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct ArenaBorder;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Playing), spawn_arena_borders)
            .add_systems(OnExit(GameState::Playing), despawn_arena_borders)
            .add_systems(
                Update,
                camera_follow_player.run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((Camera2d, GameCamera));
}

fn spawn_arena_borders(mut commands: Commands, config: Res<GameConfig>) {
    let hw = config.arena_half_width;
    let hh = config.arena_half_height;
    let thick = 6.0;
    let glow_thick = 2.0;

    // Bright neon border + inner glow layer
    let border_color = Color::srgb(0.0, 0.8, 1.0);
    let glow_color = Color::srgba(0.0, 0.5, 1.0, 0.3);

    // Helper: spawn a wall (outer bright + inner glow)
    let walls: [(Vec3, Vec2, Vec3, Vec2); 4] = [
        // Top
        (
            Vec3::new(0.0, hh + thick / 2.0, 1.0),
            Vec2::new(hw * 2.0 + thick * 2.0, thick),
            Vec3::new(0.0, hh + thick + glow_thick / 2.0, 0.5),
            Vec2::new(hw * 2.0 + thick * 2.0 + glow_thick * 2.0, glow_thick),
        ),
        // Bottom
        (
            Vec3::new(0.0, -hh - thick / 2.0, 1.0),
            Vec2::new(hw * 2.0 + thick * 2.0, thick),
            Vec3::new(0.0, -hh - thick - glow_thick / 2.0, 0.5),
            Vec2::new(hw * 2.0 + thick * 2.0 + glow_thick * 2.0, glow_thick),
        ),
        // Left
        (
            Vec3::new(-hw - thick / 2.0, 0.0, 1.0),
            Vec2::new(thick, hh * 2.0 + thick * 2.0),
            Vec3::new(-hw - thick - glow_thick / 2.0, 0.0, 0.5),
            Vec2::new(glow_thick, hh * 2.0 + thick * 2.0 + glow_thick * 2.0),
        ),
        // Right
        (
            Vec3::new(hw + thick / 2.0, 0.0, 1.0),
            Vec2::new(thick, hh * 2.0 + thick * 2.0),
            Vec3::new(hw + thick + glow_thick / 2.0, 0.0, 0.5),
            Vec2::new(glow_thick, hh * 2.0 + thick * 2.0 + glow_thick * 2.0),
        ),
    ];

    for (pos, size, glow_pos, glow_size) in walls {
        // Main border
        commands.spawn((
            ArenaBorder,
            Sprite {
                color: border_color,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_translation(pos),
        ));
        // Outer glow
        commands.spawn((
            ArenaBorder,
            Sprite {
                color: glow_color,
                custom_size: Some(glow_size),
                ..default()
            },
            Transform::from_translation(glow_pos),
        ));
    }

    // Corner accents — bright dots at each corner
    let corner_size = Vec2::splat(thick + 2.0);
    let corners = [
        Vec3::new(hw + thick / 2.0, hh + thick / 2.0, 2.0),
        Vec3::new(-hw - thick / 2.0, hh + thick / 2.0, 2.0),
        Vec3::new(hw + thick / 2.0, -hh - thick / 2.0, 2.0),
        Vec3::new(-hw - thick / 2.0, -hh - thick / 2.0, 2.0),
    ];
    for pos in corners {
        commands.spawn((
            ArenaBorder,
            Sprite {
                color: Color::srgb(0.2, 1.0, 1.0),
                custom_size: Some(corner_size),
                ..default()
            },
            Transform::from_translation(pos),
        ));
    }
}

fn despawn_arena_borders(mut commands: Commands, borders: Query<Entity, With<ArenaBorder>>) {
    for entity in &borders {
        commands.entity(entity).despawn();
    }
}

fn camera_follow_player(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<GameCamera>, Without<Player>)>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.get_single_mut() else {
        return;
    };
    let target = player_transform.translation;
    // Snappy camera — lerp 0.12 for responsive feel
    camera_transform.translation = camera_transform.translation.lerp(target, 0.12);
    camera_transform.translation.z = 999.0;
}
