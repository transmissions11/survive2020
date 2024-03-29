use crate::resources::high_scores::highscores_keys::{COVID, HORNETS, WILDFIRES};
use crate::resources::high_scores::load_scores;
use crate::states::hornets::HornetState;
use crate::states::wildfires::{WildfireState, WildfiresStateTextComponent};
use crate::*;

use crate::audio::initialise_audio;
use crate::states::{init_camera, init_level_title, LevelComponent, TimerComponent};

use crate::states::covid::{CovidState, CovidStateTextComponent};
use amethyst::ui::{Anchor, UiButton, UiButtonBuilder, UiEventType};

#[derive(Default)]
pub struct MainMenuState {
    hornets_and_highscore_button: Option<(UiButton, UiButton)>,
    wildfires_and_highscore_button: Option<(UiButton, UiButton)>,
    covid_and_highscore_button: Option<(UiButton, UiButton)>,
}

/// Creates a button for a level and displays that level's highscore.
/// Vertical padding determined by level number.
/// Level number should be >0
fn create_level_button_with_highscore(
    world: &mut World,
    title: &str,
    level_number: u32,
    high_score: u64,
) -> (UiButton, UiButton) {
    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let y_spacing = -45.0 - (level_number as f32 * 75.0);

    let height = 64.0;

    let color = create_ui_color_from_rgba(195, 130, 51, 1.0);
    let hover_color = create_ui_color_from_rgba(195, 130, 51, 0.5);
    let alt_color = create_ui_color_from_rgba(243, 180, 73, 1.0);

    let font = get_main_font(world);

    let (_, button) = UiButtonBuilder::<(), u32>::new(format!("Level {}: {}", level_number, title))
        .with_font(font.clone())
        .with_font_size(21.5)
        .with_position(((dimensions.width() * 0.6) / 2.0) + 2.5, y_spacing)
        .with_size((dimensions.width() * 0.6) - 15.0, height)
        .with_anchor(Anchor::TopLeft)
        .with_image(color.clone())
        .with_hover_image(hover_color)
        .build_from_world(&world);

    let (_, high_score) = UiButtonBuilder::<(), u32>::new(format!("High Score: {}", high_score))
        .with_font_size(18.5)
        .with_font(font)
        .with_position(((dimensions.width() * -0.4) / 2.0) - 2.5, y_spacing)
        .with_size((dimensions.width() * 0.4) - 15.0, height)
        .with_anchor(Anchor::TopRight)
        .with_image(alt_color)
        .build_from_world(&world);

    (button, high_score)
}

/// Deletes level and highscore buttons passed in.
fn delete_level_and_highscore_buttons(world: &mut World, buttons: &Option<(UiButton, UiButton)>) {
    // Delete level button
    world
        .entities()
        .delete(buttons.as_ref().unwrap().0.image_entity)
        .expect("Cannot delete UiButton's image entity.");

    world
        .entities()
        .delete(buttons.as_ref().unwrap().0.text_entity)
        .expect("Cannot delete UiButton's text entity.");

    // Delete high score button
    world
        .entities()
        .delete(buttons.as_ref().unwrap().1.image_entity)
        .expect("Cannot delete UiButton's image entity.");
    world
        .entities()
        .delete(buttons.as_ref().unwrap().1.text_entity)
        .expect("Cannot delete UiButton's text entity.");
}

impl SimpleState for MainMenuState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Register the components we won't use in any systems
        world.register::<LevelComponent>();
        world.register::<TimerComponent>();
        world.register::<WildfiresStateTextComponent>();
        world.register::<CovidStateTextComponent>();

        // Init 2d camera
        init_camera(world);

        // Init audio
        initialise_audio(world);

        // Init level title
        init_level_title(world, "logo.png");

        // Create the fonts resource.
        let font = load_font(world, "main_font.ttf");
        world.insert(FontsResource { main_font: font });

        let high_scores = load_scores();

        self.wildfires_and_highscore_button = Some(create_level_button_with_highscore(
            world,
            "Wildfires",
            1,
            high_scores.get_score(WILDFIRES),
        ));
        self.hornets_and_highscore_button = Some(create_level_button_with_highscore(
            world,
            "Murder Hornets",
            2,
            high_scores.get_score(HORNETS),
        ));
        self.covid_and_highscore_button = Some(create_level_button_with_highscore(
            world,
            "Covid-19",
            3,
            high_scores.get_score(COVID),
        ));

        world.insert(high_scores);
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        delete_all_entities_with_component::<LevelComponent>(world);

        delete_level_and_highscore_buttons(world, &self.hornets_and_highscore_button);
        delete_level_and_highscore_buttons(world, &self.wildfires_and_highscore_button);
        delete_level_and_highscore_buttons(world, &self.covid_and_highscore_button);
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match event {
            StateEvent::Ui(ui_event) => {
                if ui_event.event_type == UiEventType::Click {
                    let hornets_button = self
                        .hornets_and_highscore_button
                        .as_ref()
                        .unwrap()
                        .0
                        .image_entity;
                    let wildfires_button = self
                        .wildfires_and_highscore_button
                        .as_ref()
                        .unwrap()
                        .0
                        .image_entity;

                    let covid_button = self
                        .covid_and_highscore_button
                        .as_ref()
                        .unwrap()
                        .0
                        .image_entity;

                    if ui_event.target == wildfires_button {
                        Trans::Replace(Box::new(WildfireState::default()))
                    } else if ui_event.target == hornets_button {
                        Trans::Replace(Box::new(HornetState::default()))
                    } else if ui_event.target == covid_button {
                        Trans::Replace(Box::new(CovidState::default()))
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
