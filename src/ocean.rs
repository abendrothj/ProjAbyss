//! Gerstner wave ocean solver - CPU-side buoyancy and water mesh.

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::VertexAttributeValues;

/// Resource storing the dynamic water mesh handle for per-frame vertex updates.
#[derive(Resource)]
pub struct WaterMeshHandle(pub Handle<Mesh>);

/// Wave parameters for one Gerstner layer.
#[derive(Clone)]
pub struct GerstnerWave {
    pub wavelength: f32,
    pub amplitude: f32,
    pub speed: f32,
    pub direction: Vec2,
    pub steepness: f32,
}

impl GerstnerWave {
    pub fn new(wavelength: f32, amplitude: f32, speed: f32, direction: Vec2, steepness: f32) -> Self {
        Self {
            wavelength,
            amplitude,
            speed,
            direction: direction.normalize(),
            steepness,
        }
    }

    fn frequency(&self) -> f32 {
        std::f32::consts::TAU / self.wavelength
    }

    fn phase_constant(&self) -> f32 {
        self.speed * self.frequency()
    }
}

/// Ocean solver resource - wave height at any position.
#[derive(Resource)]
pub struct OceanSolver {
    pub time: f32,
    pub waves: Vec<GerstnerWave>,
}

impl Default for OceanSolver {
    fn default() -> Self {
        let waves = vec![
            GerstnerWave::new(60.0, 1.5, 4.0, Vec2::new(1.0, 0.2), 0.4),
            GerstnerWave::new(35.0, 0.8, 2.5, Vec2::new(0.7, 0.7), 0.6),
            GerstnerWave::new(15.0, 0.4, 3.5, Vec2::new(0.2, 1.0), 0.8),
        ];
        Self { time: 0.0, waves }
    }
}

impl OceanSolver {
    /// Returns water surface height (Y) at world position.
    /// Bevy: Y is up, horizontal plane is XZ.
    pub fn wave_height_at(&self, pos: Vec3) -> f32 {
        self.waves.iter().fold(0.0, |acc, wave| {
            let freq = wave.frequency();
            let phase = wave.phase_constant() * self.time;
            let projected = pos.x * wave.direction.x + pos.z * wave.direction.y;
            acc + (wave.amplitude * wave.steepness) * (freq * projected + phase).sin()
        })
    }
}

pub struct OceanPlugin;

impl Plugin for OceanPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OceanSolver::default())
            .add_systems(Startup, spawn_water)
            .add_systems(Update, (update_ocean_time, update_water_mesh));
    }
}

fn spawn_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Use subdivided plane for wave deformation (64x64 for large map).
    let mut plane_mesh: Mesh = Plane3d::default()
        .mesh()
        .size(1500.0, 1500.0)
        .subdivisions(64)
        .build();
    plane_mesh.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    let handle = meshes.add(plane_mesh);

    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.15, 0.4, 0.65, 0.88),
        perceptual_roughness: 0.02,
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.5,
        specular_transmission: 0.15,
        ..default()
    });

    commands.insert_resource(WaterMeshHandle(handle.clone()));
    commands.spawn((
        Mesh3d(handle),
        MeshMaterial3d(water_material),
        Transform::default(),
    ));
}

fn update_ocean_time(time: Res<Time>, mut ocean: ResMut<OceanSolver>) {
    ocean.time = time.elapsed_secs();
}

fn update_water_mesh(
    ocean: Res<OceanSolver>,
    handle: Res<WaterMeshHandle>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(mesh) = meshes.get_mut(&handle.0) else { return };
    let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };
    for pos in positions.iter_mut() {
        let world_pos = Vec3::new(pos[0], 0.0, pos[2]);
        pos[1] = ocean.wave_height_at(world_pos);
    }
}
