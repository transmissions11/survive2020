use amethyst::core::ecs::{
    Component, DenseVecStorage, Entities, LazyUpdate, Read, ReadExpect, ReadStorage, ReaderId,
    System, Write,
};

use crate::every_n_seconds;
use crate::resources::high_scores::CurrentLevelScoreResource;
use crate::systems::load_sprite_system;
use amethyst::assets::Loader;
use amethyst::core::ecs::shrev::EventChannel;
use amethyst::core::Time;
use amethyst::derive::SystemDesc;
use amethyst::ecs::prelude::*;
use amethyst::renderer::{SpriteRender, SpriteSheet, Texture};
use amethyst::ui::{Anchor, UiEvent, UiEventType, UiImage, UiTransform};
use rand::Rng;

use crate::audio::sound_keys::BEE_TAP_SOUND;
use crate::audio::{play_sound_system, SoundsResource};
use amethyst::prelude::Builder;
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
};

pub const BEE_SPRITE_HEIGHT_AND_WIDTH: f32 = 40.0;

#[derive(Default)]
pub struct Bee {
    /// The frame when the bee should be removed.
    pub expiration_frame: u64,
}
impl Component for Bee {
    type Storage = DenseVecStorage<Self>;
}

#[derive(SystemDesc)]
#[system_desc(name(HornetsSystemDesc))]
pub struct HornetsSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<UiEvent>,

    pub bee_texture: Option<SpriteRender>,
}

impl HornetsSystem {
    pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
        Self {
            reader_id,
            bee_texture: None,
        }
    }
}

impl<'s> System<'s> for HornetsSystem {
    type SystemData = (
        Read<'s, Time>,
        Entities<'s>,
        Read<'s, AssetStorage<Texture>>,
        Read<'s, AssetStorage<SpriteSheet>>,
        ReadExpect<'s, Loader>,
        ReadStorage<'s, Bee>,
        Write<'s, CurrentLevelScoreResource>,
        Read<'s, EventChannel<UiEvent>>,
        Read<'s, LazyUpdate>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, SoundsResource>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (
            time,
            entities,
            texture_storage,
            sheet_storage,
            loader,
            bee_storage,
            mut score,
            events,
            lazy,
            audio_storage,
            sounds,
            audio_output,
        ): Self::SystemData,
    ) {
        for ui_event in events.read(&mut self.reader_id) {
            if ui_event.event_type == UiEventType::Click {
                // If the UI target is a bee
                if bee_storage.get(ui_event.target).is_some() {
                    // Delete the bee
                    entities
                        .delete(ui_event.target)
                        .expect("Couldn't delete bee.");

                    // Play sound
                    play_sound_system(BEE_TAP_SOUND, &sounds, &audio_storage, &audio_output);

                    // Increase the score
                    score.score += 1;
                }
            }
        }

        let mut rng = rand::thread_rng();
        if let Some(bee_sprite) = &self.bee_texture {
            if every_n_seconds(0.5, &*time) {
                let pos_x = rng.gen_range(150., 450.);
                let pos_y = rng.gen_range(100., 500.);

                lazy.create_entity(&entities)
                    .with(UiImage::Sprite(bee_sprite.clone()))
                    .with(UiTransform::new(
                        pos_x.to_string(),
                        Anchor::BottomLeft,
                        Anchor::Middle,
                        pos_x,
                        pos_y,
                        0.,
                        BEE_SPRITE_HEIGHT_AND_WIDTH,
                        BEE_SPRITE_HEIGHT_AND_WIDTH,
                    ))
                    .with(Bee {
                        expiration_frame: time.frame_number() + rng.gen_range(50, 180),
                    })
                    .build();
            }

            for (entity, bee) in (&entities, &bee_storage).join() {
                if time.frame_number() >= bee.expiration_frame {
                    entities.delete(entity).expect("Couldn't delete bee!");
                }
            }
        } else {
            self.bee_texture = Some(load_sprite_system(
                texture_storage,
                sheet_storage,
                loader,
                "bee.png",
                0,
            ));
        }
    }
}
