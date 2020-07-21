use amethyst::{
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
};

#[derive(SystemDesc)]
pub struct WildfiresSystem;

impl<'s> System<'s> for WildfiresSystem {
    type SystemData = ();

    fn run(&mut self, _data: Self::SystemData) {}
}
