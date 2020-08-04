use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::ecs::shred::PanicHandler;
use amethyst::core::ecs::{Component, Entities, Entity, Join, Read, WriteStorage};
use amethyst::core::Transform;
use amethyst::renderer::{ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture};

pub mod ability_bar;
pub mod covid;
pub mod hornets;
pub mod wildfires;

/// Detects collisions between components and takes an `on_collide` func which can respond to the collision.
pub fn handle_collisions<T: Component>(
    entities: &Entities,
    transform_storage: &mut WriteStorage<Transform>,
    component_storage: &mut WriteStorage<T>,
    component_height_and_width: f32,
    player_height_and_width: f32,
    player_x: f32,
    player_y: f32,
    mut on_collide: impl FnMut(Entity, &mut Transform, &mut T),
) {
    for (entity, transform, component) in (entities, transform_storage, component_storage).join() {
        if distance_between_points(
            player_x,
            player_y,
            transform.translation().x,
            transform.translation().y,
        ) <= (0.5 * component_height_and_width) + (0.5 * player_height_and_width)
        {
            on_collide(entity, transform, component);
        }
    }
}

/// Calculates the distance between 2 points.
fn distance_between_points(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    ((y2 - y1) * (y2 - y1) + (x2 - x1) * (x2 - x1)).sqrt()
}

/// Load a sprite from a file and sprite number. Returns a SpriteRender.
/// Will panic if filename does not contain an extension.
pub fn load_sprite_system(
    texture_storage: &Read<AssetStorage<Texture>>,
    sheet_storage: &Read<AssetStorage<SpriteSheet>>,
    loader: &Read<Loader, PanicHandler>,
    filename: &str,
    sprite_number: usize,
) -> SpriteRender {
    assert!(
        filename.contains('.'),
        "Filename did not contain extension!"
    );

    let filename_no_extension = filename.split(".").collect::<Vec<&str>>()[0];

    // Load the texture for our sprites. We'll later need to
    // add a handle to this texture to our `SpriteRender`s, so
    // we need to keep a reference to it.
    let texture_handle = {
        loader.load(
            format!("sprites/{}", filename),
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    // Load the spritesheet definition file, which contains metadata on our
    // spritesheet texture.
    let sheet_handle = {
        loader.load(
            format!("sprites/{}.ron", filename_no_extension),
            SpriteSheetFormat(texture_handle),
            (),
            &sheet_storage,
        )
    };

    SpriteRender {
        sprite_sheet: sheet_handle,
        sprite_number,
    }
}
