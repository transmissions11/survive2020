use crate::audio::sound_keys::{BUCKET_SOUND, FIRE_OUT_SOUND, FIRE_SOUND};
use crate::audio::{play_sound_system, SoundsResource};
use crate::resources::abilities::{AbilitiesResource, AbilityType};
use crate::states::wildfires::WildfireStateResource;
use crate::states::{LevelComponent, LevelSecondsResource};
use crate::systems::ability_bar::RemoveItem;
use crate::systems::{distance_between_points, load_sprite_system};
use crate::{bound_transform_x_prepend, bound_transform_y_prepend, every_n_seconds};
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
use amethyst::winit::MouseButton;
use amethyst::{
    derive::SystemDesc,
    ecs::prelude::{System, SystemData},
};
use rand::Rng;

pub const MOVEMENT_SPEED: f32 = 4.0;
pub const ROTATION_SPEED: f32 = 0.1;

pub const PLAYER_HEIGHT_AND_WIDTH: f32 = 100.0;

pub const DROPLET_HEIGHT_AND_WIDTH: f32 = 20.0;
pub const DROPLET_SPEED: f32 = 140.0;
pub const DROPLET_MAX_SECONDS_ALIVE: f32 = 0.7;

pub const FIRE_HEIGHT_AND_WIDTH: f32 = 50.0;

pub const BUCKET_HEIGHT_AND_WIDTH: f32 = 200.0;

#[derive(Default)]
pub struct Fire;
impl Component for Fire {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct Droplet {
    /// How long the Droplet has been in the world.
    pub seconds_alive: f32,
}
impl Component for Droplet {
    type Storage = DenseVecStorage<Self>;
}

#[derive(SystemDesc, Default)]
pub struct WildfiresSystem {
    pub firefighter_entity: Option<Entity>,
    pub droplet_sprite: Option<SpriteRender>,
    pub fire_sprite: Option<SpriteRender>,

    pub bucket: Option<Entity>,
}

impl<'s> System<'s> for WildfiresSystem {
    type SystemData = (
        Entities<'s>,
        Write<'s, WildfireStateResource>,
        Read<'s, LevelSecondsResource>,
        Read<'s, Time>,
        Read<'s, LazyUpdate>,
        Read<'s, AssetStorage<Texture>>,
        Read<'s, AssetStorage<SpriteSheet>>,
        ReadExpect<'s, Loader>,
        ReadExpect<'s, ScreenDimensions>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Droplet>,
        ReadStorage<'s, Fire>,
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
            mut droplet_storage,
            fire_storage,
            input,
            mut abilities,
            audio_storage,
            sounds,
            audio_output,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();

        // All indexes in this ability will be removed from active_abilities
        let mut should_be_deactivated_abilities: Vec<usize> = Vec::new();

        let mut tri_shot_is_active = false;
        let mut range_boost_is_active = false;

        // Handle abilities
        for (index, ability) in abilities.available_abilities.iter().enumerate() {
            // If that ability is active
            if abilities.active_abilities.contains(&index) {
                let mut mouse_pos = input.mouse_position().unwrap_or((0., 0.));

                // Mouse pos height is determined from top left instead of bottom left so we have to flip this.
                mouse_pos.1 = dimensions.height() - mouse_pos.1;

                match ability.info.ability_type {
                    AbilityType::Bucket => {
                        // If the ability is about to expire
                        if ability.current_state.percentage < 0.05 {
                            // Delete the bucket sprite.
                            if let Some(bucket) = self.bucket {
                                entities
                                    .delete(bucket)
                                    .expect("Couldn't delete big swatter!");

                                self.bucket = None;
                            }
                        } else if let Some(bucket) = self.bucket {
                            let (bucket_x, bucket_y) = {
                                let bucket_transform = transform_storage.get_mut(bucket).unwrap();

                                bucket_transform.set_translation_xyz(mouse_pos.0, mouse_pos.1, 4.0);

                                (mouse_pos.0, mouse_pos.1)
                            };

                            if input.mouse_button_is_down(MouseButton::Left) {
                                // Can only use bucket once.
                                should_be_deactivated_abilities.push(index);
                                entities
                                    .delete(bucket)
                                    .expect("Couldn't delete big swatter!");
                                self.bucket = None;

                                play_sound_system(
                                    BUCKET_SOUND,
                                    &sounds,
                                    &audio_storage,
                                    &audio_output,
                                );

                                for (entity, _fire, fire_transform) in
                                    (&entities, &fire_storage, &transform_storage).join()
                                {
                                    if distance_between_points(
                                        bucket_x,
                                        bucket_y,
                                        fire_transform.translation().x,
                                        fire_transform.translation().y,
                                    ) <= (BUCKET_HEIGHT_AND_WIDTH * 0.5)
                                        + (FIRE_HEIGHT_AND_WIDTH * 0.5)
                                    {
                                        // Delete the fire
                                        entities.delete(entity).expect("Couldn't delete fire.");
                                    }
                                }
                            }
                        } else {
                            let bucket_sprite = load_sprite_system(
                                &texture_storage,
                                &sheet_storage,
                                &loader,
                                "bucket.png",
                                0,
                            );

                            let mut transform = Transform::default();

                            transform.set_translation_xyz(mouse_pos.0, mouse_pos.1, 4.0);

                            self.bucket = Some(
                                lazy.create_entity(&entities)
                                    // Tag entity with LevelComponent so it gets deleted on close.
                                    .with(LevelComponent)
                                    .with(bucket_sprite)
                                    .with(Transparent)
                                    .with(transform)
                                    .build(),
                            );
                        }
                    }

                    AbilityType::TriShot => {
                        tri_shot_is_active = true;
                    }

                    AbilityType::RangeBoost => {
                        range_boost_is_active = true;
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

        if let Some(firefighter_entity) = &self.firefighter_entity {
            // Fire collisions
            {
                for (_, fire_transform, fire_entity) in
                    (&fire_storage, &transform_storage, &entities).join()
                {
                    let firefighter_transform = transform_storage.get(*firefighter_entity).unwrap();

                    // If the fire is touching the player
                    if distance_between_points(
                        firefighter_transform.translation().x,
                        firefighter_transform.translation().y,
                        fire_transform.translation().x,
                        fire_transform.translation().y,
                    ) <= (0.5 * FIRE_HEIGHT_AND_WIDTH) + (0.5 * PLAYER_HEIGHT_AND_WIDTH)
                    {
                        entities.delete(fire_entity).expect("Couldn't delete fire!");
                        level_state.stepped_in_fire_times += 1;

                        play_sound_system(FIRE_SOUND, &sounds, &audio_storage, &audio_output);
                    }

                    'inner: for (_, droplet_transform, droplet_entity) in
                        (&droplet_storage, &transform_storage, &entities).join()
                    {
                        // If the fire is close to a droplet
                        if distance_between_points(
                            droplet_transform.translation().x,
                            droplet_transform.translation().y,
                            fire_transform.translation().x,
                            fire_transform.translation().y,
                        ) <= (0.5 * FIRE_HEIGHT_AND_WIDTH) + (0.5 * DROPLET_HEIGHT_AND_WIDTH)
                        {
                            entities.delete(fire_entity).expect("Couldn't delete fire!");
                            entities
                                .delete(droplet_entity)
                                .expect("Couldn't delete droplet!");

                            level_state.current_fires = level_state.current_fires.saturating_sub(1);

                            play_sound_system(
                                FIRE_OUT_SOUND,
                                &sounds,
                                &audio_storage,
                                &audio_output,
                            );

                            break 'inner;
                        }
                    }
                }
            }

            // Droplet physics
            {
                for (droplet, transform, entity) in
                    (&mut droplet_storage, &mut transform_storage, &entities).join()
                {
                    droplet.seconds_alive += time.delta_seconds();

                    transform.move_up(DROPLET_SPEED * time.delta_seconds());

                    transform.prepend_translation_x(rng.gen_range(-6.0, 6.0));

                    let should_delete;

                    if range_boost_is_active {
                        if droplet.seconds_alive >= (DROPLET_MAX_SECONDS_ALIVE * 2.) {
                            should_delete = true;
                        } else {
                            should_delete = false;
                        }
                    } else if droplet.seconds_alive >= DROPLET_MAX_SECONDS_ALIVE {
                        should_delete = true;
                    } else {
                        should_delete = false;
                    }

                    if should_delete {
                        entities.delete(entity).expect("Couldn't delete droplet!");
                    }
                }
            }

            let firefighter_transform = transform_storage.get_mut(*firefighter_entity).unwrap();

            // Movement and shooting
            {
                let min_height_and_width = PLAYER_HEIGHT_AND_WIDTH * 0.5;

                let max_height = dimensions.height() - PLAYER_HEIGHT_AND_WIDTH * 0.5;

                let max_width = dimensions.width() - PLAYER_HEIGHT_AND_WIDTH * 0.5;

                // Movement keys
                {
                    if input.key_is_down(VirtualKeyCode::W) {
                        bound_transform_y_prepend(
                            firefighter_transform,
                            MOVEMENT_SPEED,
                            min_height_and_width,
                            max_height,
                        );
                    }
                    if input.key_is_down(VirtualKeyCode::S) {
                        bound_transform_y_prepend(
                            firefighter_transform,
                            -MOVEMENT_SPEED,
                            min_height_and_width,
                            max_height,
                        );
                    }

                    if input.key_is_down(VirtualKeyCode::A) {
                        bound_transform_x_prepend(
                            firefighter_transform,
                            -MOVEMENT_SPEED,
                            min_height_and_width,
                            max_width,
                        );
                    }

                    if input.key_is_down(VirtualKeyCode::D) {
                        bound_transform_x_prepend(
                            firefighter_transform,
                            MOVEMENT_SPEED,
                            min_height_and_width,
                            max_width,
                        );
                    }
                }

                // Rotation keys
                {
                    if input.key_is_down(VirtualKeyCode::Left) {
                        firefighter_transform.rotate_2d(-ROTATION_SPEED);
                    }

                    if input.key_is_down(VirtualKeyCode::Right) {
                        firefighter_transform.rotate_2d(ROTATION_SPEED);
                    }
                }

                // Shooting
                {
                    if input.key_is_down(VirtualKeyCode::Up)
                        || input.key_is_down(VirtualKeyCode::Space)
                    {
                        let droplet_sprite;

                        if let Some(sprite) = &self.droplet_sprite {
                            droplet_sprite = sprite.clone();
                        } else {
                            let new_sprite = load_sprite_system(
                                &texture_storage,
                                &sheet_storage,
                                &loader,
                                "droplet.png",
                                0,
                            );

                            self.droplet_sprite = Some(new_sprite.clone());

                            droplet_sprite = new_sprite;
                        }

                        let droplet_sections_to_spawn = if tri_shot_is_active { 3 } else { 1 };

                        for n in 1..=droplet_sections_to_spawn {
                            let mut droplet_transform = (*firefighter_transform).clone();

                            droplet_transform.move_up(PLAYER_HEIGHT_AND_WIDTH * 0.5);

                            if !tri_shot_is_active || n == 2 {
                                droplet_transform.move_right(15.);
                            } else {
                                if n == 1 {
                                    droplet_transform.move_left(30.);
                                    droplet_transform.prepend_rotation_z_axis(5.);
                                }

                                if n == 3 {
                                    droplet_transform.move_right(45.);
                                    droplet_transform.prepend_rotation_z_axis(-5.);
                                }
                            }

                            lazy.create_entity(&*entities)
                                .with(droplet_sprite.clone())
                                .with(droplet_transform)
                                .with(LevelComponent)
                                .with(Droplet { seconds_alive: 0. })
                                .with(Transparent)
                                .build();
                        }
                    }
                }
            }

            // Fire spawning
            {
                if let Some(fire_sprite) = &self.fire_sprite {
                    if every_n_seconds(0.7, &*time) {
                        // Fires to spawn is from 1 to (2 + however many chunks of 20 seconds have gone by).
                        let fires_to_spawn =
                            rng.gen_range(1, 2 + (level_seconds.seconds_elapsed / 20.) as u32);

                        let mut fires_left_to_spawn = fires_to_spawn;

                        while fires_left_to_spawn != 0 {
                            let pos_x = rng.gen_range(10., 590.);
                            let pos_y = rng.gen_range(100., 500.);

                            let firefighter_transform =
                                transform_storage.get(*firefighter_entity).unwrap();

                            // Don't spawn fire on or really close to player
                            if distance_between_points(
                                pos_x,
                                pos_y,
                                firefighter_transform.translation().x,
                                firefighter_transform.translation().y,
                            ) <= ((PLAYER_HEIGHT_AND_WIDTH * 0.5) + 30.0)
                            {
                                continue;
                            }

                            let mut transform = Transform::default();

                            transform.set_translation_xyz(pos_x, pos_y, 0.0);

                            lazy.create_entity(&entities)
                                .with(fire_sprite.clone())
                                .with(transform)
                                .with(Transparent)
                                .with(LevelComponent)
                                .with(Fire)
                                .build();

                            fires_left_to_spawn -= 1;
                            level_state.current_fires += 1;
                        }
                    }
                } else {
                    // Load bee texture
                    self.fire_sprite = Some(load_sprite_system(
                        &texture_storage,
                        &sheet_storage,
                        &loader,
                        "fire.png",
                        0,
                    ));
                }
            }
        } else {
            let sprite =
                load_sprite_system(&texture_storage, &sheet_storage, &loader, "fireman.png", 0);

            let mut transform = Transform::default();

            transform.set_translation_xyz(dimensions.height() * 0.5, dimensions.width() * 0.5, 3.0);

            self.firefighter_entity = Some(
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
