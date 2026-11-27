//! Player model and in-room state.

use std::time::Instant;

use tokio::sync::mpsc::UnboundedSender;

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
    pub is_cheat: bool,
    pub is_invisible: bool,
    pub instant_answer: bool,
    pub team: Option<u8>,
    pub joined_at: Instant,
    pub disconnected_at: Option<Instant>,
    /// Power-ups still in hand for this game.
    pub power_ups: Vec<PowerUp>,
    /// Armed by `DoublePoints`, spent by the next correct answer.
    pub double_points: bool,
    /// This player's own socket. Private events (a hand, hidden options) must
    /// not go through the room broadcast or everyone would see them.
    pub outbox: Option<UnboundedSender<String>>,
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
            is_invisible: false,
            instant_answer: false,
            team: None,
            joined_at: Instant::now(),
            disconnected_at: None,
            power_ups: Vec::new(),
            double_points: false,
            outbox: None,
        }
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected_at.is_some()
    }

    /// Apply an answer and return the points it earned.
    ///
    /// `speed` is the share of the clock left when the answer landed; only
    /// speed-scored modes use it. An armed `DoublePoints` is consumed here,
    /// whether the answer is right or not: the power-up buys one attempt, not
    /// one guaranteed hit.
    pub fn award(&mut self, correct: bool, mode: GameMode, speed: f64) -> i64 {
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
        self.power_ups = PowerUp::STARTING_HAND.to_vec();
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
