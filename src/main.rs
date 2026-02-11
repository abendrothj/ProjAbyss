//! ProjAbyss - Co-op extraction survival. Sail -> Scan -> Dive -> Extract.

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler, ImageSamplerDescriptor};
use bevy::light::DirectionalLightShadowMap;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

mod ocean;
mod ship;
mod diving_bell;
mod winch;
mod world;
mod character;
mod player;
mod islands;
mod scatter;
mod marine_snow;

use bevy_rapier3d::prelude::*;

use ocean::OceanPlugin;
use ship::ShipPlugin;
use diving_bell::DivingBellPlugin;
use character::CharacterPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.42, 0.6, 0.88)))
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, setup_scene)
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(OceanPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(ShipPlugin)
        .add_plugins(DivingBellPlugin)
        .add_plugins(winch::WinchPlugin)
        .add_plugins(CharacterPlugin)
        .add_plugins(scatter::ScatterPlugin)
        .add_plugins(marine_snow::MarineSnowPlugin)
        .run();
}

fn create_terrain_noise_texture(size: u32, base_r: f32, base_g: f32, base_b: f32, variation: f32) -> Image {
    let size = size as usize;
    let mut data = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let u = x as f32 / size as f32;
            let v = y as f32 / size as f32;
            let n = (u * 12.0 + 1.0).sin() * (v * 8.0 + 0.5).cos()
                + (u * 20.0 + v * 15.0).sin() * 0.3;
            let v = (n + 1.0) * 0.5 * variation;
            let r = ((base_r + v) * 255.0).clamp(0.0, 255.0) as u8;
            let g = ((base_g + v * 0.8) * 255.0).clamp(0.0, 255.0) as u8;
            let b = ((base_b + v * 0.6) * 255.0).clamp(0.0, 255.0) as u8;
            data.extend([r, g, b, 255]);
        }
    }
    let mut image = Image::new(
        Extent3d { width: size as u32, height: size as u32, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::default());
    image
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Ambient light – warm sky fill
    commands.insert_resource(AmbientLight {
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

    // Seafloor – subdivided plane with FBM vertex displacement (submerged, ~80m deep).
    let seafloor_mesh = islands::create_seafloor_mesh(&mut meshes, world::MAP_SIZE, 64, 2.5, 7.3);
    let seafloor_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.28, 0.35),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });
    commands.spawn((
        RigidBody::Fixed,
        Collider::cuboid(world::MAP_SIZE * 0.5, 0.5, world::MAP_SIZE * 0.5),
        Mesh3d(seafloor_mesh),
        MeshMaterial3d(seafloor_mat),
        Transform::from_xyz(0.0, world::MAP_FLOOR_Y - 0.5, 0.0),
    ));

    // Materials for islands – procedural noise textures for variation
    let island_tex = images.add(create_terrain_noise_texture(64, 0.35, 0.45, 0.3, 0.12));
    let rock_tex = images.add(create_terrain_noise_texture(64, 0.4, 0.38, 0.35, 0.1));
    let island_material = materials.add(StandardMaterial {
        base_color_texture: Some(island_tex),
        base_color: Color::WHITE,
        perceptual_roughness: 0.85,
        metallic: 0.0,
        ..default()
    });
    let rock_material = materials.add(StandardMaterial {
        base_color_texture: Some(rock_tex),
        base_color: Color::WHITE,
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
