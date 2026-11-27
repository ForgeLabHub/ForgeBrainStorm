# Dynamic Difficulty Adjustment (DDA)

Dynamic Difficulty Adjustment (DDA) is an intelligent system that automatically adapts question difficulty based on player performance in real-time. This feature ensures players remain engaged by providing an optimal challenge level.

## Overview

DDA analyzes player performance metrics and adjusts the difficulty of subsequent questions to maintain an appropriate challenge level. The system uses a sliding window approach to track performance and make data-driven difficulty adjustments.

## How It Works

### Performance Tracking

The DDA system tracks three key metrics for each player:

1. **Accuracy**: Percentage of correct answers
2. **Speed**: Average time to answer (normalized)
3. **Streak**: Consecutive correct/incorrect answers

These metrics are combined into a performance score ranging from 0.0 to 1.0, where:
- **0.0 - 0.3**: Struggling player (Easy difficulty)
- **0.3 - 0.7**: Average player (Medium difficulty)
- **0.7 - 1.0**: Advanced player (Hard difficulty)

### Sliding Window

DDA uses a sliding window of the last N questions (default: 5) to calculate performance. This ensures:
- Recent performance is weighted more heavily
- The system adapts quickly to changing player skill
- Outliers don't skew the difficulty too much

### Difficulty Adjustment

Based on the performance score, DDA determines the optimal difficulty level:

```
Performance Score → Difficulty Level
0.0 - 0.3        → Easy
0.3 - 0.7        → Medium
0.7 - 1.0        → Hard
```

The system then requests questions from the AI generator with the appropriate difficulty parameter.

## Usage

### Enabling DDA

DDA is enabled by creating a room with the `adaptive` mode:

```rust
// Via WebSocket API
{
    "type": "create_room",
    "topic": "World History",
    "mode": "adaptive"
}
```

Or via REST API:

```bash
curl -X POST http://localhost:5000/api/rooms \
  -H "Content-Type: application/json" \
  -d '{"topic": "World History", "mode": "adaptive"}'
```

### Player Experience

When DDA is enabled:

1. **Game Start**: All players are added to DDA tracking
2. **Answer Submission**: Each answer is recorded with accuracy and timing data
3. **Performance Calculation**: After each question, the system recalculates performance scores
4. **Difficulty Adjustment**: The next question batch is generated with adjusted difficulty

### Example Scenario

```
Player starts game → Medium difficulty
↓
Answers 5/5 correctly, quickly → Performance score: 0.9
↓
Next questions → Hard difficulty
↓
Answers 1/5 correctly, slowly → Performance score: 0.3
↓
Next questions → Easy difficulty
↓
Answers 4/5 correctly → Performance score: 0.7
↓
Next questions → Medium difficulty
```

## Architecture

### Components

1. **PerformanceTracker**: Tracks individual player performance
   - Records answers (correct/incorrect, time taken)
   - Calculates performance score using weighted metrics
   - Maintains sliding window of recent answers

2. **DifficultyEngine**: Determines optimal difficulty
   - Maps performance scores to difficulty levels
   - Configurable thresholds for easy/medium/hard

3. **DdaManager**: Coordinates DDA for a room
   - Manages multiple PerformanceTrackers (one per player)
   - Aggregates player performance for room-wide difficulty
   - Integrates with AI question generation

### Integration Points

```
┌─────────────────┐
│  Room Manager   │
└────────┬────────┘
         │
         ├─> Creates DdaManager when room mode = "adaptive"
         │
         ├─> Adds players to tracking on game start
         │
         └─> Records answers in submit_answer()
                 │
                 └─> Updates DdaManager
                         │
                         └─> Recalculates performance
                                 │
                                 └─> Adjusts next question difficulty
```

## Configuration

### Performance Thresholds

Default thresholds can be customized:

```rust
let engine = DifficultyEngine::with_thresholds(
    0.3,  // easy_threshold
    0.7,  // hard_threshold
);
```

### Window Size

The sliding window size can be adjusted:

```rust
let dda_manager = DdaManager::new(10); // Track last 10 questions
```

### Metric Weights

Performance score calculation uses weighted metrics:

```rust
let score = (accuracy_score * 0.6) +     // 60% accuracy
            (speed_score * 0.2) +        // 20% speed
            (streak_bonus * 0.2);        // 20% streak
```

These weights can be adjusted in the `calculate_performance_score()` method.

## API Reference

### PerformanceTracker

```rust
// Create a new tracker
let tracker = PerformanceTracker::new(user_id, username, window_size);

// Record an answer
tracker.record_answer(question_index, is_correct, time_ms);

// Get performance snapshot
let perf = tracker.get_performance();
// Returns: PlayerPerformance { score, accuracy, speed, streak, ... }
```

### DifficultyEngine

```rust
// Create engine with default thresholds
let engine = DifficultyEngine::new();

// Determine difficulty from score
let difficulty = engine.determine_difficulty(0.85); // Returns DifficultyLevel::Hard
```

### DdaManager

```rust
// Create manager
let manager = DdaManager::new(window_size);

// Add player
manager.add_player(user_id, username);

// Record answer
manager.record_answer(user_id, question_index, is_correct, time_ms);

// Get room-wide difficulty
let difficulty = manager.get_room_difficulty();
```

## Monitoring

DDA provides metrics for monitoring:

### Logs

```rust
INFO  DDA: initialized tracking for 4 players
DEBUG DDA: recorded answer user_id=123 question_index=2 is_correct=true time_ms=4500
INFO  DDA: performance score updated user_id=123 score=0.85 difficulty=Hard
```

### Metrics

Expose these metrics via your monitoring system:

- `dd_answers_total`: Total answers recorded
- `dd_performance_score`: Current performance score per player
- `dd_difficulty_adjustments`: Number of difficulty changes
- `dd_room_difficulty`: Current room-wide difficulty level

## Testing

Run DDA unit tests:

```bash
cargo test --lib services::ai::dynamic_difficulty
```

Run integration tests:

```bash
cargo test --test dda_integration_test
```

## Future Enhancements

Potential improvements for DDA:

1. **Machine Learning**: Use ML models to predict optimal difficulty
2. **Category-Specific DDA**: Different difficulty per question category
3. **Team DDA**: Aggregate team performance for cooperative modes
4. **Adaptive Timer**: Adjust time limits based on difficulty
5. **Player Feedback**: Allow players to rate question difficulty
6. **Historical Analysis**: Use past game data to improve initial difficulty

## Related Features

- **AI Question Generation**: DDA works with AI to generate questions at the right difficulty
- **Player Stats**: DDA data can be used to improve player statistics
- **Leaderboards**: DDA-adjusted games can have separate leaderboards

## Troubleshooting

### DDA not adjusting difficulty

- Check that room mode is set to "adaptive"
- Verify DdaManager is initialized (check logs)
- Ensure at least 1 answer has been recorded

### Difficulty too high/low

- Adjust performance thresholds in DifficultyEngine
- Increase window size for more stable adjustments
- Check metric weights in performance calculation

### Performance score not updating

- Verify answers are being recorded in submit_answer()
- Check that DdaManager is not None
- Review logs for recording errors

## References

- [Flow Theory](https://en.wikipedia.org/wiki/Flow_(psychology)) - Psychological basis for DDA
- [Dynamic Game Difficulty Balancing](https://en.wikipedia.org/wiki/Dynamic_game_difficulty_balancing) - Game design concept
- [AI Question Generation](./ai_question_generation.md) - How DDA integrates with AI
