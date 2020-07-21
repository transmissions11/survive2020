use amethyst::core::ecs::{System, SystemData};

use amethyst::derive::SystemDesc;

#[derive(SystemDesc)]
pub struct HornetsSystem;

impl<'s> System<'s> for HornetsSystem {
    type SystemData = ();

    fn run(&mut self, _data: Self::SystemData) {}
}
