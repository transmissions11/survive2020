use amethyst::{
    assets::Loader,
    audio::{AudioSink, OggFormat, SourceHandle},
    ecs::{World, WorldExt},
};

use amethyst::core::ecs::Read;

use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    shred::PanicHandler,
};
use std::collections::HashMap;
use std::ops::Deref;

use std::{iter::Cycle, vec::IntoIter};

/// Background music resource.
pub struct MusicResource {
    pub music: Cycle<IntoIter<SourceHandle>>,
}

/// Sound effects resource.
pub struct SoundsResource {
    pub sounds: HashMap<String, SourceHandle>,
}

/// Keys for the `sounds` HashMap.
pub mod sound_keys {
    pub const BEE_TAP_SOUND: &str = "audio/bee_tap.ogg";
}

pub const MUSIC_TRACKS: &[&str] = &[
    "audio/background_music_1.ogg",
    "audio/background_music_2.ogg",
];

/// Loads an ogg audio track.
fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

/// Initialise audio in the world.
pub fn initialise_audio(world: &mut World) {
    let (sound_effects, music) = {
        let loader = world.read_resource::<Loader>();

        let mut sounds = HashMap::new();

        sounds.insert(
            sound_keys::BEE_TAP_SOUND.to_string(),
            load_audio_track(&loader, &world, sound_keys::BEE_TAP_SOUND),
        );

        let mut sink = world.write_resource::<AudioSink>();
        // Music is a bit loud, reduce the volume.
        // This only affects background music.
        sink.set_volume(0.25);

        let music = MUSIC_TRACKS
            .iter()
            .map(|file| load_audio_track(&loader, &world, file))
            .collect::<Vec<_>>()
            .into_iter()
            .cycle();

        (SoundsResource { sounds }, MusicResource { music })
    };

    world.insert(sound_effects);
    world.insert(music);
}

/// Play a sound based on its key. (Meant for systems to use, as this func takes Read<T>)
pub fn play_sound_system(
    key: &str,
    sounds: &Read<SoundsResource, PanicHandler>,
    storage: &Read<AssetStorage<Source>>,
    output: &Option<Read<Output>>,
) {
    play_score_sound(key, sounds, storage, output.as_ref().map(|o| o.deref()));
}

/// Play a sound based on its key. (Meant for systems)
pub fn play_score_sound(
    key: &str,
    sounds: &SoundsResource,
    storage: &AssetStorage<Source>,
    output: Option<&Output>,
) {
    if let Some(ref output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.sounds.get(key).unwrap()) {
            output.play_once(sound, 1.0);
        }
    }
}
