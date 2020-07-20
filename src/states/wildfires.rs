use crate::states::hornets::HornetState;
use crate::states::{
    create_optional_systems_dispatcher, init_camera, init_level_title, push_to_level_on_key,
    run_systems, LevelTitle,
};

use crate::systems::wildfires::WildfiresSystem;
use amethyst::ecs::Dispatcher;
use amethyst::prelude::*;

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

        // Register the LevelTitle component as we won't use this in any systems
        world.register::<LevelTitle>();

        init_camera(world);

        init_level_title(world, "wildfires_title.png");
    }

    fn on_resume(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        init_level_title(data.world, "wildfires_title.png");
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        push_to_level_on_key(event, HornetState::default())
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        run_systems(data.world, &mut self.dispatcher);

        Trans::None
    }
}
