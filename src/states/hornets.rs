use crate::states::{
    create_optional_systems_dispatcher, init_level_title, push_to_level_on_key, run_systems,
};
use crate::systems::hornets::HornetsSystem;
use amethyst::ecs::Dispatcher;
use amethyst::prelude::*;

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
    }

    fn on_resume(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        init_level_title(data.world, "hornets_title.png");
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
