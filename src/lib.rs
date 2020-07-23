pub mod audio;
pub mod resources;
pub mod states;
pub mod systems;

use amethyst::core::Time;
use amethyst::renderer::palette::Srgba;
use amethyst::ui::{FontHandle, TtfFormat, UiImage};
use amethyst::{
    assets::{AssetStorage, Loader},
    ecs::prelude::Join,
    ecs::Component,
    prelude::*,
    renderer::{ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};

/// Load a font from a file. Returns a FontHandle.
/// Will panic if the filename does not end with ".ttf".
pub fn load_font(world: &mut World, filename: &str) -> FontHandle {
    assert!(
        filename.contains(".ttf"),
        "Font filename must end with .ttf!"
    );

    world.read_resource::<Loader>().load(
        format!("fonts/{}", filename),
        TtfFormat,
        (),
        &world.read_resource(),
    )
}

/// Load a sprite from a file and sprite number. Returns a SpriteRender.
/// Will panic if filename does not contain an extension.
pub fn load_sprite(world: &mut World, filename: &str, sprite_number: usize) -> SpriteRender {
    assert!(
        filename.contains('.'),
        "Filename did not contain extension!"
    );

    let filename_no_extension = filename.split(".").collect::<Vec<&str>>()[0];

    // Load the texture for our sprites. We'll later need to
    // add a handle to this texture to our `SpriteRender`s, so
    // we need to keep a reference to it.
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
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
        let loader = world.read_resource::<Loader>();
        let sheet_storage = world.read_resource::<AssetStorage<SpriteSheet>>();
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

/// Deletes all entities with the associated component.
pub fn delete_all_entities_with_component<T: Component>(world: &mut World) {
    let to_delete = {
        let mut result = Vec::new();
        let titles = world.read_storage::<T>();
        let entities = world.entities();

        for (_title, entity) in (&titles, &entities).join() {
            result.push(entity);
        }

        result
    };
    if !to_delete.is_empty() {
        world
            .delete_entities(to_delete.as_slice())
            .expect("Couldn't delete title entities!");
    }
}

/// Gets the main font from a resource.
pub fn get_main_font(world: &mut World) -> FontHandle {
    let font = world.read_resource::<FontHandle>();

    (*font).clone()
}

/// Creates a UiImage::SolidColor from rgba.
/// r, g, b should be max 255
/// a should be 0.0 - 1.0
pub fn create_ui_color_from_rgba(r: u32, g: u32, b: u32, a: f32) -> UiImage {
    let (r, g, b, a) = Srgba::new(r as f32 / 255., g as f32 / 255., b as f32 / 255., a)
        .into_linear()
        .into_components();

    UiImage::SolidColor([r, g, b, a])
}

/// Returns true if the `time.frame_number()` is a multiple of 60 * n
pub fn every_n_seconds(n: f64, time: &Time) -> bool {
    (time.frame_number() as f64 % (60. * n)) == 0.0
}
