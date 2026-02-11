//! Player mode: on foot (character) vs in boat (ship). E to toggle.

use bevy::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::render::view::Hdr;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::character::MarineCharacter;
use crate::ship::Ship;

#[derive(Resource, Default)]
pub struct PlayerMode {
    pub in_boat: bool,
}

#[derive(Component)]
pub struct PlayerCamera;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerMode::default())
            .add_systems(Startup, cursor_lock)
            .add_systems(Update, (toggle_boat_enter, cursor_toggle));
    }
}


fn cursor_lock(mut query: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    for mut opts in query.iter_mut() {
        opts.visible = false;
        opts.grab_mode = CursorGrabMode::Locked;
    }
}

fn cursor_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    for mut opts in query.iter_mut() {
        opts.visible = true;
        opts.grab_mode = CursorGrabMode::None;
    }
}

fn toggle_boat_enter(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<PlayerMode>,
    mut commands: Commands,
    camera_query: Query<Entity, With<PlayerCamera>>,
    character_query: Query<Entity, With<MarineCharacter>>,
    ship_query: Query<Entity, With<Ship>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    mode.in_boat = !mode.in_boat;

    let Some(cam_id) = camera_query.iter().next() else { return };
    let Some(char_id) = character_query.iter().next() else { return };
    let Some(ship_id) = ship_query.iter().next() else { return };

    commands.entity(cam_id).despawn();

    let camera_components = (
        Camera3d::default(),
        Hdr,
        Tonemapping::AgX,
        PlayerCamera,
    );
    if mode.in_boat {
        let cam_id = commands.spawn((
            camera_components,
            Transform::from_xyz(0.0, 4.0, 12.0).looking_at(Vec3::new(0.0, 0.0, -5.0), Vec3::Y),
        )).id();
        commands.entity(ship_id).add_children(&[cam_id]);
    } else {
        let cam_id = commands.spawn((
            camera_components,
            Transform::from_xyz(0.0, 0.9, 0.0),
        )).id();
        commands.entity(char_id).add_children(&[cam_id]);
    }
}
