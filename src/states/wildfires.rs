use crate::*;

use crate::resources::abilities::{
    AbilitiesResource, Ability, AbilityInfo, AbilityState, AbilityType,
};
use crate::systems::ability_bar::{init_abilities_bar, AbilityBarComponent};
use crate::systems::wildfires::WildfiresSystem;

use crate::resources::high_scores::highscores_keys::WILDFIRES;

use crate::states::{
    create_optional_systems_dispatcher, init_level_title, init_timer_text,
    return_to_main_menu_on_escape, update_timer_and_set_high_score, TimerComponent,
};
use amethyst::shred::Dispatcher;

pub const MAX_SECONDS: f32 = 10.0;

#[derive(Default)]
pub struct WildfireState<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
    seconds_elapsed: f32,
    score: u64,
}

impl<'a, 'b> SimpleState for WildfireState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        init_level_title(world, "wildfires_title.png");

        init_timer_text(world, MAX_SECONDS);

        let vaccine_sprite = load_sprite(world, "vaccine_ability.png", 0);
        init_abilities_bar(
            world,
            AbilitiesResource::new(vec![Ability {
                info: AbilityInfo {
                    ability_type: AbilityType::Vaccine,
                    seconds_to_charge: 1,
                    duration: Some(1),
                    icon: vaccine_sprite,
                    max_uses: Some(5),
                },
                current_state: AbilityState::start_on_cooldown(),
            }]),
        );

        self.dispatcher = create_optional_systems_dispatcher(world, |builder| {
            builder.add(WildfiresSystem, "wildfires", &[])
        });
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        delete_all_entities_with_component::<AbilityBarComponent>(data.world);
        delete_all_entities_with_component::<TimerComponent>(data.world);
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

        update_timer_and_set_high_score(
            *world,
            &mut self.seconds_elapsed,
            MAX_SECONDS,
            self.score,
            WILDFIRES,
        )
    }
}
