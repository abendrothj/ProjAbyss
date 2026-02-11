//! Diving bell - heavy object with oxygen drain underwater.

use bevy::prelude::*;

#[derive(Component)]
pub struct DivingBell {
    pub max_oxygen: f32,
    pub current_oxygen: f32,
    pub oxygen_drain_rate: f32,
}

#[derive(Component)]
pub struct DivingBellVelocity(pub Vec3);

pub struct DivingBellPlugin;

impl Plugin for DivingBellPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_diving_bell)
            .add_systems(Update, diving_bell_oxygen);
    }
}

fn spawn_diving_bell(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cylinder::new(1.5, 3.0)),
            material: materials.add(Color::srgb_u8(80, 80, 90)),
            transform: Transform::from_xyz(0.0, -5.0, 0.0),
            ..default()
        },
        DivingBell {
            max_oxygen: 100.0,
            current_oxygen: 100.0,
            oxygen_drain_rate: 2.0,
        },
        DivingBellVelocity(Vec3::ZERO),
    ));
}

fn diving_bell_oxygen(
    mut query: Query<(&Transform, &mut DivingBell)>,
    time: Res<Time>,
) {
    for (transform, mut bell) in query.iter_mut() {
        if transform.translation.y < 0.0 {
            bell.current_oxygen = (bell.current_oxygen - bell.oxygen_drain_rate * time.delta_seconds()).max(0.0);
        }
    }
}
