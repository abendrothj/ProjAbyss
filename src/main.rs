//! ProjAbyss - Co-op extraction survival. Sail -> Scan -> Dive -> Extract.

use bevy::prelude::*;
use bevy::light::DirectionalLightShadowMap;

mod ocean;
mod ship;
mod diving_bell;
mod character;
mod player;
mod islands;
mod scatter;

use ocean::OceanPlugin;
use ship::ShipPlugin;
use diving_bell::DivingBellPlugin;
use character::CharacterPlugin;
use player::PlayerPlugin;
use islands::ColliderShape;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.42, 0.6, 0.88)))
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, setup_scene)
        .add_plugins(DefaultPlugins)
        .add_plugins(OceanPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ShipPlugin)
        .add_plugins(DivingBellPlugin)
        .add_plugins(CharacterPlugin)
        .add_plugins(scatter::ScatterPlugin)
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

    // Seafloor – dark sand / sediment (submerged, ~80m deep). Collisions + driveable.
    commands.spawn((
        ColliderShape::Box {
            half_extents: Vec3::new(800.0, 0.5, 800.0),
        },
        Mesh3d(meshes.add(Cuboid::new(1600.0, 1.0, 1600.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.22, 0.28, 0.35),
            perceptual_roughness: 0.95,
            metallic: 0.0,
            ..default()
        })),
        Transform::from_xyz(0.0, -80.5, 0.0),
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

    crate::islands::spawn_all_islands(
        &mut commands,
        &mut meshes,
        &island_material,
        &rock_material,
        &sand_material,
    );
}
