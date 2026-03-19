use bevy::prelude::*;

use crate::components::gol::LifeCell;
use crate::components::player::{BoostFuel, Boosting, Player};
use crate::components::tail::TailChain;
use crate::resources::game_config::GameConfig;
use crate::resources::level_config::CurrentLevel;
use crate::states::GameState;

pub struct UiPlugin;

#[derive(Component)]
struct MenuUi;

#[derive(Component)]
struct HudUi;

#[derive(Component)]
struct HudCellCount;

#[derive(Component)]
struct HudTailLength;

#[derive(Component)]
struct HudLevelNumber;

#[derive(Component)]
struct HudBoostBar;

#[derive(Component)]
struct HudBoostFill;

#[derive(Component)]
struct HudNukeIndicator;

#[derive(Component)]
struct GameOverUi;

#[derive(Component)]
struct LevelCompleteUi;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(OnExit(GameState::Menu), despawn_tagged::<MenuUi>)
            .add_systems(Update, menu_input.run_if(in_state(GameState::Menu)))
            .add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(OnExit(GameState::Playing), despawn_tagged::<HudUi>)
            .add_systems(Update, (update_hud, pause_input).run_if(in_state(GameState::Playing)))
            .add_systems(OnEnter(GameState::Paused), setup_pause_overlay)
            .add_systems(OnExit(GameState::Paused), despawn_tagged::<GameOverUi>)
            .add_systems(Update, unpause_input.run_if(in_state(GameState::Paused)))
            .add_systems(OnEnter(GameState::GameOver), setup_game_over)
            .add_systems(OnExit(GameState::GameOver), despawn_tagged::<GameOverUi>)
            .add_systems(Update, game_over_input.run_if(in_state(GameState::GameOver)))
            .add_systems(OnEnter(GameState::LevelComplete), setup_level_complete)
            .add_systems(OnExit(GameState::LevelComplete), despawn_tagged::<LevelCompleteUi>)
            .add_systems(Update, level_complete_input.run_if(in_state(GameState::LevelComplete)));
    }
}

fn despawn_tagged<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn setup_menu(mut commands: Commands) {
    commands
        .spawn((
            MenuUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("LIFE'S END"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
            parent.spawn((
                Text::new("The Monad never gives up."),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            parent.spawn((
                Text::new("WASD: Move  |  SHIFT: Boost  |  SPACE/Click: Shoot  |  S: Brake  |  Q: Nuke"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
            parent.spawn((
                Text::new("[SPACE] Start"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.8, 1.0)),
            ));
        });
}

fn menu_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::Playing);
    }
}

fn setup_hud(mut commands: Commands, level: Option<Res<CurrentLevel>>) {
    let level_num = level.map(|l| l.level_number).unwrap_or(1);

    // Top bar: stats
    commands
        .spawn((
            HudUi,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                HudTailLength,
                Text::new("Tail: 0"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.2, 0.6, 1.0)),
            ));
            parent.spawn((
                HudLevelNumber,
                Text::new(format!("Level {}", level_num)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.2)),
            ));
            parent.spawn((
                HudCellCount,
                Text::new("Cells: 0"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

    // Bottom bar: boost meter
    commands
        .spawn((
            HudUi,
            Node {
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(15.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("BOOST"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
            // Boost bar background
            parent
                .spawn((
                    HudBoostBar,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(10.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                ))
                .with_children(|bar| {
                    // Boost fill
                    bar.spawn((
                        HudBoostFill,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.0, 0.8, 1.0)),
                    ));
                });
            // Nuke indicator
            parent.spawn((
                HudNukeIndicator,
                Text::new("[Q] NUKE"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.4, 0.4, 0.4, 0.4)),
            ));
        });
}

fn update_hud(
    cells: Query<&LifeCell>,
    player: Query<(&TailChain, &BoostFuel, &Boosting), With<Player>>,
    mut player_sprite: Query<&mut Sprite, With<Player>>,
    mut cell_text: Query<(&mut Text, &mut TextColor), With<HudCellCount>>,
    mut tail_text: Query<&mut Text, (With<HudTailLength>, Without<HudCellCount>, Without<HudNukeIndicator>)>,
    mut boost_fill: Query<&mut Node, With<HudBoostFill>>,
    mut boost_bg: Query<&mut BackgroundColor, With<HudBoostFill>>,
    mut nuke_text: Query<&mut TextColor, (With<HudNukeIndicator>, Without<HudCellCount>)>,
    config: Res<GameConfig>,
) {
    let cell_count = cells.iter().count();
    let limit = config.swarm_limit;
    if let Ok((mut text, mut color)) = cell_text.get_single_mut() {
        **text = format!("Cells: {} / {}", cell_count, limit);
        let ratio = cell_count as f32 / limit as f32;
        if ratio > 0.75 {
            color.0 = Color::srgb(1.0, 0.2, 0.2); // Red — danger
        } else if ratio > 0.5 {
            color.0 = Color::srgb(1.0, 0.8, 0.0); // Yellow — warning
        } else {
            color.0 = Color::srgb(0.0, 1.0, 0.0); // Green — safe
        }
    }

    if let Ok((chain, fuel, boosting)) = player.get_single() {
        let tail_len = chain.segments.len();
        let nuke_ready = tail_len >= 10;

        if let Ok(mut text) = tail_text.get_single_mut() {
            **text = format!("Tail: {}", tail_len);
        }

        // Nuke indicator: bright when ready, dim when not
        if let Ok(mut color) = nuke_text.get_single_mut() {
            if nuke_ready {
                color.0 = Color::srgb(1.0, 0.3, 0.1); // Bright orange-red
            } else {
                color.0 = Color::srgba(0.4, 0.4, 0.4, 0.4); // Dim
            }
        }

        // Boost bar width
        let pct = (fuel.current / fuel.max * 100.0).clamp(0.0, 100.0);
        if let Ok(mut node) = boost_fill.get_single_mut() {
            node.width = Val::Percent(pct);
        }

        // Boost bar color: cyan when full, orange when low
        if let Ok(mut bg) = boost_bg.get_single_mut() {
            if boosting.0 {
                bg.0 = Color::srgb(1.0, 0.5, 0.0); // Orange while boosting
            } else if pct < 25.0 {
                bg.0 = Color::srgb(1.0, 0.2, 0.2); // Red when low
            } else {
                bg.0 = Color::srgb(0.0, 0.8, 1.0); // Cyan normal
            }
        }

        // Player body color: glow orange when boosting
        if let Ok(mut sprite) = player_sprite.get_single_mut() {
            if boosting.0 {
                sprite.color = Color::srgb(1.0, 0.5, 0.05);
            } else {
                sprite.color = Color::srgb(0.15, 0.45, 0.9);
            }
        }
    }
}

fn pause_input(keyboard: Res<ButtonInput<KeyCode>>, mut next_state: ResMut<NextState<GameState>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

fn setup_pause_overlay(mut commands: Commands) {
    commands
        .spawn((
            GameOverUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            parent.spawn((
                Text::new("[ESC] Resume"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn unpause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }
}

fn setup_game_over(mut commands: Commands) {
    commands
        .spawn((
            GameOverUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.3, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("THE MONAD PREVAILS"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
            ));
            parent.spawn((
                Text::new("[R] Retry  |  [ESC] Menu"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        next_state.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }
}

fn setup_level_complete(mut commands: Commands) {
    commands
        .spawn((
            LevelCompleteUi,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.1, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("THE MONAD RETREATS"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
            parent.spawn((
                Text::new("...but it will return.  [SPACE] Continue  |  [ESC] Menu"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn level_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Menu);
    }
}
