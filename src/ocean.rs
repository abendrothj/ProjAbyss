//! Gerstner wave ocean solver - CPU-side buoyancy and water mesh.

use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::VertexAttributeValues;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

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

/// Sea level (Y). Water surface oscillates around this.
pub const SEA_LEVEL: f32 = -2.0;

/// Ocean solver resource - wave height at any position.
#[derive(Resource)]
pub struct OceanSolver {
    pub time: f32,
    pub waves: Vec<GerstnerWave>,
}

impl Default for OceanSolver {
    fn default() -> Self {
        let waves = vec![
            GerstnerWave::new(60.0, 0.6, 4.0, Vec2::new(1.0, 0.2), 0.4),
            GerstnerWave::new(35.0, 0.35, 2.5, Vec2::new(0.7, 0.7), 0.6),
            GerstnerWave::new(15.0, 0.2, 3.5, Vec2::new(0.2, 1.0), 0.8),
        ];
        Self { time: 0.0, waves }
    }
}

impl OceanSolver {
    /// Returns water surface height (Y) at world position.
    /// Bevy: Y is up, horizontal plane is XZ.
    pub fn wave_height_at(&self, pos: Vec3) -> f32 {
        let wave_offset = self.waves.iter().fold(0.0, |acc, wave| {
            let freq = wave.frequency();
            let phase = wave.phase_constant() * self.time;
            let projected = pos.x * wave.direction.x + pos.z * wave.direction.y;
            acc + (wave.amplitude * wave.steepness) * (freq * projected + phase).sin()
        });
        SEA_LEVEL + wave_offset
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

/// Grid size for wave mesh. Bevy Plane3d.subdivisions(n) gives (n+2)×(n+2) vertices.
/// subdivisions(64) → 66×66 = 4356 vertices.
const WAVE_GRID: usize = 66;

/// Tiling factor for water normal map (how many times it repeats across the ocean).
const WATER_NORMAL_TILES: f32 = 24.0;

/// Creates a procedural water normal map with fine ripples. Tangent-space; 0-255 RGBA.
fn create_water_normal_map(size: u32) -> Image {
    use std::f32::consts::TAU;
    let size = size as usize;
    let mut data = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let u = x as f32 / size as f32;
            let v = y as f32 / size as f32;
            // Multi-octave sine ripples for organic water surface
            let dx = 0.08 * (TAU * 4.0 * u).sin()
                + 0.05 * (TAU * 7.0 * v + 1.0).sin()
                + 0.03 * (TAU * 12.0 * (u + 0.3 * v)).sin();
            let dy = 0.06 * (TAU * 5.0 * v).sin()
                + 0.04 * (TAU * 9.0 * u + 0.5).sin()
                + 0.03 * (TAU * 11.0 * (v - 0.2 * u)).sin();
            let len = (1.0 + dx * dx + dy * dy).sqrt();
            let (nx, ny, nz) = (dx / len, dy / len, 1.0 / len);
            // Tangent-space normal to 0-255: R=(nx+1)/2, G=(ny+1)/2, B=(nz+1)/2
            data.push(((nx + 1.0) * 0.5 * 255.0) as u8);
            data.push(((ny + 1.0) * 0.5 * 255.0) as u8);
            data.push(((nz + 1.0) * 0.5 * 255.0) as u8);
            data.push(255);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor::default());
    image
}

fn spawn_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Use subdivided plane for wave deformation (64x64 for large map).
    let mut plane_mesh: Mesh = Plane3d::default()
        .mesh()
        .size(1500.0, 1500.0)
        .subdivisions(64)
        .build();
    plane_mesh.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;

    // Scale UVs so the normal map tiles across the ocean for fine ripples
    if let Some(VertexAttributeValues::Float32x2(uvs)) =
        plane_mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    {
        for uv in uvs.iter_mut() {
            uv[0] *= WATER_NORMAL_TILES;
            uv[1] *= WATER_NORMAL_TILES;
        }
    }

    let n = WAVE_GRID * WAVE_GRID;
    let water_tint = [0.2, 0.4, 0.6, 0.98];
    plane_mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        (0..n).map(|_| water_tint).collect::<Vec<[f32; 4]>>(),
    );
    let handle = meshes.add(plane_mesh);

    let water_normal = images.add(create_water_normal_map(128));
    let water_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.08, 0.25, 0.45, 0.95),
        perceptual_roughness: 0.02,
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.5,
        specular_transmission: 0.2,
        thickness: 2.0,
        normal_map_texture: Some(water_normal),
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
    let n = WAVE_GRID;
    let cell_size = 1500.0 / 65.0; // 65 cells between 66 vertices (subdivisions=64)
    let water_tint = [0.2, 0.4, 0.6, 0.98];
    let foam_tint = [1.0, 1.0, 1.0, 1.0];
    let steepness_lo = 0.012;
    let steepness_hi = 0.06;

    let mut heights = vec![0.0f32; n * n];
    {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        else {
            return;
        };
        for i in 0..n {
            for j in 0..n {
                let idx = i + j * n;
                let world_pos = Vec3::new(positions[idx][0], 0.0, positions[idx][2]);
                let h = ocean.wave_height_at(world_pos);
                positions[idx][1] = h;
                heights[idx] = h;
            }
        }
    }

    {
        let Some(VertexAttributeValues::Float32x4(colors)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
        else {
            return;
        };
        for i in 0..n {
            for j in 0..n {
                let idx = i + j * n;
                let h = heights[idx];
                let h_l = if i > 0 { heights[idx - 1] } else { h };
                let h_r = if i + 1 < n { heights[idx + 1] } else { h };
                let h_d = if j > 0 { heights[idx - n] } else { h };
                let h_u = if j + 1 < n { heights[idx + n] } else { h };
                let grad_x = (h_r - h_l) / (2.0 * cell_size);
                let grad_z = (h_u - h_d) / (2.0 * cell_size);
                let steepness = (grad_x * grad_x + grad_z * grad_z).sqrt();
                let foam = ((steepness - steepness_lo) / (steepness_hi - steepness_lo))
                    .clamp(0.0, 1.0);
                for c in 0..4 {
                    colors[idx][c] = water_tint[c] * (1.0 - foam) + foam_tint[c] * foam;
                }
            }
        }
    }
}
