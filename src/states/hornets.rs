use amethyst::ecs::Dispatcher;

use crate::systems::ability_bar::init_abilities_bar;
use crate::systems::hornets::HornetsSystemDesc;

use crate::resources::abilities::{
    AbilitiesResource, Ability, AbilityInfo, AbilityState, AbilityType,
};
use crate::resources::high_scores::highscores_keys::HORNETS;
use crate::resources::high_scores::CurrentLevelScoreResource;
use crate::states::{
    create_optional_systems_dispatcher, init_level_background, init_level_title,
    init_timer_and_score_text, return_to_main_menu_on_escape, run_systems,
    update_timer_and_set_high_score, LevelComponent, LevelSecondsResource,
};
use crate::*;

pub const MAX_SECONDS: f32 = 60.0 * 2.5;

#[derive(Default)]
pub struct HornetState<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for HornetState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        init_level_background(world, "hornets_background.png");

        init_level_title(world, "hornets_title.png");

        init_timer_and_score_text(world, MAX_SECONDS);

        world.insert(CurrentLevelScoreResource::default());
        world.insert(LevelSecondsResource::default());

        let bug_spray_sprite = load_sprite(world, "bug_spray_ability.png", 0);
        let swatter_sprite = load_sprite(world, "swatter_ability.png", 0);
        let hive_trap_sprite = load_sprite(world, "hive_trap_ability.png", 0);
        init_abilities_bar(
            world,
            AbilitiesResource::new(vec![
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::BugSpray,
                        seconds_to_charge: 20,
                        duration: None,
                        icon: bug_spray_sprite,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::FlySwatter,
                        seconds_to_charge: 15,
                        duration: Some(4),
                        icon: swatter_sprite,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
                Ability {
                    info: AbilityInfo {
                        ability_type: AbilityType::HiveTrap,
                        seconds_to_charge: 7,
                        duration: Some(4),
                        icon: hive_trap_sprite,
                        max_uses: None,
                    },
                    current_state: AbilityState::default(),
                },
            ]),
        );

        self.dispatcher = create_optional_systems_dispatcher(world, |builder, world| {
            builder.add(
                HornetsSystemDesc {
                    bee_texture: None,
                    swatter: None,
                    hive: None,
                }
                .build(world),
                "hornets",
                &[],
            );
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

        run_systems(world, &mut self.dispatcher);

        update_timer_and_set_high_score(world, MAX_SECONDS, HORNETS)
    }
}
