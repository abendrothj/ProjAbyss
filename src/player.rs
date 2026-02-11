//! Player mode: on foot (character) vs in boat (ship). E to toggle.
//! UX: "Press E to enter" prompt when in range; "Press E to exit" when in vehicle.

use bevy::prelude::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::post_process::bloom::Bloom;
use bevy::render::view::{ColorGrading, Hdr};
use bevy::text::TextLayout;

use crate::artifacts::Inventory;
use crate::audio::ArtifactPickupEvent;
use crate::character::MarineCharacter;
use crate::game_state::GameState;
use crate::interaction::{
    nearest_interactable_in_range, nearest_interactable_out_of_range, Interactable, InteractKind,
};
use crate::ocean::SEA_LEVEL;
use crate::settings::InputBindings;

/// Distance (m) at which E can enter ship or sub.
pub const VEHICLE_ENTER_RANGE: f32 = 6.0;

#[derive(Resource, Default)]
pub struct PlayerMode {
    pub in_boat: bool,
    pub in_submersible: bool,
}

impl PlayerMode {
    pub fn in_vehicle(&self) -> bool {
        self.in_boat || self.in_submersible
    }
}

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
struct InteractPrompt;

#[derive(Resource)]
struct InteractPromptRoot(Entity);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerMode::default())
            .add_systems(Startup, spawn_interact_prompt)
            .add_systems(
                Update,
                (
                    toggle_boat_enter.run_if(in_state(GameState::Playing)),
                    update_interact_prompt.run_if(in_state(GameState::Playing)),
                    update_depth_color_grading,
                    update_depth_fog,
                ),
            );
    }
}


/// Depth (m) over which color grading transitions from surface to full deep-blue.
const DEPTH_COLOR_TRANSITION: f32 = 25.0;

fn update_depth_color_grading(
    mut camera_query: Query<(&mut ColorGrading, &GlobalTransform), With<PlayerCamera>>,
) {
    for (mut grading, global) in camera_query.iter_mut() {
        let y = global.translation().y;
        let depth = SEA_LEVEL - y;
        let depth_factor = if depth > 0.0 {
            let t = (depth / DEPTH_COLOR_TRANSITION).min(1.0);
            let smooth = t * t * (3.0 - 2.0 * t);
            (depth.min(80.0) / 80.0).powf(0.6) * smooth
        } else {
            0.0
        };
        grading.global.temperature = -0.6 * depth_factor;
        grading.global.post_saturation = 1.0 + 0.25 * depth_factor;
    }
}

fn update_depth_fog(
    mut camera_query: Query<(&mut bevy::pbr::DistanceFog, &GlobalTransform), With<PlayerCamera>>,
) {
    use bevy::pbr::FogFalloff;
    for (mut fog, global) in camera_query.iter_mut() {
        let y = global.translation().y;
        let depth = SEA_LEVEL - y;
        if depth > 0.0 {
            let d = depth.min(80.0);
            let density = 0.006 + 0.012 * (d / 80.0).powf(0.7);
            let t = (depth / DEPTH_COLOR_TRANSITION).min(1.0);
            let smooth = t * t * (3.0 - 2.0 * t);
            fog.color = Color::srgba(
                0.2 * smooth + 0.5 * (1.0 - smooth),
                0.35 * smooth + 0.6 * (1.0 - smooth),
                0.6 * smooth + 0.8 * (1.0 - smooth),
                0.35 * smooth + 0.2 * (1.0 - smooth),
            );
            fog.falloff = FogFalloff::Exponential { density };
        } else {
            fog.color = Color::srgba(0.5, 0.6, 0.8, 0.2);
            fog.falloff = FogFalloff::Exponential { density: 0.008 };
        }
    }
}

fn spawn_interact_prompt(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let text_id = commands
        .spawn((
            Text::new(""),
            TextFont { font, ..default() },
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.95)),
            TextLayout::default(),
        ))
        .id();
    let root_id = commands
        .spawn((
            Node {
                position_type: bevy::ui::PositionType::Absolute,
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(120.0),
                height: Val::Px(32.0),
                justify_content: bevy::ui::JustifyContent::Center,
                align_items: bevy::ui::AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.08, 0.12, 0.75)),
            InteractPrompt,
            Visibility::Hidden,
        ))
        .add_child(text_id)
        .id();
    commands.insert_resource(InteractPromptRoot(root_id));
}

fn update_interact_prompt(
    mode: Res<PlayerMode>,
    prompt: Res<InteractPromptRoot>,
    character_query: Query<&Transform, With<MarineCharacter>>,
    interactable_query: Query<(Entity, &Transform, &Interactable)>,
    mut visibility_query: Query<&mut Visibility>,
    mut text_query: Query<&mut Text>,
    children_query: Query<&Children>,
) {
    let root = prompt.0;
    let Ok(mut vis) = visibility_query.get_mut(root) else { return };
    let children = children_query.get(root).ok();
    let text_entity = children.and_then(|c| c.first().copied()).unwrap_or(root);

    // In vehicle: show "Press E to exit"
    if mode.in_vehicle() {
        *vis = Visibility::Visible;
        if let Ok(mut text) = text_query.get_mut(text_entity) {
            *text = Text::new("Press E to exit vehicle");
        }
        return;
    }

    // On foot: use generic Interactable system
    let Ok(char_tf) = character_query.single() else {
        *vis = Visibility::Hidden;
        return;
    };
    let char_pos = char_tf.translation;

    if let Some((_, kind, _)) =
        nearest_interactable_in_range(char_pos, interactable_query.iter())
    {
        *vis = Visibility::Visible;
        if let Ok(mut text) = text_query.get_mut(text_entity) {
            *text = Text::new(kind.prompt());
        }
    } else if let Some(_) = nearest_interactable_out_of_range(
        char_pos,
        VEHICLE_ENTER_RANGE,
        15.0,
        interactable_query.iter(),
    ) {
        *vis = Visibility::Visible;
        if let Ok(mut text) = text_query.get_mut(text_entity) {
            *text = Text::new(format!(
                "Move closer to enter ({:.0}m)",
                VEHICLE_ENTER_RANGE
            ));
        }
    } else {
        *vis = Visibility::Hidden;
    }
}

fn toggle_boat_enter(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut mode: ResMut<PlayerMode>,
    mut inventory: ResMut<Inventory>,
    mut pickup_events: MessageWriter<ArtifactPickupEvent>,
    mut commands: Commands,
    camera_query: Query<Entity, With<PlayerCamera>>,
    character_query: Query<(Entity, &Transform), With<MarineCharacter>>,
    interactable_query: Query<(Entity, &Transform, &Interactable)>,
) {
    if !keyboard.just_pressed(bindings.interact) {
        return;
    }

    let Some(cam_id) = camera_query.iter().next() else { return };
    let Some((char_id, char_tf)) = character_query.iter().next() else { return };
    let char_pos = char_tf.translation;

    // Exit vehicle -> character
    if mode.in_vehicle() {
        mode.in_boat = false;
        mode.in_submersible = false;
        commands.entity(cam_id).despawn();
        let cam_id = commands.spawn((
            Camera3d::default(),
            Hdr,
            Tonemapping::AgX,
            DepthPrepass,
            Bloom::NATURAL,
            ColorGrading::default(),
            bevy::pbr::DistanceFog {
                color: Color::srgba(0.5, 0.6, 0.8, 0.2),
                falloff: bevy::pbr::FogFalloff::Exponential { density: 0.008 },
                ..default()
            },
            PlayerCamera,
            Transform::from_xyz(0.0, 0.9, 0.0),
        )).id();
        commands.entity(char_id).add_children(&[cam_id]);
        return;
    }

    // On foot -> enter nearest interactable vehicle
    let camera_components = (
        Camera3d::default(),
        Hdr,
        Tonemapping::AgX,
        DepthPrepass,
        Bloom::NATURAL,
        ColorGrading::default(),
        bevy::pbr::DistanceFog {
            color: Color::srgba(0.5, 0.6, 0.8, 0.2),
            falloff: bevy::pbr::FogFalloff::Exponential { density: 0.008 },
            ..default()
        },
        PlayerCamera,
    );

    let nearest = nearest_interactable_in_range(char_pos, interactable_query.iter());

    commands.entity(cam_id).despawn();
    if let Some((target_id, kind, _)) = nearest {
        match kind {
            InteractKind::EnterShip => {
                mode.in_boat = true;
                let cam_id = commands.spawn((
                    camera_components,
                    Transform::from_xyz(0.0, 4.0, 12.0).looking_at(Vec3::new(0.0, 0.0, -5.0), Vec3::Y),
                )).id();
                commands.entity(target_id).add_children(&[cam_id]);
            }
            InteractKind::EnterSubmersible => {
                mode.in_submersible = true;
                let cam_id = commands.spawn((
                    camera_components,
                    Transform::from_xyz(0.0, 1.5, 8.0).looking_at(Vec3::new(0.0, 0.0, -6.0), Vec3::Y),
                )).id();
                commands.entity(target_id).add_children(&[cam_id]);
            }
            InteractKind::Pickup { item_id } => {
                inventory.items.push(item_id.clone());
                pickup_events.write(ArtifactPickupEvent);
                commands.entity(target_id).despawn();
                let cam_id = commands.spawn((camera_components, Transform::from_xyz(0.0, 0.9, 0.0))).id();
                commands.entity(char_id).add_children(&[cam_id]);
            }
            #[allow(unreachable_patterns)]
            _ => {
                // Unhandled kind: ensure camera stays on character
                let cam_id = commands.spawn((camera_components, Transform::from_xyz(0.0, 0.9, 0.0))).id();
                commands.entity(char_id).add_children(&[cam_id]);
            }
        }
    } else {
        let cam_id = commands.spawn((
            camera_components,
            Transform::from_xyz(0.0, 0.9, 0.0),
        )).id();
        commands.entity(char_id).add_children(&[cam_id]);
    }
}
