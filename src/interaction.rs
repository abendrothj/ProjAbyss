//! Generic interaction system. E to interact with nearby Interactables.
//!
//! Extensible for vehicle enter, pickup, winch use, etc.
//!
//! ## Usage
//!
//! 1. Add `Interactable { kind, range }` to an entity.
//! 2. Query `(Entity, &Transform, &Interactable)` where needed.
//! 3. Use `nearest_interactable_in_range()` for prompts and action dispatch.
//!
//! ## Extending
//!
//! Add variant to `InteractKind`, implement `prompt()`, add match arm in `toggle_boat_enter` (player.rs).
//!
//! See docs/FOUNDATION.md for full system documentation.

use bevy::prelude::*;

/// Marks an entity as interactable. Kind determines the action and prompt.
#[derive(Component, Clone)]
pub struct Interactable {
    pub kind: InteractKind,
    pub range: f32,
}

#[derive(Clone, PartialEq)]
pub enum InteractKind {
    EnterShip,
    EnterSubmersible,
    Pickup { item_id: String },
}

impl InteractKind {
    pub fn prompt(&self) -> String {
        match self {
            InteractKind::EnterShip => "Press E to enter ship".into(),
            InteractKind::EnterSubmersible => "Press E to enter submersible".into(),
            InteractKind::Pickup { item_id } => format!("Press E to pick up {}", item_id),
        }
    }
}

/// Find nearest interactable within range. Returns (entity, kind, distance_sq).
pub fn nearest_interactable_in_range<'a>(
    pos: Vec3,
    query: impl Iterator<Item = (Entity, &'a Transform, &'a Interactable)>,
) -> Option<(Entity, &'a InteractKind, f32)> {
    let mut nearest: Option<(Entity, &'a InteractKind, f32)> = None;
    for (entity, tf, interactable) in query {
        let d_sq = (tf.translation - pos).length_squared();
        let range_sq = interactable.range * interactable.range;
        if d_sq <= range_sq {
            if nearest.map(|(_, _, nd)| d_sq < nd).unwrap_or(true) {
                nearest = Some((entity, &interactable.kind, d_sq));
            }
        }
    }
    nearest
}

/// Find nearest interactable within hint_range but beyond interact range (for "move closer" prompt).
pub fn nearest_interactable_out_of_range<'a>(
    pos: Vec3,
    interact_range: f32,
    hint_range: f32,
    query: impl Iterator<Item = (Entity, &'a Transform, &'a Interactable)>,
) -> Option<f32> {
    let range_sq = interact_range * interact_range;
    let hint_sq = hint_range * hint_range;
    let mut nearest_d_sq = f32::MAX;
    for (_, tf, _) in query {
        let d_sq = (tf.translation - pos).length_squared();
        if d_sq > range_sq && d_sq < hint_sq {
            nearest_d_sq = nearest_d_sq.min(d_sq);
        }
    }
    if nearest_d_sq < f32::MAX {
        Some(nearest_d_sq)
    } else {
        None
    }
}
