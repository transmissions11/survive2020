pub mod resources;
pub mod states;
pub mod systems;

use crate::states::main_menu::MainMenuState;
use amethyst::input::{is_key_down, VirtualKeyCode};
use amethyst::{
    assets::{AssetStorage, Loader},
    core::transform::Transform,
    core::ArcThreadPool,
    ecs::prelude::Join,
    ecs::{Component, DenseVecStorage, Dispatcher, DispatcherBuilder},
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
    delete_all_entities_with_component::<LevelTitle>(world);

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

/// Return to main menu on escape.
pub fn return_to_main_menu_on_escape(event: StateEvent) -> SimpleTrans {
    if let StateEvent::Window(event) = &event {
        if is_key_down(event, VirtualKeyCode::Escape) {
            Trans::Replace(Box::new(MainMenuState::default()))
        } else {
            Trans::None
        }
    } else {
        Trans::None
    }
}

/// Creates a systems dispatcher. Takes a closure where the caller adds systems.
pub fn create_systems_dispatcher<'a, 'b>(
    world: &mut World,
    add_systems: impl FnOnce(&mut DispatcherBuilder),
) -> Dispatcher<'a, 'b> {
    let mut builder = DispatcherBuilder::new();

    add_systems(&mut builder);

    let mut dispatcher = builder
        .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
        .build();
    dispatcher.setup(world);

    dispatcher
}

/// Creates a systems dispatcher. Takes a closure where the caller adds systems. Returns a Some(DispatchBuilder).
pub fn create_optional_systems_dispatcher<'a, 'b>(
    world: &mut World,
    add_systems: impl FnOnce(&mut DispatcherBuilder),
) -> Option<Dispatcher<'a, 'b>> {
    Some(create_systems_dispatcher(world, add_systems))
}

/// Take's a state's dispatcher and if it exists, runs all of its systems.
pub fn run_systems(world: &World, dispatcher: &mut Option<Dispatcher>) {
    if let Some(dispatcher) = dispatcher.as_mut() {
        dispatcher.dispatch(world);
    }
}

/// Creates the 2D camera.
pub fn init_camera(world: &mut World) {
    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 1.);

    world
        .create_entity()
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build();
}
