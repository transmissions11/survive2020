use crate::*;

use crate::systems::wildfires::WildfiresSystem;

use crate::resources::high_scores::highscores_keys::WILDFIRES;

use crate::resources::high_scores::{update_high_score_if_greater, CurrentLevelScoreResource};
use crate::states::main_menu::MainMenuState;
use crate::states::{
    create_optional_systems_dispatcher, init_level_background, init_level_title,
    return_to_main_menu_on_escape, run_systems, LevelComponent,
};

use amethyst::shred::Dispatcher;

pub const MAX_SECONDS: f32 = 60.0 * 5.0;

pub const MAX_FIRES: u64 = 60;

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
    seconds_elapsed: f32,
    max_fires: u64,
}
impl<'a, 'b> Default for WildfireState<'a, 'b> {
    fn default() -> Self {
        WildfireState {
            dispatcher: None,
            seconds_elapsed: 0.,
            max_fires: MAX_FIRES,
        }
    }
}

impl<'a, 'b> SimpleState for WildfireState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        init_level_background(world, "wildfires_background.png");

        init_level_title(world, "wildfires_title.png");

        // Init the current level score.
        world.insert(CurrentLevelScoreResource::default());

        // Init the resource storing data about the player's progress on the level
        world.insert(WildfireStateResource::default());

        // let vaccine_sprite = load_sprite(world, "vaccine_ability.png", 0);
        // init_abilities_bar(
        //     world,
        //     AbilitiesResource::new(vec![Ability {
        //         info: AbilityInfo {
        //             ability_type: AbilityType::Vaccine,
        //             seconds_to_charge: 1,
        //             duration: Some(1),
        //             icon: vaccine_sprite,
        //             max_uses: Some(5),
        //         },
        //         current_state: AbilityState::start_on_cooldown(),
        //     }]),
        // );

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

        // Update the max_fires field and the score
        let current_fires = {
            let state = world.read_resource::<WildfireStateResource>();

            // Update the max amount of fires based on how many times the user has stepped in a fire
            self.max_fires = MAX_FIRES.saturating_sub(state.stepped_in_fire_times);

            let mut score = world.write_resource::<CurrentLevelScoreResource>();
            // Update the level score based on seconds elapsed and fires stepped in
            score.score = (self.seconds_elapsed as u64).saturating_sub(state.stepped_in_fire_times);

            println!("stepped: {}", state.stepped_in_fire_times);
            println!("{}/{}", state.current_fires, self.max_fires);

            state.current_fires
        };

        // End the level if the player has not put out enough fires
        if current_fires > self.max_fires {
            Trans::Replace(Box::new(MainMenuState::default()))
        } else {
            run_systems(world, &mut self.dispatcher);
            Trans::None
        }
    }
}
