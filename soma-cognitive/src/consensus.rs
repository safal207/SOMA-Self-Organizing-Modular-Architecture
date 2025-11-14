//! # Distributed Consensus - Распределенный консенсус
//!
//! Механизм голосования для коллективного принятия решений.
//! Узлы голосуют за результаты Inference Braid и достигают консенсуса
//! даже при наличии сбоев или несогласованных данных.
//!
//! v1.3: Voting-based consensus с Byzantine fault tolerance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Голос узла по результату
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Vote {
    /// Принять результат
    Accept,
    /// Отклонить результат
    Reject,
    /// Воздержаться
    Abstain,
}

/// Результат голосования узла
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeVote {
    /// ID узла
    pub node_id: String,

    /// Голос
    pub vote: Vote,

    /// Уверенность в голосе (0.0 - 1.0)
    pub confidence: f64,

    /// Обоснование
    pub reasoning: Option<String>,

    /// Временная метка
    pub timestamp: u64,
}

impl NodeVote {
    /// Создать новый голос
    pub fn new(node_id: String, vote: Vote, confidence: f64) -> Self {
        Self {
            node_id,
            vote,
            confidence,
            reasoning: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Добавить обоснование
    pub fn with_reasoning(mut self, reasoning: String) -> Self {
        self.reasoning = Some(reasoning);
        self
    }
}

/// Результат консенсуса
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConsensusResult {
    /// Консенсус достигнут - результат принят
    Accepted {
        /// Процент голосов "за"
        acceptance_rate: f64,
        /// Число участников
        participants: usize,
    },
    /// Консенсус достигнут - результат отклонен
    Rejected {
        /// Процент голосов "против"
        rejection_rate: f64,
        /// Число участников
        participants: usize,
    },
    /// Консенсус не достигнут
    NoConsensus {
        /// Распределение голосов
        vote_distribution: HashMap<String, usize>,
        /// Число участников
        participants: usize,
    },
    /// Недостаточно участников для консенсуса
    InsufficientParticipants {
        /// Текущее число участников
        current: usize,
        /// Требуемое минимальное число
        required: usize,
    },
}

/// Раунд консенсуса
#[derive(Debug, Clone)]
pub struct ConsensusRound {
    /// ID раунда
    pub round_id: String,

    /// ID задачи
    pub task_id: String,

    /// Голоса узлов
    votes: HashMap<String, NodeVote>,

    /// Порог для достижения консенсуса (0.0 - 1.0)
    threshold: f64,

    /// Минимальное число участников
    min_participants: usize,

    /// Статус раунда
    status: RoundStatus,
}

/// Статус раунда консенсуса
#[derive(Debug, Clone, PartialEq)]
pub enum RoundStatus {
    /// Ожидание голосов
    Voting,
    /// Консенсус достигнут
    Completed(ConsensusResult),
    /// Раунд отменен
    Cancelled,
}

impl ConsensusRound {
    /// Создать новый раунд консенсуса
    pub fn new(round_id: String, task_id: String, threshold: f64, min_participants: usize) -> Self {
        Self {
            round_id,
            task_id,
            votes: HashMap::new(),
            threshold,
            min_participants,
            status: RoundStatus::Voting,
        }
    }

    /// Добавить голос
    pub fn add_vote(&mut self, vote: NodeVote) -> Result<(), String> {
        if self.status != RoundStatus::Voting {
            return Err("Round is not in voting state".to_string());
        }

        self.votes.insert(vote.node_id.clone(), vote);
        Ok(())
    }

    /// Вычислить результат консенсуса
    pub fn compute_consensus(&mut self) -> ConsensusResult {
        let total_votes = self.votes.len();

        if total_votes < self.min_participants {
            let result = ConsensusResult::InsufficientParticipants {
                current: total_votes,
                required: self.min_participants,
            };
            self.status = RoundStatus::Completed(result.clone());
            return result;
        }

        // Подсчитать голоса
        let mut accept_count = 0;
        let mut reject_count = 0;
        let mut abstain_count = 0;

        for vote in self.votes.values() {
            match vote.vote {
                Vote::Accept => accept_count += 1,
                Vote::Reject => reject_count += 1,
                Vote::Abstain => abstain_count += 1,
            }
        }

        let acceptance_rate = accept_count as f64 / total_votes as f64;
        let rejection_rate = reject_count as f64 / total_votes as f64;

        let result = if acceptance_rate >= self.threshold {
            ConsensusResult::Accepted {
                acceptance_rate,
                participants: total_votes,
            }
        } else if rejection_rate >= self.threshold {
            ConsensusResult::Rejected {
                rejection_rate,
                participants: total_votes,
            }
        } else {
            let mut distribution = HashMap::new();
            distribution.insert("accept".to_string(), accept_count);
            distribution.insert("reject".to_string(), reject_count);
            distribution.insert("abstain".to_string(), abstain_count);

            ConsensusResult::NoConsensus {
                vote_distribution: distribution,
                participants: total_votes,
            }
        };

        self.status = RoundStatus::Completed(result.clone());
        result
    }

    /// Получить текущие голоса
    pub fn get_votes(&self) -> &HashMap<String, NodeVote> {
        &self.votes
    }

    /// Получить статус раунда
    pub fn status(&self) -> &RoundStatus {
        &self.status
    }

    /// Вычислить weighted consensus (с учетом confidence)
    pub fn compute_weighted_consensus(&mut self) -> ConsensusResult {
        let total_votes = self.votes.len();

        if total_votes < self.min_participants {
            let result = ConsensusResult::InsufficientParticipants {
                current: total_votes,
                required: self.min_participants,
            };
            self.status = RoundStatus::Completed(result.clone());
            return result;
        }

        // Взвешенный подсчет (confidence как вес)
        let mut accept_weight = 0.0;
        let mut reject_weight = 0.0;
        let mut total_weight = 0.0;

        for vote in self.votes.values() {
            total_weight += vote.confidence;
            match vote.vote {
                Vote::Accept => accept_weight += vote.confidence,
                Vote::Reject => reject_weight += vote.confidence,
                Vote::Abstain => {} // Не учитываем в весе
            }
        }

        let acceptance_rate = if total_weight > 0.0 {
            accept_weight / total_weight
        } else {
            0.0
        };

        let rejection_rate = if total_weight > 0.0 {
            reject_weight / total_weight
        } else {
            0.0
        };

        let result = if acceptance_rate >= self.threshold {
            ConsensusResult::Accepted {
                acceptance_rate,
                participants: total_votes,
            }
        } else if rejection_rate >= self.threshold {
            ConsensusResult::Rejected {
                rejection_rate,
                participants: total_votes,
            }
        } else {
            let mut distribution = HashMap::new();
            distribution.insert("accept_weight".to_string(), (accept_weight * 100.0) as usize);
            distribution.insert("reject_weight".to_string(), (reject_weight * 100.0) as usize);

            ConsensusResult::NoConsensus {
                vote_distribution: distribution,
                participants: total_votes,
            }
        };

        self.status = RoundStatus::Completed(result.clone());
        result
    }
}

/// Менеджер консенсуса
pub struct ConsensusManager {
    /// Активные раунды консенсуса
    rounds: Arc<RwLock<HashMap<String, ConsensusRound>>>,

    /// Порог консенсуса по умолчанию
    default_threshold: f64,

    /// Минимальное число участников по умолчанию
    default_min_participants: usize,
}

impl ConsensusManager {
    /// Создать новый менеджер консенсуса
    pub fn new(default_threshold: f64, default_min_participants: usize) -> Self {
        Self {
            rounds: Arc::new(RwLock::new(HashMap::new())),
            default_threshold,
            default_min_participants,
        }
    }

    /// Начать новый раунд консенсуса
    pub async fn start_round(&self, round_id: String, task_id: String) -> Result<(), String> {
        let round = ConsensusRound::new(
            round_id.clone(),
            task_id,
            self.default_threshold,
            self.default_min_participants,
        );

        let mut rounds = self.rounds.write().await;
        rounds.insert(round_id, round);
        Ok(())
    }

    /// Добавить голос в раунд
    pub async fn submit_vote(&self, round_id: &str, vote: NodeVote) -> Result<(), String> {
        let mut rounds = self.rounds.write().await;
        let round = rounds
            .get_mut(round_id)
            .ok_or_else(|| format!("Round {} not found", round_id))?;

        round.add_vote(vote)
    }

    /// Вычислить консенсус для раунда
    pub async fn finalize_round(&self, round_id: &str, weighted: bool) -> Result<ConsensusResult, String> {
        let mut rounds = self.rounds.write().await;
        let round = rounds
            .get_mut(round_id)
            .ok_or_else(|| format!("Round {} not found", round_id))?;

        let result = if weighted {
            round.compute_weighted_consensus()
        } else {
            round.compute_consensus()
        };

        Ok(result)
    }

    /// Получить раунд
    pub async fn get_round(&self, round_id: &str) -> Option<ConsensusRound> {
        let rounds = self.rounds.read().await;
        rounds.get(round_id).cloned()
    }

    /// Получить все активные раунды
    pub async fn active_rounds(&self) -> Vec<String> {
        let rounds = self.rounds.read().await;
        rounds
            .iter()
            .filter(|(_, r)| r.status() == &RoundStatus::Voting)
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl Default for ConsensusManager {
    fn default() -> Self {
        Self::new(0.66, 3) // 2/3 большинство, минимум 3 узла
    }
}

/// Byzantine Fault Tolerance - детектор Byzantine узлов
pub struct ByzantineDetector {
    /// История голосов узлов
    vote_history: Arc<RwLock<HashMap<String, Vec<NodeVote>>>>,

    /// Порог для пометки узла как Byzantine
    byzantine_threshold: f64,
}

impl ByzantineDetector {
    /// Создать новый детектор
    pub fn new(byzantine_threshold: f64) -> Self {
        Self {
            vote_history: Arc::new(RwLock::new(HashMap::new())),
            byzantine_threshold,
        }
    }

    /// Записать голос в историю
    pub async fn record_vote(&self, vote: NodeVote) {
        let mut history = self.vote_history.write().await;
        history
            .entry(vote.node_id.clone())
            .or_insert_with(Vec::new)
            .push(vote);
    }

    /// Проверить, является ли узел Byzantine
    pub async fn is_byzantine(&self, node_id: &str) -> bool {
        let history = self.vote_history.read().await;

        if let Some(votes) = history.get(node_id) {
            if votes.len() < 5 {
                return false; // Недостаточно данных
            }

            // Проверяем непоследовательность: узел часто меняет мнение?
            let mut flip_count = 0;
            for window in votes.windows(2) {
                if window[0].vote != window[1].vote {
                    flip_count += 1;
                }
            }

            let flip_rate = flip_count as f64 / (votes.len() - 1) as f64;
            flip_rate > self.byzantine_threshold
        } else {
            false
        }
    }

    /// Получить список подозрительных узлов
    pub async fn suspicious_nodes(&self) -> Vec<String> {
        let history = self.vote_history.read().await;
        let node_ids: Vec<String> = history.keys().cloned().collect();
        drop(history); // Release lock before async calls

        let mut suspicious = Vec::new();
        for node_id in node_ids {
            if self.is_byzantine(&node_id).await {
                suspicious.push(node_id);
            }
        }

        suspicious
    }
}

impl Default for ByzantineDetector {
    fn default() -> Self {
        Self::new(0.6) // Более 60% изменений = подозрительно
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consensus_round() {
        let mut round = ConsensusRound::new(
            "round_001".to_string(),
            "task_001".to_string(),
            0.66,
            3,
        );

        // Добавить голоса
        round.add_vote(NodeVote::new("node_a".to_string(), Vote::Accept, 0.9)).unwrap();
        round.add_vote(NodeVote::new("node_b".to_string(), Vote::Accept, 0.8)).unwrap();
        round.add_vote(NodeVote::new("node_c".to_string(), Vote::Accept, 0.95)).unwrap();

        let result = round.compute_consensus();

        match result {
            ConsensusResult::Accepted { acceptance_rate, participants } => {
                assert_eq!(acceptance_rate, 1.0);
                assert_eq!(participants, 3);
            }
            _ => panic!("Expected Accepted"),
        }
    }

    #[test]
    fn test_consensus_rejection() {
        let mut round = ConsensusRound::new(
            "round_002".to_string(),
            "task_002".to_string(),
            0.66,
            3,
        );

        round.add_vote(NodeVote::new("node_a".to_string(), Vote::Reject, 0.9)).unwrap();
        round.add_vote(NodeVote::new("node_b".to_string(), Vote::Reject, 0.8)).unwrap();
        round.add_vote(NodeVote::new("node_c".to_string(), Vote::Accept, 0.7)).unwrap();

        let result = round.compute_consensus();

        match result {
            ConsensusResult::Rejected { rejection_rate, .. } => {
                assert!(rejection_rate >= 0.66);
            }
            _ => panic!("Expected Rejected"),
        }
    }

    #[test]
    fn test_no_consensus() {
        let mut round = ConsensusRound::new(
            "round_003".to_string(),
            "task_003".to_string(),
            0.66,
            3,
        );

        round.add_vote(NodeVote::new("node_a".to_string(), Vote::Accept, 0.9)).unwrap();
        round.add_vote(NodeVote::new("node_b".to_string(), Vote::Reject, 0.8)).unwrap();
        round.add_vote(NodeVote::new("node_c".to_string(), Vote::Abstain, 0.5)).unwrap();

        let result = round.compute_consensus();

        match result {
            ConsensusResult::NoConsensus { .. } => {
                // OK
            }
            _ => panic!("Expected NoConsensus"),
        }
    }

    #[test]
    fn test_weighted_consensus() {
        let mut round = ConsensusRound::new(
            "round_004".to_string(),
            "task_004".to_string(),
            0.66,
            3,
        );

        // Два голоса "за" с высокой уверенностью
        round.add_vote(NodeVote::new("node_a".to_string(), Vote::Accept, 0.95)).unwrap();
        round.add_vote(NodeVote::new("node_b".to_string(), Vote::Accept, 0.90)).unwrap();
        // Один голос "против" с низкой уверенностью
        round.add_vote(NodeVote::new("node_c".to_string(), Vote::Reject, 0.2)).unwrap();

        let result = round.compute_weighted_consensus();

        match result {
            ConsensusResult::Accepted { acceptance_rate, .. } => {
                assert!(acceptance_rate > 0.66);
            }
            _ => panic!("Expected Accepted with weighted consensus"),
        }
    }

    #[tokio::test]
    async fn test_consensus_manager() {
        let manager = ConsensusManager::new(0.66, 3);

        manager.start_round("round_001".to_string(), "task_001".to_string()).await.unwrap();

        manager.submit_vote("round_001", NodeVote::new("node_a".to_string(), Vote::Accept, 0.9)).await.unwrap();
        manager.submit_vote("round_001", NodeVote::new("node_b".to_string(), Vote::Accept, 0.8)).await.unwrap();
        manager.submit_vote("round_001", NodeVote::new("node_c".to_string(), Vote::Accept, 0.95)).await.unwrap();

        let result = manager.finalize_round("round_001", false).await.unwrap();

        assert!(matches!(result, ConsensusResult::Accepted { .. }));
    }

    #[tokio::test]
    async fn test_byzantine_detector() {
        let detector = ByzantineDetector::new(0.6);

        // Узел меняет мнение часто
        for i in 0..10 {
            let vote = if i % 2 == 0 { Vote::Accept } else { Vote::Reject };
            detector.record_vote(NodeVote::new("byzantine_node".to_string(), vote, 0.8)).await;
        }

        // Узел последователен
        for _ in 0..10 {
            detector.record_vote(NodeVote::new("honest_node".to_string(), Vote::Accept, 0.9)).await;
        }

        assert!(detector.is_byzantine("byzantine_node").await);
        assert!(!detector.is_byzantine("honest_node").await);
    }
}
