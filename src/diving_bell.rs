//! Submersible - drivable, collidable, oxygen drain underwater.

use bevy::gltf::GltfAssetLabel;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::scene::SceneRoot;

use bevy_rapier3d::prelude::*;
use crate::ocean::SEA_LEVEL;
use crate::player::PlayerMode;

#[derive(Component)]
pub struct Submersible {
    pub drive_power: f32,
    pub turn_speed: f32,
    pub ascend_speed: f32,
    pub current_throttle: f32,
    pub current_steering: f32,
    pub current_vertical: f32,
}

#[derive(Component)]
pub struct SubmersibleVelocity(pub Vec3);

#[derive(Component)]
pub struct DivingBell {
    pub max_oxygen: f32,
    pub current_oxygen: f32,
    pub oxygen_drain_rate: f32,
}

pub struct DivingBellPlugin;

impl Plugin for DivingBellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_diving_bell)
            .add_systems(
                Update,
                (
                    diving_bell_oxygen,
                    submersible_input.run_if(|mode: Res<PlayerMode>| mode.in_submersible),
                    submersible_mouse_look.run_if(|mode: Res<PlayerMode>| mode.in_submersible),
                    submersible_movement,
                ),
            );
    }
}

fn spawn_diving_bell(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/submersible.glb"));
    let light_id = commands.spawn((
        PointLight {
            intensity: 8_000.0,
            range: 40.0,
            color: Color::srgba(1.0, 0.95, 0.85, 1.0),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 4.0),
    )).id();

    let sub_id = commands
        .spawn((
            RigidBody::KinematicVelocityBased,
            Collider::cylinder(3.0, 2.5),
            GravityScale(0.0),
            Velocity::default(),
            SceneRoot(scene),
            Transform::from_xyz(0.0, -4.0, -25.0).with_scale(Vec3::splat(4.0)),
            Submersible {
            drive_power: 15.0,
            turn_speed: 1.2,
            ascend_speed: 8.0,
            current_throttle: 0.0,
            current_steering: 0.0,
            current_vertical: 0.0,
        },
        SubmersibleVelocity(Vec3::ZERO),
        DivingBell {
            max_oxygen: 100.0,
            current_oxygen: 100.0,
            oxygen_drain_rate: 2.0,
        },
    ))
    .id();

    commands.entity(sub_id).add_child(light_id);
}

fn diving_bell_oxygen(
    mut query: Query<(&Transform, &mut DivingBell)>,
    time: Res<Time>,
) {
    for (transform, mut bell) in query.iter_mut() {
        if transform.translation.y < SEA_LEVEL {
            bell.current_oxygen = (bell.current_oxygen - bell.oxygen_drain_rate * time.delta_secs()).max(0.0);
        }
    }
}

fn submersible_movement(
    mut query: Query<(&Submersible, &mut SubmersibleVelocity, &mut Transform, &mut Velocity)>,
    time: Res<Time>,
) {
    const WATER_DRAG: f32 = 0.95;    // decay when idle; sub holds depth/position (neutral buoyancy)

    for (sub, mut vel, transform, mut rb_vel) in query.iter_mut() {
        let forward = transform.forward();
        vel.0 += forward * sub.drive_power * sub.current_throttle * time.delta_secs();
        vel.0.y += sub.ascend_speed * sub.current_vertical * time.delta_secs();

        // Neutral buoyancy: no input = no sink/rise. Sub is trim-able and holds depth.
        vel.0 *= WATER_DRAG;

        rb_vel.linvel = vel.0;
        rb_vel.angvel = Vec3::new(0.0, sub.turn_speed * sub.current_steering, 0.0);
    }
}

fn submersible_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Submersible>,
) {
    for mut sub in query.iter_mut() {
        sub.current_throttle = if keyboard.pressed(KeyCode::KeyW) {
            1.0
        } else if keyboard.pressed(KeyCode::KeyS) {
            -1.0
        } else {
            0.0
        };
        sub.current_steering = if keyboard.pressed(KeyCode::KeyA) {
            1.0
        } else if keyboard.pressed(KeyCode::KeyD) {
            -1.0
        } else {
            0.0
        };
        sub.current_vertical = if keyboard.pressed(KeyCode::Space) {
            1.0
        } else if keyboard.pressed(KeyCode::ShiftLeft) {
            -1.0
        } else {
            0.0
        };
    }
}

fn submersible_mouse_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<&mut Transform, With<Submersible>>,
) {
    let delta_x = mouse_motion.delta.x;
    if delta_x == 0.0 {
        return;
    }
    const SENSITIVITY: f32 = 0.002;
    for mut transform in query.iter_mut() {
        transform.rotate_y(-delta_x * SENSITIVITY);
    }
}

