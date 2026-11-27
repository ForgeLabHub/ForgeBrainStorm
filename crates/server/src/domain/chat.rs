//! In-room chat: sanitising, flood control and the emoji palette.
//!
//! The server owns every rule here. A modified client can send whatever it
//! likes; the room only ever broadcasts text this module produced.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use serde::Serialize;

/// Longest chat message the server keeps, in characters (not bytes).
pub const MAX_CHAT_CHARS: usize = 200;
/// Messages replayed to a player who joins (or reloads) mid-game.
pub const CHAT_HISTORY: usize = 50;
/// Minimum gap between two messages from the same player.
pub const CHAT_MIN_GAP: Duration = Duration::from_millis(400);
/// Sliding window for flood control: at most `CHAT_BURST` messages per window.
pub const CHAT_BURST: usize = 6;
pub const CHAT_WINDOW: Duration = Duration::from_secs(10);

/// Emojis a player may throw. Anything outside this list is rejected, so the
/// emoji channel cannot be used to smuggle arbitrary text to the room.
pub const EMOJI_PALETTE: [&str; 8] = ["😂", "🔥", "😱", "👏", "💀", "🤡", "❤️", "🎯"];

/// Directional formatting characters. They are invisible but reorder the text
/// around them, which is how chat spoofing ("Аня wrote: ...") works.
const BIDI_CONTROLS: [char; 9] = [
    '\u{202A}', '\u{202B}', '\u{202C}', '\u{202D}', '\u{202E}', '\u{2066}', '\u{2067}', '\u{2068}',
    '\u{2069}',
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatError {
    /// Nothing left after sanitising.
    Empty,
    /// Same player spoke too recently.
    TooFast,
    /// Too many messages in the sliding window.
    Flood,
    /// Emoji is not in [`EMOJI_PALETTE`].
    NotAllowed,
}

impl ChatError {
    pub fn code(self) -> &'static str {
        match self {
            ChatError::Empty => "chat_empty",
            ChatError::TooFast => "chat_too_fast",
            ChatError::Flood => "chat_flood",
            ChatError::NotAllowed => "emoji_not_allowed",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            ChatError::Empty => "Сообщение пустое или слишком короткое",
            ChatError::TooFast => "Не так быстро — подождите секунду",
            ChatError::Flood => "Слишком много сообщений, сделайте паузу",
            ChatError::NotAllowed => "Такой эмодзи недоступен",
        }
    }
}

/// One line in the room chat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChatMessage {
    pub seq: u64,
    pub from: String,
    pub text: String,
    /// `text` or `emoji`; the client renders them differently.
    pub kind: &'static str,
}

/// Trim, drop control characters, collapse whitespace and cut to the limit.
pub fn sanitize(raw: &str) -> Option<String> {
    let cleaned: String = raw
        .chars()
        .map(|c| {
            if c.is_control() || BIDI_CONTROLS.contains(&c) {
                ' '
            } else {
                c
            }
        })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return None;
    }
    Some(collapsed.chars().take(MAX_CHAT_CHARS).collect())
}

/// Chat history plus the per-player rate limiting state for one room.
#[derive(Debug, Default)]
pub struct ChatLog {
    messages: VecDeque<ChatMessage>,
    seq: u64,
    last_at: HashMap<String, Instant>,
    window: HashMap<String, VecDeque<Instant>>,
}

impl ChatLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, sid: &str, from: &str, raw: &str) -> Result<ChatMessage, ChatError> {
        self.push_at(sid, from, raw, Instant::now())
    }

    /// Same as [`Self::push`] with an explicit clock, so tests can drive time.
    pub fn push_at(
        &mut self,
        sid: &str,
        from: &str,
        raw: &str,
        now: Instant,
    ) -> Result<ChatMessage, ChatError> {
        let text = sanitize(raw).ok_or(ChatError::Empty)?;
        self.throttle(sid, now)?;
        Ok(self.record(from, text, "text"))
    }

    pub fn push_emoji(
        &mut self,
        sid: &str,
        from: &str,
        emoji: &str,
    ) -> Result<ChatMessage, ChatError> {
        self.push_emoji_at(sid, from, emoji, Instant::now())
    }

    pub fn push_emoji_at(
        &mut self,
        sid: &str,
        from: &str,
        emoji: &str,
        now: Instant,
    ) -> Result<ChatMessage, ChatError> {
        if !EMOJI_PALETTE.contains(&emoji) {
            return Err(ChatError::NotAllowed);
        }
        self.throttle(sid, now)?;
        Ok(self.record(from, emoji.to_owned(), "emoji"))
    }

    /// Messages still worth replaying, oldest first.
    pub fn history(&self) -> Vec<ChatMessage> {
        self.messages.iter().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

    fn record(&mut self, from: &str, text: String, kind: &'static str) -> ChatMessage {
        self.seq += 1;
        let message = ChatMessage {
            seq: self.seq,
            from: from.to_owned(),
            text,
            kind,
        };
        self.messages.push_back(message.clone());
        while self.messages.len() > CHAT_HISTORY {
            self.messages.pop_front();
        }
        message
    }

    fn throttle(&mut self, sid: &str, now: Instant) -> Result<(), ChatError> {
        if let Some(last) = self.last_at.get(sid) {
            if now.saturating_duration_since(*last) < CHAT_MIN_GAP {
                return Err(ChatError::TooFast);
            }
        }

        let window = self.window.entry(sid.to_owned()).or_default();
        while window
            .front()
            .is_some_and(|sent| now.saturating_duration_since(*sent) > CHAT_WINDOW)
        {
            window.pop_front();
        }
        if window.len() >= CHAT_BURST {
            return Err(ChatError::Flood);
        }
        window.push_back(now);
        self.last_at.insert(sid.to_owned(), now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(base: Instant, millis: u64) -> Instant {
        base + Duration::from_millis(millis)
    }

    #[test]
    fn sanitize_strips_control_characters_and_collapses_whitespace() {
        assert_eq!(sanitize("  привет\n\tмир  ").as_deref(), Some("привет мир"));
        assert_eq!(
            sanitize("\u{202E}перевёрнутый").as_deref(),
            Some("перевёрнутый")
        );
        assert_eq!(sanitize("   \n\t "), None);
    }

    #[test]
    fn sanitize_cuts_long_messages_by_characters_not_bytes() {
        let long = "я".repeat(MAX_CHAT_CHARS + 50);
        let clean = sanitize(&long).expect("not empty");
        assert_eq!(clean.chars().count(), MAX_CHAT_CHARS);
    }

    #[test]
    fn a_message_reaches_the_history_with_a_growing_seq() {
        let mut log = ChatLog::new();
        let now = Instant::now();
        let first = log.push_at("sid", "Аня", "привет", now).expect("sent");
        let second = log
            .push_at("sid", "Аня", "как дела", at(now, 1_000))
            .expect("sent");
        assert_eq!((first.seq, second.seq), (1, 2));
        assert_eq!(log.history().len(), 2);
        assert_eq!(log.history()[0].kind, "text");
    }

    #[test]
    fn the_same_player_cannot_spam_two_messages_at_once() {
        let mut log = ChatLog::new();
        let now = Instant::now();
        log.push_at("sid", "Аня", "раз", now).expect("sent");
        assert_eq!(
            log.push_at("sid", "Аня", "два", at(now, 10)),
            Err(ChatError::TooFast)
        );
        assert!(log.push_at("sid", "Аня", "три", at(now, 500)).is_ok());
    }

    #[test]
    fn the_burst_window_stops_a_flood_even_with_gaps() {
        let mut log = ChatLog::new();
        let now = Instant::now();
        for index in 0..CHAT_BURST {
            let when = at(now, index as u64 * 500);
            log.push_at("sid", "Аня", "спам", when)
                .expect("within budget");
        }
        let blocked = at(now, CHAT_BURST as u64 * 500);
        assert_eq!(
            log.push_at("sid", "Аня", "ещё", blocked),
            Err(ChatError::Flood)
        );
        // Once the window slides past the oldest messages, the player may speak again.
        assert!(log
            .push_at(
                "sid",
                "Аня",
                "снова",
                at(now, CHAT_WINDOW.as_millis() as u64 + 1_000)
            )
            .is_ok());
    }

    #[test]
    fn one_player_flooding_does_not_mute_another() {
        let mut log = ChatLog::new();
        let now = Instant::now();
        log.push_at("a", "Аня", "раз", now).expect("sent");
        assert!(log.push_at("b", "Боря", "раз", now).is_ok());
    }

    #[test]
    fn history_is_capped_and_keeps_the_newest() {
        let mut log = ChatLog::new();
        let now = Instant::now();
        for index in 0..(CHAT_HISTORY + 10) {
            log.push_at(
                "sid",
                "Аня",
                &format!("msg {index}"),
                at(now, index as u64 * 5_000),
            )
            .expect("sent");
        }
        let history = log.history();
        assert_eq!(history.len(), CHAT_HISTORY);
        assert!(history.last().expect("not empty").text.ends_with("59"));
    }

    #[test]
    fn only_palette_emojis_are_accepted() {
        let mut log = ChatLog::new();
        let now = Instant::now();
        assert_eq!(
            log.push_emoji_at("sid", "Аня", "💥", now),
            Err(ChatError::NotAllowed)
        );
        let sent = log
            .push_emoji_at("sid", "Аня", "🔥", at(now, 1_000))
            .expect("palette emoji");
        assert_eq!(sent.kind, "emoji");
        assert_eq!(sent.text, "🔥");
    }

    #[test]
    fn clearing_keeps_the_sequence_monotonic() {
        let mut log = ChatLog::new();
        let now = Instant::now();
        log.push_at("sid", "Аня", "раз", now).expect("sent");
        log.clear();
        let next = log
            .push_at("sid", "Аня", "два", at(now, 1_000))
            .expect("sent");
        assert_eq!(next.seq, 2);
        assert_eq!(log.len(), 1);
    }
}
