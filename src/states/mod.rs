pub mod hornets;
pub mod main_menu;
pub mod wildfires;

use crate::resources::high_scores::{update_high_score_if_greater, CurrentLevelScoreResource};
use crate::states::main_menu::MainMenuState;

use amethyst::core::Time;
use amethyst::input::{is_key_down, VirtualKeyCode};

use crate::{get_main_font, load_sprite};
use amethyst::ui::{Anchor, LineMode, UiText, UiTransform};
use amethyst::{
    core::transform::Transform,
    core::ArcThreadPool,
    ecs::prelude::Join,
    ecs::{Component, DenseVecStorage, Dispatcher, DispatcherBuilder},
    prelude::*,
    renderer::Camera,
    window::ScreenDimensions,
};

/// A component to tag a timer text component.
pub struct TimerComponent;
impl Component for TimerComponent {
    type Storage = DenseVecStorage<Self>;
}

/// Update the elapsed time using delta seconds and set the high score if max time is passed and the score is the highest.
/// It also updates the score counter when updating the time.
pub fn update_timer_and_set_high_score(
    world: &mut World,
    elapsed_time: &mut f32,
    max_time: f32,
    high_score_key: &str,
) -> SimpleTrans {
    // Old time + delta seconds.
    let new_time = *elapsed_time + world.read_resource::<Time>().delta_seconds();

    let rounded_new_time = (new_time * 10.0).round() / 10.0;
    let rounded_old_time = (*elapsed_time * 10.0).round() / 10.0;

    // Whether or not the rounded times changed
    let rounded_times_changed = rounded_new_time > rounded_old_time.floor();

    // If the timer is maxed out.
    let level_is_over = *elapsed_time >= max_time;

    // Update the elapsed time
    *elapsed_time = new_time;

    let timer_entity = {
        if rounded_times_changed || level_is_over {
            let mut ui_texts = world.write_storage::<UiText>();
            let timer_components = world.read_storage::<TimerComponent>();

            let score = world.read_resource::<CurrentLevelScoreResource>();

            let entities = world.entities();

            let mut timer_entity = None;

            for (ui_text, _, entity) in (&mut ui_texts, &timer_components, &entities).join() {
                ui_text.text = format!(
                    "{}s / {}s - Score: {}",
                    rounded_new_time, max_time, score.score
                );

                if level_is_over {
                    timer_entity = Some(entity);
                }
            }

            timer_entity
        } else {
            None
        }
    };

    if level_is_over {
        update_high_score_if_greater(world, high_score_key);

        // Delete the timer entity.
        if let Some(entity) = timer_entity {
            world
                .delete_entity(entity)
                .expect("Couldn't delete timer text entity!");
        }

        Trans::Replace(Box::new(MainMenuState::default()))
    } else {
        Trans::None
    }
}

/// Create timer/score text with default value of "0s / {max_seconds}s - Score: 0"
/// Tagged with TimerComponent.
/// It will automatically get deleted when used with `update_timer_and_set_high_score` when the timer ends.
pub fn init_timer_and_score_text(world: &mut World, max_seconds: f32) {
    let font = get_main_font(world);

    let transform = UiTransform::new(
        "timer_text".to_string(),
        Anchor::TopMiddle,
        Anchor::TopMiddle,
        0.0,
        -55.0,
        0.0,
        600.0,
        50.0,
    );
    let ui_text = UiText::new(
        font,
        format!("0s /{}s - Score: 0", max_seconds),
        [1.0, 1.0, 1.0, 1.0],
        25.0,
        LineMode::Single,
        Anchor::Middle,
    );

    world
        .create_entity()
        .with(TimerComponent)
        .with(transform)
        .with(ui_text)
        .build();
}

/// Creates the 2D camera.
pub fn init_camera(world: &mut World) {
    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.5, 10.);

    world
        .create_entity()
        .with(Camera::standard_2d(dimensions.width(), dimensions.height()))
        .with(transform)
        .build();
}

/// Creates a systems dispatcher. Takes a closure where the caller adds systems.
pub fn create_systems_dispatcher<'a, 'b>(
    world: &mut World,
    add_systems: impl FnOnce(&mut DispatcherBuilder, &mut World),
) -> Dispatcher<'a, 'b> {
    let mut builder = DispatcherBuilder::new();

    add_systems(&mut builder, world);

    let mut dispatcher = builder
        .with_pool((*world.read_resource::<ArcThreadPool>()).clone())
        .build();
    dispatcher.setup(world);

    dispatcher
}

/// Creates a systems dispatcher. Takes a closure where the caller adds systems. Returns a Some(DispatchBuilder).
pub fn create_optional_systems_dispatcher<'a, 'b>(
    world: &mut World,
    add_systems: impl FnOnce(&mut DispatcherBuilder, &mut World),
) -> Option<Dispatcher<'a, 'b>> {
    Some(create_systems_dispatcher(world, add_systems))
}

/// Take's a state's dispatcher and if it exists, runs all of its systems.
pub fn run_systems(world: &World, dispatcher: &mut Option<Dispatcher>) {
    if let Some(dispatcher) = dispatcher.as_mut() {
        dispatcher.dispatch(world);
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

/// Tag a component as level assets (level title and background)
pub struct LevelAsset;
impl Component for LevelAsset {
    type Storage = DenseVecStorage<Self>;
}

/// Displays the level background in the center of the screen.
pub fn init_level_background(world: &mut World, filename: &str) {
    let background = load_sprite(world, filename, 0);

    let (width, height) = {
        let dimensions = world.read_resource::<ScreenDimensions>();

        let width = dimensions.width();
        let height = dimensions.height();

        (width, height)
    };

    let mut transform = Transform::default();

    transform.set_translation_xyz(width * 0.5, height * 0.5, -10.0);

    world
        .create_entity()
        .with(LevelAsset)
        .with(transform)
        .with(background)
        .build();
}

/// Displays the level title at the top of the screen.
pub fn init_level_title(world: &mut World, filename: &str) {
    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let sprite = load_sprite(world, filename, 0);

    let mut transform = Transform::default();
    transform.set_translation_xyz(dimensions.width() * 0.5, dimensions.height() * 0.93, 0.);

    world
        .create_entity()
        .with(LevelAsset)
        .with(transform)
        .with(sprite)
        .build();
}
