//! Ship with buoyancy, engine. Pure Rust, no physics plugin.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use crate::islands::IslandCollider;
use crate::ocean::OceanSolver;
use crate::player::PlayerMode;

const SHIP_COLLISION_RADIUS: f32 = 10.0;

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
            .add_systems(
                Update,
                (
                    ship_buoyancy,
                    ship_movement,
                    ship_island_collision,
                    ship_input.run_if(|mode: Res<PlayerMode>| mode.in_boat),
                    ship_mouse_look.run_if(|mode: Res<PlayerMode>| mode.in_boat),
                ),
            );
    }
}

fn spawn_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(8.0, 2.0, 15.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.45, 0.35, 0.25),
            perceptual_roughness: 0.7,
            metallic: 0.05,
            ..default()
        })),
        Transform::from_xyz(0.0, 2.0, 0.0),
        Ship {
            float_force: 4000.0,
            water_drag: 2.0,
            engine_power: 22000.0,
            turn_speed: 3500.0,
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
                vel.linear.y += depth * ship.float_force * time.delta_secs() * 0.01;
            }
        }

        if pontoons_underwater > 0 {
            let drag = ship.water_drag * time.delta_secs() * 0.1;
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
            let forward = transform.forward();
            vel.linear += forward * ship.engine_power * ship.current_throttle * time.delta_secs() * 0.001;
            vel.angular.y += ship.turn_speed * ship.current_steering * time.delta_secs() * 0.001;
        }

        vel.linear.y -= 9.8 * time.delta_secs() * 0.5;

        transform.translation += vel.linear * time.delta_secs();
        transform.rotate_y(vel.angular.y * time.delta_secs());

        vel.linear *= 0.99;
        vel.angular *= 0.98;
    }
}

fn ship_mouse_look(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<&mut Transform, With<Ship>>,
) {
    let delta_x = mouse_motion.delta.x;
    if delta_x == 0.0 {
        return;
    }
    const SENSITIVITY: f32 = 0.002;
    for mut transform in query.iter_mut() {
        transform.rotate_y(-delta_x * SENSITIVITY);
    }
}

fn ship_island_collision(
    mut ship_query: Query<(&mut ShipVelocity, &mut Transform), With<Ship>>,
    island_query: Query<(&Transform, &IslandCollider), Without<Ship>>,
) {
    for (mut vel, mut ship_tf) in ship_query.iter_mut() {
        let ship_pos = ship_tf.translation;
        for (island_tf, collider) in island_query.iter() {
            let island_pos = island_tf.translation;
            let delta = ship_pos - island_pos;
            let dist_xz = delta.xz().length();
            let min_dist = SHIP_COLLISION_RADIUS + collider.radius;

            if dist_xz < min_dist {
                let push_dir = if dist_xz > 0.001 {
                    delta.xz().normalize().extend(0.0)
                } else {
                    ship_tf.forward().xz().normalize().extend(0.0)
                };
                let push_amount = min_dist - dist_xz;
                ship_tf.translation += push_dir * push_amount;
                ship_tf.translation.y = ship_pos.y;

                let into_island = vel.linear.x * push_dir.x + vel.linear.z * push_dir.z;
                if into_island < 0.0 {
                    vel.linear.x -= into_island * push_dir.x;
                    vel.linear.z -= into_island * push_dir.z;
                }
            }
        }
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
