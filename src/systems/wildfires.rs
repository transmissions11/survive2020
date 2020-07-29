use crate::resources::high_scores::CurrentLevelScoreResource;
use crate::states::LevelComponent;
use crate::systems::{distance_between_points, load_sprite_system};
use crate::{bound_transform_x_prepend, bound_transform_y_prepend, every_n_seconds};
use amethyst::assets::{AssetStorage, Loader};
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
use amethyst::{
    derive::SystemDesc,
    ecs::prelude::{System, SystemData},
};
use rand::Rng;

pub const MOVEMENT_SPEED: f32 = 4.0;
pub const ROTATION_SPEED: f32 = 0.07;

pub const PLAYER_HEIGHT_AND_WIDTH: f32 = 100.0;

pub const DROPLET_HEIGHT_AND_WIDTH: f32 = 20.0;
pub const DROPLET_SPEED: f32 = 140.0;
pub const DROPLET_MAX_SECONDS_ALIVE: f32 = 0.5;

pub const FIRE_HEIGHT_AND_WIDTH: f32 = 50.0;

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
}

impl<'s> System<'s> for WildfiresSystem {
    type SystemData = (
        Entities<'s>,
        Write<'s, CurrentLevelScoreResource>,
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
    );

    fn run(
        &mut self,
        (
            entities,
            mut score,
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
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();

        if let Some(firefighter_entity) = &self.firefighter_entity {
            // Fire collisions
            {
                for (_, droplet_transform, droplet_entity) in
                    (&droplet_storage, &transform_storage, &entities).join()
                {
                    for (_, fire_transform, fire_entity) in
                        (&fire_storage, &transform_storage, &entities).join()
                    {
                        let firefighter_transform =
                            transform_storage.get(*firefighter_entity).unwrap();

                        // If the fire is touching the player
                        if distance_between_points(
                            firefighter_transform.translation().x,
                            firefighter_transform.translation().y,
                            fire_transform.translation().x,
                            fire_transform.translation().y,
                        ) <= (0.5 * FIRE_HEIGHT_AND_WIDTH) + (0.5 * PLAYER_HEIGHT_AND_WIDTH)
                        {
                            if score.score < 30 {
                                score.score = 0;
                            } else {
                                score.score -= 30;
                            }
                        }

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

                    if droplet.seconds_alive >= DROPLET_MAX_SECONDS_ALIVE {
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
                        let mut droplet_sprite = None;

                        if let Some(sprite) = &self.droplet_sprite {
                            droplet_sprite = Some(sprite.clone());
                        } else {
                            let new_sprite = load_sprite_system(
                                &texture_storage,
                                &sheet_storage,
                                &loader,
                                "droplet.png",
                                0,
                            );

                            self.droplet_sprite = Some(new_sprite.clone());

                            droplet_sprite = Some(new_sprite);
                        }

                        let mut droplet_transform = (*firefighter_transform).clone();

                        droplet_transform.move_up(PLAYER_HEIGHT_AND_WIDTH * 0.5);
                        droplet_transform.move_right(15.);

                        lazy.create_entity(&*entities)
                            .with(droplet_sprite.unwrap())
                            .with(droplet_transform)
                            .with(Droplet { seconds_alive: 0. })
                            .with(LevelComponent)
                            .with(Transparent)
                            .build();
                    }
                }
            }

            // Fire spawning
            {
                if let Some(fire_sprite) = &self.fire_sprite {
                    if every_n_seconds(0.5, &*time) {
                        let fires_to_spawn = rng.gen_range(1, 8);

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
