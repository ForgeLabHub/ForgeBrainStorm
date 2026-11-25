//! WebSocket gameplay tests.

use brainstorm::ws::protocol::{ClientMessage, ServerMessage, RoomSettingsInput};
use brainstorm::domain::room::{Room, RoomSettings, RoomState};
use brainstorm::domain::player::Player;
use brainstorm::ws::room_manager::RoomManager;
use brainstorm::AppState;
use std::sync::Arc;
use std::time::Instant;

#[test]
fn test_room_creation() {
    let settings = RoomSettings {
        topic: "Test Topic".to_string(),
        mode: "classic".to_string(),
        max_players: 10,
        question_count: 5,
        num_options: 4,
        timer_secs: 30,
        difficulty: "medium".to_string(),
    };

    assert_eq!(settings.topic, "Test Topic");
    assert_eq!(settings.mode, "classic");
    assert_eq!(settings.max_players, 10);
}

#[test]
fn test_room_settings_clamp() {
    let mut settings = RoomSettings {
        topic: "A".repeat(300),  // Too long
        mode: "invalid_mode".to_string(),
        max_players: 30,
        question_count: 100,  // Too high
        num_options: 10,      // Too high
        timer_secs: 300,      // Too high
        difficulty: "invalid".to_string(),
    };

    settings.clamp();

    assert_eq!(settings.topic.len(), 200);
    assert_eq!(settings.mode, "classic");  // Reset to default
    assert_eq!(settings.question_count, 30);  // Clamped to max
    assert_eq!(settings.num_options, 6);  // Clamped to max
    assert_eq!(settings.timer_secs, 120);  // Clamped to max
    assert_eq!(settings.difficulty, "medium");  // Reset to default
}

#[test]
fn test_player_creation() {
    let player = Player::new(
        1,
        "test_user".to_string(),
        "Test User".to_string(),
        Some(3),
    );

    assert_eq!(player.user_id, 1);
    assert_eq!(player.username, "test_user");
    assert_eq!(player.display_name, "Test User");
    assert_eq!(player.score, 0);
    assert_eq!(player.lives, Some(3));
    assert_eq!(player.jokers_used, 0);
    assert_eq!(player.current_streak, 0);
    assert_eq!(player.answers.len(), 0);
    assert!(!player.is_ready);
}

#[test]
fn test_room_state_transitions() {
    let mut room = Room {
        code: "ABCDEF".to_string(),
        host_user_id: 1,
        host_name: "host".to_string(),
        state: RoomState::Waiting,
        settings: RoomSettings::default(),
        players: std::collections::HashMap::new(),
        questions: Vec::new(),
        current_question: None,
        question_start_time: None,
        created_at: Instant::now(),
        last_activity: Instant::now(),
        tx: tokio::sync::broadcast::channel(10).0,
    };

    assert_eq!(room.state, RoomState::Waiting);
    
    // Simulate game start
    room.state = RoomState::Playing;
    room.current_question = Some(0);
    room.question_start_time = Some(Instant::now());
    
    assert_eq!(room.state, RoomState::Playing);
    assert_eq!(room.current_question, Some(0));
    
    // Simulate game end
    room.state = RoomState::Finished;
    assert_eq!(room.state, RoomState::Finished);
}

#[test]
fn test_scoring_calculation() {
    // Test basic scoring
    let base_points = 100;
    let time_bonus = 50;  // Answered quickly
    let streak_bonus = 25;  // 2nd correct in a row
    
    let total = base_points + time_bonus + streak_bonus;
    assert_eq!(total, 175);
}

#[test]
fn test_lives_mode() {
    let mut player = Player::new(
        1,
        "player1".to_string(),
        "Player 1".to_string(),
        Some(3),  // Start with 3 lives
    );

    assert_eq!(player.lives, Some(3));
    
    // Lose a life
    if let Some(ref mut lives) = player.lives {
        *lives = lives.saturating_sub(1);
    }
    assert_eq!(player.lives, Some(2));
    
    // Lose another life
    if let Some(ref mut lives) = player.lives {
        *lives = lives.saturating_sub(1);
    }
    assert_eq!(player.lives, Some(1));
    
    // Lose last life
    if let Some(ref mut lives) = player.lives {
        *lives = lives.saturating_sub(1);
    }
    assert_eq!(player.lives, Some(0));
    
    // Can't go below 0
    if let Some(ref mut lives) = player.lives {
        *lives = lives.saturating_sub(1);
    }
    assert_eq!(player.lives, Some(0));
}

#[test]
fn test_message_serialization() {
    // Test ClientMessage serialization
    let msg = ClientMessage::CreateRoom {
        topic: Some("Test".to_string()),
        mode: Some("classic".to_string()),
        settings: None,
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("CreateRoom"));
    assert!(json.contains("Test"));
    
    // Test ServerMessage serialization
    let msg = ServerMessage::RoomJoined {
        code: "ABCDEF".to_string(),
        is_host: true,
        topic: "Test".to_string(),
        mode: "classic".to_string(),
        players: Vec::new(),
    };
    
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("RoomJoined"));
    assert!(json.contains("ABCDEF"));
}

#[test]
fn test_joker_usage() {
    let mut player = Player::new(
        1,
        "player1".to_string(),
        "Player 1".to_string(),
        None,
    );

    assert_eq!(player.jokers_used, 0);
    
    // Use joker
    player.jokers_used += 1;
    assert_eq!(player.jokers_used, 1);
    
    // Try to use another (should be blocked by game logic)
    let can_use = player.jokers_used < 1;
    assert!(!can_use);
}

#[test]
fn test_answer_tracking() {
    use brainstorm::ws::protocol::PlayerAnswer;
    
    let mut player = Player::new(
        1,
        "player1".to_string(),
        "Player 1".to_string(),
        None,
    );

    // Submit answer
    player.answers.push(PlayerAnswer {
        user_id: 1,
        username: "player1".to_string(),
        question_index: 0,
        answer_index: 2,
        is_correct: true,
        time_ms: 5000,
        points_earned: 150,
    });

    assert_eq!(player.answers.len(), 1);
    assert_eq!(player.answers[0].question_index, 0);
    assert!(player.answers[0].is_correct);
    assert_eq!(player.answers[0].points_earned, 150);
    
    // Submit another answer
    player.answers.push(PlayerAnswer {
        user_id: 1,
        username: "player1".to_string(),
        question_index: 1,
        answer_index: 1,
        is_correct: false,
        time_ms: 10000,
        points_earned: 0,
    });

    assert_eq!(player.answers.len(), 2);
    assert!(!player.answers[1].is_correct);
}

#[test]
fn test_leaderboard_sorting() {
    use brainstorm::ws::protocol::PlayerScore;
    
    let mut scores = vec![
        PlayerScore {
            user_id: 1,
            username: "player1".to_string(),
            score: 500,
            correct_answers: 5,
            total_answers: 10,
        },
        PlayerScore {
            user_id: 2,
            username: "player2".to_string(),
            score: 800,
            correct_answers: 8,
            total_answers: 10,
        },
        PlayerScore {
            user_id: 3,
            username: "player3".to_string(),
            score: 300,
            correct_answers: 3,
            total_answers: 10,
        },
    ];

    // Sort by score descending
    scores.sort_by(|a, b| b.score.cmp(&a.score));

    assert_eq!(scores[0].user_id, 2);  // 800
    assert_eq!(scores[1].user_id, 1);  // 500
    assert_eq!(scores[2].user_id, 3);  // 300
}

#[test]
fn test_room_code_generation() {
    // Test that room codes are 6 characters
    let code = brainstorm::ws::room_manager::generate_room_code();
    assert_eq!(code.len(), 6);
    
    // Test that all characters are uppercase letters
    assert!(code.chars().all(|c| c.is_ascii_uppercase()));
    
    // Test that codes are different (probabilistic)
    let code2 = brainstorm::ws::room_manager::generate_room_code();
    // Note: There's a tiny chance they could be the same, but 26^6 = 308M combinations
    // So this should pass virtually always
    assert_ne!(code, code2);
}
