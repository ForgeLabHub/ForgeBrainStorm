//! Player model and in-room state.

use std::time::Instant;

use tokio::sync::mpsc::UnboundedSender;

use crate::domain::cheat::CheatPowers;
use crate::domain::mode::{GameMode, SURVIVAL_LIVES};
use crate::domain::power_up::PowerUp;

/// A player within a room.
#[derive(Debug, Clone)]
pub struct Player {
    pub sid: String,
    pub user_id: Option<i64>,
    pub username: String,
    pub display_name: String,
    pub score: i64,
    pub streak: u32,
    pub total_correct: u32,
    pub answered: bool,
    pub answer_index: Option<usize>,
    pub answer_time: Option<Instant>,
    pub lives: u8,
    pub infinite_lives: bool,
    pub is_spectator: bool,
    /// Account has the `cheater` role; the server, never the client, sets this.
    pub is_cheat: bool,
    /// Cheater powers left in this game.
    pub cheat: CheatPowers,
    pub team: Option<u8>,
    pub joined_at: Instant,
    pub disconnected_at: Option<Instant>,
    /// Power-ups still in hand for this game.
    pub power_ups: Vec<PowerUp>,
    /// Armed by `DoublePoints`, spent by the next correct answer.
    pub double_points: bool,
    /// Chat silenced by a cheater for the rest of the game.
    pub muted: bool,
    /// Cursed by a cheater: the next answer scores nothing even when correct.
    pub jinxed: bool,
    /// Account taken over by a cheater; the player watches as a spectator.
    pub hijacked: bool,
    /// Until when a cheater's `blind` keeps this player from answering.
    pub blind_until: Option<Instant>,
    /// This player's own socket. Private events (a hand, hidden options) must
    /// not go through the room broadcast or everyone would see them.
    pub outbox: Option<UnboundedSender<String>>,
}

/// State that must survive a page reload. A reload opens a new socket, and
/// handing out a fresh cheat budget (or lifting a curse) on reconnect would be
/// a free exploit, so the room copies this across instead of starting over.
#[derive(Debug, Clone)]
pub struct ReconnectState {
    pub score: i64,
    pub streak: u32,
    pub total_correct: u32,
    pub lives: u8,
    pub power_ups: Vec<PowerUp>,
    pub double_points: bool,
    pub is_spectator: bool,
    pub infinite_lives: bool,
    pub muted: bool,
    pub jinxed: bool,
    pub hijacked: bool,
    pub blind_until: Option<Instant>,
    pub cheat: CheatPowers,
    pub team: Option<u8>,
}

impl Player {
    pub fn new(
        sid: String,
        display_name: String,
        user_id: Option<i64>,
        username: Option<String>,
    ) -> Self {
        Self {
            sid,
            user_id,
            username: username.unwrap_or_default(),
            display_name,
            score: 0,
            streak: 0,
            total_correct: 0,
            answered: false,
            answer_index: None,
            answer_time: None,
            lives: SURVIVAL_LIVES,
            infinite_lives: false,
            is_spectator: false,
            is_cheat: false,
            cheat: CheatPowers::new(),
            team: None,
            joined_at: Instant::now(),
            disconnected_at: None,
            power_ups: Vec::new(),
            double_points: false,
            muted: false,
            jinxed: false,
            hijacked: false,
            blind_until: None,
            outbox: None,
        }
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected_at.is_some()
    }

    /// True while a `blind` power is in force. The deadline lives on the
    /// server, so a client that ignores the event still cannot answer.
    pub fn is_blind(&self) -> bool {
        self.blind_until.is_some_and(|until| until > Instant::now())
    }

    /// Apply an answer and return the points it earned.
    ///
    /// `speed` is the share of the clock left when the answer landed; only
    /// speed-scored modes use it. An armed `DoublePoints` is consumed here,
    /// whether the answer is right or not: the power-up buys one attempt, not
    /// one guaranteed hit.
    pub fn award(&mut self, correct: bool, mode: GameMode, speed: f64) -> i64 {
        // A curse is spent on the first answer after it lands, right or wrong.
        // An armed double survives it, so the player is not robbed of both.
        if std::mem::take(&mut self.jinxed) {
            self.streak = 0;
            if mode.lives().is_some() {
                self.lives = self.lives.saturating_sub(1);
            }
            return 0;
        }
        let doubled = std::mem::take(&mut self.double_points);
        if !correct {
            self.streak = 0;
            if mode.lives().is_some() {
                self.lives = self.lives.saturating_sub(1);
            }
            return 0;
        }
        self.streak += 1;
        self.total_correct += 1;
        let mut points = mode.points(speed, self.streak);
        if doubled {
            points *= 2;
        }
        self.score += points;
        points
    }

    /// Take `kind` out of the hand; false when it is not there.
    pub fn spend(&mut self, kind: PowerUp) -> bool {
        match self.power_ups.iter().position(|owned| *owned == kind) {
            Some(index) => {
                self.power_ups.remove(index);
                true
            }
            None => false,
        }
    }

    /// Reset everything that belongs to a single game, and deal a new hand.
    pub fn reset_for_new_game(&mut self) {
        self.score = 0;
        self.streak = 0;
        self.total_correct = 0;
        self.answered = false;
        self.answer_index = None;
        self.answer_time = None;
        self.lives = SURVIVAL_LIVES;
        self.is_spectator = false;
        self.double_points = false;
        self.muted = false;
        self.jinxed = false;
        self.hijacked = false;
        self.blind_until = None;
        self.power_ups = PowerUp::STARTING_HAND.to_vec();
        self.cheat.reset();
    }

    pub fn reconnect_state(&self) -> ReconnectState {
        ReconnectState {
            score: self.score,
            streak: self.streak,
            total_correct: self.total_correct,
            lives: self.lives,
            power_ups: self.power_ups.clone(),
            double_points: self.double_points,
            is_spectator: self.is_spectator,
            infinite_lives: self.infinite_lives,
            muted: self.muted,
            jinxed: self.jinxed,
            hijacked: self.hijacked,
            blind_until: self.blind_until,
            cheat: self.cheat,
            team: self.team,
        }
    }

    pub fn apply_reconnect(&mut self, state: ReconnectState) {
        self.score = state.score;
        self.streak = state.streak;
        self.total_correct = state.total_correct;
        self.lives = state.lives;
        self.power_ups = state.power_ups;
        self.double_points = state.double_points;
        self.is_spectator = state.is_spectator;
        self.infinite_lives = state.infinite_lives;
        self.muted = state.muted;
        self.jinxed = state.jinxed;
        self.hijacked = state.hijacked;
        self.blind_until = state.blind_until;
        self.cheat = state.cheat;
        self.team = state.team;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player() -> Player {
        Player::new("sid".into(), "Аня".into(), Some(1), Some("anya".into()))
    }

    #[test]
    fn a_survival_game_takes_one_life_per_wrong_answer() {
        let mut p = player();
        for left in [2, 1, 0] {
            p.award(false, GameMode::Lives, 1.0);
            assert_eq!(p.lives, left);
        }
    }

    #[test]
    fn a_classic_game_never_touches_lives() {
        let mut p = player();
        p.award(false, GameMode::Classic, 1.0);
        assert_eq!(p.lives, SURVIVAL_LIVES);
    }

    #[test]
    fn a_new_game_brings_a_knocked_out_player_back() {
        let mut p = player();
        p.lives = 0;
        p.is_spectator = true;
        p.score = 400;

        p.reset_for_new_game();
        assert_eq!(p.lives, SURVIVAL_LIVES);
        assert!(!p.is_spectator);
        assert_eq!(p.score, 0);
    }

    #[test]
    fn double_points_doubles_the_next_correct_answer() {
        let mut p = player();
        p.double_points = true;
        assert_eq!(p.award(true, GameMode::Classic, 1.0), 200);
        assert_eq!(p.score, 200);
        // Spent: the answer after it scores normally again.
        assert_eq!(p.award(true, GameMode::Classic, 1.0), 100);
        assert_eq!(p.score, 300);
    }

    #[test]
    fn a_cursed_player_scores_nothing_even_when_right() {
        let mut p = player();
        p.jinxed = true;
        assert_eq!(p.award(true, GameMode::Lives, 1.0), 0);
        assert_eq!(p.score, 0);
        assert_eq!(p.lives, 2);
        assert_eq!(p.streak, 0);
        // The curse is spent: the next answer scores normally.
        assert!(p.award(true, GameMode::Lives, 1.0) > 0);
    }

    #[test]
    fn a_curse_does_not_swallow_an_armed_double() {
        let mut p = player();
        p.jinxed = true;
        p.double_points = true;
        assert_eq!(p.award(true, GameMode::Classic, 1.0), 0);
        // The double is still armed and pays out on the next correct answer.
        assert!(p.double_points);
        assert_eq!(p.award(true, GameMode::Classic, 1.0), 200);
    }

    #[test]
    fn a_new_game_clears_mute_curse_and_hijack() {
        let mut p = player();
        p.muted = true;
        p.jinxed = true;
        p.hijacked = true;
        p.reset_for_new_game();
        assert!(!p.muted && !p.jinxed && !p.hijacked);
    }

    #[test]
    fn double_points_is_lost_on_a_wrong_answer() {
        let mut p = player();
        p.double_points = true;
        assert_eq!(p.award(false, GameMode::Classic, 1.0), 0);
        assert_eq!(p.award(true, GameMode::Classic, 1.0), 100);
    }

    #[test]
    fn wrong_answer_resets_streak_and_costs_a_life_in_lives_mode() {
        let mut p = player();
        p.award(true, GameMode::Lives, 1.0);
        p.award(true, GameMode::Lives, 1.0);
        assert_eq!(p.streak, 2);
        p.award(false, GameMode::Lives, 1.0);
        assert_eq!(p.streak, 0);
        assert_eq!(p.lives, 2);
        // Classic mode has no lives to lose.
        p.award(false, GameMode::Classic, 1.0);
        assert_eq!(p.lives, 2);
    }

    #[test]
    fn spending_removes_exactly_one_or_nothing() {
        let mut p = player();
        p.reset_for_new_game();
        assert!(p.spend(PowerUp::FreezeTime));
        assert_eq!(p.power_ups.len(), 2);
        assert!(!p.spend(PowerUp::FreezeTime));
    }

    #[test]
    fn new_game_deals_a_full_hand_and_clears_score() {
        let mut p = player();
        p.award(true, GameMode::Classic, 1.0);
        p.power_ups.clear();
        p.reset_for_new_game();
        assert_eq!(p.score, 0);
        assert_eq!(p.power_ups, PowerUp::STARTING_HAND.to_vec());
    }
}
