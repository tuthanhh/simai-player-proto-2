use bevy::prelude::*;
use bevy_kira_audio::AudioSource as KiraAudioSource;
#[derive(Resource, Default)]
pub struct SoundResources {
    // Define your sound resources here
    pub hit: Handle<KiraAudioSource>,
}
