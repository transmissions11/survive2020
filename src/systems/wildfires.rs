use amethyst::{
    derive::SystemDesc,
    ecs::prelude::{System, SystemData},
};

#[derive(SystemDesc)]
pub struct WildfiresSystem;

impl<'s> System<'s> for WildfiresSystem {
    type SystemData = ();

    fn run(&mut self, _data: Self::SystemData) {}
}
