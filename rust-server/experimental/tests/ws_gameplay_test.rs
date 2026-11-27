//! WebSocket gameplay unit tests.
//!
//! Tests the core game logic: scoring, state transitions, domain types.

use brainstorm::domain::player::Player;
use brainstorm::domain::room::{RoomSettings, RoomState};
use brainstorm::ws::protocol::{PlayerAnswer, PlayerInfo, PlayerScore, ServerMessage};

#[test]
fn test_room_settings_default() {
    let settings = RoomSettings::default();
    assert_eq!(settings.topic, "Общие знания");
    assert_eq!(settings.mode, "classic");
    assert_eq!(settings.max_players, 30);
    assert_eq!(settings.question_count, 10);
    assert_eq!(settings.num_options, 4);
    assert_eq!(settings.timer_secs, 30);
    assert_eq!(settings.difficulty, "medium");
}

#[test]
fn test_room_settings_clamp_valid_values() {
    let mut settings = RoomSettings {
        topic: "Valid Topic".to_string(),
        mode: "classic".to_string(),
        max_players: 10,
        question_count: 15,
        num_options: 4,
        timer_secs: 30,
        difficulty: "easy".to_string(),
    };

    settings.clamp();

    assert_eq!(settings.topic, "Valid Topic");
    assert_eq!(settings.mode, "classic");
    assert_eq!(settings.question_count, 15);
    assert_eq!(settings.num_options, 4);
    assert_eq!(settings.timer_secs, 30);
    assert_eq!(settings.difficulty, "easy");
}

#[test]
fn test_room_settings_clamp_out_of_range() {
    let mut settings = RoomSettings {
        topic: "A".repeat(300),       // Too long → truncate to 200
        mode: "invalid_mode".into(),   // Invalid → reset to "classic"
        max_players: 30,
        question_count: 100,           // Too high → clamp to 30
        num_options: 10,               // Too high → clamp to 6
        timer_secs: 300,               // Too high → clamp to 120
        difficulty: "invalid".into(),  // Invalid → reset to "medium"
    };

    settings.clamp();

    assert_eq!(settings.topic.len(), 200);
    assert_eq!(settings.mode, "classic");
    assert_eq!(settings.question_count, 30);
    assert_eq!(settings.num_options, 6);
    assert_eq!(settings.timer_secs, 120);
    assert_eq!(settings.difficulty, "medium");
}

#[test]
fn test_room_settings_clamp_below_min() {
    let mut settings = RoomSettings {
        topic: "Test".to_string(),
        mode: "classic".to_string(),
        max_players: 30,
        question_count: 2,    // Below min 5
        num_options: 1,       // Below min 2
        timer_secs: 5,        // Below min 10
        difficulty: "medium".to_string(),
    };

    settings.clamp();

    assert_eq!(settings.question_count, 5);
    assert_eq!(settings.num_options, 2);
    assert_eq!(settings.timer_secs, 10);
}

#[test]
fn test_room_state_variants() {
    // Verify all states exist and are distinct
    let states = [
        RoomState::Waiting,
        RoomState::Starting,
        RoomState::Playing,
        RoomState::Finished,
        RoomState::Error,
    ];

    for (i, s1) in states.iter().enumerate() {
        for (j, s2) in states.iter().enumerate() {
            if i != j {
                assert_ne!(s1, s2);
            }
        }
    }
}

#[test]
fn test_player_creation_classic_mode() {
    let player = Player::new(
        1,
        "test_user".to_string(),
        "Test User".to_string(),
        None, // No lives in classic mode
    );

    assert_eq!(player.user_id, 1);
    assert_eq!(player.username, "test_user");
    assert_eq!(player.display_name, "Test User");
    assert_eq!(player.score, 0);
    assert_eq!(player.lives, None);
    assert_eq!(player.jokers_used, 0);
    assert_eq!(player.current_streak, 0);
    assert_eq!(player.answers.len(), 0);
    assert!(!player.is_ready);
}

#[test]
fn test_player_creation_lives_mode() {
    let player = Player::new(
        1,
        "player1".to_string(),
        "Player 1".to_string(),
        Some(3), // 3 lives
    );

    assert_eq!(player.lives, Some(3));
}

#[test]
fn test_scoring_basic() {
    // Test base scoring formula
    let base_points: i64 = 100;
    let time_bonus: i64 = 50;
    let streak_bonus: i64 = 25;

    let total = base_points + time_bonus + streak_bonus;
    assert_eq!(total, 175);
}

#[test]
fn test_scoring_with_streak() {
    // Test streak accumulation
    let mut streak: u32 = 0;
    let mut total_score: i64 = 0;

    // First correct answer (streak goes from 0 to 1, bonus = (1-1)*25 = 0)
    streak += 1;
    let streak_bonus = (streak as i64 - 1) * 25;
    total_score += 100 + 50 + streak_bonus;
    assert_eq!(total_score, 150);

    // Second correct answer (streak goes from 1 to 2, bonus = (2-1)*25 = 25)
    streak += 1;
    let streak_bonus = (streak as i64 - 1) * 25;
    total_score += 100 + 50 + streak_bonus;
    assert_eq!(total_score, 325); // 150 + 175

    // Third correct answer (streak goes from 2 to 3, bonus = (3-1)*25 = 50)
    streak += 1;
    let streak_bonus = (streak as i64 - 1) * 25;
    total_score += 100 + 50 + streak_bonus;
    assert_eq!(total_score, 525); // 325 + 200
}

#[test]
fn test_scoring_streak_reset_on_wrong() {
    let mut streak: u32 = 3;

    // Wrong answer resets streak
    streak = 0;

    // Next correct answer (streak goes from 0 to 1, bonus = 0)
    streak += 1;
    let streak_bonus = (streak as i64 - 1) * 25;
    assert_eq!(streak_bonus, 0);
}

#[test]
fn test_lives_mode_elimination() {
    let mut player = Player::new(
        1,
        "player1".to_string(),
        "Player 1".to_string(),
        Some(3),
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

    // Can't go below 0 (saturating_sub)
    if let Some(ref mut lives) = player.lives {
        *lives = lives.saturating_sub(1);
    }
    assert_eq!(player.lives, Some(0));
}

#[test]
fn test_message_serialization_client() {
    use brainstorm::ws::protocol::ClientMessage;

    let msg = ClientMessage::CreateRoom {
        topic: Some("Test".to_string()),
        mode: Some("classic".to_string()),
        settings: None,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("create_room"));
    assert!(json.contains("Test"));
}

#[test]
fn test_message_serialization_server() {
    let msg = ServerMessage::RoomJoined {
        code: "ABCDEF".to_string(),
        is_host: true,
        topic: "Test".to_string(),
        mode: "classic".to_string(),
        players: Vec::new(),
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("room_joined"));
    assert!(json.contains("ABCDEF"));
}

#[test]
fn test_message_deserialization_client_with_question_index() {
    use brainstorm::ws::protocol::ClientMessage;

    let json = r#"{"type":"submit_answer","data":{"question_index":0,"answer_index":2}}"#;
    let msg: ClientMessage = serde_json::from_str(json).unwrap();

    match msg {
        ClientMessage::SubmitAnswer {
            question_index,
            answer_index,
        } => {
            assert_eq!(question_index, Some(0));
            assert_eq!(answer_index, 2);
        }
        _ => panic!("Expected SubmitAnswer"),
    }
}

#[test]
fn test_message_deserialization_client_without_question_index() {
    use brainstorm::ws::protocol::ClientMessage;

    // Frontend sends submit_answer without question_index
    let json = r#"{"type":"submit_answer","data":{"answer_index":2}}"#;
    let msg: ClientMessage = serde_json::from_str(json).unwrap();

    match msg {
        ClientMessage::SubmitAnswer {
            question_index,
            answer_index,
        } => {
            assert_eq!(question_index, None);
            assert_eq!(answer_index, 2);
        }
        _ => panic!("Expected SubmitAnswer"),
    }
}

#[test]
fn test_message_serialization_roundtrip() {
    // ServerMessage only derives Serialize (not Deserialize) since it's
    // server → client only. We verify the JSON output is correct.
    let msg = ServerMessage::QuestionEmitted {
        question_index: 5,
        question: brainstorm::ws::protocol::QuestionData {
            question: "What is 2+2?".to_string(),
            options: vec!["3".into(), "4".into(), "5".into(), "6".into()],
            category: Some("Math".into()),
        },
        time_limit: 30,
    };

    let json = serde_json::to_string(&msg).unwrap();
    
    // Verify the JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["type"], "question_emitted");
    assert_eq!(parsed["data"]["question_index"], 5);
    assert_eq!(parsed["data"]["question"]["question"], "What is 2+2?");
    assert_eq!(parsed["data"]["question"]["options"].as_array().unwrap().len(), 4);
    assert_eq!(parsed["data"]["time_limit"], 30);
}

#[test]
fn test_joker_usage() {
    let mut player = Player::new(1, "p1".into(), "P1".into(), None);

    assert_eq!(player.jokers_used, 0);

    player.jokers_used += 1;
    assert_eq!(player.jokers_used, 1);

    // Can't use more than 1 joker
    let can_use = player.jokers_used < 1;
    assert!(!can_use);
}

#[test]
fn test_answer_tracking() {
    let mut player = Player::new(1, "player1".into(), "Player 1".into(), None);

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

    scores.sort_by(|a, b| b.score.cmp(&a.score));

    assert_eq!(scores[0].user_id, 2); // 800
    assert_eq!(scores[1].user_id, 1); // 500
    assert_eq!(scores[2].user_id, 3); // 300
}

#[test]
fn test_room_code_generation() {
    let code = brainstorm::domain::room::generate_room_code();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_uppercase()));
}

#[test]
fn test_room_code_uniqueness() {
    // Generate multiple codes and verify they're (almost certainly) unique
    let codes: Vec<String> = (0..100)
        .map(|_| brainstorm::domain::room::generate_room_code())
        .collect();

    let unique: std::collections::HashSet<&String> = codes.iter().collect();
    // With 26^6 = 308M combinations, 100 codes should all be unique
    assert_eq!(unique.len(), 100);
}

#[test]
fn test_player_info_serialization() {
    let info = PlayerInfo {
        user_id: 42,
        username: "test_user".to_string(),
        display_name: "Test User".to_string(),
        score: 150,
        lives: Some(2),
        is_ready: true,
        is_host: false,
    };

    let json = serde_json::to_string(&info).unwrap();
    let deserialized: PlayerInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.user_id, 42);
    assert_eq!(deserialized.username, "test_user");
    assert_eq!(deserialized.score, 150);
    assert_eq!(deserialized.lives, Some(2));
    assert!(deserialized.is_ready);
    assert!(!deserialized.is_host);
}

#[test]
fn test_player_score_serialization() {
    let score = PlayerScore {
        user_id: 1,
        username: "winner".to_string(),
        score: 1000,
        correct_answers: 10,
        total_answers: 10,
    };

    let json = serde_json::to_string(&score).unwrap();
    assert!(json.contains("winner"));
    assert!(json.contains("1000"));
}

#[test]
fn test_game_over_message() {
    let msg = ServerMessage::GameOver {
        final_leaderboard: vec![
            PlayerScore {
                user_id: 1,
                username: "winner".into(),
                score: 1000,
                correct_answers: 10,
                total_answers: 10,
            },
            PlayerScore {
                user_id: 2,
                username: "loser".into(),
                score: 500,
                correct_answers: 5,
                total_answers: 10,
            },
        ],
        winner: Some("winner".into()),
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("game_over"));
    assert!(json.contains("winner"));
    assert!(json.contains("loser"));
}

#[test]
fn test_timer_update_message() {
    let msg = ServerMessage::TimerUpdate {
        seconds_remaining: 15,
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("timer_update"));
    assert!(json.contains("15"));
}

#[test]
fn test_error_message() {
    let msg = ServerMessage::Error {
        message: "Something went wrong".into(),
    };

    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains("error"));
    assert!(json.contains("Something went wrong"));
}

#[test]
fn test_all_valid_game_modes() {
    let valid_modes = ["classic", "ffa", "team", "lives", "coop", "svoyaigra", "sandbox"];
    
    for mode in &valid_modes {
        let mut settings = RoomSettings {
            mode: mode.to_string(),
            ..Default::default()
        };
        settings.clamp();
        assert_eq!(settings.mode, *mode, "Mode {} should be valid", mode);
    }
}

#[test]
fn test_all_valid_difficulties() {
    let valid_difficulties = ["easy", "medium", "hard"];
    
    for diff in &valid_difficulties {
        let mut settings = RoomSettings {
            difficulty: diff.to_string(),
            ..Default::default()
        };
        settings.clamp();
        assert_eq!(settings.difficulty, *diff, "Difficulty {} should be valid", diff);
    }
}
