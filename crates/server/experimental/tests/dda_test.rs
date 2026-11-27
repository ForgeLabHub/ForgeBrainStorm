//! Dynamic Difficulty Adjustment (DDA) integration tests.
//!
//! These tests verify the DDA system works correctly in various scenarios.

use brainstorm::services::ai::dynamic_difficulty::{DdaManager, DifficultyEngine, DifficultyLevel, PerformanceTracker};

#[test]
fn test_dda_performance_tracking() {
    // Test that performance tracker correctly records and calculates performance
    let mut tracker = PerformanceTracker::new(1, "player1".to_string(), 5);

    // Perfect performance: 5 correct answers, fast
    for i in 0..5 {
        tracker.record_answer(i, true, 3000); // 3 seconds per answer
    }

    let perf = tracker.get_performance();
    assert_eq!(perf.total_questions, 5);
    assert_eq!(perf.correct_answers, 5);
    assert!(perf.performance_score > 0.8, "Perfect performance should have high score");
}

#[test]
fn test_dda_poor_performance() {
    // Test that poor performance results in low score
    let mut tracker = PerformanceTracker::new(2, "player2".to_string(), 5);

    // Poor performance: 5 incorrect answers, slow
    for i in 0..5 {
        tracker.record_answer(i, false, 25000); // 25 seconds per answer
    }

    let perf = tracker.get_performance();
    assert_eq!(perf.correct_answers, 0);
    assert!(perf.performance_score < 0.3, "Poor performance should have low score");
}

#[test]
fn test_dda_mixed_performance() {
    // Test mixed performance results in medium score
    let mut tracker = PerformanceTracker::new(3, "player3".to_string(), 5);

    // Mixed: 3 correct, 2 incorrect, average speed
    tracker.record_answer(0, true, 10000);
    tracker.record_answer(1, false, 15000);
    tracker.record_answer(2, true, 12000);
    tracker.record_answer(3, true, 11000);
    tracker.record_answer(4, false, 18000);

    let perf = tracker.get_performance();
    assert_eq!(perf.correct_answers, 3);
    assert!(perf.performance_score > 0.3 && perf.performance_score < 0.7,
        "Mixed performance should have medium score");
}

#[test]
fn test_dda_difficulty_engine() {
    let engine = DifficultyEngine::new();

    // Test threshold boundaries
    assert_eq!(engine.determine_difficulty(0.2), DifficultyLevel::Easy);
    assert_eq!(engine.determine_difficulty(0.5), DifficultyLevel::Medium);
    assert_eq!(engine.determine_difficulty(0.8), DifficultyLevel::Hard);

    // Test exact boundaries
    assert_eq!(engine.determine_difficulty(0.3), DifficultyLevel::Medium); // At boundary
    assert_eq!(engine.determine_difficulty(0.7), DifficultyLevel::Hard);  // At boundary
}

#[test]
fn test_dda_custom_thresholds() {
    // Test custom thresholds
    let engine = DifficultyEngine::with_thresholds(0.4, 0.6);

    assert_eq!(engine.determine_difficulty(0.3), DifficultyLevel::Easy);
    assert_eq!(engine.determine_difficulty(0.5), DifficultyLevel::Medium);
    assert_eq!(engine.determine_difficulty(0.7), DifficultyLevel::Hard);
}

#[test]
fn test_dda_manager_multiple_players() {
    let mut manager = DdaManager::new(5);

    // Add two players with different skill levels
    manager.add_player(1, "expert".to_string());
    manager.add_player(2, "beginner".to_string());

    // Expert: 5 correct, fast
    for i in 0..5 {
        manager.record_answer(1, i, true, 3000);
    }

    // Beginner: 1 correct, slow
    manager.record_answer(2, 0, true, 20000);
    for i in 1..5 {
        manager.record_answer(2, i, false, 25000);
    }

    // Expert should have high score
    let expert_perf = manager.get_player_performance(1).unwrap();
    assert!(expert_perf.performance_score > 0.8);

    // Beginner should have low score
    let beginner_perf = manager.get_player_performance(2).unwrap();
    assert!(beginner_perf.performance_score < 0.3);

    // Room difficulty should be medium (average of expert and beginner)
    let room_diff = manager.get_room_difficulty();
    assert_eq!(room_diff, DifficultyLevel::Medium);
}

#[test]
fn test_dda_sliding_window() {
    let mut tracker = PerformanceTracker::new(1, "player1".to_string(), 3);

    // First 3 answers: all correct
    tracker.record_answer(0, true, 5000);
    tracker.record_answer(1, true, 5000);
    tracker.record_answer(2, true, 5000);

    let perf1 = tracker.get_performance();
    assert_eq!(perf1.correct_answers, 3);
    assert!(perf1.performance_score > 0.8);

    // Next 3 answers: all incorrect (should replace first 3 in window)
    tracker.record_answer(3, false, 5000);
    tracker.record_answer(4, false, 5000);
    tracker.record_answer(5, false, 5000);

    let perf2 = tracker.get_performance();
    assert_eq!(perf2.correct_answers, 0, "Sliding window should have replaced old answers");
    assert!(perf2.performance_score < 0.3);
}

#[test]
fn test_dda_streak_tracking() {
    let mut tracker = PerformanceTracker::new(1, "player1".to_string(), 10);

    // 5 correct in a row
    for i in 0..5 {
        tracker.record_answer(i, true, 5000);
    }

    let perf = tracker.get_performance();
    assert_eq!(perf.current_streak, 5, "Should track positive streak");

    // Now 2 incorrect
    tracker.record_answer(5, false, 5000);
    tracker.record_answer(6, false, 5000);

    let perf2 = tracker.get_performance();
    assert_eq!(perf2.current_streak, -2, "Should track negative streak");
}

#[test]
fn test_dda_empty_tracker() {
    let tracker = PerformanceTracker::new(1, "player1".to_string(), 5);
    let perf = tracker.get_performance();

    assert_eq!(perf.total_questions, 0);
    assert_eq!(perf.performance_score, 0.5, "Empty tracker should have neutral score");
}

#[test]
fn test_dda_player_removal() {
    let mut manager = DdaManager::new(5);

    manager.add_player(1, "player1".to_string());
    manager.add_player(2, "player2".to_string());

    // Record answers for both
    manager.record_answer(1, 0, true, 5000);
    manager.record_answer(2, 0, false, 15000);

    // Remove player 1
    manager.remove_player(1);

    // Should only have player 2's performance
    assert!(manager.get_player_performance(1).is_none());
    assert!(manager.get_player_performance(2).is_some());

    // Room difficulty should be based only on player 2
    let room_diff = manager.get_room_difficulty();
    assert_eq!(room_diff, DifficultyLevel::Easy, "Should be Easy based on player 2's poor performance");
}

#[test]
fn test_dda_all_performances() {
    let mut manager = DdaManager::new(5);

    manager.add_player(1, "player1".to_string());
    manager.add_player(2, "player2".to_string());
    manager.add_player(3, "player3".to_string());

    manager.record_answer(1, 0, true, 3000);
    manager.record_answer(2, 0, true, 5000);
    manager.record_answer(3, 0, false, 15000);

    let performances = manager.get_all_performances();
    assert_eq!(performances.len(), 3, "Should return all player performances");
}
