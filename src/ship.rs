//! Ship with buoyancy, engine. Rapier Dynamic + ExternalForce.

use bevy::gltf::GltfAssetLabel;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::scene::SceneRoot;

use bevy_rapier3d::prelude::*;
use crate::game_state::GameState;
use crate::interaction::{Interactable, InteractKind};
use crate::ocean::OceanSolver;
use crate::player::{PlayerMode, VEHICLE_ENTER_RANGE};
use crate::world::{MAP_SCALE_FROM_LEGACY, SPAWN_ISLAND_X, SPAWN_ISLAND_Z};

/// Ship anchored near Safe Island: offset from island center.
const SHIP_ANCHOR_OFFSET: Vec3 = Vec3::new(3.0, 0.0, -2.0);

/// Hull corners for buoyancy (rowboat ≈ 2.5 scale).
const PONTOON_OFFSETS: [Vec3; 4] = [
    Vec3::new(-0.9, -0.25, -1.5),
    Vec3::new(0.9, -0.25, -1.5),
    Vec3::new(-0.9, -0.25, 1.5),
    Vec3::new(0.9, -0.25, 1.5),
];

#[derive(Component)]
pub struct Ship {
    pub float_force: f32,
    pub water_drag: f32,
    pub engine_power: f32,
    pub turn_speed: f32,
    pub current_throttle: f32,
    pub current_steering: f32,
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ship)
            .add_systems(
                Update,
                (
                    ship_buoyancy.run_if(in_state(GameState::Playing)),
                    ship_input
                        .run_if(in_state(GameState::Playing))
                        .run_if(|mode: Res<PlayerMode>| mode.in_boat),
                    ship_mouse_look
                        .run_if(in_state(GameState::Playing))
                        .run_if(|mode: Res<PlayerMode>| mode.in_boat),
                    ship_movement.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

fn spawn_ship(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/boat-row-small.glb"));
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(2.5, 0.7, 4.0),
        ColliderMassProperties::Density(150.0),
        GravityScale(1.0),
        Damping {
            linear_damping: 2.0,
            angular_damping: 3.0,
        },
        ExternalForce::default(),
        SceneRoot(scene),
        Transform::from_xyz(
            SPAWN_ISLAND_X + SHIP_ANCHOR_OFFSET.x * MAP_SCALE_FROM_LEGACY,
            -0.5,
            SPAWN_ISLAND_Z + SHIP_ANCHOR_OFFSET.z * MAP_SCALE_FROM_LEGACY,
        ).with_scale(Vec3::splat(2.5)),
        Ship {
            float_force: 22000.0,
            water_drag: 2.0,
            engine_power: 22000.0,
            turn_speed: 3500.0,
            current_throttle: 0.0,
            current_steering: 0.0,
        },
        Interactable {
            kind: InteractKind::EnterShip,
            range: VEHICLE_ENTER_RANGE,
        },
    ));
}

fn ship_buoyancy(
    ocean: Res<OceanSolver>,
    mut query: Query<(&Transform, &Ship, &mut ExternalForce), With<Ship>>,
) {
    for (transform, ship, mut ext_force) in query.iter_mut() {
        let mut buoyancy = Vec3::ZERO;
        let mut pontoons_underwater = 0u32;

        for offset in PONTOON_OFFSETS {
            let pos = transform.translation + transform.rotation * offset;
            let wave_height = ocean.wave_height_at(pos);

            if pos.y < wave_height {
                pontoons_underwater += 1;
                let depth = wave_height - pos.y;
                buoyancy.y += depth * ship.float_force;
            }
        }

        ext_force.torque = Vec3::ZERO;
        if pontoons_underwater > 0 {
            ext_force.force = buoyancy;
        } else {
            ext_force.force = Vec3::ZERO;
        }
    }
}

fn ship_movement(
    ocean: Res<OceanSolver>,
    mut query: Query<(&Ship, &Transform, &mut ExternalForce), With<Ship>>,
    time: Res<Time>,
) {
    for (ship, transform, mut ext_force) in query.iter_mut() {
        let mut pontoons_underwater = 0u32;
        for offset in PONTOON_OFFSETS {
            let pos = transform.translation + transform.rotation * offset;
            let wave_height = ocean.wave_height_at(pos);
            if pos.y < wave_height {
                pontoons_underwater += 1;
            }
        }

        if pontoons_underwater > 0 {
            let forward = transform.forward();
            let thrust = forward * ship.engine_power * ship.current_throttle * time.delta_secs() * 0.001;
            let torque_y = ship.turn_speed * ship.current_steering * time.delta_secs() * 0.001;
            ext_force.force += thrust;
            ext_force.torque += Vec3::new(0.0, torque_y, 0.0);
        }
    }
}

fn ship_mouse_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    mut query: Query<&mut ExternalForce, With<Ship>>,
) {
    let delta_x = mouse_motion.delta.x;
    if delta_x == 0.0 {
        return;
    }
    const SENSITIVITY: f32 = 0.002;
    let dt = time.delta_secs().max(0.001);
    let ang = -delta_x * SENSITIVITY / dt;
    for mut ext_force in query.iter_mut() {
        ext_force.torque.y += ang * 5000.0;
    }
}

fn ship_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Ship, With<Ship>>,
) {
    for mut ship in query.iter_mut() {
        ship.current_throttle = if keyboard.pressed(KeyCode::KeyW) {
            1.0
        } else if keyboard.pressed(KeyCode::KeyS) {
            -1.0
        } else {
            0.0
        };
        ship.current_steering = if keyboard.pressed(KeyCode::KeyA) {
            1.0
        } else if keyboard.pressed(KeyCode::KeyD) {
            -1.0
        } else {
            0.0
        };
    }
}
