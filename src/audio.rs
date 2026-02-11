//! Audio – playback on pickup. Add assets/audio/pickup.ogg to enable.

use bevy::prelude::*;

/// Message sent when player picks up an artifact. Audio system plays pickup sound.
#[derive(Message)]
pub struct ArtifactPickupEvent;

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ArtifactPickupEvent>()
            .add_systems(Update, play_pickup_sound);
    }
}

fn play_pickup_sound(
    mut events: MessageReader<ArtifactPickupEvent>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for _ in events.read() {
        // Load and play. Add assets/audio/pickup.ogg for actual sound.
        let source = asset_server.load("audio/pickup.ogg");
        commands.spawn((
            AudioPlayer::new(source),
            PlaybackSettings::DESPAWN,
        ));
    }
}
