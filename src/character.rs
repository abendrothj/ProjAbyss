//! Marine character - first person, WASD, jump, mouse look.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;

use crate::islands::ColliderShape;
use crate::ocean::OceanSolver;
use crate::player::{PlayerCamera, PlayerMode};

const CHARACTER_COLLISION_RADIUS: f32 = 0.5;
const SWIM_SPEED: f32 = 2.5;
const SWIM_ASCEND_SPEED: f32 = 3.0;
const SWIM_DESCEND_SPEED: f32 = 2.0;
const SWIM_DRAG: f32 = 4.0;

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
                    character_mouse_look.run_if(|mode: Res<PlayerMode>| !mode.in_vehicle()),
                    character_movement.run_if(|mode: Res<PlayerMode>| !mode.in_vehicle()),
                    character_island_collision.run_if(|mode: Res<PlayerMode>| !mode.in_vehicle()),
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
            bevy::core_pipeline::prepass::DepthPrepass,
            bevy::post_process::bloom::Bloom::NATURAL,
            bevy::render::view::ColorGrading::default(),
            bevy::pbr::DistanceFog {
                color: Color::srgba(0.5, 0.6, 0.8, 0.2),
                falloff: bevy::pbr::FogFalloff::Exponential { density: 0.008 },
                ..default()
            },
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
    ocean: Res<OceanSolver>,
    mut query: Query<(&MarineCharacter, &mut CharacterVelocity, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (char, mut vel, mut transform) in query.iter_mut() {
        let pos = transform.translation;
        let wave_height = ocean.wave_height_at(pos);
        let underwater = pos.y < wave_height - 0.1;

        if underwater {
            // Swimming: no gravity, Space ascend, Shift descend, WASD swim
            vel.0.y *= 1.0 - SWIM_DRAG * dt;

            if keyboard.pressed(KeyCode::Space) {
                vel.0.y += SWIM_ASCEND_SPEED * dt;
            }
            if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
                vel.0.y -= SWIM_DESCEND_SPEED * dt;
            }
            vel.0.y = vel.0.y.clamp(-SWIM_DESCEND_SPEED * 2.0, SWIM_ASCEND_SPEED * 2.0);

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
                vel.0.x = dir.x * SWIM_SPEED;
                vel.0.z = dir.z * SWIM_SPEED;
            } else {
                vel.0.x *= 1.0 - SWIM_DRAG * dt;
                vel.0.z *= 1.0 - SWIM_DRAG * dt;
            }
        } else {
            // Walking: gravity, jump, WASD
            vel.0.y -= 9.8 * dt;

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
        }

        transform.translation += vel.0 * dt;

        // Floor at seafloor
        const FLOOR_Y: f32 = -80.0;
        if transform.translation.y < FLOOR_Y {
            transform.translation.y = FLOOR_Y;
            vel.0.y = 0.0;
        }
    }
}

fn character_island_collision(
    mut char_query: Query<(&mut CharacterVelocity, &mut Transform), With<MarineCharacter>>,
    collider_query: Query<(&GlobalTransform, &ColliderShape), Without<MarineCharacter>>,
) {
    for (mut vel, mut char_tf) in char_query.iter_mut() {
        let char_pos = char_tf.translation;
        let margin = CHARACTER_COLLISION_RADIUS;

        for (global, shape) in collider_query.iter() {
            if let Some(push) = shape.penetration(
                global.translation(),
                global.rotation(),
                char_pos,
                margin,
            ) {
                char_tf.translation += push;
                char_tf.translation.y = char_pos.y;

                let push_xz = push.xz();
                if push_xz.length_squared() > 0.0001 {
                    let push_dir = push_xz.normalize();
                    let into_island = vel.0.x * push_dir.x + vel.0.z * push_dir.y;
                    if into_island < 0.0 {
                        vel.0.x -= into_island * push_dir.x;
                        vel.0.z -= into_island * push_dir.y;
                    }
                }
            }
        }
    }
}
