use crate::states::hornets::ProgressBar;
use crate::states::LevelTitle;
use amethyst::{
    core::math::Vector3,
    core::timing::Time,
    core::transform::Transform,
    derive::SystemDesc,
    ecs::prelude::{Join, Read, ReadStorage, System, SystemData, WriteStorage},
};

#[derive(SystemDesc)]
pub struct HornetsSystem;

impl<'s> System<'s> for HornetsSystem {
    type SystemData = (
        WriteStorage<'s, ProgressBar>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
    );

    fn run(&mut self, (mut progress_bars, mut transforms, time): Self::SystemData) {
        for (progress_bar, transform) in (&mut progress_bars, &mut transforms).join() {
            let mut new_percentage = progress_bar.percentage - (time.delta_seconds() / 2.);

            if new_percentage < 0.0 {
                new_percentage = 1.;
            }

            progress_bar.percentage = new_percentage;

            transform.set_translation_xyz(
                // Half the width of the window - half the width of max progress bar times the remaining amount of charge percentage
                (600. * 0.5) - ((1. - new_percentage) * (47. * 0.5)),
                // 10% of the width of the screen - 20 pixels (to align the progress bar with the bottom of the ability frame)
                (600. * 0.1) - 20.,
                0.,
            );
            transform.set_scale(Vector3::new(new_percentage, 1., 1.));
        }
    }
}
