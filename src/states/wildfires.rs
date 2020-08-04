use crate::*;

use crate::systems::wildfires::WildfiresSystem;

use crate::resources::high_scores::highscores_keys::WILDFIRES;

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
use amethyst::core::ecs::DenseVecStorage;
use amethyst::shred::Dispatcher;
use amethyst::ui::{Anchor, LineMode, UiText, UiTransform};

pub const MAX_FIRES: u64 = 60;

/// Tags a component as the wildfire state text.
pub struct WildfiresStateTextComponent;
impl Component for WildfiresStateTextComponent {
    type Storage = DenseVecStorage<Self>;
}

fn init_wildfires_state_text(world: &mut World, max_fires: u64) {
    let font = get_main_font(world);

    let transform = UiTransform::new(
        "wildfire_state".to_string(),
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
        format!("0 FIRES / {} MAX", max_fires),
        [1.0, 1.0, 1.0, 1.0],
        25.0,
        LineMode::Single,
        Anchor::Middle,
    );

    world
        .create_entity()
        .with(WildfiresStateTextComponent)
        .with(LevelComponent)
        .with(transform)
        .with(ui_text)
        .build();
}

fn update_wildfire_state(world: &mut World, current_fires: u64, max_fires: u64) {
    let mut ui_texts = world.write_storage::<UiText>();
    let state_text_components = world.read_storage::<WildfiresStateTextComponent>();

    for (ui_text, _) in (&mut ui_texts, &state_text_components).join() {
        ui_text.text = format!("{} FIRES / {} MAX", current_fires, max_fires);
    }
}

/// A resource for storing some level state for the Wildfires level.
pub struct WildfireStateResource {
    pub current_fires: u64,
    pub stepped_in_fire_times: u64,
}

impl Default for WildfireStateResource {
    fn default() -> Self {
        WildfireStateResource {
            current_fires: 0,
            stepped_in_fire_times: 0,
        }
    }
}

pub struct WildfireState<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
    max_fires: u64,
}
impl<'a, 'b> Default for WildfireState<'a, 'b> {
    fn default() -> Self {
        WildfireState {
            dispatcher: None,
            max_fires: MAX_FIRES,
        }
    }
}

impl<'a, 'b> SimpleState for WildfireState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        init_wildfires_state_text(world, self.max_fires);

        init_level_background(world, "wildfires_background.png");

        init_level_title(world, "wildfires_title.png");

        world.insert(CurrentLevelScoreResource::default());
        world.insert(LevelSecondsResource::default());

        // Init the resource storing data about the player's progress on the level
        world.insert(WildfireStateResource::default());

        let bucket_sprite = load_sprite(world, "bucket_ability.png", 0);
        let tri_shot_sprite = load_sprite(world, "tri_shot_ability.png", 0);
        let range_boost_sprite = load_sprite(world, "range_boost_ability.png", 0);
        init_abilities_bar(
            world,
            AbilitiesResource::new(vec![
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::Bucket,
                        seconds_to_charge: 5,
                        duration: Some(5),
                        icon: bucket_sprite,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::TriShot,
                        seconds_to_charge: 8,
                        duration: Some(6),
                        icon: tri_shot_sprite,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::RangeBoost,
                        seconds_to_charge: 10,
                        duration: Some(7),
                        icon: range_boost_sprite,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
            ]),
        );

        self.dispatcher = create_optional_systems_dispatcher(world, |builder, _| {
            builder.add(WildfiresSystem::default(), "wildfires", &[])
        });
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        delete_all_entities_with_component::<LevelComponent>(world);

        update_high_score_if_greater(world, WILDFIRES);
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

        // Update the max_fires field and the score
        let current_fires = {
            let state = world.read_resource::<WildfireStateResource>();

            // Update the max amount of fires based on how many times the user has stepped in a fire
            self.max_fires = MAX_FIRES.saturating_sub(state.stepped_in_fire_times);

            let mut score = world.write_resource::<CurrentLevelScoreResource>();
            // Update the level score based on seconds elapsed and fires stepped in
            score.score = (seconds_elapsed as u64).saturating_sub(state.stepped_in_fire_times);

            state.current_fires
        };

        // End the level if the player has not put out enough fires
        if current_fires > self.max_fires {
            Trans::Replace(Box::new(MainMenuState::default()))
        } else {
            run_systems(world, &mut self.dispatcher);
            update_wildfire_state(world, current_fires, self.max_fires);
            Trans::None
        }
    }
}
