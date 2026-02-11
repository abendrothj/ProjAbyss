//! ProjAbyss - Co-op extraction survival. Sail -> Scan -> Dive -> Extract.

use bevy::prelude::*;

mod ocean;
mod ship;
mod diving_bell;
mod character;

use ocean::OceanPlugin;
use ship::ShipPlugin;
use diving_bell::DivingBellPlugin;
use character::CharacterPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(OceanPlugin)
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
    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 3000.0,
            range: 50.0,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 10.0, 5.0),
        ..default()
    });

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Ocean plane (visual) - use circle rotated flat
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(50.0)),
        material: materials.add(Color::srgb(0.2, 0.4, 0.7)),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    // Ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(200.0, 1.0, 200.0)),
        material: materials.add(Color::srgb(0.3, 0.25, 0.2)),
        transform: Transform::from_xyz(0.0, -0.5, 0.0),
        ..default()
    });
}
