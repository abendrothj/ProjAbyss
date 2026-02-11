//! Islands with varied shapes and collision.

use bevy::prelude::*;

/// Collision shape for islands. Used for ship and character collision.
#[derive(Component)]
pub struct IslandCollider {
    /// Radius for sphere collision (simplified; works for all shapes).
    pub radius: f32,
}
