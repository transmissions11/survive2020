use crate::*;

use crate::resources::high_scores::highscores_keys::COVID;

use crate::resources::high_scores::{update_high_score_if_greater, CurrentLevelScoreResource};
use crate::states::main_menu::MainMenuState;
use crate::states::{
    create_optional_systems_dispatcher, init_level_background, init_level_title,
    return_to_main_menu_on_escape, run_systems, LevelComponent, LevelSecondsResource,
};

use crate::resources::abilities::{
    AbilitiesResource, Ability, AbilityInfo, AbilityState, AbilityType,
};
use crate::systems::ability_bar::init_abilities_bar;
use crate::systems::covid::CovidSystem;
use amethyst::core::ecs::DenseVecStorage;
use amethyst::shred::Dispatcher;
use amethyst::ui::{Anchor, LineMode, UiText, UiTransform};

pub const HEALTH_POOL: u64 = 100;

/// Tags a component as the covid state text.
pub struct CovidStateTextComponent;
impl Component for CovidStateTextComponent {
    type Storage = DenseVecStorage<Self>;
}

fn init_covid_state_text(world: &mut World, health_pool: u64) {
    let font = get_main_font(world);

    let transform = UiTransform::new(
        "covid_state".to_string(),
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
        format!("{} HP / {} MAX HEALTH", health_pool, health_pool),
        [1.0, 1.0, 1.0, 1.0],
        25.0,
        LineMode::Single,
        Anchor::Middle,
    );

    world
        .create_entity()
        .with(CovidStateTextComponent)
        .with(LevelComponent)
        .with(transform)
        .with(ui_text)
        .build();
}

fn update_covid_state(world: &mut World, current_health: u64, health_pool: u64) {
    let mut ui_texts = world.write_storage::<UiText>();
    let state_text_components = world.read_storage::<CovidStateTextComponent>();

    for (ui_text, _) in (&mut ui_texts, &state_text_components).join() {
        ui_text.text = format!("{} HP / {} MAX HEALTH", current_health, health_pool);
    }
}

/// A resource for storing some level state for the COVID level.
pub struct CovidStateResource {
    pub current_health: u64,
}

impl Default for CovidStateResource {
    fn default() -> Self {
        CovidStateResource {
            current_health: HEALTH_POOL,
        }
    }
}

#[derive(Default)]
pub struct CovidState<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for CovidState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        init_covid_state_text(world, HEALTH_POOL);

        init_level_background(world, "covid_background.png");

        init_level_title(world, "covid_title.png");

        world.insert(CurrentLevelScoreResource::default());
        world.insert(LevelSecondsResource::default());

        // Init the resource storing data about the player's progress on the level
        world.insert(CovidStateResource::default());

        let mask_sprite = load_sprite(world, "mask_ability.png", 0);
        let spray_bottle = load_sprite(world, "spray_bottle_ability.png", 0);

        init_abilities_bar(
            world,
            AbilitiesResource::new(vec![
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::Mask,
                        seconds_to_charge: 17,
                        duration: Some(5),
                        icon: mask_sprite,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::SprayBottle,
                        seconds_to_charge: 17,
                        duration: Some(7),
                        icon: spray_bottle,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
            ]),
        );

        self.dispatcher = create_optional_systems_dispatcher(world, |builder, _| {
            builder.add(CovidSystem::default(), "covid", &[])
        });
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        delete_all_entities_with_component::<LevelComponent>(world);

        update_high_score_if_greater(world, COVID);
    }

    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        return_to_main_menu_on_escape(event)
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = &mut data.world;

        // Update seconds elapsed
        let seconds_elapsed = {
            let mut seconds = world.write_resource::<LevelSecondsResource>();

            seconds.seconds_elapsed += world.read_resource::<Time>().delta_seconds();

            seconds.seconds_elapsed
        };

        // Update the score
        let current_health = {
            let state = world.read_resource::<CovidStateResource>();

            let mut score = world.write_resource::<CurrentLevelScoreResource>();

            // Update the level score based on seconds elapsed.
            score.score = seconds_elapsed as u64;

            state.current_health
        };

        // End the level if the player has not put out enough fires
        if current_health <= 0 {
            Trans::Replace(Box::new(MainMenuState::default()))
        } else {
            run_systems(world, &mut self.dispatcher);
            update_covid_state(world, current_health, HEALTH_POOL);
            Trans::None
        }
    }
}
