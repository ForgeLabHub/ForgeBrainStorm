//! Power-ups: one-shot abilities a player spends during a round.

use serde::{Deserialize, Serialize};

/// Seconds added to the current round by [`PowerUp::FreezeTime`].
pub const FREEZE_SECS: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerUp {
    /// Doubles the points of the next correct answer.
    DoublePoints,
    /// Adds [`FREEZE_SECS`] to the round clock.
    FreezeTime,
    /// Hides two wrong options from the player who used it.
    RemoveTwoOptions,
}

impl PowerUp {
    /// Hand dealt to every player when a game starts.
    pub const STARTING_HAND: [PowerUp; 3] = [
        PowerUp::DoublePoints,
        PowerUp::FreezeTime,
        PowerUp::RemoveTwoOptions,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            PowerUp::DoublePoints => "double_points",
            PowerUp::FreezeTime => "freeze_time",
            PowerUp::RemoveTwoOptions => "remove_two_options",
        }
    }

    /// Parse the wire name sent by the client.
    pub fn parse(value: &str) -> Option<Self> {
        Self::STARTING_HAND
            .into_iter()
            .find(|kind| kind.as_str() == value)
    }
}

/// Indexes of two wrong options the player may cross out.
///
/// Shuffled, because a stable order would let the answer leak through repeated
/// use. Returns what the question allows: a two-option question only has one
/// wrong option to hide.
pub fn wrong_options(question: &serde_json::Value) -> Vec<usize> {
    use rand::seq::SliceRandom;

    let Some(options) = question["options"].as_array() else {
        return Vec::new();
    };
    let correct = question["correct_index"].as_u64().unwrap_or(0) as usize;
    let mut wrong: Vec<usize> = (0..options.len())
        .filter(|index| *index != correct)
        .collect();
    wrong.shuffle(&mut rand::thread_rng());
    wrong.truncate(2);
    wrong
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn wire_names_round_trip() {
        for kind in PowerUp::STARTING_HAND {
            assert_eq!(PowerUp::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(PowerUp::parse("teleport"), None);
    }

    #[test]
    fn hides_two_wrong_options() {
        let question = json!({"options": ["a", "b", "c", "d"], "correct_index": 2});
        let hidden = wrong_options(&question);
        assert_eq!(hidden.len(), 2);
        assert!(!hidden.contains(&2));
        assert_ne!(hidden[0], hidden[1]);
    }

    #[test]
    fn hides_only_what_exists() {
        let question = json!({"options": ["a", "b"], "correct_index": 0});
        assert_eq!(wrong_options(&question), vec![1]);
        assert!(wrong_options(&json!({})).is_empty());
    }
}
