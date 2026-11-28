//! Proposal system for collaborative room settings.
//!
//! Players can propose changes to room settings (topic, mode, etc.)
//! and other players can vote on them. The host can accept proposals
//! to apply them to the room.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Types of proposals that can be made.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalType {
    Topic,
    Mode,
    Difficulty,
    QuestionCount,
    NumOptions,
    TimerSecs,
    AiProvider,
}

/// A proposal to change a room setting.
#[derive(Debug, Clone, Serialize)]
pub struct Proposal {
    /// Unique proposal ID.
    pub id: String,
    /// Player who created the proposal.
    pub proposer_sid: String,
    /// Player name (for display).
    pub proposer_name: String,
    /// What setting is being proposed.
    pub proposal_type: ProposalType,
    /// Proposed value (serialized as string for flexibility).
    pub value: String,
    /// Human-readable description of the proposal.
    pub description: String,
    /// Players who voted for this proposal (sid -> timestamp).
    pub votes: HashMap<String, Instant>,
    /// When the proposal was created.
    pub created_at: Instant,
    /// Whether this proposal is still active.
    pub active: bool,
}

impl Proposal {
    /// Create a new proposal.
    pub fn new(
        id: String,
        proposer_sid: String,
        proposer_name: String,
        proposal_type: ProposalType,
        value: String,
        description: String,
    ) -> Self {
        let mut votes = HashMap::new();
        // Proposer automatically votes for their own proposal.
        votes.insert(proposer_sid.clone(), Instant::now());

        Self {
            id,
            proposer_sid,
            proposer_name,
            proposal_type,
            value,
            description,
            votes,
            created_at: Instant::now(),
            active: true,
        }
    }

    /// Add a vote from a player. Returns true if vote was added.
    pub fn vote(&mut self, player_sid: &str) -> bool {
        if self.active && !self.votes.contains_key(player_sid) {
            self.votes.insert(player_sid.to_string(), Instant::now());
            true
        } else {
            false
        }
    }

    /// Remove a vote from a player. Returns true if vote was removed.
    pub fn unvote(&mut self, player_sid: &str) -> bool {
        if self.active && self.votes.remove(player_sid).is_some() {
            true
        } else {
            false
        }
    }

    /// Get vote count.
    pub fn vote_count(&self) -> usize {
        self.votes.len()
    }

    /// Check if a player has voted.
    pub fn has_voted(&self, player_sid: &str) -> bool {
        self.votes.contains_key(player_sid)
    }
}

/// Serializable proposal info for sending to clients.
#[derive(Debug, Clone, Serialize)]
pub struct ProposalInfo {
    pub id: String,
    pub proposer_sid: String,
    pub proposer_name: String,
    pub proposal_type: ProposalType,
    pub value: String,
    pub description: String,
    pub vote_count: usize,
    pub voted_by: Vec<String>,
    pub active: bool,
    pub created_at_secs: u64,
}

impl From<&Proposal> for ProposalInfo {
    fn from(proposal: &Proposal) -> Self {
        Self {
            id: proposal.id.clone(),
            proposer_sid: proposal.proposer_sid.clone(),
            proposer_name: proposal.proposer_name.clone(),
            proposal_type: proposal.proposal_type.clone(),
            value: proposal.value.clone(),
            description: proposal.description.clone(),
            vote_count: proposal.vote_count(),
            voted_by: proposal.votes.keys().cloned().collect(),
            active: proposal.active,
            created_at_secs: proposal.created_at.elapsed().as_secs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposal_creation() {
        let proposal = Proposal::new(
            "prop_1".to_string(),
            "player_1".to_string(),
            "Alice".to_string(),
            ProposalType::Topic,
            "History".to_string(),
            "Change topic to History".to_string(),
        );

        assert_eq!(proposal.proposer_sid, "player_1");
        assert_eq!(proposal.proposal_type, ProposalType::Topic);
        assert_eq!(proposal.value, "History");
        assert!(proposal.active);
        assert_eq!(proposal.vote_count(), 1); // Proposer auto-votes
        assert!(proposal.has_voted("player_1"));
    }

    #[test]
    fn test_proposal_voting() {
        let mut proposal = Proposal::new(
            "prop_1".to_string(),
            "player_1".to_string(),
            "Alice".to_string(),
            ProposalType::Mode,
            "classic".to_string(),
            "Change mode to Classic".to_string(),
        );

        assert!(proposal.vote("player_2"));
        assert_eq!(proposal.vote_count(), 2);
        assert!(proposal.has_voted("player_2"));

        // Cannot vote twice
        assert!(!proposal.vote("player_2"));
        assert_eq!(proposal.vote_count(), 2);
    }

    #[test]
    fn test_proposal_unvoting() {
        let mut proposal = Proposal::new(
            "prop_1".to_string(),
            "player_1".to_string(),
            "Alice".to_string(),
            ProposalType::Difficulty,
            "hard".to_string(),
            "Change difficulty to Hard".to_string(),
        );

        proposal.vote("player_2");
        assert_eq!(proposal.vote_count(), 2);

        assert!(proposal.unvote("player_2"));
        assert_eq!(proposal.vote_count(), 1);
        assert!(!proposal.has_voted("player_2"));
    }

    #[test]
    fn test_proposal_serialization() {
        let proposal = Proposal::new(
            "prop_1".to_string(),
            "player_1".to_string(),
            "Alice".to_string(),
            ProposalType::Topic,
            "Science".to_string(),
            "Change topic to Science".to_string(),
        );

        let info = ProposalInfo::from(&proposal);
        assert_eq!(info.id, "prop_1");
        assert_eq!(info.proposer_name, "Alice");
        assert_eq!(info.vote_count, 1);
    }
}
