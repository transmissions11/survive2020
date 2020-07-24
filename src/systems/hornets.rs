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
use crate::resources::abilities::{AbilitiesResource, AbilityType};
use crate::systems::ability_bar::AbilityBarComponent;
use amethyst::input::{InputHandler, StringBindings};
use amethyst::prelude::Builder;
use amethyst::window::ScreenDimensions;
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
};

pub const BEE_SPRITE_HEIGHT_AND_WIDTH: f32 = 40.0;

pub const SWATTER_HEIGHT_AND_WIDTH: f32 = 300.0;

#[derive(Default)]
pub struct Bee {
    /// The frame when the bee should be removed.
    pub expiration_frame: u64,
}
impl Component for Bee {
    type Storage = DenseVecStorage<Self>;
}

/// Create a UiTransform for the swatter entity.
fn create_swatter_ui_transform(x_pos: f32, y_pos: f32) -> UiTransform {
    UiTransform::new(
        "big_swatter".to_string(),
        Anchor::BottomLeft,
        Anchor::BottomRight,
        x_pos,
        y_pos,
        0.0,
        SWATTER_HEIGHT_AND_WIDTH,
        SWATTER_HEIGHT_AND_WIDTH,
    )
}

#[derive(SystemDesc)]
#[system_desc(name(HornetsSystemDesc))]
pub struct HornetsSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<UiEvent>,

    pub bee_texture: Option<SpriteRender>,

    pub swatter: Option<Entity>,
}

impl HornetsSystem {
    pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
        Self {
            reader_id,
            bee_texture: None,
            swatter: None,
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
        WriteStorage<'s, UiTransform>,
        Write<'s, CurrentLevelScoreResource>,
        Read<'s, EventChannel<UiEvent>>,
        Read<'s, LazyUpdate>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, SoundsResource>,
        Option<Read<'s, Output>>,
        Write<'s, AbilitiesResource>,
        Read<'s, InputHandler<StringBindings>>,
        ReadExpect<'s, ScreenDimensions>,
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
            mut ui_transform_storage,
            mut score,
            events,
            lazy,
            audio_storage,
            sounds,
            audio_output,
            abilities,
            input,
            dimensions,
        ): Self::SystemData,
    ) {
        for (index, ability) in abilities.available_abilities.iter().enumerate() {
            // If that ability is active
            if abilities.active_abilities.contains(&index) {
                match ability.info.ability_type {
                    AbilityType::FlySwatter => {
                        let mut mouse_pos = input.mouse_position().unwrap_or((0., 0.));

                        // Mouse pos height is determined from top left instead of bottom left so we have to flip this.
                        mouse_pos.1 = dimensions.height() - mouse_pos.1;

                        // If the ability is about to expire
                        if ability.current_state.percentage < 0.05 {
                            // Delete the swatter.
                            if let Some(swatter) = self.swatter {
                                entities
                                    .delete(swatter)
                                    .expect("Couldn't delete big swatter!");

                                self.swatter = None;
                            }
                        } else if let Some(fly_swatter) = self.swatter {
                            let ui_transform = ui_transform_storage.get_mut(fly_swatter).unwrap();

                            *ui_transform = create_swatter_ui_transform(mouse_pos.0, mouse_pos.1);
                        } else {
                            let swatter_sprite = load_sprite_system(
                                &texture_storage,
                                &sheet_storage,
                                &loader,
                                "big_swatter.png",
                                0,
                            );

                            let ui_transform =
                                create_swatter_ui_transform(mouse_pos.0, mouse_pos.1);

                            self.swatter = Some(
                                lazy.create_entity(&entities)
                                    // Tag entity with AbilityBarComponent so it gets deleted on close.
                                    .with(AbilityBarComponent)
                                    .with(UiImage::Sprite(swatter_sprite))
                                    .with(ui_transform)
                                    .build(),
                            );
                        }
                    }
                    AbilityType::BugSpray => {}
                    _ => {}
                }
            }
        }

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
                &texture_storage,
                &sheet_storage,
                &loader,
                "bee.png",
                0,
            ));
        }
    }
}
