use savefile::prelude::*;

use savefile_derive::Savefile;

#[derive(Default, Savefile)]
pub struct HighScores {
    pub wildfires_high_score: u64,
    pub hornets_high_score: u64,
}

pub fn save_scores(scores: &HighScores) {
    save_file("high_scores.txt", 0, scores).expect("Couldn't save high scores file.");
}

pub fn load_scores() -> HighScores {
    load_file("high_scores.txt", 0).unwrap_or_default()
}
