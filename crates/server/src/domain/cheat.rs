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
    /// Take over another player's account: their points move to the cheater
    /// and they spend the rest of the game as a spectator.
    Hijack,
    /// Replace the current question with one the cheater writes. Answers
    /// already given are dropped, so the whole room answers the new text.
    Rewrite,
    /// Silence a player's chat for the rest of the game.
    Mute,
    /// Curse a player: their next answer scores nothing, even when correct.
    Jinx,
}

impl CheatPower {
    pub const ALL: [CheatPower; 8] = [
        CheatPower::Peek,
        CheatPower::Freeze,
        CheatPower::Steal,
        CheatPower::Double,
        CheatPower::Hijack,
        CheatPower::Rewrite,
        CheatPower::Mute,
        CheatPower::Jinx,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            CheatPower::Peek => "peek",
            CheatPower::Freeze => "freeze",
            CheatPower::Steal => "steal",
            CheatPower::Double => "double",
            CheatPower::Hijack => "hijack",
            CheatPower::Rewrite => "rewrite",
            CheatPower::Mute => "mute",
            CheatPower::Jinx => "jinx",
        }
    }

    /// Powers that need another player picked as the victim.
    pub fn needs_target(self) -> bool {
        matches!(
            self,
            CheatPower::Hijack | CheatPower::Mute | CheatPower::Jinx
        )
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
            CheatPower::Hijack => 1,
            CheatPower::Rewrite => 2,
            CheatPower::Mute => 2,
            CheatPower::Jinx => 2,
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
    hijack: u8,
    rewrite: u8,
    mute: u8,
    jinx: u8,
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
            hijack: CheatPower::Hijack.budget(),
            rewrite: CheatPower::Rewrite.budget(),
            mute: CheatPower::Mute.budget(),
            jinx: CheatPower::Jinx.budget(),
        }
    }

    pub fn remaining(&self, power: CheatPower) -> u8 {
        match power {
            CheatPower::Peek => self.peek,
            CheatPower::Freeze => self.freeze,
            CheatPower::Steal => self.steal,
            CheatPower::Double => self.double,
            CheatPower::Hijack => self.hijack,
            CheatPower::Rewrite => self.rewrite,
            CheatPower::Mute => self.mute,
            CheatPower::Jinx => self.jinx,
        }
    }

    /// Spend one use. Returns `false` when the budget is exhausted.
    pub fn spend(&mut self, power: CheatPower) -> bool {
        let slot = match power {
            CheatPower::Peek => &mut self.peek,
            CheatPower::Freeze => &mut self.freeze,
            CheatPower::Steal => &mut self.steal,
            CheatPower::Double => &mut self.double,
            CheatPower::Hijack => &mut self.hijack,
            CheatPower::Rewrite => &mut self.rewrite,
            CheatPower::Mute => &mut self.mute,
            CheatPower::Jinx => &mut self.jinx,
        };
        if *slot == 0 {
            return false;
        }
        *slot -= 1;
        true
    }

    /// Give one use back: used when the target turned out to be invalid, so a
    /// mistyped name does not burn the budget.
    pub fn refund(&mut self, power: CheatPower) {
        let slot = match power {
            CheatPower::Peek => &mut self.peek,
            CheatPower::Freeze => &mut self.freeze,
            CheatPower::Steal => &mut self.steal,
            CheatPower::Double => &mut self.double,
            CheatPower::Hijack => &mut self.hijack,
            CheatPower::Rewrite => &mut self.rewrite,
            CheatPower::Mute => &mut self.mute,
            CheatPower::Jinx => &mut self.jinx,
        };
        *slot = slot.saturating_add(1).min(power.budget());
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Wire representation for the player's private event.
    pub fn to_json(self) -> serde_json::Value {
        let mut powers = serde_json::Map::new();
        for power in CheatPower::ALL {
            powers.insert(power.as_str().to_owned(), self.remaining(power).into());
        }
        serde_json::Value::Object(powers)
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

    #[test]
    fn the_aggressive_powers_are_rationed() {
        let mut powers = CheatPowers::new();
        assert_eq!(powers.remaining(CheatPower::Hijack), 1);
        assert!(powers.spend(CheatPower::Hijack));
        assert!(!powers.spend(CheatPower::Hijack));
        // Taking an account must not eat the other budgets.
        assert_eq!(powers.remaining(CheatPower::Rewrite), 2);
        assert_eq!(powers.remaining(CheatPower::Mute), 2);
        assert_eq!(powers.remaining(CheatPower::Jinx), 2);
    }

    #[test]
    fn only_victim_powers_ask_for_a_target() {
        for power in CheatPower::ALL {
            let expected = matches!(
                power,
                CheatPower::Hijack | CheatPower::Mute | CheatPower::Jinx
            );
            assert_eq!(power.needs_target(), expected, "{power:?}");
        }
    }

    #[test]
    fn a_refund_cannot_exceed_the_starting_budget() {
        let mut powers = CheatPowers::new();
        powers.refund(CheatPower::Hijack);
        assert_eq!(powers.remaining(CheatPower::Hijack), 1);
    }

    #[test]
    fn the_wire_budget_covers_every_power() {
        let json = CheatPowers::new().to_json();
        for power in CheatPower::ALL {
            assert!(json[power.as_str()].is_u64(), "missing {power:?}");
        }
        assert_eq!(
            json.as_object().map(|map| map.len()),
            Some(CheatPower::ALL.len())
        );
    }
}
