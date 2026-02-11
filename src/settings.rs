//! Input bindings and game settings. Extensible for settings UI.

use bevy::prelude::*;

/// Input key bindings. Use in systems instead of raw KeyCode.
#[derive(Resource)]
pub struct InputBindings {
    pub interact: KeyCode,
    pub save: KeyCode,
    pub load: KeyCode,
    pub forward: KeyCode,
    pub back: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub jump: KeyCode,
    pub ascend: KeyCode,
    pub descend: KeyCode,
    pub reel_in: KeyCode,
    pub reel_out: KeyCode,
    pub pause: KeyCode,
    pub menu_start: KeyCode,
}

impl Default for InputBindings {
    fn default() -> Self {
        Self {
            interact: KeyCode::KeyE,
            save: KeyCode::F5,
            load: KeyCode::F9,
            forward: KeyCode::KeyW,
            back: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            jump: KeyCode::Space,
            ascend: KeyCode::Space,
            descend: KeyCode::ShiftLeft,
            reel_in: KeyCode::KeyR,
            reel_out: KeyCode::KeyT,
            pause: KeyCode::Escape,
            menu_start: KeyCode::Enter,
        }
    }
}


/// Game settings. Persist to file for full impl.
#[derive(Resource)]
pub struct GameSettings {
    pub mouse_sensitivity: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.002,
        }
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputBindings::default())
            .insert_resource(GameSettings::default());
    }
}
