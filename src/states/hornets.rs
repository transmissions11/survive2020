use crate::states::{
    create_optional_systems_dispatcher, init_level_title, load_sprite, push_to_level_on_key,
    run_systems,
};

use amethyst::ecs::{Component, DenseVecStorage};
use amethyst::{
    core::math::Vector3, core::transform::Transform, ecs::Dispatcher, prelude::*,
    window::ScreenDimensions,
};

use crate::systems::hornets::HornetsSystem;

#[derive(Debug, Default)]
pub struct ProgressBar {
    pub(crate) percentage: f32,
}
impl Component for ProgressBar {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct HornetState<'a, 'b> {
    dispatcher: Option<Dispatcher<'a, 'b>>,
}

fn init_abilities_bar(world: &mut World, charged_percentage: f32) {
    assert!(
        charged_percentage <= 1. && charged_percentage >= 0.,
        "Charge Percentage must be between 1 and 0!"
    );

    let dimensions = (*world.read_resource::<ScreenDimensions>()).clone();

    let ability_frame = load_sprite(world, "ability_frame.png", 0);

    let mut ability_frame_transform = Transform::default();
    ability_frame_transform.set_translation_xyz(
        dimensions.width() * 0.5,
        dimensions.height() * 0.1,
        0.,
    );

    world
        .create_entity()
        .with(ability_frame_transform)
        .with(ability_frame)
        .build();

    let progress_bar = load_sprite(world, "progress_bar.png", 0);
    let mut progress_bar_transform = Transform::default();
    progress_bar_transform.set_translation_xyz(
        // Half the width of the window - half the width of max progress bar times the remaining amount of charge percentage
        (dimensions.width() * 0.5) - ((1. - charged_percentage) * (47. * 0.5)),
        // 10% of the width of the screen - 20 pixels (to align the progress bar with the bottom of the ability frame)
        (dimensions.height() * 0.1) - 20.,
        0.,
    );
    progress_bar_transform.set_scale(Vector3::new(charged_percentage, 1., 1.));

    world
        .create_entity()
        .with(progress_bar_transform)
        .with(progress_bar)
        .with(ProgressBar {
            percentage: charged_percentage,
        })
        .build();
}

impl<'a, 'b> SimpleState for HornetState<'a, 'b> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        self.dispatcher = create_optional_systems_dispatcher(world, |builder| {
            builder.add(HornetsSystem, "hornets", &[])
        });

        init_level_title(world, "hornets_title.png");

        init_abilities_bar(world, 1.);
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
