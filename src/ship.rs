//! Ship with buoyancy, engine. Pure Rust, no physics plugin.

use bevy::prelude::*;

use crate::ocean::OceanSolver;

const PONTOON_OFFSETS: [Vec3; 4] = [
    Vec3::new(-1.5, -0.5, -2.0),
    Vec3::new(1.5, -0.5, -2.0),
    Vec3::new(-1.5, -0.5, 2.0),
    Vec3::new(1.5, -0.5, 2.0),
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

#[derive(Component)]
pub struct ShipVelocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ship)
            .add_systems(Update, (ship_buoyancy, ship_movement, ship_input));
    }
}

fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(8.0, 2.0, 15.0)),
            material: materials.add(Color::srgb_u8(100, 80, 60)),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        },
        Ship {
            float_force: 4000.0,
            water_drag: 2.0,
            engine_power: 5000.0,
            turn_speed: 2000.0,
            current_throttle: 0.0,
            current_steering: 0.0,
        },
        ShipVelocity {
            linear: Vec3::ZERO,
            angular: Vec3::ZERO,
        },
    ));
}

fn ship_buoyancy(
    ocean: Res<OceanSolver>,
    mut query: Query<(&Transform, &mut Ship, &mut ShipVelocity), With<Ship>>,
    time: Res<Time>,
) {
    for (transform, ship, mut vel) in query.iter_mut() {
        let mut pontoons_underwater = 0u32;

        for offset in PONTOON_OFFSETS {
            let pos = transform.translation + transform.rotation * offset;
            let wave_height = ocean.wave_height_at(pos);

            if pos.y < wave_height {
                pontoons_underwater += 1;
                let depth = wave_height - pos.y;
                vel.linear.y += depth * ship.float_force * time.delta_seconds() * 0.01;
            }
        }

        if pontoons_underwater > 0 {
            let drag = ship.water_drag * time.delta_seconds() * 0.1;
            vel.linear = vel.linear * (1.0 - drag);
        }
    }
}

fn ship_movement(
    mut query: Query<(&Ship, &mut ShipVelocity, &mut Transform), With<Ship>>,
    time: Res<Time>,
) {
    for (ship, mut vel, mut transform) in query.iter_mut() {
        let mut pontoons_underwater = 0u32;
        for offset in PONTOON_OFFSETS {
            let pos = transform.translation + transform.rotation * offset;
            if pos.y < 2.0 {
                pontoons_underwater += 1;
            }
        }

        if pontoons_underwater > 0 {
            let forward = -transform.forward();
            vel.linear += forward * ship.engine_power * ship.current_throttle * time.delta_seconds() * 0.001;
            vel.angular.y += ship.turn_speed * ship.current_steering * time.delta_seconds() * 0.001;
        }

        vel.linear.y -= 9.8 * time.delta_seconds() * 0.5;

        transform.translation += vel.linear * time.delta_seconds();
        transform.rotate_y(vel.angular.y * time.delta_seconds());

        vel.linear *= 0.99;
        vel.angular *= 0.98;
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
