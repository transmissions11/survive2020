use amethyst::ecs::Dispatcher;

use crate::systems::ability_bar::{init_abilities_bar, AbilityBarComponent};
use crate::systems::hornets::{Bee, HornetsSystemDesc};

use crate::resources::abilities::{
    AbilitiesResource, Ability, AbilityInfo, AbilityState, AbilityType,
};
use crate::resources::high_scores::highscores_keys::HORNETS;
use crate::resources::high_scores::CurrentLevelScoreResource;
use crate::states::{
    create_optional_systems_dispatcher, init_level_title, init_timer_and_score_text,
    return_to_main_menu_on_escape, run_systems, update_timer_and_set_high_score, TimerComponent,
};
use crate::*;

pub const MAX_SECONDS: f32 = 60.0 * 2.5;

#[derive(Default)]
pub struct HornetState<'a, 'b> {
    seconds_elapsed: f32,
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for HornetState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Init the current level score.
        world.insert(CurrentLevelScoreResource::default());

        init_level_title(world, "hornets_title.png");

        init_timer_and_score_text(world, MAX_SECONDS);

        let bug_spray_sprite = load_sprite(world, "bug_spray_ability.png", 0);
        let swatter_sprite = load_sprite(world, "swatter_ability.png", 0);
        init_abilities_bar(
            world,
            AbilitiesResource::new(vec![
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::BugSpray,
                        seconds_to_charge: 60,
                        duration: None,
                        icon: bug_spray_sprite,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::FlySwatter,
                        seconds_to_charge: 30,
                        duration: Some(4),
                        icon: swatter_sprite,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
            ]),
        );

        self.dispatcher = create_optional_systems_dispatcher(world, |builder, world| {
            builder.add(
                HornetsSystemDesc { bee_texture: None }.build(world),
                "hornets",
                &[],
            );
        });
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        delete_all_entities_with_component::<AbilityBarComponent>(data.world);
        delete_all_entities_with_component::<TimerComponent>(data.world);
        delete_all_entities_with_component::<Bee>(data.world);
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

        run_systems(world, &mut self.dispatcher);

        update_timer_and_set_high_score(world, &mut self.seconds_elapsed, MAX_SECONDS, HORNETS)
    }
}
