use amethyst::renderer::SpriteRender;
use amethyst::ui::UiButton;

/// All available abilities and all active abilities.
#[derive(Default)]
pub struct AbilitiesResource {
    /// All available abilities.
    pub available_abilities: Vec<Ability>,
    /// All active abilities (their index in available_abilities).
    pub active_abilities: Vec<usize>,
}

impl AbilitiesResource {
    pub fn new(available_abilities: Vec<Ability>) -> Self {
        AbilitiesResource {
            available_abilities,
            active_abilities: vec![],
        }
    }
}

/// The ability's info and it's current state.
pub struct Ability {
    pub info: AbilityInfo,
    pub current_state: AbilityState,
}

/// Type of ability.
pub enum AbilityType {
    Vaccine,
    FlySwatter,
    BugSpray,
}

/// Information about an ability.
pub struct AbilityInfo {
    /// Seconds for the ability to charge.
    pub seconds_to_charge: u32,
    /// Seconds for the ability to be active (and be stored in the current abilities vector).
    /// If there is no duration,
    /// **the System managing this ability will have to
    /// manually remove it from the `active_abilities` vector.**
    pub duration: Option<u32>,
    /// The type of ability.
    pub ability_type: AbilityType,
    /// The icon to be shown for this ability.
    pub icon: SpriteRender,
    /// The maximum amount of times this ability can be used.
    pub max_uses: Option<u32>,
}

/// The current state of an ability.
pub struct AbilityState {
    /// Charge/duration percentage.
    pub percentage: f32,
    /// The UI button that can be tapped to trigger the ability.
    pub ui_button: Option<UiButton>,
    /// How many times the ability has been used.
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
