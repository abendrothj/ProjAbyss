//! Procedural scatter: rocks, seaweed, debris around islands and seafloor.

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::scene::SceneRoot;

use crate::islands::{IslandCollider, SafeIsland};
use crate::ocean::SEA_LEVEL;
use crate::world::MAP_FLOOR_Y;

/// Scatter props around island bases and on seafloor.
pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_scatter);
    }
}

fn spawn_scatter(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    island_query: Query<(&Transform, &IslandCollider), Without<SafeIsland>>,
) {
    let buoy_scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/buoy.glb"));
    let rock_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.32, 0.28),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });
    let seaweed_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.45, 0.3),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });
    let debris_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.32, 0.22),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });

    // Rocks: irregular boulders (squashed spheres) + pebbles (scaled cuboids)
    // Boulders ~1–2m, pebbles ~0.3–0.5m for visibility at map scale
    let rock_boulder_mesh = meshes.add(Sphere::new(0.8).mesh().uv(8, 6));
    let rock_pebble_mesh = meshes.add(Cuboid::new(0.4, 0.25, 0.5).mesh().build());
    // Seaweed: tapered capsule (thicker base, thinner tip) ~1–2m tall
    let seaweed_mesh = meshes.add(Capsule3d::new(0.08, 1.2).mesh().build());
    // Debris: varied crate/barrel shapes (~0.8–1m for visibility)
    let debris_crate_mesh = meshes.add(Cuboid::new(0.5, 0.4, 0.6).mesh().build());
    let debris_barrel_mesh = meshes.add(Cylinder::new(0.25, 0.5).mesh().resolution(8));

    for (transform, collider) in island_query.iter() {
        let center = transform.translation;
        let radius = collider.radius;

        // Scatter rocks – mix of boulders and pebbles, varied scale/rotation
        let rock_count = (radius * 0.8) as usize;
        for i in 0..rock_count {
            let angle = (i as f32 * 2.1) % std::f32::consts::TAU;
            let dist = radius * 0.7 + (i as f32 * 0.3 % 0.4);
            let x = center.x + angle.cos() * dist;
            let z = center.z + angle.sin() * dist;
            let scale = 0.8 + (i as f32 * 0.15 % 1.2);
            let is_boulder = (i % 3) != 0;
            let mesh = if is_boulder {
                rock_boulder_mesh.clone()
            } else {
                rock_pebble_mesh.clone()
            };
            let scale_vec = if is_boulder {
                Vec3::new(scale * 1.1, scale * 0.9, scale * 1.0) // squashed
            } else {
                Vec3::splat(scale)
            };
            // Rocks at shore (island-relative): just above waterline
            let rock_y = center.y + SEA_LEVEL + 1.5;
            commands.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(rock_mat.clone()),
                Transform::from_xyz(x, rock_y, z)
                    .with_scale(scale_vec)
                    .with_rotation(Quat::from_euler(
                        EulerRot::ZXY,
                        angle * 0.3,
                        angle * 0.5,
                        angle * 0.2,
                    )),
            ));
        }

        // Scatter seaweed (tapered capsules) in shallow water (~0.5m below surface)
        if radius > 15.0 {
            let seaweed_count = (radius * 0.3) as usize;
            let seaweed_y = center.y + SEA_LEVEL + 0.5;
            for i in 0..seaweed_count {
                let angle = (i as f32 * 3.7) % std::f32::consts::TAU;
                let dist = radius * 0.6 + (i as f32 * 0.2 % 0.6);
                let x = center.x + angle.cos() * dist;
                let z = center.z + angle.sin() * dist;
                let scale = 0.8 + (i as f32 * 0.2 % 0.8);
                commands.spawn((
                    Mesh3d(seaweed_mesh.clone()),
                    MeshMaterial3d(seaweed_mat.clone()),
                    Transform::from_xyz(x, seaweed_y, z)
                        .with_scale(Vec3::new(scale, scale * 1.4, scale))
                        .with_rotation(Quat::from_rotation_z((i as f32 * 0.5) % 0.4)),
                ));
            }
        }
    }

    // Scatter buoys in open water across the map
    for i in 0..24 {
        let angle = (i as f32 * 1.7) * std::f32::consts::TAU / 24.0;
        let dist = 400.0 + (i as f32 * 150.0) % 1800.0;
        let x = angle.cos() * dist;
        let z = angle.sin() * dist;
        commands.spawn((
            SceneRoot(buoy_scene.clone()),
            Transform::from_xyz(x, 0.5, z)
                .with_scale(Vec3::splat(2.0))
                .with_rotation(Quat::from_rotation_y(angle)),
        ));
    }

    // Scatter debris on seafloor – crates and barrels across the map
    for gx in -8..=8 {
        for gz in -8..=8 {
            let x = gx as f32 * 120.0 + 80.0;
            let z = gz as f32 * 120.0 - 60.0;
            let hash = ((x * 7.0 + z * 13.0) as u32) % 100;
            if hash < 15 {
                let is_crate = hash % 2 == 0;
                let (mesh, scale_vec) = if is_crate {
                    (debris_crate_mesh.clone(), Vec3::new(1.2, 1.0, 1.3))
                } else {
                    (debris_barrel_mesh.clone(), Vec3::new(1.2, 1.0, 1.2))
                };
                commands.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(debris_mat.clone()),
                    Transform::from_xyz(x, MAP_FLOOR_Y + 0.05, z)
                        .with_scale(scale_vec)
                        .with_rotation(Quat::from_rotation_y(x * 0.1)),
                ));
            }
        }
    }
}
