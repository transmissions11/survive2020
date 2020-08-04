use crate::audio::{play_sound_system, SoundsResource};
use crate::resources::abilities::{AbilitiesResource, AbilityType};
use crate::states::covid::CovidStateResource;

use crate::states::{LevelComponent, LevelSecondsResource};

use crate::systems::{distance_between_points, load_sprite_system};
use crate::{bound, bound_transform_x_prepend, bound_transform_y_prepend, every_n_seconds};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::audio::output::Output;
use amethyst::audio::Source;
use amethyst::core::ecs::{
    Component, DenseVecStorage, Entities, Entity, Join, LazyUpdate, Read, ReadExpect, ReadStorage,
    Write, WriteStorage,
};
use amethyst::core::{Time, Transform};
use amethyst::input::{InputHandler, StringBindings};
use amethyst::prelude::Builder;
use amethyst::renderer::rendy::wsi::winit::VirtualKeyCode;
use amethyst::renderer::{SpriteRender, SpriteSheet, Texture, Transparent};
use amethyst::window::ScreenDimensions;

use crate::audio::sound_keys::{BUCKET_SOUND, FIRE_OUT_SOUND, FIRE_SOUND};

use crate::systems::ability_bar::RemoveItem;
use amethyst::{
    derive::SystemDesc,
    ecs::prelude::{System, SystemData},
};
use rand::Rng;

pub const MOVEMENT_SPEED: f32 = 140.0;
pub const ROTATION_SPEED: f32 = 4.2;

pub const PLAYER_HEIGHT_AND_WIDTH: f32 = 80.0;

pub const SPREADER_HEIGHT_AND_WIDTH: f32 = 170.0;

pub const COVID_SPEED: f32 = 40.0;
pub const COVID_HEIGHT_AND_WIDTH: f32 = 40.0;

pub const HEALTH_PACK_HEIGHT_AND_WIDTH: f32 = 40.0;

/// Tags an entity as a super spreader.
pub struct SuperSpreaderComponent {
    pub expiration_frame: u64,
}
impl Component for SuperSpreaderComponent {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Copy, Clone)]
pub enum CovidDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Tags an entity as a covid cell.
pub struct CovidCellComponent {
    pub direction: CovidDirection,
}

impl Component for CovidCellComponent {
    type Storage = DenseVecStorage<Self>;
}

/// Tags an entity as a health pack;
pub struct HealthPackComponent;
impl Component for HealthPackComponent {
    type Storage = DenseVecStorage<Self>;
}

#[derive(SystemDesc, Default)]
pub struct CovidSystem {
    pub player_entity: Option<Entity>,
    pub masked_player_sprite: Option<SpriteRender>,
    pub player_sprite: Option<SpriteRender>,
    pub covid_sprite: Option<SpriteRender>,
    pub spreader_sprite: Option<SpriteRender>,
    pub health_pack_sprite: Option<SpriteRender>,
}

impl<'s> System<'s> for CovidSystem {
    type SystemData = (
        Entities<'s>,
        Write<'s, CovidStateResource>,
        Read<'s, LevelSecondsResource>,
        Read<'s, Time>,
        Read<'s, LazyUpdate>,
        Read<'s, AssetStorage<Texture>>,
        Read<'s, AssetStorage<SpriteSheet>>,
        ReadExpect<'s, Loader>,
        ReadExpect<'s, ScreenDimensions>,
        WriteStorage<'s, Transform>,
        ReadStorage<'s, SuperSpreaderComponent>,
        ReadStorage<'s, CovidCellComponent>,
        ReadStorage<'s, HealthPackComponent>,
        WriteStorage<'s, SpriteRender>,
        Read<'s, InputHandler<StringBindings>>,
        Write<'s, AbilitiesResource>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, SoundsResource>,
        Option<Read<'s, Output>>,
    );

    fn run(
        &mut self,
        (
            entities,
            mut level_state,
            level_seconds,
            time,
            lazy,
            texture_storage,
            sheet_storage,
            loader,
            dimensions,
            mut transform_storage,
            spreader_storage,
            covid_storage,
            health_pack_storage,
            mut sprite_render_storage,
            input,
            mut abilities,
            audio_storage,
            sounds,
            audio_output,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();

        if let Some(player_entity) = &self.player_entity {
            let (player_x, player_y, player_z) = {
                let player_transform = transform_storage.get(*player_entity).unwrap();

                (
                    player_transform.translation().x,
                    player_transform.translation().y,
                    player_transform.translation().z,
                )
            };

            // All indexes in this ability will be removed from active_abilities
            let mut should_be_deactivated_abilities: Vec<usize> = Vec::new();

            let mut mask_is_active = false;

            // Handle abilities
            for (index, ability) in abilities.available_abilities.iter().enumerate() {
                // If that ability is active
                if abilities.active_abilities.contains(&index) {
                    match ability.info.ability_type {
                        AbilityType::Mask => {
                            // If the ability is about to expire
                            if ability.current_state.percentage < 0.05 {
                                // Get the player sprite
                                if let Some(player_sprite) = &self.player_sprite {
                                    let current_player_sprite =
                                        sprite_render_storage.get_mut(*player_entity).unwrap();

                                    // If the player's sprite is not already set to default
                                    if current_player_sprite.sprite_sheet.id()
                                        != player_sprite.sprite_sheet.id()
                                    {
                                        // Set the player sprite to a masked version
                                        *current_player_sprite = player_sprite.clone();
                                    }
                                } else {
                                    self.player_sprite = Some(load_sprite_system(
                                        &texture_storage,
                                        &sheet_storage,
                                        &loader,
                                        "covid_player.png",
                                        0,
                                    ));
                                }
                            }
                            // If the ability has just been cast
                            else if ability.current_state.percentage > 0.99 {
                                // Get the masked player sprite
                                if let Some(masked_player_sprite) = &self.masked_player_sprite {
                                    let current_player_sprite =
                                        sprite_render_storage.get_mut(*player_entity).unwrap();

                                    // If the player's sprite is not already set to mask
                                    if current_player_sprite.sprite_sheet.id()
                                        != masked_player_sprite.sprite_sheet.id()
                                    {
                                        // Set the player sprite to a masked version
                                        *current_player_sprite = masked_player_sprite.clone();
                                    }
                                } else {
                                    self.masked_player_sprite = Some(load_sprite_system(
                                        &texture_storage,
                                        &sheet_storage,
                                        &loader,
                                        "masked_covid_player.png",
                                        0,
                                    ));
                                }
                            } else {
                                mask_is_active = true;
                            }
                        }

                        AbilityType::Bucket => {}

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

            // Health pack collisions
            {
                for (_, health_pack_transform, entity) in
                    (&health_pack_storage, &transform_storage, &entities).join()
                {
                    // Health pack collisions
                    if !mask_is_active
                        && distance_between_points(
                            player_x,
                            player_y,
                            health_pack_transform.translation().x,
                            health_pack_transform.translation().y,
                        ) <= (0.5 * COVID_HEIGHT_AND_WIDTH) + (0.5 * PLAYER_HEIGHT_AND_WIDTH)
                    {
                        entities.delete(entity).expect("Couldn't delete covid!");
                        level_state.current_health =
                            bound(level_state.current_health as f32 + 10., 0., 100.) as u64;

                        // TODO CHANGE
                        play_sound_system(BUCKET_SOUND, &sounds, &audio_storage, &audio_output);
                    }
                }
            }
            // Health pack spawning
            {
                if let Some(health_pack_sprite) = &self.health_pack_sprite {
                    if every_n_seconds(10.0, &*time) {
                        let pos_x = rng.gen_range(10., 590.);
                        let pos_y = rng.gen_range(100., 500.);

                        // Don't spawn health packs on or really close to player
                        if distance_between_points(pos_x, pos_y, player_x, player_y)
                            <= ((PLAYER_HEIGHT_AND_WIDTH * 0.5)
                                + (HEALTH_PACK_HEIGHT_AND_WIDTH * 0.5))
                        {
                            return;
                        }

                        let mut transform = Transform::default();

                        transform.set_translation_xyz(pos_x, pos_y, 2.0);

                        lazy.create_entity(&entities)
                            .with(health_pack_sprite.clone())
                            .with(transform)
                            .with(Transparent)
                            .with(LevelComponent)
                            .with(HealthPackComponent)
                            .build();
                    }
                } else {
                    // Load health pack texture
                    self.health_pack_sprite = Some(load_sprite_system(
                        &texture_storage,
                        &sheet_storage,
                        &loader,
                        "health_pack.png",
                        0,
                    ));
                }
            }

            // Covid collisions
            {
                for (_, covid_transform, entity) in
                    (&covid_storage, &transform_storage, &entities).join()
                {
                    // Covid collisions
                    if !mask_is_active
                        && distance_between_points(
                            player_x,
                            player_y,
                            covid_transform.translation().x,
                            covid_transform.translation().y,
                        ) <= (0.5 * COVID_HEIGHT_AND_WIDTH) + (0.5 * PLAYER_HEIGHT_AND_WIDTH)
                    {
                        entities.delete(entity).expect("Couldn't delete covid!");
                        level_state.current_health = level_state.current_health.saturating_sub(10);

                        // TODO CHANGE
                        play_sound_system(FIRE_OUT_SOUND, &sounds, &audio_storage, &audio_output);
                    }
                }
            }
            // Covid movement
            {
                for (covid, covid_transform, entity) in
                    (&covid_storage, &mut transform_storage, &entities).join()
                {
                    match covid.direction {
                        CovidDirection::Up => {
                            covid_transform.move_up(COVID_SPEED * time.delta_seconds());
                        }
                        CovidDirection::Down => {
                            covid_transform.move_down(COVID_SPEED * time.delta_seconds());
                        }
                        CovidDirection::Left => {
                            covid_transform.move_left(COVID_SPEED * time.delta_seconds());
                        }
                        CovidDirection::Right => {
                            covid_transform.move_right(COVID_SPEED * time.delta_seconds());
                        }
                    }

                    if covid_transform.translation().x >= dimensions.width()
                        || covid_transform.translation().y >= dimensions.height()
                    {
                        entities
                            .delete(entity)
                            .expect("Couldn't delete covid cell.")
                    }
                }
            }
            // Covid spawning
            {
                if let Some(covid_sprite) = &self.covid_sprite {
                    if every_n_seconds(1.0, &*time) {
                        let spawn_locations = vec![
                            ((rng.gen_range(10., 590.), 120.), CovidDirection::Up),
                            ((rng.gen_range(10., 590.), 480.), CovidDirection::Down),
                            ((30., rng.gen_range(100., 500.)), CovidDirection::Right),
                            ((570., rng.gen_range(100., 500.)), CovidDirection::Left),
                        ];

                        let chosen_location =
                            &spawn_locations[rng.gen_range(0, spawn_locations.len())];

                        let pos_x = (chosen_location.0).0;
                        let pos_y = (chosen_location.0).1;

                        // Don't spawn covid on or really close to player
                        if distance_between_points(pos_x, pos_y, player_x, player_y)
                            <= ((PLAYER_HEIGHT_AND_WIDTH * 0.5) + (COVID_HEIGHT_AND_WIDTH * 0.5))
                        {
                            return;
                        }

                        let mut transform = Transform::default();

                        transform.set_translation_xyz(pos_x, pos_y, 2.0);

                        lazy.create_entity(&entities)
                            .with(covid_sprite.clone())
                            .with(transform)
                            .with(Transparent)
                            .with(LevelComponent)
                            .with(CovidCellComponent {
                                direction: chosen_location.1,
                            })
                            .build();
                    }
                } else {
                    // Load covid texture
                    self.covid_sprite = Some(load_sprite_system(
                        &texture_storage,
                        &sheet_storage,
                        &loader,
                        "covid.png",
                        0,
                    ));
                }
            }

            // Super spreader collisions and deletion
            {
                for (spreader, spreader_transform, entity) in
                    (&spreader_storage, &transform_storage, &entities).join()
                {
                    // Delete stale spreaders
                    if time.frame_number() >= spreader.expiration_frame {
                        entities.delete(entity).expect("Could not delete spreader!");
                    }

                    // Spreader collisions
                    if !mask_is_active
                        && distance_between_points(
                            player_x,
                            player_y,
                            spreader_transform.translation().x,
                            spreader_transform.translation().y,
                        ) <= (0.5 * SPREADER_HEIGHT_AND_WIDTH) + (0.5 * PLAYER_HEIGHT_AND_WIDTH)
                    {
                        entities.delete(entity).expect("Couldn't delete spreader!");
                        level_state.current_health = level_state.current_health.saturating_sub(10);

                        // TODO CHANGE
                        play_sound_system(FIRE_SOUND, &sounds, &audio_storage, &audio_output);
                    }
                }
            }
            // Super spreader spawning
            {
                if let Some(spreader_sprite) = &self.spreader_sprite {
                    if every_n_seconds(2., &*time) {
                        // Spreaders to spawn is from 1 to (2 + however many chunks of 40 seconds have gone by).
                        let spreaders_to_spawn =
                            rng.gen_range(1, 2 + (level_seconds.seconds_elapsed / 40.) as u32);

                        let mut spreaders_left_to_spawn = spreaders_to_spawn;

                        while spreaders_left_to_spawn != 0 {
                            let pos_x = rng.gen_range(10., 590.);
                            let pos_y = rng.gen_range(100., 500.);

                            // Don't spawn spreaders on or really close to player
                            if distance_between_points(pos_x, pos_y, player_x, player_y)
                                <= ((PLAYER_HEIGHT_AND_WIDTH * 0.5)
                                    + (SPREADER_HEIGHT_AND_WIDTH * 0.5))
                            {
                                continue;
                            }

                            let mut transform = Transform::default();

                            transform.set_translation_xyz(pos_x, pos_y, 0.0);

                            lazy.create_entity(&entities)
                                .with(spreader_sprite.clone())
                                .with(transform)
                                .with(Transparent)
                                .with(LevelComponent)
                                .with(SuperSpreaderComponent {
                                    expiration_frame: time.frame_number() + rng.gen_range(60, 640),
                                })
                                .build();

                            spreaders_left_to_spawn -= 1;
                        }
                    }
                } else {
                    // Load spreader texture
                    self.spreader_sprite = Some(load_sprite_system(
                        &texture_storage,
                        &sheet_storage,
                        &loader,
                        "super_spreader.png",
                        0,
                    ));
                }
            }

            // Movement and shooting
            {
                let player_transform_mut = transform_storage.get_mut(*player_entity).unwrap();

                let min_height_and_width = PLAYER_HEIGHT_AND_WIDTH * 0.5;

                let max_height = dimensions.height() - PLAYER_HEIGHT_AND_WIDTH * 0.5;

                let max_width = dimensions.width() - PLAYER_HEIGHT_AND_WIDTH * 0.5;

                // Movement keys
                {
                    if input.key_is_down(VirtualKeyCode::W) {
                        bound_transform_y_prepend(
                            player_transform_mut,
                            MOVEMENT_SPEED * time.delta_seconds(),
                            min_height_and_width,
                            max_height,
                        );
                    }
                    if input.key_is_down(VirtualKeyCode::S) {
                        bound_transform_y_prepend(
                            player_transform_mut,
                            -MOVEMENT_SPEED * time.delta_seconds(),
                            min_height_and_width,
                            max_height,
                        );
                    }

                    if input.key_is_down(VirtualKeyCode::A) {
                        bound_transform_x_prepend(
                            player_transform_mut,
                            -MOVEMENT_SPEED * time.delta_seconds(),
                            min_height_and_width,
                            max_width,
                        );
                    }

                    if input.key_is_down(VirtualKeyCode::D) {
                        bound_transform_x_prepend(
                            player_transform_mut,
                            MOVEMENT_SPEED * time.delta_seconds(),
                            min_height_and_width,
                            max_width,
                        );
                    }
                }

                // Rotation keys
                {
                    if input.key_is_down(VirtualKeyCode::Left) {
                        player_transform_mut.rotate_2d(-ROTATION_SPEED * time.delta_seconds());
                    }

                    if input.key_is_down(VirtualKeyCode::Right) {
                        player_transform_mut.rotate_2d(ROTATION_SPEED * time.delta_seconds());
                    }
                }
            }
        } else {
            let sprite = load_sprite_system(
                &texture_storage,
                &sheet_storage,
                &loader,
                "covid_player.png",
                0,
            );

            let mut transform = Transform::default();

            transform.set_translation_xyz(dimensions.height() * 0.5, dimensions.width() * 0.5, 3.0);

            self.player_entity = Some(
                lazy.create_entity(&*entities)
                    .with(sprite)
                    .with(transform)
                    .with(Transparent)
                    .with(LevelComponent)
                    .build(),
            );
        }
    }
}
