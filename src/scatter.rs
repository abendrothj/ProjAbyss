//! Procedural scatter: rocks, seaweed, debris around islands and seafloor.

use bevy::gltf::GltfAssetLabel;
use bevy::prelude::*;
use bevy::scene::SceneRoot;

use crate::islands::{IslandCollider, SafeIsland};

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

    let rock_mesh = meshes.add(Cuboid::new(0.4, 0.3, 0.5).mesh().build());
    let seaweed_mesh = meshes.add(Cylinder::new(0.08, 0.8).mesh().resolution(6));
    let debris_mesh = meshes.add(Cuboid::new(0.25, 0.1, 0.4).mesh().build());

    for (transform, collider) in island_query.iter() {
        let center = transform.translation;
        let radius = collider.radius;

        // Scatter rocks around island perimeter
        let rock_count = (radius * 0.8) as usize;
        for i in 0..rock_count {
            let angle = (i as f32 * 2.1) % std::f32::consts::TAU;
            let dist = radius * 0.7 + (i as f32 * 0.3 % 0.4);
            let x = center.x + angle.cos() * dist;
            let z = center.z + angle.sin() * dist;
            let scale = 0.5 + (i as f32 * 0.1 % 0.8);
            commands.spawn((
                Mesh3d(rock_mesh.clone()),
                MeshMaterial3d(rock_mat.clone()),
                Transform::from_xyz(x, 0.2, z)
                    .with_scale(Vec3::splat(scale))
                    .with_rotation(Quat::from_rotation_y(angle * 0.5)),
            ));
        }

        // Scatter seaweed (small cylinders) in shallow water
        if radius > 15.0 {
            let seaweed_count = (radius * 0.3) as usize;
            for i in 0..seaweed_count {
                let angle = (i as f32 * 3.7) % std::f32::consts::TAU;
                let dist = radius * 0.6 + (i as f32 * 0.2 % 0.6);
                let x = center.x + angle.cos() * dist;
                let z = center.z + angle.sin() * dist;
                let scale = 0.6 + (i as f32 * 0.15 % 0.6);
                commands.spawn((
                    Mesh3d(seaweed_mesh.clone()),
                    MeshMaterial3d(seaweed_mat.clone()),
                    Transform::from_xyz(x, 0.1, z)
                        .with_scale(Vec3::new(scale, scale * 1.2, scale))
                        .with_rotation(Quat::from_rotation_z(0.2)),
                ));
            }
        }
    }

    // Scatter buoys in open water
    for i in 0..12 {
        let angle = (i as f32 * 1.7) * std::f32::consts::TAU / 12.0;
        let dist = 80.0 + (i as f32 * 11.0) % 60.0;
        let x = angle.cos() * dist;
        let z = angle.sin() * dist;
        commands.spawn((
            SceneRoot(buoy_scene.clone()),
            Transform::from_xyz(x, 0.5, z)
                .with_scale(Vec3::splat(1.5))
                .with_rotation(Quat::from_rotation_y(angle)),
        ));
    }

    // Scatter debris on seafloor (random grid, seafloor at y ≈ -80)
    const SEAFLOOR_Y: f32 = -80.0;
    for gx in -3..=3 {
        for gz in -3..=3 {
            let x = gx as f32 * 35.0 + 12.0;
            let z = gz as f32 * 35.0 - 8.0;
            let hash = ((x * 7.0 + z * 13.0) as u32) % 100;
            if hash < 15 {
                commands.spawn((
                    Mesh3d(debris_mesh.clone()),
                    MeshMaterial3d(debris_mat.clone()),
                    Transform::from_xyz(x, SEAFLOOR_Y + 0.05, z)
                        .with_scale(Vec3::new(0.8, 0.5, 1.2))
                        .with_rotation(Quat::from_rotation_y(x * 0.1)),
                ));
            }
        }
    }
}
