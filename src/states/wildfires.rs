use crate::*;

use crate::resources::abilities::{
    AbilitiesResource, Ability, AbilityInfo, AbilityState, AbilityType,
};
use crate::systems::ability_bar::{init_abilities_bar, AbilityBarComponent};
use crate::systems::wildfires::WildfiresSystem;

use amethyst::prelude::*;
use amethyst::shred::Dispatcher;

#[derive(Default)]
pub struct WildfireState<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for WildfireState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.dispatcher = create_optional_systems_dispatcher(world, |builder| {
            builder.add(WildfiresSystem, "wildfires", &[])
        });

        init_level_title(world, "wildfires_title.png");

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
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        delete_all_entities_with_component::<AbilityBarComponent>(data.world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        run_systems(data.world, &mut self.dispatcher);

        Trans::None
    }

    fn handle_event(
        &mut self,
        _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        return_to_main_menu_on_escape(event)
    }
}
