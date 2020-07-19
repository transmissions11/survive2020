pub mod hornets;
pub mod wildfires;

use amethyst::{
    assets::{AssetStorage, Loader},
    core::transform::Transform,
    ecs::prelude::Join,
    ecs::{Component, DenseVecStorage},
    input::{is_key_down, VirtualKeyCode},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    window::ScreenDimensions,
};

#[derive(Debug)]
pub struct LevelTitle;
impl Component for LevelTitle {
    type Storage = DenseVecStorage<Self>;
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

/// Displays the level title at the top of the screen.
pub fn init_level_title(world: &mut World, filename: &str) {
    // Delete previous titles
    delete_level_title(world);

    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let sprite = load_sprite(world, filename, 0);

    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.9, 0.);

    world
        .create_entity()
        .with(LevelTitle)
        .with(transform)
        .with(sprite)
        .build();
}

/// Deletes all level titles visible
pub fn delete_level_title(world: &mut World) {
    let to_delete = {
        let mut result = Vec::new();
        let titles = world.read_storage::<LevelTitle>();
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

/// Pushes to a new level when the Z key is pressed.
pub fn push_to_level_on_key(
    event: StateEvent,
    new_state: impl SimpleState + 'static,
) -> SimpleTrans {
    if let StateEvent::Window(event) = &event {
        if is_key_down(event, VirtualKeyCode::Z) {
            Trans::Push(Box::new(new_state))
        } else {
            Trans::None
        }
    } else {
        Trans::None
    }
}

/// Creates the 2D camera.
fn init_camera(world: &mut World) {
    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);

    world
        .create_entity()
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build();
}
