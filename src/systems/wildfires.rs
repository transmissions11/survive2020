use crate::states::LevelTitle;
use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
};

#[derive(SystemDesc)]
pub struct WildfiresSystem;

impl<'s> System<'s> for WildfiresSystem {
    type SystemData = (
        ReadStorage<'s, LevelTitle>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (level_titles, mut locals, time): Self::SystemData) {
        for (_, local) in (&level_titles, &mut locals).join() {
            local.prepend_translation_x(5. * time.delta_seconds());
            local.prepend_translation_y(5. * time.delta_seconds());
        }
    }
}
