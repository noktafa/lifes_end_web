use bevy::prelude::*;

#[derive(Resource)]
pub struct GameConfig {
    pub cell_size: f32,
    // Movement — Rocket League style: snappy, fast, responsive
    pub thrust_force: f32,
    pub boost_force: f32,
    pub brake_force: f32,
    pub rotation_speed: f32,
    pub max_velocity: f32,
    pub max_boost_velocity: f32,
    pub friction: f32,
    pub drift_friction: f32,
    // Combat
    pub projectile_speed: f32,
    // Tail
    pub player_base_mass: f32,
    pub mass_per_segment: f32,
    // Arena
    pub arena_half_width: f32,
    pub arena_half_height: f32,
    pub bounce_damping: f32,
    pub safe_spawn_radius: f32,
    // Swarm
    pub swarm_limit: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            cell_size: 16.0,
            thrust_force: 800.0,
            boost_force: 1800.0,
            brake_force: 600.0,
            rotation_speed: 3.8,
            max_velocity: 500.0,
            max_boost_velocity: 800.0,
            friction: 0.985,
            drift_friction: 0.95,
            projectile_speed: 700.0,
            player_base_mass: 1.0,
            mass_per_segment: 0.08,
            arena_half_width: 500.0,
            arena_half_height: 350.0,
            bounce_damping: 0.8,
            safe_spawn_radius: 5.0,
            swarm_limit: 100,
        }
    }
}
