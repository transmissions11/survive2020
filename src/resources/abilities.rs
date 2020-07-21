use amethyst::renderer::SpriteRender;
use amethyst::ui::UiButton;

pub struct Abilities {
    pub current_abilities: Vec<Ability>,
}

pub struct Ability {
    pub info: AbilityInfo,
    pub current_state: AbilityState,
}

pub enum AbilityType {
    Vaccine,
}

pub struct AbilityInfo {
    pub speed: u32,
    pub ability_type: AbilityType,
    pub icon: Option<SpriteRender>,
    pub max_uses: Option<u32>,
}

pub struct AbilityState {
    pub percentage: f32,
    pub ui_button: Option<UiButton>,
    pub uses: u32,
}

impl AbilityState {
    pub fn default() -> Self {
        AbilityState {
            percentage: 1.0,
            ui_button: None,
            uses: 0,
        }
    }
    pub fn start_on_cooldown() -> Self {
        AbilityState {
            percentage: 0.0,
            ui_button: None,
            uses: 0,
        }
    }
}
