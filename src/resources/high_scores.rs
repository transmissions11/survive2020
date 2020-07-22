use savefile::prelude::*;

use amethyst::core::ecs::{World, WorldExt};
use savefile_derive::Savefile;
use std::collections::HashMap;

/// Keys for the `high_scores` HashMap.
pub mod highscores_keys {
    pub const WILDFIRES: &str = "wildfires";
    pub const HORNETS: &str = "hornets";
}

#[derive(Default, Savefile)]
pub struct HighScores {
    /// Keys:
    /// wildfires
    /// hornets
    pub high_scores: HashMap<String, u64>,
}

impl HighScores {
    pub fn get_score(&self, key: &str) -> u64 {
        *self.high_scores.get(key).unwrap_or(&0)
    }
}

/// Save HighScores to file.
pub fn save_scores(scores: &HighScores) {
    save_file("high_scores.txt", 0, scores).expect("Couldn't save high scores file.");
}

/// Load HighScores from file.
pub fn load_scores() -> HighScores {
    load_file("high_scores.txt", 0).unwrap_or_default()
}

/// Updates the high score for a level based on it's key (only if the new score is higher!)
pub fn update_high_score_if_greater(world: &mut World, key: &str, new_score: u64) {
    let mut resource = world.write_resource::<HighScores>();

    let current_score = resource.get_score(key);

    if new_score > current_score {
        resource.high_scores.insert(key.to_string(), new_score);

        save_scores(&*resource);
    }
}
