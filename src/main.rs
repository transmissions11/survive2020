use amethyst::audio::AudioBundle;
use amethyst::input::{InputBundle, StringBindings};
use amethyst::renderer::RenderFlat2D;
use amethyst::{
    core::transform::TransformBundle,
    prelude::*,
    renderer::{plugins::RenderToWindow, types::DefaultBackend, RenderingBundle},
    ui::{RenderUi, UiBundle},
    utils::application_root_dir,
};

use amethyst::audio::DjSystemDesc;

use survive2020::audio::MusicResource;
use survive2020::states::main_menu::MainMenuState;
use survive2020::systems::ability_bar::AbilityBarSystemDesc;

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let resources = app_root.join("assets");
    let display_config = resources.join("display_config.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(UiBundle::<StringBindings>::new())?
        .with_bundle(AudioBundle::default())?
        // Background music
        .with_system_desc(
            DjSystemDesc::new(|music: &mut MusicResource| music.music.next()),
            "background_music",
            &[],
        )
        .with_system_desc(AbilityBarSystemDesc::default(), "ability_bar", &[])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config)?.with_clear([0., 0., 0., 1.0]),
                )
                .with_plugin(RenderFlat2D::default())
                .with_plugin(RenderUi::default()),
        )?;

    let mut game = Application::new(resources, MainMenuState::default(), game_data)?;
    game.run();

    Ok(())
}
