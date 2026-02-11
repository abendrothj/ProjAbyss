//! Winch – RopeJoint tether between ship and submersible.
//!
//! The cable constrains the sub to stay within max distance of the ship.
//! R / T to reel in/out when in boat. Visual cable drawn between anchors.

use bevy::prelude::*;

use bevy_rapier3d::prelude::*;
use crate::artifacts::{Artifact, AttachedArtifact, Inventory};
use crate::audio::ArtifactPickupEvent;
use crate::diving_bell::Submersible;
use crate::game_state::GameState;
use crate::settings::InputBindings;
use crate::player::PlayerMode;
use crate::ship::Ship;

/// Max cable length (m). Sub cannot go further than this from the ship.
const MAX_CABLE_LENGTH: f32 = 100.0;

/// Min cable length (m). Reel won't go shorter.
const MIN_CABLE_LENGTH: f32 = 5.0;

/// Reel speed (m/sec).
const REEL_SPEED: f32 = 8.0;

/// Winch attachment on ship (local space): stern, above deck.
const SHIP_ANCHOR: Vec3 = Vec3::new(0.0, 0.6, 2.5);

/// Winch attachment on sub (local space): top center.
const SUB_ANCHOR: Vec3 = Vec3::new(0.0, 2.5, 0.0);

#[derive(Resource)]
pub struct WinchState {
    pub cable_length: f32,
}

/// Cable mesh entity – updated each frame to span ship anchor to sub anchor.
#[derive(Resource)]
struct CableVisual(Entity);

pub struct WinchPlugin;

impl Plugin for WinchPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WinchState {
            cable_length: MAX_CABLE_LENGTH,
        })
        .add_systems(Startup, (spawn_winch_joint, spawn_cable_visual))
        .add_systems(
            Update,
            (
                winch_controls
                    .run_if(in_state(GameState::Playing))
                    .run_if(|mode: Res<PlayerMode>| mode.in_boat),
                update_winch_joint_length.run_if(in_state(GameState::Playing)),
                update_cable_visual.run_if(in_state(GameState::Playing)),
                deliver_attached_artifact.run_if(in_state(GameState::Playing)),
            ),
        );
    }
}

fn spawn_winch_joint(
    mut commands: Commands,
    ship_query: Query<Entity, With<Ship>>,
    sub_query: Query<Entity, With<Submersible>>,
    winch: Res<WinchState>,
) {
    let Ok(ship_id) = ship_query.single() else { return };
    let Ok(sub_id) = sub_query.single() else { return };

    let rope = RopeJointBuilder::new(winch.cable_length)
        .local_anchor1(SHIP_ANCHOR)
        .local_anchor2(SUB_ANCHOR);
    let joint = ImpulseJoint::new(ship_id, rope);

    commands.entity(sub_id).insert(joint);
}

fn winch_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut winch: ResMut<WinchState>,
    time: Res<Time>,
) {
    let reel_in = keyboard.pressed(bindings.reel_in);
    let reel_out = keyboard.pressed(bindings.reel_out);
    let delta = time.delta_secs() * REEL_SPEED;

    if reel_in && !reel_out {
        winch.cable_length = (winch.cable_length - delta).max(MIN_CABLE_LENGTH);
    } else if reel_out && !reel_in {
        winch.cable_length = (winch.cable_length + delta).min(MAX_CABLE_LENGTH);
    }
}

fn spawn_cable_visual(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cable_mesh = meshes.add(
        Cylinder::new(0.08, 1.0)
            .mesh()
            .resolution(8),
    );
    let cable_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.3, 0.25, 0.2, 0.9),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        ..default()
    });
    let id = commands
        .spawn((
            Mesh3d(cable_mesh),
            MeshMaterial3d(cable_mat),
            Transform::default(),
        ))
        .id();
    commands.insert_resource(CableVisual(id));
}

fn update_cable_visual(
    cable: Res<CableVisual>,
    ship_query: Query<&Transform, With<Ship>>,
    sub_query: Query<&Transform, With<Submersible>>,
    mut transform_query: Query<&mut Transform>,
) {
    let Ok(ship_tf) = ship_query.single() else { return };
    let Ok(sub_tf) = sub_query.single() else { return };

    let from = ship_tf.translation + ship_tf.rotation * SHIP_ANCHOR;
    let to = sub_tf.translation + sub_tf.rotation * SUB_ANCHOR;
    let delta = to - from;
    let len = delta.length().max(0.1);

    let Ok(mut cable_tf) = transform_query.get_mut(cable.0) else { return };

    cable_tf.translation = (from + to) * 0.5;
    cable_tf.scale = Vec3::new(1.0, len * 0.5, 1.0);
    cable_tf.rotation = Quat::from_rotation_arc(Vec3::Y, delta.normalize());
}

/// When cable is reeled to min length with an artifact attached, deliver to inventory.
fn deliver_attached_artifact(
    winch: Res<WinchState>,
    mut attached: ResMut<AttachedArtifact>,
    mut inventory: ResMut<Inventory>,
    mut pickup_events: MessageWriter<ArtifactPickupEvent>,
    artifact_query: Query<&Artifact>,
    mut commands: Commands,
) {
    if winch.cable_length > MIN_CABLE_LENGTH + 0.5 {
        return;
    }
    let Some(art_id) = attached.0 else { return };
    let Ok(artifact) = artifact_query.get(art_id) else { return };
    inventory.items.push(artifact.item_id.clone());
    pickup_events.write(ArtifactPickupEvent);
    commands.entity(art_id).despawn();
    attached.0 = None;
}

/// Update the RopeJoint's max length when WinchState changes.
fn update_winch_joint_length(
    winch: Res<WinchState>,
    mut joint_query: Query<&mut ImpulseJoint, With<Submersible>>,
) {
    if !winch.is_changed() {
        return;
    }

    for mut joint in joint_query.iter_mut() {
        if let TypedJoint::RopeJoint(rope) = &mut joint.data {
            rope.set_max_distance(winch.cable_length);
        }
    }
}
