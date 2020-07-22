use amethyst::prelude::*;

use crate::states::hornets::HornetState;
use crate::states::wildfires::WildfireState;
use crate::systems::ability_bar::AbilityBarComponent;
use crate::*;
use amethyst::ui::{Anchor, UiButton, UiButtonBuilder, UiEventType, UiImage};

#[derive(Default)]
pub struct MainMenuState {
    hornet_button: Option<UiButton>,
    wildfires_button: Option<UiButton>,
}

pub fn create_level_button(world: &mut World, title: &str, level_number: u32) -> UiButton {
    let (_, button) = UiButtonBuilder::<(), u32>::new(format!("Level {}: {}", level_number, title))
        .with_font_size(32.0)
        .with_position(0.0, -156.0 - (level_number as f32 * 80.0))
        .with_size(64.0 * 6.0, 64.0)
        .with_anchor(Anchor::TopMiddle)
        .with_image(UiImage::SolidColor([0.8, 0.6, 0.3, 1.0]))
        .with_hover_image(UiImage::SolidColor([0.8, 0.6, 0.3, 0.5]))
        .build_from_world(&world);

    button
}

pub fn delete_ui_button(world: &mut World, button: &Option<UiButton>) {
    world
        .entities()
        .delete(button.as_ref().unwrap().image_entity)
        .expect("Cannot delete UiButton's image entity.");

    world
        .entities()
        .delete(button.as_ref().unwrap().text_entity)
        .expect("Cannot delete UiButton's text entity.");
}

impl SimpleState for MainMenuState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Init 2d camera
        init_camera(world);

        // Register the components we won't use in any systems
        world.register::<LevelTitle>();
        world.register::<AbilityBarComponent>();

        init_level_title(world, "logo.png");

        self.wildfires_button = Some(create_level_button(world, "Wildfires", 1));
        self.hornet_button = Some(create_level_button(world, "Murder Hornets", 2));
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        delete_ui_button(world, &self.hornet_button);
        delete_ui_button(world, &self.wildfires_button);
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match event {
            StateEvent::Ui(ui_event) => {
                if ui_event.event_type == UiEventType::Click {
                    let hornets_button = self.hornet_button.as_ref().unwrap().image_entity;
                    let wildfires_button = self.wildfires_button.as_ref().unwrap().image_entity;

                    if ui_event.target == wildfires_button {
                        Trans::Replace(Box::new(WildfireState::default()))
                    } else if ui_event.target == hornets_button {
                        Trans::Replace(Box::new(HornetState::default()))
                    } else {
                        Trans::None
                    }
                } else {
                    Trans::None
                }
            }
            _ => Trans::None,
        }
    }
}
