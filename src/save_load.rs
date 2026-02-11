//! Save/load game state. F5 save, F9 load.

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::character::MarineCharacter;
use crate::player::PlayerCamera;
use serde::{Deserialize, Serialize};

use crate::artifacts::Inventory;
use crate::diving_bell::Submersible;
use crate::settings::InputBindings;
use crate::game_state::GameState;
use crate::player::PlayerMode;
use crate::ship::Ship;
use crate::winch::WinchState;

const SAVE_PATH: &str = "save.ron";

#[derive(Resource, Default)]
struct LoadRequest(Option<SaveData>);

#[derive(Serialize, Deserialize, Default)]
pub struct SaveData {
    pub ship: EntitySave,
    pub sub: EntitySave,
    pub character: EntitySave,
    pub player_mode: PlayerModeSave,
    pub winch_cable_length: f32,
    #[serde(default)]
    pub inventory_items: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub struct EntitySave {
    pub translation: [f32; 3],
    pub rotation: [f32; 4],
    pub velocity: [f32; 3],
    pub angvel: [f32; 3],
}

#[derive(Serialize, Deserialize, Default)]
pub struct PlayerModeSave {
    pub in_boat: bool,
    pub in_submersible: bool,
}

impl From<Transform> for EntitySave {
    fn from(t: Transform) -> Self {
        let r = t.rotation;
        Self {
            translation: t.translation.to_array(),
            rotation: [r.x, r.y, r.z, r.w],
            velocity: [0.0; 3],
            angvel: [0.0; 3],
        }
    }
}

impl EntitySave {
    pub fn to_translation(&self) -> Vec3 {
        Vec3::from_slice(&self.translation)
    }
    pub fn to_rotation(&self) -> Quat {
        Quat::from_xyzw(self.rotation[0], self.rotation[1], self.rotation[2], self.rotation[3])
    }
}

fn save_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    ship_query: Query<(&Transform, &Velocity), With<Ship>>,
    sub_query: Query<(&Transform, &Velocity), With<Submersible>>,
    character_query: Query<&Transform, With<MarineCharacter>>,
    mode: Res<PlayerMode>,
    winch: Res<WinchState>,
    inventory: Res<Inventory>,
) {
    if !keyboard.just_pressed(bindings.save) {
        return;
    }

    let ship = ship_query
        .iter()
        .next()
        .map(|(t, v)| EntitySave {
            translation: t.translation.to_array(),
            rotation: [
                t.rotation.x,
                t.rotation.y,
                t.rotation.z,
                t.rotation.w,
            ],
            velocity: v.linvel.to_array(),
            angvel: v.angvel.to_array(),
        })
        .unwrap_or_default();

    let sub = sub_query
        .iter()
        .next()
        .map(|(t, v)| EntitySave {
            translation: t.translation.to_array(),
            rotation: [
                t.rotation.x,
                t.rotation.y,
                t.rotation.z,
                t.rotation.w,
            ],
            velocity: v.linvel.to_array(),
            angvel: v.angvel.to_array(),
        })
        .unwrap_or_default();

    let character = character_query
        .iter()
        .next()
        .map(|t| EntitySave {
            translation: t.translation.to_array(),
            rotation: [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
            velocity: [0.0; 3],
            angvel: [0.0; 3],
        })
        .unwrap_or_default();

    let data = SaveData {
        ship,
        sub,
        character,
        player_mode: PlayerModeSave {
            in_boat: mode.in_boat,
            in_submersible: mode.in_submersible,
        },
        winch_cable_length: winch.cable_length,
        inventory_items: inventory.items.clone(),
    };

    if let Ok(s) = ron::ser::to_string_pretty(&data, ron::ser::PrettyConfig::default()) {
        if std::fs::write(SAVE_PATH, s).is_ok() {
            bevy::log::info!("Saved to {}", SAVE_PATH);
        }
    }
}

fn load_request_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut load_request: ResMut<LoadRequest>,
) {
    if !keyboard.just_pressed(bindings.load) {
        return;
    }
    let Ok(s) = std::fs::read_to_string(SAVE_PATH) else {
        bevy::log::warn!("No save file found at {}", SAVE_PATH);
        return;
    };
    let Ok(data) = ron::from_str::<SaveData>(&s) else {
        bevy::log::warn!("Failed to parse save file");
        return;
    };
    load_request.0 = Some(data);
}

fn apply_load_system(world: &mut World) {
    let Some(data) = world.remove_resource::<LoadRequest>().and_then(|r| r.0) else {
        return;
    };

    let mut ship_query = world.query_filtered::<(&mut Transform, &mut Velocity), With<Ship>>();
    if let Some((mut tf, mut vel)) = ship_query.iter_mut(world).next() {
        tf.translation = data.ship.to_translation();
        tf.rotation = data.ship.to_rotation();
        vel.linvel = Vec3::from_slice(&data.ship.velocity);
        vel.angvel = Vec3::from_slice(&data.ship.angvel);
    }
    let mut sub_query = world.query_filtered::<(&mut Transform, &mut Velocity), With<Submersible>>();
    if let Some((mut tf, mut vel)) = sub_query.iter_mut(world).next() {
        tf.translation = data.sub.to_translation();
        tf.rotation = data.sub.to_rotation();
        vel.linvel = Vec3::from_slice(&data.sub.velocity);
        vel.angvel = Vec3::from_slice(&data.sub.angvel);
    }
    let mut character_query = world.query_filtered::<&mut Transform, With<MarineCharacter>>();
    if let Some(mut tf) = character_query.iter_mut(world).next() {
        tf.translation = data.character.to_translation();
        tf.rotation = data.character.to_rotation();
    }

    if let Some(mut mode) = world.get_resource_mut::<PlayerMode>() {
        mode.in_boat = data.player_mode.in_boat;
        mode.in_submersible = data.player_mode.in_submersible;
    }
    if let Some(mut winch) = world.get_resource_mut::<WinchState>() {
        winch.cable_length = data.winch_cable_length.clamp(5.0, 100.0);
    }
    if let Some(mut inventory) = world.get_resource_mut::<Inventory>() {
        inventory.items = data.inventory_items.clone();
    }

    let mut camera_query = world.query_filtered::<Entity, With<PlayerCamera>>();
    let mut character_entity_query = world.query_filtered::<Entity, With<MarineCharacter>>();
    let Some(cam_id) = camera_query.iter(world).next() else { return };
    let Some(char_id) = character_entity_query.iter(world).next() else { return };
    world.entity_mut(cam_id).despawn();
    let cam_id = world.spawn((
        Camera3d::default(),
        bevy::render::view::Hdr,
        bevy::core_pipeline::tonemapping::Tonemapping::AgX,
        bevy::core_pipeline::prepass::DepthPrepass,
        bevy::post_process::bloom::Bloom::NATURAL,
        bevy::render::view::ColorGrading::default(),
        bevy::pbr::DistanceFog {
            color: Color::srgba(0.5, 0.6, 0.8, 0.2),
            falloff: bevy::pbr::FogFalloff::Exponential { density: 0.008 },
            ..default()
        },
        PlayerCamera,
        Transform::from_xyz(0.0, 0.9, 0.0),
    )).id();
    world.entity_mut(char_id).add_children(&[cam_id]);
    if let Some(mut mode) = world.get_resource_mut::<PlayerMode>() {
        mode.in_boat = false;
        mode.in_submersible = false;
    }

    world.insert_resource(LoadRequest::default());
    bevy::log::info!("Loaded from {}", SAVE_PATH);
}

pub struct SaveLoadPlugin;

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoadRequest>()
            .add_systems(
                Update,
                load_request_system.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                save_system.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                apply_load_system.run_if(in_state(GameState::Playing)),
            );
    }
}
