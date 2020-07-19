use crate::states::{delete_level_title, init_level_title, push_to_level_on_key};
use amethyst::prelude::*;

pub struct HornetState;
impl SimpleState for HornetState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        init_level_title(world, "hornets_title.png");
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        push_to_level_on_key(event, HornetState)
    }
}
