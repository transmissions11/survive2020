use crate::states::LevelAsset;
use crate::systems::load_sprite_system;
use crate::{bound_transform_x_prepend, bound_transform_y_prepend};
use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::ecs::{Entities, Entity, LazyUpdate, Read, ReadExpect, WriteStorage};
use amethyst::core::Transform;
use amethyst::input::{InputHandler, StringBindings};
use amethyst::prelude::Builder;
use amethyst::renderer::rendy::wsi::winit::VirtualKeyCode;
use amethyst::renderer::{SpriteSheet, Texture};
use amethyst::window::ScreenDimensions;
use amethyst::{
    derive::SystemDesc,
    ecs::prelude::{System, SystemData},
};

pub const MOVEMENT_SPEED: f32 = 4.0;
pub const ROTATION_SPEED: f32 = 0.1;

pub const PLAYER_HEIGHT_AND_WIDTH: f32 = 100.0;

#[derive(SystemDesc, Default)]
pub struct WildfiresSystem {
    pub firefighter_entity: Option<Entity>,
}

impl<'s> System<'s> for WildfiresSystem {
    type SystemData = (
        Entities<'s>,
        Read<'s, LazyUpdate>,
        Read<'s, AssetStorage<Texture>>,
        Read<'s, AssetStorage<SpriteSheet>>,
        ReadExpect<'s, Loader>,
        ReadExpect<'s, ScreenDimensions>,
        WriteStorage<'s, Transform>,
        Read<'s, InputHandler<StringBindings>>,
    );

    fn run(
        &mut self,
        (
            entities,
            lazy,
            texture_storage,
            sheet_storage,
            loader,
            dimensions,
            mut transform_storage,
            input,
        ): Self::SystemData,
    ) {
        if let Some(firefighter_entity) = &self.firefighter_entity {
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
        } else {
            let sprite =
                load_sprite_system(&texture_storage, &sheet_storage, &loader, "fireman.png", 0);

            let mut transform = Transform::default();

            transform.set_translation_xyz(dimensions.height() * 0.5, dimensions.width() * 0.5, 0.0);
            transform.rotate_2d(10.0);

            self.firefighter_entity = Some(
                lazy.create_entity(&*entities)
                    .with(sprite)
                    .with(transform)
                    .with(LevelAsset)
                    .build(),
            );
        }
    }
}
