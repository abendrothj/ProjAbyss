//! Gerstner wave ocean solver - CPU-side buoyancy.

use bevy::prelude::*;

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
            .add_systems(Update, update_ocean_time);
    }
}

fn update_ocean_time(time: Res<Time>, mut ocean: ResMut<OceanSolver>) {
    ocean.time = time.elapsed_seconds();
}
