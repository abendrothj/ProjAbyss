//! ProjAbyss - Co-op extraction survival. Sail -> Scan -> Dive -> Extract.

use bevy::prelude::*;
use bevy::light::DirectionalLightShadowMap;

mod ocean;
mod ship;
mod diving_bell;
mod character;
mod player;
mod islands;

use ocean::OceanPlugin;
use ship::ShipPlugin;
use diving_bell::DivingBellPlugin;
use character::CharacterPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.42, 0.6, 0.88)))
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .add_plugins(OceanPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ShipPlugin)
        .add_plugins(DivingBellPlugin)
        .add_plugins(CharacterPlugin)
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ambient light – warm sky fill
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.65, 0.78, 0.95),
        brightness: 500.0,
        ..default()
    });

    // Sun – directional light with shadows
    commands.spawn((
        DirectionalLight {
            illuminance: 25_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(30.0, 50.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Fill light (soften shadows)
    commands.spawn((
        PointLight {
            intensity: 12_000.0,
            range: 100.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-15.0, 20.0, 15.0),
    ));

    // Seafloor – dark sand / sediment (submerged)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1600.0, 1.0, 1600.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.28, 0.35),
            perceptual_roughness: 0.95,
            metallic: 0.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    // Materials for islands
    let island_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.45, 0.3),
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });
    let rock_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.38, 0.35),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });
    let sand_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.76, 0.7, 0.5),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });

    use crate::islands::IslandCollider;

    // Varied island shapes – elongated, volcanic, rocky
    // (center, mesh_fn, scale, collision_radius)
    let island_defs: Vec<(Vec3, fn(&mut Assets<Mesh>) -> Handle<Mesh>, Vec3, f32)> = vec![
        // Large elongated island (cuboid)
        (
            Vec3::new(-120.0, 2.0, -200.0),
            |m| m.add(Cuboid::new(40.0, 8.0, 60.0).mesh()),
            Vec3::ONE,
            45.0,
        ),
        // Volcanic cone
        (
            Vec3::new(180.0, 3.0, 150.0),
            |m| m.add(Cone::default().mesh().resolution(24)),
            Vec3::new(25.0, 30.0, 25.0),
            35.0,
        ),
        // Rocky spire (tall cuboid)
        (
            Vec3::new(80.0, 4.0, -300.0),
            |m| m.add(Cuboid::new(15.0, 25.0, 12.0).mesh()),
            Vec3::ONE,
            25.0,
        ),
        // Atoll-like ring (torus)
        (
            Vec3::new(-250.0, 1.0, 100.0),
            |m| m.add(Torus::new(20.0, 8.0).mesh().minor_resolution(16).major_resolution(32)),
            Vec3::ONE,
            35.0,
        ),
        // Low sand cay (flat capsule)
        (
            Vec3::new(300.0, 0.8, -80.0),
            |m| m.add(Capsule3d::new(8.0, 25.0).mesh()),
            Vec3::new(1.0, 0.5, 1.0),
            20.0,
        ),
        // Chunk of rock (irregular cuboid)
        (
            Vec3::new(-80.0, 1.5, 220.0),
            |m| m.add(Cuboid::new(22.0, 6.0, 18.0).mesh()),
            Vec3::ONE,
            20.0,
        ),
        // Small forested hill (sphere)
        (
            Vec3::new(50.0, 4.0, 280.0),
            |m| m.add(Sphere::new(18.0).mesh().uv(24, 16)),
            Vec3::ONE,
            22.0,
        ),
        // Long reef (elongated box)
        (
            Vec3::new(-350.0, 0.5, -150.0),
            |m| m.add(Cuboid::new(80.0, 2.0, 15.0).mesh()),
            Vec3::ONE,
            45.0,
        ),
        // Rocky outcrop cluster
        (
            Vec3::new(220.0, 2.0, 250.0),
            |m| m.add(Cuboid::new(18.0, 10.0, 14.0).mesh()),
            Vec3::ONE,
            18.0,
        ),
        // Island near spawn
        (
            Vec3::new(-15.0, 1.5, 10.0),
            |m| m.add(Cuboid::new(12.0, 4.0, 8.0).mesh()),
            Vec3::ONE,
            12.0,
        ),
    ];

    for (center, mesh_fn, scale, coll_radius) in island_defs {
        let mesh = mesh_fn(&mut meshes);
        let is_rock = center.y > 2.5 || coll_radius < 15.0;
        let mat = if is_rock {
            rock_material.clone()
        } else {
            island_material.clone()
        };
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_translation(center).with_scale(scale),
            IslandCollider { radius: coll_radius },
        ));
        // Sand beach ring for larger islands
        if coll_radius >= 20.0 && !is_rock {
            let ring_radius = coll_radius * 1.15;
            commands.spawn((
                Mesh3d(meshes.add(Cylinder::new(ring_radius, 0.5))),
                MeshMaterial3d(sand_material.clone()),
                Transform::from_translation(center),
                IslandCollider { radius: ring_radius },
            ));
        }
    }
}
