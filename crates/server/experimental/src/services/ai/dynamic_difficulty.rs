//! Dynamic Difficulty Adjustment (DDA) for AI-generated questions.
//!
//! Analyzes player performance in real-time and adjusts question difficulty
//! to maintain optimal challenge level. This improves user experience by
//! keeping players in the "flow state" — not too easy, not too hard.
//!
//! ## Algorithm
//!
//! The DDA system uses a sliding window approach:
//! 1. Track player performance over the last N questions
//! 2. Calculate performance score (0.0 to 1.0)
//! 3. Map performance score to difficulty level
//! 4. Adjust AI generation parameters accordingly
//!
//! ## Performance Metrics
//!
//! - **Accuracy**: Percentage of correct answers
//! - **Speed**: Average time to answer (normalized)
//! - **Streak**: Consecutive correct/incorrect answers
//!
//! ## Difficulty Levels
//!
//! - `easy`: 0.0 - 0.3 (beginners, struggling players)
//! - `medium`: 0.3 - 0.7 (average players)
//! - `hard`: 0.7 - 1.0 (advanced players, experts)

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Player performance snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPerformance {
    pub user_id: i64,
    pub username: String,
    pub total_questions: usize,
    pub correct_answers: usize,
    pub average_time_ms: u64,
    pub current_streak: i32, // positive = correct, negative = incorrect
    pub performance_score: f32, // 0.0 to 1.0
}

/// Difficulty level for question generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
}

impl DifficultyLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DifficultyLevel::Easy => "easy",
            DifficultyLevel::Medium => "medium",
            DifficultyLevel::Hard => "hard",
        }
    }
}

/// Answer record for performance tracking.
#[derive(Debug, Clone)]
struct AnswerRecord {
    question_index: usize,
    is_correct: bool,
    time_ms: u64,
    timestamp: Instant,
}

/// Performance tracker for a single player.
#[derive(Debug)]
pub struct PerformanceTracker {
    user_id: i64,
    username: String,
    answers: VecDeque<AnswerRecord>,
    window_size: usize,
    created_at: Instant,
}

impl PerformanceTracker {
    /// Create a new performance tracker.
    pub fn new(user_id: i64, username: String, window_size: usize) -> Self {
        Self {
            user_id,
            username,
            answers: VecDeque::with_capacity(window_size),
            window_size,
            created_at: Instant::now(),
        }
    }

    /// Record an answer.
    pub fn record_answer(&mut self, question_index: usize, is_correct: bool, time_ms: u64) {
        let record = AnswerRecord {
            question_index,
            is_correct,
            time_ms,
            timestamp: Instant::now(),
        };

        // Maintain sliding window
        if self.answers.len() >= self.window_size {
            self.answers.pop_front();
        }
        self.answers.push_back(record);

        debug!(
            user_id = self.user_id,
            question_index,
            is_correct,
            time_ms,
            window_size = self.answers.len(),
            "Recorded answer"
        );
    }

    /// Calculate current performance score (0.0 to 1.0).
    pub fn calculate_performance_score(&self) -> f32 {
        if self.answers.is_empty() {
            return 0.5; // Neutral starting point
        }

        let total = self.answers.len() as f32;
        let correct = self.answers.iter().filter(|a| a.is_correct).count() as f32;
        
        // Base accuracy score
        let accuracy_score = correct / total;

        // Speed bonus/penalty (normalized to 0-1 range, where 1.0 is fastest)
        let avg_time_ms = self.answers.iter().map(|a| a.time_ms).sum::<u64>() as f32 / total;
        let speed_score = if avg_time_ms < 5000.0 {
            1.0 // Very fast
        } else if avg_time_ms < 10000.0 {
            0.8 // Fast
        } else if avg_time_ms < 20000.0 {
            0.6 // Normal
        } else if avg_time_ms < 30000.0 {
            0.4 // Slow
        } else {
            0.2 // Very slow
        };

        // Streak bonus (positive or negative)
        let streak_bonus = self.calculate_streak_bonus();

        // Combined score (weighted average)
        let score = (accuracy_score * 0.6) + (speed_score * 0.2) + (streak_bonus * 0.2);
        
        score.clamp(0.0, 1.0)
    }

    /// Calculate streak bonus (-1.0 to 1.0).
    fn calculate_streak_bonus(&self) -> f32 {
        if self.answers.is_empty() {
            return 0.0;
        }

        let mut streak = 0i32;
        let last_correct = self.answers.back().map(|a| a.is_correct).unwrap_or(false);

        for answer in self.answers.iter().rev() {
            if answer.is_correct == last_correct {
                streak += if last_correct { 1 } else { -1 };
            } else {
                break;
            }
        }

        // Normalize streak to -1.0 to 1.0 range
        // Max streak of 5 maps to +/- 1.0
        (streak as f32 / 5.0).clamp(-1.0, 1.0)
    }

    /// Get performance snapshot.
    pub fn get_performance(&self) -> PlayerPerformance {
        let total = self.answers.len();
        let correct = self.answers.iter().filter(|a| a.is_correct).count();
        let avg_time = if total > 0 {
            self.answers.iter().map(|a| a.time_ms).sum::<u64>() / total as u64
        } else {
            0
        };

        let current_streak = if self.answers.is_empty() {
            0
        } else {
            let last_correct = self.answers.back().unwrap().is_correct;
            let mut streak = 0i32;
            
            for answer in self.answers.iter().rev() {
                if answer.is_correct == last_correct {
                    streak += if last_correct { 1 } else { -1 };
                } else {
                    break;
                }
            }
            streak
        };

        PlayerPerformance {
            user_id: self.user_id,
            username: self.username.clone(),
            total_questions: total,
            correct_answers: correct,
            average_time_ms: avg_time,
            current_streak,
            performance_score: self.calculate_performance_score(),
        }
    }
}

/// Difficulty adjustment engine.
///
/// Analyzes player performance and determines optimal difficulty level.
pub struct DifficultyEngine {
    /// Performance thresholds for difficulty levels.
    easy_threshold: f32,
    hard_threshold: f32,
}

impl DifficultyEngine {
    /// Create a new difficulty engine with default thresholds.
    pub fn new() -> Self {
        Self {
            easy_threshold: 0.3,
            hard_threshold: 0.7,
        }
    }

    /// Create a difficulty engine with custom thresholds.
    pub fn with_thresholds(easy_threshold: f32, hard_threshold: f32) -> Self {
        Self {
            easy_threshold,
            hard_threshold,
        }
    }

    /// Determine difficulty level based on performance score.
    pub fn determine_difficulty(&self, performance_score: f32) -> DifficultyLevel {
        if performance_score < self.easy_threshold {
            DifficultyLevel::Easy
        } else if performance_score < self.hard_threshold {
            DifficultyLevel::Medium
        } else {
            DifficultyLevel::Hard
        }
    }

    /// Get recommended difficulty for a player.
    pub fn get_recommended_difficulty(&self, performance: &PlayerPerformance) -> DifficultyLevel {
        self.determine_difficulty(performance.performance_score)
    }
}

impl Default for DifficultyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// DDA manager for a room.
///
/// Tracks performance of all players and provides room-wide difficulty recommendations.
pub struct DdaManager {
    trackers: std::collections::HashMap<i64, PerformanceTracker>,
    engine: DifficultyEngine,
    window_size: usize,
}

impl DdaManager {
    /// Create a new DDA manager.
    pub fn new(window_size: usize) -> Self {
        Self {
            trackers: std::collections::HashMap::new(),
            engine: DifficultyEngine::new(),
            window_size,
        }
    }

    /// Add a player to tracking.
    pub fn add_player(&mut self, user_id: i64, username: String) {
        if !self.trackers.contains_key(&user_id) {
            self.trackers.insert(
                user_id,
                PerformanceTracker::new(user_id, username, self.window_size),
            );
            info!(user_id, "Added player to DDA tracking");
        }
    }

    /// Remove a player from tracking.
    pub fn remove_player(&mut self, user_id: i64) {
        if self.trackers.remove(&user_id).is_some() {
            info!(user_id, "Removed player from DDA tracking");
        }
    }

    /// Record an answer for a player.
    pub fn record_answer(&mut self, user_id: i64, question_index: usize, is_correct: bool, time_ms: u64) {
        if let Some(tracker) = self.trackers.get_mut(&user_id) {
            tracker.record_answer(question_index, is_correct, time_ms);
        }
    }

    /// Get performance for a specific player.
    pub fn get_player_performance(&self, user_id: i64) -> Option<PlayerPerformance> {
        self.trackers.get(&user_id).map(|t| t.get_performance())
    }

    /// Get recommended difficulty for the room.
    ///
    /// Uses the average performance of all players to determine room-wide difficulty.
    pub fn get_room_difficulty(&self) -> DifficultyLevel {
        if self.trackers.is_empty() {
            return DifficultyLevel::Medium;
        }

        let avg_score = self
            .trackers
            .values()
            .map(|t| t.calculate_performance_score())
            .sum::<f32>()
            / self.trackers.len() as f32;

        self.engine.determine_difficulty(avg_score)
    }

    /// Get performance stats for all players.
    pub fn get_all_performances(&self) -> Vec<PlayerPerformance> {
        self.trackers.values().map(|t| t.get_performance()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_tracker_empty() {
        let tracker = PerformanceTracker::new(1, "test".to_string(), 10);
        let perf = tracker.get_performance();
        
        assert_eq!(perf.total_questions, 0);
        assert_eq!(perf.correct_answers, 0);
        assert_eq!(perf.performance_score, 0.5); // Neutral
    }

    #[test]
    fn test_performance_tracker_perfect() {
        let mut tracker = PerformanceTracker::new(1, "test".to_string(), 10);
        
        // Record 5 perfect answers
        for i in 0..5 {
            tracker.record_answer(i, true, 3000);
        }

        let perf = tracker.get_performance();
        assert_eq!(perf.total_questions, 5);
        assert_eq!(perf.correct_answers, 5);
        assert!(perf.performance_score > 0.8);
        assert!(perf.current_streak > 0);
    }

    #[test]
    fn test_performance_tracker_poor() {
        let mut tracker = PerformanceTracker::new(1, "test".to_string(), 10);
        
        // Record 5 incorrect answers
        for i in 0..5 {
            tracker.record_answer(i, false, 25000);
        }

        let perf = tracker.get_performance();
        assert_eq!(perf.total_questions, 5);
        assert_eq!(perf.correct_answers, 0);
        assert!(perf.performance_score < 0.3);
        assert!(perf.current_streak < 0);
    }

    #[test]
    fn test_difficulty_engine() {
        let engine = DifficultyEngine::new();

        assert_eq!(engine.determine_difficulty(0.1), DifficultyLevel::Easy);
        assert_eq!(engine.determine_difficulty(0.5), DifficultyLevel::Medium);
        assert_eq!(engine.determine_difficulty(0.9), DifficultyLevel::Hard);
    }

    #[test]
    fn test_dda_manager() {
        let mut manager = DdaManager::new(10);
        
        manager.add_player(1, "player1".to_string());
        manager.add_player(2, "player2".to_string());

        // Player 1: perfect performance
        for i in 0..5 {
            manager.record_answer(1, i, true, 3000);
        }

        // Player 2: poor performance
        for i in 0..5 {
            manager.record_answer(2, i, false, 25000);
        }

        let room_diff = manager.get_room_difficulty();
        assert_eq!(room_diff, DifficultyLevel::Medium); // Average of easy and hard

        let perf1 = manager.get_player_performance(1).unwrap();
        assert!(perf1.performance_score > 0.8);

        let perf2 = manager.get_player_performance(2).unwrap();
        assert!(perf2.performance_score < 0.3);
    }
}
