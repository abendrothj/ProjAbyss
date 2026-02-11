//! World scale constants – single source of truth for map size.

/// Horizontal extent of the ocean (water & seafloor) in meters.
/// 5km × 5km for a big, explorable map.
pub const MAP_SIZE: f32 = 5000.0;

/// Seafloor depth (Y). Seafloor plane at MAP_FLOOR_Y.
pub const MAP_FLOOR_Y: f32 = -80.0;

/// Scale factor from legacy 1500m map to current map.
/// Use when converting old positions: new_pos = old_pos * MAP_SCALE_FROM_LEGACY.
pub const MAP_SCALE_FROM_LEGACY: f32 = MAP_SIZE / 1500.0;

/// Safe Island position (XZ). Ship and character spawn near here.
pub const SPAWN_ISLAND_X: f32 = -15.0 * MAP_SCALE_FROM_LEGACY;
pub const SPAWN_ISLAND_Z: f32 = 10.0 * MAP_SCALE_FROM_LEGACY;
