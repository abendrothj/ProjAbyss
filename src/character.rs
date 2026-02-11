//! Marine character - first person, WASD, jump.

use bevy::prelude::*;

#[derive(Component)]
pub struct MarineCharacter {
    pub walk_speed: f32,
    pub jump_velocity: f32,
}

#[derive(Component)]
pub struct CharacterVelocity(pub Vec3);

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_character)
            .add_systems(Update, character_movement);
    }
}

fn spawn_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.4, 0.9)),
            material: materials.add(Color::srgb_u8(200, 150, 100)),
            transform: Transform::from_xyz(3.0, 1.0, 5.0),
            ..default()
        },
        MarineCharacter {
            walk_speed: 4.0,
            jump_velocity: 6.0,
        },
        CharacterVelocity(Vec3::ZERO),
    ));
}

fn character_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&MarineCharacter, &mut CharacterVelocity, &mut Transform)>,
    time: Res<Time>,
) {
    for (char, mut vel, mut transform) in query.iter_mut() {
        vel.0.y -= 9.8 * time.delta_seconds();

        if keyboard.just_pressed(KeyCode::Space) {
            vel.0.y = char.jump_velocity;
        }

        let mut input = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            input.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            input.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            input.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
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

        transform.translation += vel.0 * time.delta_seconds();

        if transform.translation.y < 0.9 {
            transform.translation.y = 0.9;
            vel.0.y = 0.0;
        }
    }
}
