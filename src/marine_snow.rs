//! Marine snow particles – floating debris when underwater.
//! Adds scale and atmosphere to the abyss (design doc: Midnight Zone).

use bevy::prelude::*;

use crate::ocean::OceanSolver;
use crate::player::PlayerCamera;

const PARTICLE_COUNT: usize = 400;
const SPHERE_RADIUS: f32 = 10.0;
const RECYCLE_RADIUS: f32 = 12.0;
const DRIFT_SPEED: f32 = 0.15;
const SIZE_MIN: f32 = 0.01;
const SIZE_MAX: f32 = 0.035;

#[derive(Component)]
struct MarineSnowRoot;

#[derive(Component)]
struct MarineSnowParticle {
    velocity: Vec3,
}

pub struct MarineSnowPlugin;

impl Plugin for MarineSnowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_marine_snow)
            .add_systems(Update, update_marine_snow);
    }
}

fn spawn_marine_snow(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0)); // unit cube, scale per-particle
    let material_white = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.45),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let material_gray = materials.add(StandardMaterial {
        base_color: Color::srgba(0.88, 0.9, 0.95, 0.35),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let mut rng = FastNoise::new(12345);
    let root = commands
        .spawn((
            MarineSnowRoot,
            Visibility::Hidden,
            Transform::default(),
        ))
        .id();

    let mut children = Vec::with_capacity(PARTICLE_COUNT);
    for _ in 0..PARTICLE_COUNT {
        let pos = random_in_sphere(&mut rng, SPHERE_RADIUS);
        let vel = Vec3::new(
            rng.next() * 2.0 - 1.0,
            rng.next() * 2.0 - 1.0,
            rng.next() * 2.0 - 1.0,
        ) * DRIFT_SPEED * 0.5;
        let base_scale = SIZE_MIN + rng.next() * (SIZE_MAX - SIZE_MIN);
        let mat = if rng.next() > 0.7 {
            material_gray.clone()
        } else {
            material_white.clone()
        };

        let child = commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(pos).with_scale(Vec3::splat(base_scale)),
                MarineSnowParticle { velocity: vel },
            ))
            .id();
        children.push(child);
    }

    commands.entity(root).add_children(&children);
}

fn update_marine_snow(
    ocean: Res<OceanSolver>,
    time: Res<Time>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    mut root_query: Query<(&mut Transform, &mut Visibility), With<MarineSnowRoot>>,
    mut particle_query: Query<(&mut Transform, &mut MarineSnowParticle), Without<MarineSnowRoot>>,
) {
    let Some(cam_global) = camera_query.iter().next() else { return };
    let Some((mut root_tf, mut root_vis)) = root_query.iter_mut().next() else { return };

    let cam_pos = cam_global.translation();
    let wave_height = ocean.wave_height_at(cam_pos);
    let underwater = cam_pos.y < wave_height - 0.3;

    *root_vis = if underwater {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    root_tf.translation = cam_pos;

    if !underwater {
        return;
    }

    let dt = time.delta_secs();
    let mut rng = FastNoise::new(0);

    for (mut tf, mut particle) in particle_query.iter_mut() {
        particle.velocity += Vec3::new(
            (rng.next() - 0.5) * 2.0,
            (rng.next() - 0.5) * 2.0,
            (rng.next() - 0.5) * 2.0,
        ) * DRIFT_SPEED * 0.3;
        particle.velocity = particle.velocity.clamp_length_max(DRIFT_SPEED * 2.0);

        tf.translation += particle.velocity * dt;

        if tf.translation.length() > RECYCLE_RADIUS {
            tf.translation = random_in_sphere(&mut rng, SPHERE_RADIUS * 0.8);
            particle.velocity = Vec3::new(
                rng.next() * 2.0 - 1.0,
                rng.next() * 2.0 - 1.0,
                rng.next() * 2.0 - 1.0,
            ) * DRIFT_SPEED * 0.5;
        }
    }
}

fn random_in_sphere(rng: &mut FastNoise, radius: f32) -> Vec3 {
    Vec3::new(
        (rng.next() * 2.0 - 1.0) * radius,
        (rng.next() * 2.0 - 1.0) * radius,
        (rng.next() * 2.0 - 1.0) * radius,
    )
}

/// Simple deterministic RNG for reproducible snow.
struct FastNoise(u32);

impl FastNoise {
    fn new(seed: u32) -> Self {
        Self(seed)
    }
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1103515245).wrapping_add(12345);
        ((self.0 >> 16) & 0x7FFF) as f32 / 32767.0
    }
}
