//! Cheater powers: extra abilities available only to accounts with role `cheater`.
//!
//! Every power is spent from a per-game budget the server owns, so a modified
//! client cannot grant itself more uses. Effects that touch other players
//! (stealing points, freezing the clock) are applied by the server and then
//! announced, exactly like ordinary power-ups.

use serde::{Deserialize, Serialize};

/// Points moved from the current leader to the cheater by [`CheatPower::Steal`].
pub const STEAL_POINTS: i64 = 50;
/// Seconds added to the round clock by [`CheatPower::Freeze`].
pub const CHEAT_FREEZE_SECS: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheatPower {
    /// Reveal the correct option to the cheater only.
    Peek,
    /// Add [`CHEAT_FREEZE_SECS`] to the round clock for everyone.
    Freeze,
    /// Take [`STEAL_POINTS`] from the current leader and keep them.
    Steal,
    /// Double the points of the next correct answer.
    Double,
}

impl CheatPower {
    pub const ALL: [CheatPower; 4] = [
        CheatPower::Peek,
        CheatPower::Freeze,
        CheatPower::Steal,
        CheatPower::Double,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            CheatPower::Peek => "peek",
            CheatPower::Freeze => "freeze",
            CheatPower::Steal => "steal",
            CheatPower::Double => "double",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|power| power.as_str() == value)
    }

    /// Uses granted at the start of every game.
    pub fn budget(self) -> u8 {
        match self {
            CheatPower::Peek => 3,
            CheatPower::Freeze => 2,
            CheatPower::Steal => 1,
            CheatPower::Double => 2,
        }
    }
}

/// Remaining uses of each cheater power for one player in one game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheatPowers {
    peek: u8,
    freeze: u8,
    steal: u8,
    double: u8,
}

impl Default for CheatPowers {
    fn default() -> Self {
        Self::new()
    }
}

impl CheatPowers {
    pub fn new() -> Self {
        Self {
            peek: CheatPower::Peek.budget(),
            freeze: CheatPower::Freeze.budget(),
            steal: CheatPower::Steal.budget(),
            double: CheatPower::Double.budget(),
        }
    }

    pub fn remaining(&self, power: CheatPower) -> u8 {
        match power {
            CheatPower::Peek => self.peek,
            CheatPower::Freeze => self.freeze,
            CheatPower::Steal => self.steal,
            CheatPower::Double => self.double,
        }
    }

    /// Spend one use. Returns `false` when the budget is exhausted.
    pub fn spend(&mut self, power: CheatPower) -> bool {
        let slot = match power {
            CheatPower::Peek => &mut self.peek,
            CheatPower::Freeze => &mut self.freeze,
            CheatPower::Steal => &mut self.steal,
            CheatPower::Double => &mut self.double,
        };
        if *slot == 0 {
            return false;
        }
        *slot -= 1;
        true
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Wire representation for the player's private event.
    pub fn to_json(self) -> serde_json::Value {
        serde_json::json!({
            "peek": self.peek,
            "freeze": self.freeze,
            "steal": self.steal,
            "double": self.double,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spending_stops_at_zero() {
        let mut powers = CheatPowers::new();
        assert!(powers.spend(CheatPower::Steal));
        assert!(!powers.spend(CheatPower::Steal));
        assert_eq!(powers.remaining(CheatPower::Steal), 0);
    }

    #[test]
    fn budgets_are_independent() {
        let mut powers = CheatPowers::new();
        while powers.spend(CheatPower::Peek) {}
        assert_eq!(powers.remaining(CheatPower::Peek), 0);
        assert_eq!(powers.remaining(CheatPower::Freeze), 2);
    }

    #[test]
    fn wire_names_round_trip() {
        for power in CheatPower::ALL {
            assert_eq!(CheatPower::parse(power.as_str()), Some(power));
        }
        assert_eq!(CheatPower::parse("nope"), None);
    }

    #[test]
    fn reset_restores_everything() {
        let mut powers = CheatPowers::new();
        powers.spend(CheatPower::Peek);
        powers.spend(CheatPower::Steal);
        powers.reset();
        assert_eq!(powers, CheatPowers::new());
    }
}
