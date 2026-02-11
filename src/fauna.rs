//! Fauna – fish and creatures. Phase 2: Boids (schooling), flee behavior.
//!
//! VISION: Small fish school with cohesion, separation, alignment. Flee from player/sub.

use bevy::prelude::*;

use crate::character::MarineCharacter;
use crate::diving_bell::Submersible;
use crate::ship::Ship;
use crate::game_state::GameState;
use crate::ocean::SEA_LEVEL;
use crate::world::{MAP_FLOOR_Y, MAP_SCALE_FROM_LEGACY};

/// Small schooling fish. Boids algorithm: cohesion, separation, alignment, flee.
#[derive(Component)]
pub struct Boid {
    pub velocity: Vec3,
    pub school_id: u32,
}

/// Boids steering weights.
const COHESION_WEIGHT: f32 = 0.5;
const SEPARATION_WEIGHT: f32 = 1.2;
const ALIGNMENT_WEIGHT: f32 = 0.8;
const FLEE_WEIGHT: f32 = 2.5;
const COHESION_RADIUS: f32 = 6.0;
const SEPARATION_RADIUS: f32 = 1.5;
const ALIGNMENT_RADIUS: f32 = 5.0;
const FLEE_RADIUS: f32 = 8.0;
const MAX_SPEED: f32 = 2.5;
const MAX_FLEE_SPEED: f32 = 6.0;

/// Snapshot of boid state for neighbor checks (avoids query borrow conflicts).
#[derive(Resource, Default)]
struct BoidSnapshot(Vec<(Entity, Vec3, Vec3, u32)>);

pub struct FaunaPlugin;

impl Plugin for FaunaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BoidSnapshot>()
            .add_systems(Startup, spawn_boid_schools)
            .add_systems(
                Update,
                copy_boid_snapshot.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                boids_steering.run_if(in_state(GameState::Playing)),
            );
    }
}

fn copy_boid_snapshot(
    mut snapshot: ResMut<BoidSnapshot>,
    boid_query: Query<(Entity, &Transform, &Boid)>,
) {
    snapshot.0.clear();
    for (entity, transform, boid) in boid_query.iter() {
        snapshot.0.push((
            entity,
            transform.translation,
            boid.velocity,
            boid.school_id,
        ));
    }
}

fn boids_steering(
    time: Res<Time>,
    snapshot: Res<BoidSnapshot>,
    character_query: Query<&Transform, With<MarineCharacter>>,
    sub_query: Query<&Transform, With<Submersible>>,
    mut boid_query: Query<(&mut Boid, &mut Transform), (With<Boid>, Without<Submersible>, Without<Ship>, Without<MarineCharacter>)>,
) {
    let dt = time.delta_secs();
    let boids = &snapshot.0;

    let threat_positions: Vec<Vec3> = character_query
        .iter()
        .chain(sub_query.iter())
        .filter(|t| t.translation.y < SEA_LEVEL - 0.5)
        .map(|t| t.translation)
        .collect();

    for (idx, &(entity, pos, vel, school_id)) in boids.iter().enumerate() {
        let Ok((mut boid, mut transform)) = boid_query.get_mut(entity) else { continue };

        let mut steer = Vec3::ZERO;

        // Flee from player/sub when underwater
        for threat in &threat_positions {
            let to_boid = pos - *threat;
            let dist = to_boid.length();
            if dist < FLEE_RADIUS && dist > 0.01 {
                let strength = 1.0 - (dist / FLEE_RADIUS).powf(0.5);
                steer += to_boid.normalize() * strength * FLEE_WEIGHT;
            }
        }

        // Boids rules (only with same school)
        let mut cohesion_sum = Vec3::ZERO;
        let mut cohesion_count = 0;
        let mut separation_sum = Vec3::ZERO;
        let mut separation_count = 0;
        let mut alignment_sum = Vec3::ZERO;
        let mut alignment_count = 0;

        for (j, &(_, other_pos, other_vel, other_school)) in boids.iter().enumerate() {
            if other_school != school_id || j == idx {
                continue;
            }
            let delta = other_pos - pos;
            let dist = delta.length();
            if dist < 0.01 {
                continue;
            }

            if dist < COHESION_RADIUS {
                cohesion_sum += other_pos;
                cohesion_count += 1;
            }
            if dist < SEPARATION_RADIUS {
                separation_sum -= delta / dist;
                separation_count += 1;
            }
            if dist < ALIGNMENT_RADIUS {
                alignment_sum += other_vel;
                alignment_count += 1;
            }
        }

        if cohesion_count > 0 {
            let center = cohesion_sum / cohesion_count as f32;
            let to_center = center - pos;
            steer += to_center.normalize_or_zero() * COHESION_WEIGHT;
        }
        if separation_count > 0 {
            steer += separation_sum.normalize_or_zero() * SEPARATION_WEIGHT;
        }
        if alignment_count > 0 {
            let avg_vel = alignment_sum / alignment_count as f32;
            steer += (avg_vel - vel).normalize_or_zero() * ALIGNMENT_WEIGHT;
        }

        let max_speed = if threat_positions.iter().any(|t| (pos - *t).length() < FLEE_RADIUS) {
            MAX_FLEE_SPEED
        } else {
            MAX_SPEED
        };

        boid.velocity += steer * dt * 4.0;
        let speed = boid.velocity.length();
        if speed > max_speed {
            boid.velocity = boid.velocity.normalize() * max_speed;
        }
        if speed < 0.3 && steer.length_squared() < 0.01 {
            boid.velocity += Vec3::new(0.1, 0.0, 0.1) * (school_id as f32 * 0.1).sin();
        }

        // Keep in water column
        let mut new_pos = pos + boid.velocity * dt;
        new_pos.y = new_pos.y.clamp(MAP_FLOOR_Y + 2.0, SEA_LEVEL - 1.0);

        transform.translation = new_pos;
        transform.rotation = quat_from_forward(boid.velocity.normalize_or_zero());
    }
}

fn spawn_boid_schools(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let fish_mesh = meshes.add(
        Capsule3d::new(0.08, 0.2)
            .mesh()
            .build(),
    );
    let fish_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.6, 0.85),
        perceptual_roughness: 0.6,
        metallic: 0.0,
        ..default()
    });

    // Schools spawn in underwater zones (Y < SEA_LEVEL). 10–30 cm fish.
    let school_centers = [
        Vec3::new(80.0 * MAP_SCALE_FROM_LEGACY, -15.0, 120.0 * MAP_SCALE_FROM_LEGACY),
        Vec3::new(-60.0 * MAP_SCALE_FROM_LEGACY, -25.0, -80.0 * MAP_SCALE_FROM_LEGACY),
        Vec3::new(150.0 * MAP_SCALE_FROM_LEGACY, -8.0, 50.0 * MAP_SCALE_FROM_LEGACY),
        Vec3::new(-100.0 * MAP_SCALE_FROM_LEGACY, -35.0, 40.0 * MAP_SCALE_FROM_LEGACY),
        Vec3::new(30.0 * MAP_SCALE_FROM_LEGACY, -20.0, -120.0 * MAP_SCALE_FROM_LEGACY),
    ];

    for (school_id, center) in school_centers.iter().enumerate() {
        let id = school_id as u32;
        let count = 12 + (school_id % 5) * 3;
        for i in 0..count {
            let offset = Vec3::new(
                (i as f32 * 1.7).sin() * 3.0 - 1.0,
                (i as f32 * 0.9).cos() * 2.0,
                (i as f32 * 1.3).cos() * 3.0 + 0.5,
            );
            let pos = *center + offset;
            let vel = Vec3::new(
                (i as f32 * 0.5).sin() * 0.5,
                0.1,
                (i as f32 * 0.7).cos() * 0.5,
            );
            commands.spawn((
                Mesh3d(fish_mesh.clone()),
                MeshMaterial3d(fish_mat.clone()),
                Transform::from_translation(pos)
                    .with_rotation(quat_from_forward(vel.normalize_or_zero())),
                Boid {
                    velocity: vel,
                    school_id: id,
                },
            ));
        }
    }
}

fn quat_from_forward(forward: Vec3) -> Quat {
    if forward.length_squared() < 0.01 {
        return Quat::IDENTITY;
    }
    let f = forward.normalize();
    let up = Vec3::Y;
    let right = up.cross(f).normalize_or_zero();
    let up = f.cross(right).normalize_or_zero();
    Quat::from_mat3(&Mat3::from_cols(right, up, -f))
}
