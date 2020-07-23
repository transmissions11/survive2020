use amethyst::core::ecs::{
    Component, DenseVecStorage, Entities, Join, Read, System, SystemData, WriteStorage,
};

use crate::every_n_seconds;
use crate::systems::load_sprite_system;
use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::{Time, Transform};
use amethyst::derive::SystemDesc;
use amethyst::renderer::{SpriteRender, SpriteSheet, Texture};
use amethyst::shred::ReadExpect;
use rand::Rng;

#[derive(SystemDesc, Default)]
pub struct HornetsSystem {
    bee_texture: Option<SpriteRender>,
}

#[derive(Default)]
pub struct Bee {
    /// The frame when the bee should be removed.
    pub expiration_frame: u64,
}
impl Component for Bee {
    type Storage = DenseVecStorage<Self>;
}

impl<'s> System<'s> for HornetsSystem {
    type SystemData = (
        Read<'s, Time>,
        Entities<'s>,
        Read<'s, AssetStorage<Texture>>,
        Read<'s, AssetStorage<SpriteSheet>>,
        ReadExpect<'s, Loader>,
        WriteStorage<'s, SpriteRender>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Bee>,
    );

    fn run(
        &mut self,
        (
            time,
            entities,
            texture_storage,
            sheet_storage,
            loader,
            mut sprite_render_storage,
            mut transform_storage,
            mut bee_storage,
        ): Self::SystemData,
    ) {
        let mut rng = rand::thread_rng();
        if let Some(bee_sprite) = &self.bee_texture {
            if every_n_seconds(0.5, &*time) {
                let mut transform = Transform::default();

                transform.set_translation_xyz(
                    rng.gen_range(100., 500.),
                    rng.gen_range(100., 500.),
                    0.0,
                );

                entities
                    .build_entity()
                    .with(
                        Bee {
                            expiration_frame: time.frame_number() + rng.gen_range(50, 180),
                        },
                        &mut bee_storage,
                    )
                    .with(bee_sprite.clone(), &mut sprite_render_storage)
                    .with(transform, &mut transform_storage)
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
