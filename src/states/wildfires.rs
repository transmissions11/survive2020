use crate::states::hornets::HornetState;
use crate::states::{
    create_optional_systems_dispatcher, init_camera, init_level_title, push_to_next_level_on_key,
    run_systems, LevelTitle,
};

use crate::resources::abilities::{
    AbilitiesResource, Ability, AbilityInfo, AbilityState, AbilityType,
};
use crate::systems::ability_bar::{init_abilities_bar, AbilityBarComponent, ProgressBar};
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

        // Register the components we won't use this in any systems
        world.register::<LevelTitle>();
        world.register::<AbilityBarComponent>();
        world.register::<ProgressBar>();

        init_camera(world);

        init_level_title(world, "wildfires_title.png");

        init_abilities_bar(
            world,
            AbilitiesResource::new(vec![Ability {
                info: AbilityInfo {
                    ability_type: AbilityType::Vaccine,
                    seconds_to_charge: 5,
                    duration: Some(5),
                    icon: None,
                    max_uses: Some(5),
                },
                current_state: AbilityState::start_on_cooldown(),
            }]),
        );
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        push_to_next_level_on_key(event, HornetState::default())
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        run_systems(data.world, &mut self.dispatcher);

        Trans::None
    }
}
