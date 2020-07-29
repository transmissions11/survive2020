use crate::states::LevelComponent;
use crate::systems::load_sprite_system;
use crate::{bound_transform_x_prepend, bound_transform_y_prepend};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::ecs::{
    Component, DenseVecStorage, Entities, Entity, Join, LazyUpdate, Read, ReadExpect, WriteStorage,
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

pub const DROPLET_SPEED: f32 = 100.0;

pub const DROPLET_MAX_SECONDS_ALIVE: f32 = 1.0;

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
}

impl<'s> System<'s> for WildfiresSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, Time>,
        Read<'s, LazyUpdate>,
        Read<'s, AssetStorage<Texture>>,
        Read<'s, AssetStorage<SpriteSheet>>,
        ReadExpect<'s, Loader>,
        ReadExpect<'s, ScreenDimensions>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Droplet>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(
        &mut self,
        (
            entities,
            time,
            lazy,
            texture_storage,
            sheet_storage,
            loader,
            dimensions,
            mut transform_storage,
            mut droplet_storage,
            input,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();

        if let Some(firefighter_entity) = &self.firefighter_entity {
            // Droplet physics
            {
                for (droplet, transform, entity) in
                    (&mut droplet_storage, &mut transform_storage, &entities).join()
                {
                    droplet.seconds_alive += time.delta_seconds();

                    transform.move_up(DROPLET_SPEED * time.delta_seconds());

                    transform.prepend_translation_x(rng.gen_range(-3.0, 3.0));

                    if droplet.seconds_alive >= DROPLET_MAX_SECONDS_ALIVE {
                        entities.delete(entity).expect("Couldn't delete droplet!");
                    }
                }
            }

            // Movement and shooting
            {
                let transform = transform_storage.get_mut(*firefighter_entity).unwrap();

                let min_height_and_width = PLAYER_HEIGHT_AND_WIDTH * 0.5;

                let max_height = dimensions.height() - PLAYER_HEIGHT_AND_WIDTH * 0.5;

                let max_width = dimensions.width() - PLAYER_HEIGHT_AND_WIDTH * 0.5;

                // Movement keys
                {
                    if input.key_is_down(VirtualKeyCode::W) {
                        bound_transform_y_prepend(
                            transform,
                            MOVEMENT_SPEED,
                            min_height_and_width,
                            max_height,
                        );
                    }
                    if input.key_is_down(VirtualKeyCode::S) {
                        bound_transform_y_prepend(
                            transform,
                            -MOVEMENT_SPEED,
                            min_height_and_width,
                            max_height,
                        );
                    }

                    if input.key_is_down(VirtualKeyCode::A) {
                        bound_transform_x_prepend(
                            transform,
                            -MOVEMENT_SPEED,
                            min_height_and_width,
                            max_width,
                        );
                    }

                    if input.key_is_down(VirtualKeyCode::D) {
                        bound_transform_x_prepend(
                            transform,
                            MOVEMENT_SPEED,
                            min_height_and_width,
                            max_width,
                        );
                    }
                }

                // Rotation keys
                {
                    if input.key_is_down(VirtualKeyCode::Left) {
                        transform.rotate_2d(-ROTATION_SPEED);
                    }

                    if input.key_is_down(VirtualKeyCode::Right) {
                        transform.rotate_2d(ROTATION_SPEED);
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

                        let mut droplet_transform = (*transform).clone();

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
        } else {
            let sprite =
                load_sprite_system(&texture_storage, &sheet_storage, &loader, "fireman.png", 0);

            let mut transform = Transform::default();

            transform.set_translation_xyz(dimensions.height() * 0.5, dimensions.width() * 0.5, 0.0);

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
