//! Artifacts – pickable objects for the extraction loop.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::game_state::GameState;
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

/// Heavy artifact currently attached to winch (child of sub). Cleared on detach or deliver.
#[derive(Resource, Default)]
pub struct AttachedArtifact(pub Option<Entity>);

#[derive(Component)]
struct InventoryUiRoot;

#[derive(Resource)]
struct InventoryUiRootRes(Entity);

pub struct ArtifactsPlugin;

impl Plugin for ArtifactsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Inventory::default())
            .insert_resource(AttachedArtifact::default())
            .add_systems(Startup, (spawn_artifacts, spawn_inventory_ui))
            .add_systems(Update, update_inventory_ui.run_if(in_state(GameState::Playing)));
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

    // Heavy artifacts – attach to winch from submersible, reel in to deliver
    let heavy_positions = [
        Vec3::new(0.0 * MAP_SCALE_FROM_LEGACY, -40.0, 100.0 * MAP_SCALE_FROM_LEGACY),
        Vec3::new(-80.0 * MAP_SCALE_FROM_LEGACY, -55.0, -60.0 * MAP_SCALE_FROM_LEGACY),
    ];
    let heavy_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.35, 0.1),
        metallic: 0.7,
        perceptual_roughness: 0.5,
        ..default()
    });
    for (i, pos) in heavy_positions.iter().enumerate() {
        let item_id = format!("Heavy Artifact {}", i + 1);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.2))),
            MeshMaterial3d(heavy_mat.clone()),
            Transform::from_translation(*pos),
            RigidBody::Fixed,
            Collider::cuboid(0.5, 0.5, 0.6),
            Artifact {
                item_id: item_id.clone(),
            },
            Interactable {
                kind: InteractKind::AttachToWinch {
                    item_id: item_id.clone(),
                },
                range: VEHICLE_ENTER_RANGE,
            },
        ));
    }
}

fn spawn_inventory_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_id = commands
        .spawn((
            Text::new(""),
            TextFont { font, ..default() },
            TextColor(Color::srgba(0.9, 0.9, 0.95, 0.9)),
            TextLayout::default(),
        ))
        .id();
    let root_id = commands
        .spawn((
            Node {
                position_type: bevy::ui::PositionType::Absolute,
                right: Val::Px(20.0),
                bottom: Val::Px(120.0),
                flex_direction: bevy::ui::FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.75)),
            InventoryUiRoot,
        ))
        .add_child(text_id)
        .id();
    commands.insert_resource(InventoryUiRootRes(root_id));
}

fn update_inventory_ui(
    inventory: Res<Inventory>,
    ui_root: Res<InventoryUiRootRes>,
    mut text_query: Query<&mut Text>,
    children_query: Query<&Children>,
    mut visibility_query: Query<&mut Visibility, With<InventoryUiRoot>>,
) {
    let children = children_query.get(ui_root.0).ok();
    let text_entity = children.and_then(|c| c.first().copied()).unwrap_or(ui_root.0);
    let Ok(mut vis) = visibility_query.get_mut(ui_root.0) else { return };
    let Ok(mut text) = text_query.get_mut(text_entity) else { return };

    if inventory.items.is_empty() {
        *vis = Visibility::Hidden;
        return;
    }
    *vis = Visibility::Visible;
    let list: String = inventory
        .items
        .iter()
        .enumerate()
        .map(|(i, name)| format!("{}. {}", i + 1, name))
        .collect::<Vec<_>>()
        .join("\n");
    *text = Text::new(format!("Inventory ({})\n{}", inventory.items.len(), list));
}
