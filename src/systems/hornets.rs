use amethyst::core::ecs::{
    Component, DenseVecStorage, Entities, LazyUpdate, Read, ReadExpect, ReaderId, System, Write,
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

use crate::audio::sound_keys::{BEE_TAP_SOUND, BUG_SPRAY_SOUND, FLY_SWAT_SOUND, HIVE_TRAP_SOUND};
use crate::audio::{play_sound_system, SoundsResource};
use crate::resources::abilities::{AbilitiesResource, AbilityType};
use crate::states::LevelComponent;
use crate::systems::ability_bar::RemoveItem;
use amethyst::input::{InputHandler, StringBindings};
use amethyst::prelude::Builder;
use amethyst::window::ScreenDimensions;
use amethyst::winit::MouseButton;
use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
};

pub const BEE_SPRITE_HEIGHT_AND_WIDTH: f32 = 40.0;

pub const SWATTER_HEIGHT_AND_WIDTH: f32 = 240.0;

pub const HIVE_HEIGHT_AND_WIDTH: f32 = 100.0;

#[derive(Default)]
pub struct Bee {
    /// The frame when the bee should be removed.
    pub expiration_frame: u64,
}
impl Component for Bee {
    type Storage = DenseVecStorage<Self>;
}
/// Calculates the distance between 2 points.
fn distance_between_points(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((y2 - y1) * (y2 - y1) + (x2 - x1) * (x2 - x1)).sqrt()
}

/// Create a UiTransform easily.
fn create_ui_transform(x_pos: f32, y_pos: f32, height_and_width: f32) -> UiTransform {
    UiTransform::new(
        (x_pos + y_pos).to_string(),
        Anchor::BottomLeft,
        Anchor::Middle,
        x_pos,
        y_pos,
        0.0,
        height_and_width,
        height_and_width,
    )
}

#[derive(SystemDesc)]
#[system_desc(name(HornetsSystemDesc))]
pub struct HornetsSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<UiEvent>,

    pub bee_texture: Option<SpriteRender>,

    pub swatter: Option<Entity>,

    pub hive: Option<Entity>,
}

impl HornetsSystem {
    pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
        Self {
            reader_id,
            bee_texture: None,
            swatter: None,
            hive: None,
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
        WriteStorage<'s, Bee>,
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
            mut bee_storage,
            mut ui_transform_storage,
            mut score,
            events,
            lazy,
            audio_storage,
            sounds,
            audio_output,
            mut abilities,
            input,
            dimensions,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();

        // All indexes in this ability will be removed from active_abilities
        let mut should_be_deactivated_abilities: Vec<usize> = Vec::new();

        // Handle abilities
        for (index, ability) in abilities.available_abilities.iter().enumerate() {
            // If that ability is active
            if abilities.active_abilities.contains(&index) {
                let mut mouse_pos = input.mouse_position().unwrap_or((0., 0.));

                // Mouse pos height is determined from top left instead of bottom left so we have to flip this.
                mouse_pos.1 = dimensions.height() - mouse_pos.1;

                match ability.info.ability_type {
                    AbilityType::FlySwatter => {
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
                            let (swatter_x, swatter_y) = {
                                let swatter_ui_transform =
                                    ui_transform_storage.get_mut(fly_swatter).unwrap();

                                *swatter_ui_transform = create_ui_transform(
                                    mouse_pos.0,
                                    mouse_pos.1,
                                    SWATTER_HEIGHT_AND_WIDTH,
                                );

                                (mouse_pos.0, mouse_pos.1)
                            };

                            if input.mouse_button_is_down(MouseButton::Left) {
                                // Can only use swatter once.
                                should_be_deactivated_abilities.push(index);
                                entities
                                    .delete(fly_swatter)
                                    .expect("Couldn't delete big swatter!");
                                self.swatter = None;

                                play_sound_system(
                                    FLY_SWAT_SOUND,
                                    &sounds,
                                    &audio_storage,
                                    &audio_output,
                                );

                                for (entity, _bee, bee_ui_transform) in
                                    (&entities, &bee_storage, &ui_transform_storage).join()
                                {
                                    if distance_between_points(
                                        swatter_x,
                                        swatter_y,
                                        bee_ui_transform.pixel_x(),
                                        bee_ui_transform.pixel_y(),
                                    ) <= SWATTER_HEIGHT_AND_WIDTH * 0.5
                                    {
                                        // Delete the bee
                                        entities.delete(entity).expect("Couldn't delete bee.");

                                        // Increase the score
                                        score.score += 1;
                                    }
                                }
                            }
                        } else {
                            let swatter_sprite = load_sprite_system(
                                &texture_storage,
                                &sheet_storage,
                                &loader,
                                "big_swatter.png",
                                0,
                            );

                            let ui_transform = create_ui_transform(
                                mouse_pos.0,
                                mouse_pos.1,
                                SWATTER_HEIGHT_AND_WIDTH,
                            );

                            self.swatter = Some(
                                lazy.create_entity(&entities)
                                    // Tag entity with LevelComponent so it gets deleted on close.
                                    .with(LevelComponent)
                                    .with(UiImage::Sprite(swatter_sprite))
                                    .with(ui_transform)
                                    .build(),
                            );
                        }
                    }
                    AbilityType::BugSpray => {
                        // Can only use swatter once.
                        should_be_deactivated_abilities.push(index);

                        play_sound_system(BUG_SPRAY_SOUND, &sounds, &audio_storage, &audio_output);

                        for (entity, _bee) in (&entities, &bee_storage).join() {
                            // Delete the bee
                            entities.delete(entity).expect("Couldn't delete bee.");

                            // Increase the score
                            score.score += 1;
                        }
                    }
                    AbilityType::HiveTrap => {
                        // If the ability is about to expire
                        if ability.current_state.percentage < 0.05 {
                            // Delete the hive.
                            if let Some(swatter) = self.swatter {
                                entities
                                    .delete(swatter)
                                    .expect("Couldn't delete hive trap!");

                                self.swatter = None;
                            }
                        } else if let Some(hive_trap) = self.hive {
                            let (hive_trap_x, hive_trap_y) = {
                                let hive_trap_transform =
                                    ui_transform_storage.get_mut(hive_trap).unwrap();

                                *hive_trap_transform = create_ui_transform(
                                    mouse_pos.0,
                                    mouse_pos.1,
                                    HIVE_HEIGHT_AND_WIDTH,
                                );

                                (mouse_pos.0, mouse_pos.1)
                            };

                            if input.mouse_button_is_down(MouseButton::Left) {
                                // Can only use hive once.
                                should_be_deactivated_abilities.push(index);
                                entities
                                    .delete(hive_trap)
                                    .expect("Couldn't delete hive trap!");
                                self.hive = None;

                                play_sound_system(
                                    HIVE_TRAP_SOUND,
                                    &sounds,
                                    &audio_storage,
                                    &audio_output,
                                );

                                for (bee, bee_ui_transform) in
                                    (&mut bee_storage, &mut ui_transform_storage).join()
                                {
                                    // If the bee is nearby
                                    if distance_between_points(
                                        hive_trap_x,
                                        hive_trap_y,
                                        bee_ui_transform.pixel_x(),
                                        bee_ui_transform.pixel_y(),
                                    ) <= 200.0
                                    {
                                        // Move the bee close to the hive
                                        *bee_ui_transform = create_ui_transform(
                                            hive_trap_x + rng.gen_range(-10., 10.),
                                            hive_trap_y + rng.gen_range(-10., 10.),
                                            BEE_SPRITE_HEIGHT_AND_WIDTH,
                                        );

                                        // Extend the bee's lifetime
                                        bee.expiration_frame += rng.gen_range(60, 120);
                                    }
                                }
                            }
                        } else {
                            let hive_trap = load_sprite_system(
                                &texture_storage,
                                &sheet_storage,
                                &loader,
                                "hive_trap.png",
                                0,
                            );

                            let ui_transform = create_ui_transform(
                                mouse_pos.0,
                                mouse_pos.1,
                                HIVE_HEIGHT_AND_WIDTH,
                            );

                            self.hive = Some(
                                lazy.create_entity(&entities)
                                    // Tag entity with LevelComponent so it gets deleted on close.
                                    .with(LevelComponent)
                                    .with(UiImage::Sprite(hive_trap))
                                    .with(ui_transform)
                                    .build(),
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        // Remove abilities that have been used
        for index in should_be_deactivated_abilities {
            abilities.available_abilities[index]
                .current_state
                .percentage = 0.0;
            abilities.active_abilities.remove_first_found_item(&index);
        }

        // Handle clicking on bees
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

        if let Some(bee_sprite) = &self.bee_texture {
            // Spawn new bees and delete old ones
            if every_n_seconds(0.5, &*time) {
                let bees_to_spawn = rng.gen_range(1, 6);

                let mut bees_left_to_spawn = bees_to_spawn;

                while bees_left_to_spawn != 0 {
                    let pos_x = rng.gen_range(10., 590.);
                    let pos_y = rng.gen_range(100., 500.);

                    lazy.create_entity(&entities)
                        .with(UiImage::Sprite(bee_sprite.clone()))
                        .with(create_ui_transform(
                            pos_x,
                            pos_y,
                            BEE_SPRITE_HEIGHT_AND_WIDTH,
                        ))
                        .with(LevelComponent)
                        .with(Bee {
                            expiration_frame: time.frame_number() + rng.gen_range(50, 180),
                        })
                        .build();

                    bees_left_to_spawn -= 1;
                }
            }

            for (entity, bee) in (&entities, &bee_storage).join() {
                if time.frame_number() >= bee.expiration_frame {
                    entities.delete(entity).expect("Couldn't delete bee!");
                }
            }
        } else {
            // Load bee texture
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
