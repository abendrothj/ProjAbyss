//! Artifacts – pickable objects for the extraction loop.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::interaction::{Interactable, InteractKind};
use crate::player::VEHICLE_ENTER_RANGE;
use crate::world::MAP_SCALE_FROM_LEGACY;

#[derive(Component)]
pub struct Artifact {
    pub item_id: String,
}

#[derive(Resource, Default)]
pub struct Inventory {
    pub items: Vec<String>,
}

pub struct ArtifactsPlugin;

impl Plugin for ArtifactsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Inventory::default())
            .add_systems(Startup, spawn_artifacts);
    }
}

fn spawn_artifacts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.5, 0.2),
        metallic: 0.6,
        perceptual_roughness: 0.4,
        ..default()
    });

    // Spawn a few artifacts on the seafloor
    let positions = [
        Vec3::new(100.0 * MAP_SCALE_FROM_LEGACY, -25.0, 80.0 * MAP_SCALE_FROM_LEGACY),
        Vec3::new(-120.0 * MAP_SCALE_FROM_LEGACY, -35.0, -90.0 * MAP_SCALE_FROM_LEGACY),
        Vec3::new(50.0 * MAP_SCALE_FROM_LEGACY, -15.0, 150.0 * MAP_SCALE_FROM_LEGACY),
    ];

    for (i, pos) in positions.iter().enumerate() {
        let item_id = format!("Artifact {}", i + 1);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.8))),
            MeshMaterial3d(mat.clone()),
            Transform::from_translation(*pos),
            RigidBody::Fixed,
            Collider::cuboid(0.25, 0.25, 0.4),
            Artifact {
                item_id: item_id.clone(),
            },
            Interactable {
                kind: InteractKind::Pickup {
                    item_id: item_id.clone(),
                },
                range: VEHICLE_ENTER_RANGE,
            },
        ));
    }
}
