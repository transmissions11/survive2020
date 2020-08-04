use crate::resources::abilities::AbilitiesResource;
use crate::states::LevelComponent;
use crate::*;
use amethyst::core::ecs::{Component, DenseVecStorage, World};
use amethyst::core::shrev::EventChannel;
use amethyst::core::Transform;
use amethyst::renderer::{SpriteRender, Transparent};
use amethyst::ui::{Anchor, UiButton, UiButtonBuilder, UiEvent, UiEventType, UiImage, UiTransform};
use amethyst::window::ScreenDimensions;
use amethyst::{core::timing::Time, derive::SystemDesc, ecs::prelude::*};

pub const ABILITY_FRAME_HEIGHT_AND_WITH: f32 = 52.;
pub const PROGRESS_BAR_MAX_WIDTH: f32 = 47.;
pub const PROGRESS_BAR_HEIGHT: f32 = 7.;
/// The extra spacing between ability frames.
pub const ABILITY_FRAME_SPACING: f32 = 10.;

pub trait RemoveItem<T> {
    fn remove_first_found_item(&mut self, item: &T) -> Option<T>;
}

impl<T: PartialEq> RemoveItem<T> for Vec<T> {
    fn remove_first_found_item(&mut self, item: &T) -> Option<T> {
        let pos = match self.iter().position(|x| *x == *item) {
            Some(x) => x,
            None => return None,
        };
        Some(self.remove(pos))
    }
}

#[derive(Default)]
pub struct ProgressBar {
    pub ability_index: usize,
    pub x_offset: f32,
}
impl Component for ProgressBar {
    type Storage = DenseVecStorage<Self>;
}

/// Creates an ability bar based off of a vector of abilities. Updates the Abilities resource with the new abilities.
pub fn init_abilities_bar(world: &mut World, mut abilities: AbilitiesResource) {
    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let mut base_offset = 0.0;

    while base_offset
        != dimensions.width()
            - (((abilities.available_abilities.len() - 1) as f32
                * (ABILITY_FRAME_HEIGHT_AND_WITH + ABILITY_FRAME_SPACING))
                + base_offset)
    {
        base_offset += 1.0;
    }

    for (i, ability) in abilities.available_abilities.iter_mut().enumerate() {
        ability.current_state.ui_button = Some(create_ability_item(
            world,
            ability.info.icon.clone(),
            base_offset + ((ABILITY_FRAME_HEIGHT_AND_WITH + ABILITY_FRAME_SPACING) * i as f32),
            i,
        ));
    }

    // insert() overrides if already exists.
    world.insert(abilities);
}

/// Creates a UI transform for a progress bar.
pub fn create_progress_bar_transform(
    x_padding: f32,
    percent: f32,
    arena_height: f32,
) -> UiTransform {
    UiTransform::new(
        percent.to_string(),
        Anchor::BottomLeft,
        Anchor::MiddleLeft,
        x_padding - (0.5 * PROGRESS_BAR_MAX_WIDTH),
        (arena_height * 0.05) - ABILITY_FRAME_HEIGHT_AND_WITH / 2.5,
        0.,
        PROGRESS_BAR_MAX_WIDTH * percent,
        PROGRESS_BAR_HEIGHT,
    )
}

/// Creates an ability item button at the padding location with the associated index.
pub fn create_ability_item(
    world: &mut World,
    icon: SpriteRender,
    x_padding: f32,
    index: usize,
) -> UiButton {
    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let ability_frame = load_sprite(world, "ability_frame.png", 0);
    let selected_ability_frame = load_sprite(world, "selected_ability_frame.png", 0);
    let tapped_ability_frame = load_sprite(world, "tapped_ability_frame.png", 0);

    let progress_bar = load_sprite(world, "progress_bar.png", 0);

    world
        .create_entity()
        .with(UiImage::Sprite(progress_bar))
        .with(create_progress_bar_transform(
            x_padding,
            1.0,
            dimensions.height(),
        ))
        .with(ProgressBar {
            ability_index: index,
            x_offset: x_padding,
        })
        .with(LevelComponent)
        .build();

    let button_parent = world.create_entity().with(LevelComponent).build();

    let (_id, button) = UiButtonBuilder::<(), u32>::new(String::new())
        .with_anchor(Anchor::BottomLeft)
        .with_layer(3.0)
        .with_position(x_padding, dimensions.height() * 0.05)
        .with_size(52., 52.)
        .with_image(UiImage::Sprite(ability_frame))
        .with_hover_image(UiImage::Sprite(selected_ability_frame))
        .with_press_image(UiImage::Sprite(tapped_ability_frame))
        .with_parent(button_parent)
        .build_from_world(&world);

    let background = load_sprite(world, "ability_frame_background.png", 0);

    // Create the background in the center of the frame.
    let mut background_transform = Transform::default();
    background_transform.set_translation_xyz(x_padding, dimensions.height() * 0.05, 1.0);

    world
        .create_entity()
        .with(background_transform.clone())
        .with(background)
        .with(LevelComponent)
        .build();

    // Create the icon in the upper center of the frame.
    let mut icon_transform = Transform::default();
    icon_transform.set_translation_xyz(
        x_padding,
        (dimensions.height() * 0.05) + PROGRESS_BAR_HEIGHT / 2.5,
        2.0,
    );

    world
        .create_entity()
        .with(icon_transform)
        .with(Transparent)
        .with(icon)
        .with(LevelComponent)
        .build();

    button
}

/// Updates a progress bar and the ability at the index.
/// It will increase the progress bar at the speed specified if the ability is on cooldown.
/// It will decrease the progress bar if the ability has a duration and is active.
/// It will enforce max uses if the ability has a limit.
/// If the ability has a duration and that duration is over, it will remove the ability from the active abilities vector.
pub fn update_progress_bar(
    progress_bar: &ProgressBar,
    transform: &mut UiTransform,
    abilities: &mut AbilitiesResource,
    time: &Time,
    arena_height: f32,
) {
    let ability = &mut abilities.available_abilities[progress_bar.ability_index];

    // If the ability is active:
    if abilities
        .active_abilities
        .contains(&progress_bar.ability_index)
        && ability.info.duration.is_some()
    {
        // Lower the progress bar until the ability duration is complete.
        let mut new_percentage = ability.current_state.percentage
            - (time.delta_seconds() / ability.info.duration.unwrap() as f32);

        if new_percentage <= 0.0 {
            new_percentage = 0.0;
            // Remove the ability from being active.
            abilities
                .active_abilities
                .remove_first_found_item(&progress_bar.ability_index);
        }

        ability.current_state.percentage = new_percentage;

        *transform =
            create_progress_bar_transform(progress_bar.x_offset, new_percentage, arena_height);
    } else {
        let mut new_percentage = ability.current_state.percentage
            + (time.delta_seconds() / ability.info.seconds_to_charge as f32);

        if new_percentage > 1.0 {
            new_percentage = 1.0;
        }

        // If the ability has a max use set
        if let Some(max_uses) = ability.info.max_uses {
            // If this ability has already been used up
            if ability.current_state.uses >= max_uses {
                // Set the charge percentage to 0
                new_percentage = 0.0;
            }
        }

        ability.current_state.percentage = new_percentage;

        *transform =
            create_progress_bar_transform(progress_bar.x_offset, new_percentage, arena_height);
    }
}

/// Updates the ability's "uses" counter
/// and sets the ability's charge percentage to 0 (if the ability does not have a duration).
/// Adds the index of the ability to the `active_abilities` vector.
pub fn use_ability(
    progress_bar: &ProgressBar,
    transform: &mut UiTransform,
    abilities: &mut AbilitiesResource,

    arena_height: f32,
) {
    let ability = &mut abilities.available_abilities[progress_bar.ability_index];

    // If ability is off cooldown (and is not active)
    if ability.current_state.percentage == 1.0 {
        // If ability has a duration:
        if let Some(_) = ability.info.duration {
            // Just update the uses.
            ability.current_state.uses += 1;
        } else {
            // Set the charge percentage to 0 and update the uses.
            ability.current_state.percentage = 0.0;
            ability.current_state.uses += 1;
            *transform = create_progress_bar_transform(progress_bar.x_offset, 0.0, arena_height);
        }

        // Level-specific systems will do as they please with the ability being active:
        // If the ability does not have a duration, the system will have to manually remove the ability.
        abilities.active_abilities.push(progress_bar.ability_index);
    }
}

#[derive(SystemDesc)]
#[system_desc(name(AbilityBarSystemDesc))]
pub struct AbilityBarSystem {
    #[system_desc(event_channel_reader)]
    reader_id: ReaderId<UiEvent>,
}

impl AbilityBarSystem {
    pub fn new(reader_id: ReaderId<UiEvent>) -> Self {
        Self { reader_id }
    }
}

impl<'s> System<'s> for AbilityBarSystem {
    type SystemData = (
        Read<'s, EventChannel<UiEvent>>,
        ReadStorage<'s, ProgressBar>,
        WriteStorage<'s, UiTransform>,
        Read<'s, Time>,
        ReadExpect<'s, ScreenDimensions>,
        Option<Write<'s, AbilitiesResource>>,
    );

    fn run(
        &mut self,
        (events, progress_bars, mut transforms, time, dimensions, abilities): Self::SystemData,
    ) {
        if let Some(mut abilities) = abilities {
            let mut clicked_abilities: Vec<usize> = Vec::new();

            for ui_event in events.read(&mut self.reader_id) {
                if ui_event.event_type == UiEventType::Click {
                    for (i, ability) in abilities.available_abilities.iter_mut().enumerate() {
                        let button = ability
                            .current_state
                            .ui_button
                            .as_ref()
                            .unwrap()
                            .image_entity;

                        if ui_event.target == button {
                            clicked_abilities.push(i);
                        }
                    }
                }
            }

            for (progress_bar, transform) in (&progress_bars, &mut transforms).join() {
                if clicked_abilities.contains(&progress_bar.ability_index) {
                    use_ability(
                        progress_bar,
                        transform,
                        &mut *abilities,
                        dimensions.height(),
                    );
                } else {
                    update_progress_bar(
                        progress_bar,
                        transform,
                        &mut *abilities,
                        &*time,
                        dimensions.height(),
                    );
                }
            }
        }
    }
}
