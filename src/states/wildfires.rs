use crate::*;

use crate::systems::wildfires::WildfiresSystem;

use crate::resources::high_scores::highscores_keys::WILDFIRES;

use crate::resources::high_scores::CurrentLevelScoreResource;
use crate::states::{
    create_optional_systems_dispatcher, init_level_background, init_level_title,
    init_timer_and_score_text, return_to_main_menu_on_escape, run_systems,
    update_timer_and_set_high_score, LevelComponent,
};
use amethyst::shred::Dispatcher;

pub const MAX_SECONDS: f32 = 60.0 * 5.0;

#[derive(Default)]
pub struct WildfireState<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
    seconds_elapsed: f32,
}

impl<'a, 'b> SimpleState for WildfireState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        init_level_background(world, "wildfires_background.png");

        init_level_title(world, "wildfires_title.png");

        init_timer_and_score_text(world, MAX_SECONDS);

        // Init the current level score.
        world.insert(CurrentLevelScoreResource::default());

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
        delete_all_entities_with_component::<LevelComponent>(data.world);
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

        {
            // Set score to seconds elapsed
            let mut score = world.write_resource::<CurrentLevelScoreResource>();
            score.score = self.seconds_elapsed as u64;
        }

        run_systems(world, &mut self.dispatcher);

        update_timer_and_set_high_score(*world, &mut self.seconds_elapsed, MAX_SECONDS, WILDFIRES)
    }
}
