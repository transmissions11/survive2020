use crate::states::hornets::HornetState;
use crate::states::{init_camera, init_level_title, push_to_level_on_key, LevelTitle};
use amethyst::prelude::*;

pub struct WildfireState;
impl SimpleState for WildfireState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        // Register the LevelTitle component as we won't use this in any systems
        world.register::<LevelTitle>();

        init_camera(world);

        init_level_title(world, "wildfires_title.png");
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        push_to_level_on_key(event, HornetState)
    }
}
