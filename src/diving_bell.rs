//! Submersible - drivable, collidable, oxygen drain underwater.

use bevy::gltf::GltfAssetLabel;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::scene::SceneRoot;

use crate::islands::ColliderShape;
use crate::player::PlayerMode;

/// Collision radius for submersible hull.
const SUB_COLLISION_RADIUS: f32 = 2.0;

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
                    submersible_movement,
                    submersible_input.run_if(|mode: Res<PlayerMode>| mode.in_submersible),
                    submersible_mouse_look.run_if(|mode: Res<PlayerMode>| mode.in_submersible),
                    submersible_island_collision,
                ),
            );
    }
}

fn spawn_diving_bell(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/submersible.glb"));
    commands.spawn((
        SceneRoot(scene),
        Transform::from_xyz(0.0, -4.0, -25.0).with_scale(Vec3::splat(4.0)),
        ColliderShape::Cylinder {
            radius: 2.5,
            half_height: 3.0,
        },
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
    ));
}

fn diving_bell_oxygen(
    mut query: Query<(&Transform, &mut DivingBell)>,
    time: Res<Time>,
) {
    for (transform, mut bell) in query.iter_mut() {
        if transform.translation.y < 0.0 {
            bell.current_oxygen = (bell.current_oxygen - bell.oxygen_drain_rate * time.delta_secs()).max(0.0);
        }
    }
}

fn submersible_movement(
    mut query: Query<(&Submersible, &mut SubmersibleVelocity, &mut Transform)>,
    time: Res<Time>,
) {
    for (sub, mut vel, mut transform) in query.iter_mut() {
        let forward = transform.forward();
        vel.0 += forward * sub.drive_power * sub.current_throttle * time.delta_secs();
        vel.0.y += sub.ascend_speed * sub.current_vertical * time.delta_secs();
        transform.rotate_y(sub.turn_speed * sub.current_steering * time.delta_secs());

        transform.translation += vel.0 * time.delta_secs();
        vel.0 *= 0.97;
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

fn submersible_island_collision(
    mut sub_query: Query<(&mut SubmersibleVelocity, &mut Transform), With<Submersible>>,
    collider_query: Query<(&GlobalTransform, &ColliderShape), Without<Submersible>>,
) {
    for (mut vel, mut sub_tf) in sub_query.iter_mut() {
        let sub_pos = sub_tf.translation;
        let margin = SUB_COLLISION_RADIUS;

        for (global, shape) in collider_query.iter() {
            if let Some(push) = shape.penetration(
                global.translation(),
                global.rotation(),
                sub_pos,
                margin,
            ) {
                sub_tf.translation += push;

                let push_xz = push.xz();
                if push_xz.length_squared() > 0.0001 {
                    let push_dir = push_xz.normalize();
                    let into = vel.0.x * push_dir.x + vel.0.z * push_dir.y;
                    if into < 0.0 {
                        vel.0.x -= into * push_dir.x;
                        vel.0.z -= into * push_dir.y;
                    }
                }
                if push.y.abs() > 0.001 {
                    if vel.0.y * push.y < 0.0 {
                        vel.0.y = 0.0;
                    }
                }
            }
        }
    }
}
