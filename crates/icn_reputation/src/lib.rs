// File: crates/icn_reputation/src/lib.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationScore {
    score: f64,
    last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    entity_id: String,
    event_type: ReputationEventType,
    value: f64,
    timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationEventType {
    Contribution,
    Participation,
    Feedback,
    Violation,
}

pub struct ReputationSystem {
    scores: HashMap<String, ReputationScore>,
    events: Vec<ReputationEvent>,
    decay_rate: f64,
    min_score: f64,
    max_score: f64,
}

impl ReputationSystem {
    pub fn new(decay_rate: f64, min_score: f64, max_score: f64) -> Self {
        ReputationSystem {
            scores: HashMap::new(),
            events: Vec::new(),
            decay_rate,
            min_score,
            max_score,
        }
    }

    pub fn add_event(&mut self, event: ReputationEvent) {
        self.events.push(event.clone());
        self.update_score(&event.entity_id, event);
    }

    pub fn get_score(&self, entity_id: &str) -> Option<f64> {
        self.scores.get(entity_id).map(|score| score.score)
    }

    fn update_score(&mut self, entity_id: &str, event: ReputationEvent) {
        let score = self.scores.entry(entity_id.to_string()).or_insert(ReputationScore {
            score: 0.0,
            last_updated: Utc::now(),
        });

        // Apply time decay
        let time_diff = (Utc::now() - score.last_updated).num_days() as f64;
        score.score *= (1.0 - self.decay_rate).powf(time_diff);

        // Update score based on event
        match event.event_type {
            ReputationEventType::Contribution | ReputationEventType::Participation => {
                score.score += event.value;
            }
            ReputationEventType::Feedback => {
                score.score += event.value * 0.5; // Feedback has less impact
            }
            ReputationEventType::Violation => {
                score.score -= event.value;
            }
        }

        // Ensure score is within bounds
        score.score = score.score.clamp(self.min_score, self.max_score);
        score.last_updated = Utc::now();
    }

    pub fn get_top_entities(&self, limit: usize) -> Vec<(String, f64)> {
        let mut entities: Vec<(String, f64)> = self.scores
            .iter()
            .map(|(id, score)| (id.clone(), score.score))
            .collect();
        entities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        entities.truncate(limit);
        entities
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_system() {
        let mut system = ReputationSystem::new(0.01, 0.0, 100.0);

        // Add some events
        system.add_event(ReputationEvent {
            entity_id: "Alice".to_string(),
            event_type: ReputationEventType::Contribution,
            value: 10.0,
            timestamp: Utc::now(),
        });

        system.add_event(ReputationEvent {
            entity_id: "Bob".to_string(),
            event_type: ReputationEventType::Participation,
            value: 5.0,
            timestamp: Utc::now(),
        });

        // Check scores
        assert!(system.get_score("Alice").unwrap() > system.get_score("Bob").unwrap());

        // Add a violation for Alice
        system.add_event(ReputationEvent {
            entity_id: "Alice".to_string(),
            event_type: ReputationEventType::Violation,
            value: 3.0,
            timestamp: Utc::now(),
        });

        // Check top entities
        let top = system.get_top_entities(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, "Alice");
        assert_eq!(top[1].0, "Bob");
    }
}