//! Marine character - first person, WASD, jump, mouse look.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;

use crate::islands::IslandCollider;
use crate::player::{PlayerCamera, PlayerMode};

const CHARACTER_COLLISION_RADIUS: f32 = 0.5;

#[derive(Component)]
pub struct MarineCharacter {
    pub walk_speed: f32,
    pub jump_velocity: f32,
}

#[derive(Component)]
pub struct CharacterVelocity(pub Vec3);

/// Horizontal (yaw) and vertical (pitch) look angles in radians.
#[derive(Component, Default)]
pub struct CharacterLook {
    pub yaw: f32,
    pub pitch: f32,
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_character)
            .add_systems(
                Update,
                (
                    character_mouse_look.run_if(|mode: Res<PlayerMode>| !mode.in_boat),
                    character_movement.run_if(|mode: Res<PlayerMode>| !mode.in_boat),
                    character_island_collision.run_if(|mode: Res<PlayerMode>| !mode.in_boat),
                ),
            );
    }
}

fn spawn_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.4, 0.9))),
        MeshMaterial3d(materials.add(Color::srgb_u8(200, 150, 100))),
        Transform::from_xyz(3.0, 1.0, 5.0),
        MarineCharacter {
            walk_speed: 4.0,
            jump_velocity: 6.0,
        },
        CharacterVelocity(Vec3::ZERO),
        CharacterLook::default(),
        children![(
            Camera3d::default(),
            bevy::render::view::Hdr,
            bevy::core_pipeline::tonemapping::Tonemapping::AgX,
            PlayerCamera,
            Transform::from_xyz(0.0, 0.9, 0.0),
        )],
    ));
}

fn character_mouse_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<(&mut CharacterLook, &mut Transform), With<MarineCharacter>>,
) {
    let delta = mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    const SENSITIVITY: f32 = 0.002;
    for (mut look, mut transform) in query.iter_mut() {
        look.yaw -= delta.x * SENSITIVITY;
        look.pitch -= delta.y * SENSITIVITY;
        look.pitch = look.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch, 0.0);
    }
}

fn character_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&MarineCharacter, &mut CharacterVelocity, &mut Transform)>,
    time: Res<Time>,
) {
    for (char, mut vel, mut transform) in query.iter_mut() {
        vel.0.y -= 9.8 * time.delta_secs();

        if keyboard.just_pressed(KeyCode::Space) {
            vel.0.y = char.jump_velocity;
        }

        let mut input = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            input.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            input.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            input.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            input.x += 1.0;
        }

        if input.length_squared() > 0.0 {
            let dir = transform.rotation * input.normalize();
            vel.0.x = dir.x * char.walk_speed;
            vel.0.z = dir.z * char.walk_speed;
        } else {
            vel.0.x *= 0.9;
            vel.0.z *= 0.9;
        }

        transform.translation += vel.0 * time.delta_secs();

        if transform.translation.y < 0.9 {
            transform.translation.y = 0.9;
            vel.0.y = 0.0;
        }
    }
}

fn character_island_collision(
    mut char_query: Query<(&mut CharacterVelocity, &mut Transform), With<MarineCharacter>>,
    island_query: Query<(&Transform, &IslandCollider), Without<MarineCharacter>>,
) {
    for (mut vel, mut char_tf) in char_query.iter_mut() {
        let char_pos = char_tf.translation;
        for (island_tf, collider) in island_query.iter() {
            let island_pos = island_tf.translation;
            let delta = char_pos - island_pos;
            let dist_xz = delta.xz().length();
            let min_dist = CHARACTER_COLLISION_RADIUS + collider.radius;

            if dist_xz < min_dist {
                let push_dir = if dist_xz > 0.001 {
                    Vec3::new(delta.x / dist_xz, 0.0, delta.z / dist_xz)
                } else {
                    Vec3::new(1.0, 0.0, 0.0)
                };
                let push_amount = min_dist - dist_xz;
                char_tf.translation += push_dir * push_amount;
                char_tf.translation.y = char_pos.y;

                let into_island = vel.0.x * push_dir.x + vel.0.z * push_dir.z;
                if into_island < 0.0 {
                    vel.0.x -= into_island * push_dir.x;
                    vel.0.z -= into_island * push_dir.z;
                }
            }
        }
    }
}
