//! Game states: Menu, Playing, Paused.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
}

#[derive(Component)]
struct MenuOverlay;

#[derive(Component)]
struct PauseOverlay;

pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::Menu), (spawn_menu_overlay, release_cursor))
            .add_systems(OnExit(GameState::Menu), despawn_menu_overlay)
            .add_systems(OnEnter(GameState::Playing), lock_cursor)
            .add_systems(OnEnter(GameState::Paused), (spawn_pause_overlay, release_cursor))
            .add_systems(OnExit(GameState::Paused), despawn_pause_overlay)
            .add_systems(
                Update,
                (
                    menu_input.run_if(in_state(GameState::Menu)),
                    pause_input.run_if(in_state(GameState::Paused)),
                    handle_escape_pause
                        .run_if(|s: Res<State<GameState>>| {
                            matches!(s.get(), GameState::Playing | GameState::Paused)
                        }),
                ),
            );
    }
}

fn spawn_menu_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let title_id = commands
        .spawn((
            Text::new("PROJECT ABYSS"),
            TextFont {
                font: font.clone(),
                font_size: 48.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 1.0)),
            TextLayout::default(),
        ))
        .id();
    let prompt_id = commands
        .spawn((
            Text::new("Press Enter or E to Start"),
            TextFont { font, ..default() },
            TextColor(Color::srgba(0.9, 0.9, 0.95, 0.95)),
            TextLayout::default(),
        ))
        .id();
    commands
        .spawn((
            Node {
                position_type: bevy::ui::PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: bevy::ui::FlexDirection::Column,
                justify_content: bevy::ui::JustifyContent::Center,
                align_items: bevy::ui::AlignItems::Center,
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.05, 0.12, 0.92)),
            MenuOverlay,
        ))
        .add_child(title_id)
        .add_child(prompt_id);
}

fn despawn_menu_overlay(
    mut commands: Commands,
    query: Query<(Entity, &Children), With<MenuOverlay>>,
) {
    for (entity, children) in query.iter() {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
        commands.entity(entity).despawn();
    }
}

fn spawn_pause_overlay(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_id = commands
        .spawn((
            Text::new("PAUSED\nPress Escape to Resume"),
            TextFont {
                font,
                font_size: 36.0,
                ..default()
            },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.95)),
            TextLayout::default(),
        ))
        .id();
    commands
        .spawn((
            Node {
                position_type: bevy::ui::PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                flex_direction: bevy::ui::FlexDirection::Column,
                justify_content: bevy::ui::JustifyContent::Center,
                align_items: bevy::ui::AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.05, 0.12, 0.7)),
            PauseOverlay,
        ))
        .add_child(text_id);
}

fn despawn_pause_overlay(
    mut commands: Commands,
    query: Query<(Entity, &Children), With<PauseOverlay>>,
) {
    for (entity, children) in query.iter() {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
        commands.entity(entity).despawn();
    }
}

fn menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::KeyE)
    {
        next_state.set(GameState::Playing);
    }
}

fn pause_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
    }
}

fn lock_cursor(mut query: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    for mut opts in query.iter_mut() {
        opts.visible = false;
        opts.grab_mode = CursorGrabMode::Locked;
    }
}

fn release_cursor(mut query: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    for mut opts in query.iter_mut() {
        opts.visible = true;
        opts.grab_mode = CursorGrabMode::None;
    }
}

/// Escape toggles pause when Playing. Call from cursor_toggle or a dedicated system.
pub fn handle_escape_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    match state.get() {
        GameState::Playing => {
            next_state.set(GameState::Paused);
            for mut opts in query.iter_mut() {
                opts.visible = true;
                opts.grab_mode = CursorGrabMode::None;
            }
        }
        GameState::Paused => {
            next_state.set(GameState::Playing);
            for mut opts in query.iter_mut() {
                opts.visible = false;
                opts.grab_mode = CursorGrabMode::Locked;
            }
        }
        GameState::Menu => {}
    }
}
