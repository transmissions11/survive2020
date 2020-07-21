use crate::states::{create_optional_systems_dispatcher, init_level_title, run_systems};

use amethyst::{ecs::Dispatcher, prelude::*};

use crate::systems::ability_bar::init_abilities_bar;
use crate::systems::hornets::HornetsSystem;

use crate::resources::abilities::{Abilities, Ability, AbilityInfo, AbilityState, AbilityType};

#[derive(Default)]
pub struct HornetState<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

impl<'a, 'b> SimpleState for HornetState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.dispatcher = create_optional_systems_dispatcher(world, |builder| {
            builder.add(HornetsSystem, "hornets", &[])
        });

        init_level_title(world, "hornets_title.png");

        init_abilities_bar(
            world,
            Abilities {
                current_abilities: vec![
                    Ability {
                        info: AbilityInfo {
                            ability_type: AbilityType::Vaccine,
                            speed: 2,
                            icon: None,
                            max_uses: None,
                        },
                        current_state: AbilityState::default(),
                    },
                    Ability {
                        info: AbilityInfo {
                            ability_type: AbilityType::Vaccine,
                            speed: 4,
                            icon: None,
                            max_uses: None,
                        },
                        current_state: AbilityState::default(),
                    },
                    Ability {
                        info: AbilityInfo {
                            ability_type: AbilityType::Vaccine,
                            speed: 6,
                            icon: None,
                            max_uses: None,
                        },
                        current_state: AbilityState::default(),
                    },
                ],
            },
        );
    }

    // fn handle_event(
    //     &mut self,
    //     mut _data: StateData<'_, GameData<'_, '_>>,
    //     event: StateEvent,
    // ) -> SimpleTrans {
    //     push_to_next_level_on_key(event, TODO::default())
    // }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        run_systems(data.world, &mut self.dispatcher);

        Trans::None
    }
}
