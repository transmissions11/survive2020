use amethyst::assets::{AssetStorage, Loader};
use amethyst::core::ecs::shred::PanicHandler;
use amethyst::core::ecs::Read;
use amethyst::renderer::{ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture};

pub mod ability_bar;
pub mod hornets;
pub mod wildfires;

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
