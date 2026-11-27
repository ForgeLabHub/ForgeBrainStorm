//! Game modes and the rules that separate them.
//!
//! Every difference between modes lives here: lives, clock, scoring, teams and
//! whether the result is worth persisting. The rest of the server asks the mode
//! instead of comparing strings, so a new mode is one variant plus one match arm.

use serde::{Deserialize, Serialize};

/// Lives handed out in survival mode.
pub const SURVIVAL_LIVES: u8 = 3;

/// Team names by index (`Player::team`).
pub const TEAM_NAMES: [&str; 2] = ["Синие", "Красные"];

/// The modes a room can be played in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    /// Everyone answers at once, a flat 100 points per correct answer.
    #[default]
    Classic,
    /// Free-for-all: the faster the answer, the more it is worth.
    FreeForAll,
    /// Survival: three lives, the last player standing ends the round early.
    Lives,
    /// Blitz: a forced 10-second clock and a streak multiplier.
    Blitz,
    /// Two teams, scores added up per team.
    Team,
    /// Practice: no lives, no persistence, nothing on the leaderboard.
    Sandbox,
    /// Cooperative: players work together, shared score, no elimination.
    Coop,
    /// Svoyaigra (Jeopardy-style): players choose questions with point values.
    Svoyaigra,
}

impl GameMode {
    /// Modes offered by the UI, in display order.
    pub const ALL: [GameMode; 8] = [
        GameMode::Classic,
        GameMode::FreeForAll,
        GameMode::Lives,
        GameMode::Blitz,
        GameMode::Team,
        GameMode::Sandbox,
        GameMode::Coop,
        GameMode::Svoyaigra,
    ];

    /// Parse a mode sent by a client. Aliases keep old links and API clients
    /// working (`survival` → `lives`, `teams` → `team`, `practice` → `sandbox`).
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "classic" => Some(GameMode::Classic),
            "ffa" | "free_for_all" | "freeforall" => Some(GameMode::FreeForAll),
            "lives" | "survival" => Some(GameMode::Lives),
            "blitz" => Some(GameMode::Blitz),
            "team" | "teams" => Some(GameMode::Team),
            "sandbox" | "practice" => Some(GameMode::Sandbox),
            "coop" | "cooperative" => Some(GameMode::Coop),
            "svoyaigra" | "jeopardy" => Some(GameMode::Svoyaigra),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            GameMode::Classic => "classic",
            GameMode::FreeForAll => "ffa",
            GameMode::Lives => "lives",
            GameMode::Blitz => "blitz",
            GameMode::Team => "team",
            GameMode::Sandbox => "sandbox",
            GameMode::Coop => "coop",
            GameMode::Svoyaigra => "svoyaigra",
        }
    }

    /// Russian label shown in the UI.
    pub fn title(self) -> &'static str {
        match self {
            GameMode::Classic => "Классика",
            GameMode::FreeForAll => "Каждый за себя",
            GameMode::Lives => "На выживание",
            GameMode::Blitz => "Блиц",
            GameMode::Team => "Команды",
            GameMode::Sandbox => "Тренировка",
            GameMode::Coop => "Кооператив",
            GameMode::Svoyaigra => "Своя Игра",
        }
    }

    /// One-line rule summary for the lobby and the docs.
    pub fn description(self) -> &'static str {
        match self {
            GameMode::Classic => "100 очков за верный ответ",
            GameMode::FreeForAll => "Чем быстрее ответ, тем больше очков",
            GameMode::Lives => "3 жизни, ошибка их отнимает",
            GameMode::Blitz => "10 секунд на вопрос, серия умножает очки",
            GameMode::Team => "Две команды, счёт общий",
            GameMode::Sandbox => "Без рейтинга и жизней — просто поиграть",
            GameMode::Coop => "Одна общая копилка очков на всех",
            GameMode::Svoyaigra => "15 секунд на вопрос, очки растут с ответом",
        }
    }

    /// Lives per player, or `None` when the mode has no elimination.
    pub fn lives(self) -> Option<u8> {
        match self {
            GameMode::Lives => Some(SURVIVAL_LIVES),
            _ => None,
        }
    }

    /// A fixed clock for modes that define their own pace.
    pub fn timer_override(self) -> Option<u64> {
        match self {
            GameMode::Blitz => Some(10),
            GameMode::Svoyaigra => Some(15),
            _ => None,
        }
    }

    /// Team-based modes: players are seated on sides and scored per side.
    pub fn is_team(self) -> bool {
        matches!(self, GameMode::Team | GameMode::Coop)
    }

    /// Cooperative play: one shared side, so the score belongs to everyone.
    pub fn is_coop(self) -> bool {
        self == GameMode::Coop
    }

    /// Whether the finished game belongs in the leaderboard and history.
    pub fn persists(self) -> bool {
        !matches!(self, GameMode::Sandbox)
    }

    /// Flat points a correct answer is worth before speed/streak adjustments.
    pub fn base_points(self) -> i64 {
        match self {
            GameMode::Blitz => 60,
            GameMode::Sandbox | GameMode::Coop => 50,
            GameMode::Svoyaigra => 100,
            _ => 100,
        }
    }

    /// Points for one correct answer.
    ///
    /// `speed` is the share of the clock still left (1.0 = instant answer,
    /// 0.0 = answered on the buzzer); `streak` counts the answer being scored,
    /// so the first correct answer is streak 1.
    pub fn points(self, speed: f64, streak: u32) -> i64 {
        let speed = speed.clamp(0.0, 1.0);
        match self {
            // 40 points at the buzzer, 100 for an instant answer, in steps of 5.
            GameMode::FreeForAll => {
                let raw = 40.0 + 60.0 * speed;
                ((raw / 5.0).round() * 5.0) as i64
            }
            // +25% per consecutive correct answer, capped at double.
            GameMode::Blitz => {
                let multiplier = (1.0 + 0.25 * f64::from(streak.saturating_sub(1))).min(2.0);
                (f64::from(self.base_points() as i32) * multiplier).round() as i64
            }
            // Cooperative: bonus for consecutive correct answers, shared pool.
            GameMode::Coop => {
                let bonus = 1.0 + 0.1 * f64::from(streak.saturating_sub(1));
                (f64::from(self.base_points() as i32) * bonus).round() as i64
            }
            // Svoyaigra: base points, but players choose their stake.
            // Speed bonus applies: faster = more points.
            GameMode::Svoyaigra => {
                let raw = 60.0 + 40.0 * speed;
                ((raw / 10.0).round() * 10.0) as i64
            }
            _ => self.base_points(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_round_trips_every_mode() {
        for mode in GameMode::ALL {
            assert_eq!(GameMode::parse(mode.as_str()), Some(mode));
        }
    }

    #[test]
    fn parse_accepts_legacy_aliases() {
        assert_eq!(GameMode::parse("survival"), Some(GameMode::Lives));
        assert_eq!(GameMode::parse("teams"), Some(GameMode::Team));
        assert_eq!(GameMode::parse("practice"), Some(GameMode::Sandbox));
        assert_eq!(GameMode::parse("cooperative"), Some(GameMode::Coop));
        assert_eq!(GameMode::parse("jeopardy"), Some(GameMode::Svoyaigra));
    }

    #[test]
    fn ffa_rewards_speed_and_never_goes_below_floor() {
        let instant = GameMode::FreeForAll.points(1.0, 1);
        let buzzer = GameMode::FreeForAll.points(0.0, 1);
        let middle = GameMode::FreeForAll.points(0.5, 1);
        assert_eq!(instant, 100);
        assert_eq!(buzzer, 40);
        assert!(middle > buzzer && middle < instant);
    }

    #[test]
    fn blitz_streak_multiplier_caps_at_double() {
        assert_eq!(GameMode::Blitz.points(1.0, 1), 60);
        assert_eq!(GameMode::Blitz.points(1.0, 3), 90);
        assert_eq!(GameMode::Blitz.points(1.0, 9), 120);
        assert_eq!(GameMode::Blitz.points(1.0, 50), 120);
    }

    #[test]
    fn flat_modes_ignore_speed() {
        for mode in [GameMode::Classic, GameMode::Lives, GameMode::Team] {
            assert_eq!(mode.points(1.0, 1), 100);
            assert_eq!(mode.points(0.0, 7), 100);
        }
    }

    #[test]
    fn coop_rewards_streak() {
        assert_eq!(GameMode::Coop.points(1.0, 1), 50);
        assert_eq!(GameMode::Coop.points(1.0, 5), 70);
        assert_eq!(GameMode::Coop.points(0.5, 3), 60);
    }

    #[test]
    fn svoyaigra_rewards_speed() {
        let instant = GameMode::Svoyaigra.points(1.0, 1);
        let buzzer = GameMode::Svoyaigra.points(0.0, 1);
        assert_eq!(instant, 100);
        assert_eq!(buzzer, 60);
    }

    #[test]
    fn mode_specific_rules() {
        assert_eq!(GameMode::Lives.lives(), Some(3));
        assert_eq!(GameMode::Classic.lives(), None);
        assert_eq!(GameMode::Blitz.timer_override(), Some(10));
        assert_eq!(GameMode::Classic.timer_override(), None);
        assert!(GameMode::Team.is_team());
        assert!(!GameMode::Sandbox.persists());
        assert!(GameMode::Classic.persists());
    }
}
