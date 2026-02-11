//! Player mode: on foot (character) vs in boat (ship). E to toggle.

use bevy::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::post_process::bloom::Bloom;
use bevy::render::view::{ColorGrading, Hdr};
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::character::MarineCharacter;
use crate::diving_bell::Submersible;
use crate::ocean::SEA_LEVEL;
use crate::ship::Ship;

#[derive(Resource, Default)]
pub struct PlayerMode {
    pub in_boat: bool,
    pub in_submersible: bool,
}

impl PlayerMode {
    pub fn in_vehicle(&self) -> bool {
        self.in_boat || self.in_submersible
    }
}

#[derive(Component)]
pub struct PlayerCamera;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerMode::default())
            .add_systems(Startup, cursor_lock)
            .add_systems(Update, (
                toggle_boat_enter,
                cursor_toggle,
                update_depth_color_grading,
                update_depth_fog,
            ));
    }
}


fn cursor_lock(mut query: Query<&mut CursorOptions, With<PrimaryWindow>>) {
    for mut opts in query.iter_mut() {
        opts.visible = false;
        opts.grab_mode = CursorGrabMode::Locked;
    }
}

/// Depth (m) over which color grading transitions from surface to full deep-blue.
const DEPTH_COLOR_TRANSITION: f32 = 25.0;

fn update_depth_color_grading(
    mut camera_query: Query<(&mut ColorGrading, &GlobalTransform), With<PlayerCamera>>,
) {
    for (mut grading, global) in camera_query.iter_mut() {
        let y = global.translation().y;
        let depth = SEA_LEVEL - y;
        let depth_factor = if depth > 0.0 {
            let t = (depth / DEPTH_COLOR_TRANSITION).min(1.0);
            let smooth = t * t * (3.0 - 2.0 * t);
            (depth.min(80.0) / 80.0).powf(0.6) * smooth
        } else {
            0.0
        };
        grading.global.temperature = -0.6 * depth_factor;
        grading.global.post_saturation = 1.0 + 0.25 * depth_factor;
    }
}

fn update_depth_fog(
    mut camera_query: Query<(&mut bevy::pbr::DistanceFog, &GlobalTransform), With<PlayerCamera>>,
) {
    use bevy::pbr::FogFalloff;
    for (mut fog, global) in camera_query.iter_mut() {
        let y = global.translation().y;
        let depth = SEA_LEVEL - y;
        if depth > 0.0 {
            let d = depth.min(80.0);
            let density = 0.006 + 0.012 * (d / 80.0).powf(0.7);
            let t = (depth / DEPTH_COLOR_TRANSITION).min(1.0);
            let smooth = t * t * (3.0 - 2.0 * t);
            fog.color = Color::srgba(
                0.2 * smooth + 0.5 * (1.0 - smooth),
                0.35 * smooth + 0.6 * (1.0 - smooth),
                0.6 * smooth + 0.8 * (1.0 - smooth),
                0.35 * smooth + 0.2 * (1.0 - smooth),
            );
            fog.falloff = FogFalloff::Exponential { density };
        } else {
            fog.color = Color::srgba(0.5, 0.6, 0.8, 0.2);
            fog.falloff = FogFalloff::Exponential { density: 0.008 };
        }
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

const VEHICLE_ENTER_RANGE: f32 = 6.0;

fn toggle_boat_enter(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mode: ResMut<PlayerMode>,
    mut commands: Commands,
    camera_query: Query<Entity, With<PlayerCamera>>,
    character_query: Query<(Entity, &Transform), With<MarineCharacter>>,
    ship_query: Query<(Entity, &Transform), With<Ship>>,
    sub_query: Query<(Entity, &Transform), With<Submersible>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Some(cam_id) = camera_query.iter().next() else { return };
    let Some((char_id, char_tf)) = character_query.iter().next() else { return };
    let char_pos = char_tf.translation;

    // Exit vehicle -> character
    if mode.in_vehicle() {
        mode.in_boat = false;
        mode.in_submersible = false;
        commands.entity(cam_id).despawn();
        let cam_id = commands.spawn((
            Camera3d::default(),
            Hdr,
            Tonemapping::AgX,
            DepthPrepass,
            Bloom::NATURAL,
            ColorGrading::default(),
            bevy::pbr::DistanceFog {
                color: Color::srgba(0.5, 0.6, 0.8, 0.2),
                falloff: bevy::pbr::FogFalloff::Exponential { density: 0.008 },
                ..default()
            },
            PlayerCamera,
            Transform::from_xyz(0.0, 0.9, 0.0),
        )).id();
        commands.entity(char_id).add_children(&[cam_id]);
        return;
    }

    // On foot -> enter nearest vehicle
    let camera_components = (
        Camera3d::default(),
        Hdr,
        Tonemapping::AgX,
        DepthPrepass,
        Bloom::NATURAL,
        ColorGrading::default(),
        bevy::pbr::DistanceFog {
            color: Color::srgba(0.5, 0.6, 0.8, 0.2),
            falloff: bevy::pbr::FogFalloff::Exponential { density: 0.008 },
            ..default()
        },
        PlayerCamera,
    );

    let mut nearest: Option<(f32, bool, Entity)> = None; // (dist_sq, is_boat, entity)
    if let Some((ship_id, ship_tf)) = ship_query.iter().next() {
        let d = (ship_tf.translation - char_pos).length_squared();
        if d <= VEHICLE_ENTER_RANGE * VEHICLE_ENTER_RANGE {
            nearest = Some((d, true, ship_id));
        }
    }
    if let Some((sub_id, sub_tf)) = sub_query.iter().next() {
        let d = (sub_tf.translation - char_pos).length_squared();
        if d <= VEHICLE_ENTER_RANGE * VEHICLE_ENTER_RANGE {
            if nearest.map(|(nd, _, _)| d < nd).unwrap_or(true) {
                nearest = Some((d, false, sub_id));
            }
        }
    }

    commands.entity(cam_id).despawn();
    if let Some((_, is_boat, vehicle_id)) = nearest {
        if is_boat {
            mode.in_boat = true;
            let cam_id = commands.spawn((
                camera_components,
                Transform::from_xyz(0.0, 4.0, 12.0).looking_at(Vec3::new(0.0, 0.0, -5.0), Vec3::Y),
            )).id();
            commands.entity(vehicle_id).add_children(&[cam_id]);
        } else {
            mode.in_submersible = true;
            let cam_id = commands.spawn((
                camera_components,
                Transform::from_xyz(0.0, 1.5, 8.0).looking_at(Vec3::new(0.0, 0.0, -6.0), Vec3::Y),
            )).id();
            commands.entity(vehicle_id).add_children(&[cam_id]);
        }
    } else {
        let cam_id = commands.spawn((
            camera_components,
            Transform::from_xyz(0.0, 0.9, 0.0),
        )).id();
        commands.entity(char_id).add_children(&[cam_id]);
    }
}
