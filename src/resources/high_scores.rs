use savefile::prelude::*;

use amethyst::core::ecs::{World, WorldExt};
use savefile_derive::Savefile;
use std::collections::HashMap;

/// Keys for the `high_scores` HashMap.
pub mod highscores_keys {
    pub const WILDFIRES: &str = "wildfires";
    pub const HORNETS: &str = "hornets";
}

/// Each level will create this resource when it starts.
#[derive(Default)]
pub struct CurrentLevelScoreResource {
    pub(crate) score: u64,
}

#[derive(Default, Savefile)]
pub struct HighScoresResource {
    /// Keys:
    /// wildfires
    /// hornets
    pub high_scores: HashMap<String, u64>,
}

impl HighScoresResource {
    pub fn get_score(&self, key: &str) -> u64 {
        *self.high_scores.get(key).unwrap_or(&0)
    }
}

/// Save HighScores to file.
pub fn save_scores(scores: &HighScoresResource) {
    save_file("high_scores.txt", 0, scores).expect("Couldn't save high scores file.");
}

/// Load HighScores from file.
pub fn load_scores() -> HighScoresResource {
    load_file("high_scores.txt", 0).unwrap_or_default()
}

/// Updates the high score for a level based on it's key (only if the new score is higher!)
pub fn update_high_score_if_greater(world: &mut World, key: &str) {
    let new_score = world.write_resource::<CurrentLevelScoreResource>();

    let mut resource = world.write_resource::<HighScoresResource>();

    let past_score = resource.get_score(key);

    if new_score.score > past_score {
        resource
            .high_scores
            .insert(key.to_string(), new_score.score);

        save_scores(&*resource);
    }
}
