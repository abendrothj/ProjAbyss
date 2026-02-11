//! Marine character - first person, WASD, jump, mouse look.
//! Oxygen drains when swimming; respawn at Safe Island on drown.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::ui::{AlignItems, FlexDirection, JustifyContent};

use bevy_rapier3d::prelude::*;
use crate::game_state::GameState;
use crate::ocean::{OceanSolver, SEA_LEVEL};
use crate::player::{PlayerCamera, PlayerMode};
use crate::settings::{GameSettings, InputBindings};
use crate::world::{character_respawn_position, MAP_SCALE_FROM_LEGACY, SPAWN_ISLAND_X, SPAWN_ISLAND_Z};

/// Deck offset from ship center (character stands on ship).
const SHIP_DECK_OFFSET: Vec3 = Vec3::new(0.0, 0.5, 2.0);
const SHIP_ANCHOR_OFFSET: Vec3 = Vec3::new(3.0, 0.0, -2.0);

const SWIM_SPEED: f32 = 2.5;
const SWIM_ASCEND_SPEED: f32 = 4.0;
const SWIM_DESCEND_SPEED: f32 = 2.0;
const SWIM_DRAG: f32 = 4.0;
/// Stay in swim mode until this far above surface (lets player climb onto ship).
const SURFACE_EXIT_MARGIN: f32 = 0.6;

/// Depth (m) beyond which pressure increases oxygen drain. Sub required for sustained deep diving.
const PRESSURE_DEPTH_THRESHOLD: f32 = 50.0;
/// Oxygen drain multiplier when below pressure threshold (3x = ~20s at 50m+).
const PRESSURE_DRAIN_MULTIPLIER: f32 = 3.0;

#[derive(Component)]
pub struct MarineCharacter {
    pub walk_speed: f32,
    pub jump_velocity: f32,
}

#[derive(Component)]
pub struct CharacterVelocity(pub Vec3);

/// Oxygen when swimming. Refills at surface; drains underwater. Respawn on drown.
#[derive(Component)]
pub struct CharacterOxygen {
    pub max: f32,
    pub current: f32,
    pub drain_rate: f32,
    pub refill_rate: f32,
}

#[derive(Component)]
struct CharacterOxygenBarFill;

#[derive(Resource)]
struct CharacterOxygenUi {
    root: Entity,
    fill: Entity,
}

/// Horizontal (yaw) and vertical (pitch) look angles in radians.
#[derive(Component, Default)]
pub struct CharacterLook {
    pub yaw: f32,
    pub pitch: f32,
}

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_character, spawn_character_oxygen_ui))
            .add_systems(
                Update,
                (
                    character_mouse_look
                        .run_if(in_state(GameState::Playing))
                        .run_if(|mode: Res<PlayerMode>| !mode.in_vehicle()),
                    character_movement
                        .run_if(in_state(GameState::Playing))
                        .run_if(|mode: Res<PlayerMode>| !mode.in_vehicle()),
                    character_oxygen
                        .run_if(in_state(GameState::Playing))
                        .run_if(|mode: Res<PlayerMode>| !mode.in_vehicle()),
                    update_character_oxygen_ui.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

fn spawn_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        RigidBody::KinematicPositionBased,
        Collider::capsule_y(0.45, 0.4),
        KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            snap_to_ground: Some(CharacterLength::Absolute(0.2)),
            ..default()
        },
        Mesh3d(meshes.add(Capsule3d::new(0.4, 0.9))),
        MeshMaterial3d(materials.add(Color::srgb_u8(200, 150, 100))),
        Transform::from_xyz(
            SPAWN_ISLAND_X + SHIP_ANCHOR_OFFSET.x * MAP_SCALE_FROM_LEGACY + SHIP_DECK_OFFSET.x,
            SHIP_DECK_OFFSET.y,
            SPAWN_ISLAND_Z + SHIP_ANCHOR_OFFSET.z * MAP_SCALE_FROM_LEGACY + SHIP_DECK_OFFSET.z,
        ), // On ship deck (ship is child of character? No - character is separate)
        MarineCharacter {
            walk_speed: 4.0,
            jump_velocity: 6.0,
        },
        CharacterOxygen {
            max: 60.0,
            current: 60.0,
            drain_rate: 1.2,
            refill_rate: 25.0,
        },
        CharacterVelocity(Vec3::ZERO),
        CharacterLook::default(),
        children![(
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
        )],
    ));
}

fn spawn_character_oxygen_ui(mut commands: Commands) {
    let fill_id = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.2, 0.6, 0.95, 0.9)),
            CharacterOxygenBarFill,
        ))
        .id();
    let root_id = commands
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(24.0),
                position_type: bevy::ui::PositionType::Absolute,
                left: Val::Px(20.0),
                bottom: Val::Px(54.0), // Above sub oxygen bar if both shown; char bar when swimming
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.15, 0.25, 0.9)),
            Visibility::Hidden,
        ))
        .add_child(fill_id)
        .id();
    commands.insert_resource(CharacterOxygenUi { root: root_id, fill: fill_id });
}

fn character_oxygen(
    ocean: Res<OceanSolver>,
    mut query: Query<(
        &mut Transform,
        &mut CharacterOxygen,
        &mut CharacterVelocity,
    ), With<MarineCharacter>>,
    time: Res<Time>,
) {
    for (mut transform, mut oxygen, mut vel) in query.iter_mut() {
        let pos = transform.translation;
        let wave_height = ocean.wave_height_at(pos);
        let underwater = pos.y < wave_height + SURFACE_EXIT_MARGIN;

        if underwater {
            let depth = SEA_LEVEL - pos.y;
            let drain_mult = if depth > PRESSURE_DEPTH_THRESHOLD {
                PRESSURE_DRAIN_MULTIPLIER
            } else {
                1.0
            };
            oxygen.current = (oxygen.current
                - oxygen.drain_rate * drain_mult * time.delta_secs())
                .max(0.0);
            if oxygen.current <= 0.0 {
                // Respawn at Safe Island
                transform.translation = character_respawn_position();
                oxygen.current = oxygen.max;
                vel.0 = Vec3::ZERO;
            }
        } else {
            oxygen.current = (oxygen.current + oxygen.refill_rate * time.delta_secs()).min(oxygen.max);
        }
    }
}

fn update_character_oxygen_ui(
    mode: Res<PlayerMode>,
    oxygen_ui: Res<CharacterOxygenUi>,
    character_query: Query<(&Transform, &CharacterOxygen), With<MarineCharacter>>,
    ocean: Res<OceanSolver>,
    mut visibility_query: Query<&mut Visibility>,
    mut fill_query: Query<&mut Node, With<CharacterOxygenBarFill>>,
) {
    let mut root_vis = visibility_query.get_mut(oxygen_ui.root).unwrap();
    if mode.in_vehicle() {
        *root_vis = Visibility::Hidden;
        return;
    }
    let Ok((transform, oxygen)) = character_query.single() else {
        *root_vis = Visibility::Hidden;
        return;
    };
    let wave_height = ocean.wave_height_at(transform.translation);
    let underwater = transform.translation.y < wave_height + SURFACE_EXIT_MARGIN;
    if !underwater {
        *root_vis = Visibility::Hidden;
        return;
    }
    *root_vis = Visibility::Visible;
    let pct = (oxygen.current / oxygen.max).max(0.0).min(1.0);
    if let Ok(mut fill_node) = fill_query.get_mut(oxygen_ui.fill) {
        fill_node.width = Val::Percent(pct * 100.0);
    }
}

fn character_mouse_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    settings: Res<GameSettings>,
    mut query: Query<(&mut CharacterLook, &mut Transform), With<MarineCharacter>>,
) {
    let delta = mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    let sensitivity = settings.mouse_sensitivity;
    for (mut look, mut transform) in query.iter_mut() {
        look.yaw -= delta.x * sensitivity;
        look.pitch -= delta.y * sensitivity;
        look.pitch = look.pitch.clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch, 0.0);
    }
}

fn character_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    ocean: Res<OceanSolver>,
    mut query: Query<(
        &MarineCharacter,
        &mut CharacterVelocity,
        &Transform,
        &mut KinematicCharacterController,
    )>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (char, mut vel, transform, mut controller) in query.iter_mut() {
        let pos = transform.translation;
        let wave_height = ocean.wave_height_at(pos);
        let underwater = pos.y < wave_height + SURFACE_EXIT_MARGIN;

        if underwater {
            // Swimming: sink when idle, Space ascend, Shift descend.
            // Near surface: neutral buoyancy so player can tread water and climb onto ship.
            let near_surface = pos.y > wave_height - 0.3;
            let sink_rate = if near_surface { 0.3 } else { 1.8 };
            vel.0.y -= sink_rate * dt;
            vel.0.y *= 1.0 - SWIM_DRAG * dt;

            if keyboard.pressed(bindings.ascend) {
                vel.0.y += SWIM_ASCEND_SPEED * dt;
            }
            if keyboard.pressed(bindings.descend) {
                vel.0.y -= SWIM_DESCEND_SPEED * dt;
            }
            vel.0.y = vel.0.y.clamp(-SWIM_DESCEND_SPEED * 2.0, SWIM_ASCEND_SPEED * 2.0);

            let mut input = Vec3::ZERO;
            if keyboard.pressed(bindings.forward) {
                input.z -= 1.0;
            }
            if keyboard.pressed(bindings.back) {
                input.z += 1.0;
            }
            if keyboard.pressed(bindings.left) {
                input.x -= 1.0;
            }
            if keyboard.pressed(bindings.right) {
                input.x += 1.0;
            }

            if input.length_squared() > 0.0 {
                let dir = transform.rotation * input.normalize();
                vel.0.x = dir.x * SWIM_SPEED;
                vel.0.z = dir.z * SWIM_SPEED;
            } else {
                vel.0.x *= 1.0 - SWIM_DRAG * dt;
                vel.0.z *= 1.0 - SWIM_DRAG * dt;
            }
        } else {
            // Walking: gravity, jump, WASD
            vel.0.y -= 9.8 * dt;

            if keyboard.just_pressed(bindings.jump) {
                vel.0.y = char.jump_velocity;
            }

            let mut input = Vec3::ZERO;
            if keyboard.pressed(bindings.forward) {
                input.z -= 1.0;
            }
            if keyboard.pressed(bindings.back) {
                input.z += 1.0;
            }
            if keyboard.pressed(bindings.left) {
                input.x -= 1.0;
            }
            if keyboard.pressed(bindings.right) {
                input.x += 1.0;
            }

            if input.length_squared() > 0.0 {
                let dir = transform.rotation * input.normalize();
                vel.0.x = dir.x * char.walk_speed;
                vel.0.z = dir.z * char.walk_speed;
            } else {
                vel.0.x *= 0.9;
                vel.0.z *= 0.9;
            }
        }

        let mut delta = vel.0 * dt;

        // Floor clamp: don't go below seafloor
        const FLOOR_Y: f32 = -80.0;
        if pos.y + delta.y < FLOOR_Y {
            delta.y = FLOOR_Y - pos.y;
            vel.0.y = 0.0;
        }

        controller.translation = Some(delta);
    }
}
