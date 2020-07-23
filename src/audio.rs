use amethyst::{
    assets::Loader,
    audio::{OggFormat, SourceHandle},
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

/// Keys for the `sounds` HashMap.
pub mod sound_keys {
    pub const BEE_TAP_SOUND: &str = "audio/bee_tap.ogg";
}

pub struct Sounds {
    pub sounds: HashMap<String, SourceHandle>,
}

/// Loads an ogg audio track.
fn load_audio_track(loader: &Loader, world: &World, file: &str) -> SourceHandle {
    loader.load(file, OggFormat, (), &world.read_resource())
}

/// Initialise audio in the world.
pub fn initialise_audio(world: &mut World) {
    let sound_effects = {
        let loader = world.read_resource::<Loader>();

        let mut sounds = HashMap::new();

        sounds.insert(
            sound_keys::BEE_TAP_SOUND.to_string(),
            load_audio_track(&loader, &world, sound_keys::BEE_TAP_SOUND),
        );

        Sounds { sounds }
    };

    // Add sound effects to the world. We have to do this in another scope because
    // world won't let us insert new resources as long as `Loader` is borrowed.
    world.insert(sound_effects);
}

/// Play a sound based on its key. (Meant for systems to use, as this func takes Read<T>)
pub fn play_sound_system(
    key: &str,
    sounds: &Read<Sounds, PanicHandler>,
    storage: &Read<AssetStorage<Source>>,
    output: &Option<Read<Output>>,
) {
    play_score_sound(key, sounds, storage, output.as_ref().map(|o| o.deref()));
}

/// Play a sound based on its key. (Meant for systems)
pub fn play_score_sound(
    key: &str,
    sounds: &Sounds,
    storage: &AssetStorage<Source>,
    output: Option<&Output>,
) {
    if let Some(ref output) = output.as_ref() {
        if let Some(sound) = storage.get(&sounds.sounds.get(key).unwrap()) {
            output.play_once(sound, 1.0);
        }
    }
}
