//! # Decision Tracker
//!
//! Отслеживание решений Domino Luck Engine для анализа и обучения.
//! Каждое решение записывается с outcome для последующей рефлексии.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Результат выполнения решения Domino
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DecisionOutcome {
    /// Успешное выполнение
    Success {
        actual_latency_ms: f64,
        actual_quality: f64,
    },
    /// Неудачное выполнение (timeout, error, etc.)
    Failure { reason: String },
    /// Частичный успех
    Partial {
        completed_ratio: f64,
        issues: Vec<String>,
    },
    /// Еще не известен (решение только что принято)
    Pending,
}

impl DecisionOutcome {
    /// Проверить, было ли решение успешным
    pub fn is_success(&self) -> bool {
        matches!(self, DecisionOutcome::Success { .. })
    }

    /// Получить числовую оценку успешности (0.0 - 1.0)
    pub fn success_score(&self) -> f64 {
        match self {
            DecisionOutcome::Success { .. } => 1.0,
            DecisionOutcome::Partial { completed_ratio, .. } => *completed_ratio,
            DecisionOutcome::Failure { .. } => 0.0,
            DecisionOutcome::Pending => 0.5, // нейтральная оценка
        }
    }
}

/// Trace решения Domino Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominoDecisionTrace {
    /// Уникальный ID решения
    pub decision_id: String,

    /// Timestamp принятия решения
    pub timestamp: u64,

    /// Тип намерения
    pub intent_kind: String,

    /// Контекстные теги
    pub context_tags: Vec<String>,

    /// Список кандидатов (peer_id)
    pub candidates: Vec<String>,

    /// Выбранный лучший пир
    pub chosen_peer: String,

    /// Luck score решения
    pub luck_score: f32,

    /// Resistance score решения
    pub resistance_score: f32,

    /// Объяснение решения
    pub explanation: String,

    /// Результат выполнения (может быть обновлён позже)
    pub outcome: DecisionOutcome,

    /// Node ID, который принял решение
    pub node_id: String,
}

impl DominoDecisionTrace {
    /// Создать новый trace решения
    pub fn new(
        decision_id: String,
        timestamp: u64,
        intent_kind: String,
        context_tags: Vec<String>,
        candidates: Vec<String>,
        chosen_peer: String,
        luck_score: f32,
        resistance_score: f32,
        explanation: String,
        node_id: String,
    ) -> Self {
        Self {
            decision_id,
            timestamp,
            intent_kind,
            context_tags,
            candidates,
            chosen_peer,
            luck_score,
            resistance_score,
            explanation,
            outcome: DecisionOutcome::Pending,
            node_id,
        }
    }

    /// Обновить результат выполнения
    pub fn update_outcome(&mut self, outcome: DecisionOutcome) {
        self.outcome = outcome;
    }

    /// Проверить, было ли это "удачное" решение (high luck, success outcome)
    pub fn was_lucky(&self) -> bool {
        self.luck_score > 0.7 && self.outcome.is_success()
    }

    /// Проверить, было ли это "неудачное" решение (low luck, failure outcome)
    pub fn was_unlucky(&self) -> bool {
        self.luck_score < 0.4 && !self.outcome.is_success()
    }
}

/// Хранилище истории решений Domino
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionHistory {
    /// Очередь traces (FIFO)
    traces: VecDeque<DominoDecisionTrace>,

    /// Максимальный размер истории
    max_size: usize,
}

impl DecisionHistory {
    /// Создать новое хранилище с максимальным размером
    pub fn new(max_size: usize) -> Self {
        Self {
            traces: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Добавить новый trace
    pub fn add_trace(&mut self, trace: DominoDecisionTrace) {
        if self.traces.len() >= self.max_size {
            self.traces.pop_front(); // Удалить самый старый
        }
        self.traces.push_back(trace);
    }

    /// Получить все traces
    pub fn get_all(&self) -> Vec<DominoDecisionTrace> {
        self.traces.iter().cloned().collect()
    }

    /// Получить последние N traces
    pub fn get_recent(&self, n: usize) -> Vec<DominoDecisionTrace> {
        self.traces
            .iter()
            .rev()
            .take(n)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Получить traces для конкретного пира
    pub fn get_by_peer(&self, peer_id: &str) -> Vec<DominoDecisionTrace> {
        self.traces
            .iter()
            .filter(|t| t.chosen_peer == peer_id)
            .cloned()
            .collect()
    }

    /// Получить traces для конкретного типа намерения
    pub fn get_by_intent(&self, intent_kind: &str) -> Vec<DominoDecisionTrace> {
        self.traces
            .iter()
            .filter(|t| t.intent_kind == intent_kind)
            .cloned()
            .collect()
    }

    /// Получить traces в заданном временном окне (timestamp)
    pub fn get_by_time_window(&self, start: u64, end: u64) -> Vec<DominoDecisionTrace> {
        self.traces
            .iter()
            .filter(|t| t.timestamp >= start && t.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Обновить outcome для конкретного decision_id
    pub fn update_outcome(&mut self, decision_id: &str, outcome: DecisionOutcome) -> bool {
        if let Some(trace) = self.traces.iter_mut().find(|t| t.decision_id == decision_id) {
            trace.update_outcome(outcome);
            true
        } else {
            false
        }
    }

    /// Получить статистику успешности
    pub fn get_success_stats(&self) -> DecisionStats {
        let total = self.traces.len();
        if total == 0 {
            return DecisionStats::default();
        }

        let successes = self
            .traces
            .iter()
            .filter(|t| t.outcome.is_success())
            .count();

        let avg_luck_score = self.traces.iter().map(|t| t.luck_score as f64).sum::<f64>()
            / total as f64;

        let avg_resistance_score = self
            .traces
            .iter()
            .map(|t| t.resistance_score as f64)
            .sum::<f64>()
            / total as f64;

        let lucky_decisions = self.traces.iter().filter(|t| t.was_lucky()).count();

        let unlucky_decisions = self.traces.iter().filter(|t| t.was_unlucky()).count();

        DecisionStats {
            total_decisions: total,
            successful_decisions: successes,
            success_rate: successes as f64 / total as f64,
            avg_luck_score,
            avg_resistance_score,
            lucky_decisions,
            unlucky_decisions,
        }
    }

    /// Очистить всю историю
    pub fn clear(&mut self) {
        self.traces.clear();
    }

    /// Получить количество traces
    pub fn len(&self) -> usize {
        self.traces.len()
    }

    /// Проверить, пуста ли история
    pub fn is_empty(&self) -> bool {
        self.traces.is_empty()
    }
}

/// Статистика решений
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DecisionStats {
    /// Общее количество решений
    pub total_decisions: usize,

    /// Количество успешных решений
    pub successful_decisions: usize,

    /// Процент успешности (0.0 - 1.0)
    pub success_rate: f64,

    /// Средний luck_score
    pub avg_luck_score: f64,

    /// Средний resistance_score
    pub avg_resistance_score: f64,

    /// Количество "удачных" решений (high luck + success)
    pub lucky_decisions: usize,

    /// Количество "неудачных" решений (low luck + failure)
    pub unlucky_decisions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_trace(
        decision_id: &str,
        chosen_peer: &str,
        luck_score: f32,
        outcome: DecisionOutcome,
    ) -> DominoDecisionTrace {
        DominoDecisionTrace {
            decision_id: decision_id.to_string(),
            timestamp: 1000,
            intent_kind: "routing".to_string(),
            context_tags: vec![],
            candidates: vec!["peer_a".to_string(), "peer_b".to_string()],
            chosen_peer: chosen_peer.to_string(),
            luck_score,
            resistance_score: 0.2,
            explanation: "test".to_string(),
            outcome,
            node_id: "node_1".to_string(),
        }
    }

    #[test]
    fn test_decision_outcome_success_score() {
        let success = DecisionOutcome::Success {
            actual_latency_ms: 100.0,
            actual_quality: 0.95,
        };
        assert_eq!(success.success_score(), 1.0);

        let partial = DecisionOutcome::Partial {
            completed_ratio: 0.7,
            issues: vec![],
        };
        assert_eq!(partial.success_score(), 0.7);

        let failure = DecisionOutcome::Failure {
            reason: "timeout".to_string(),
        };
        assert_eq!(failure.success_score(), 0.0);
    }

    #[test]
    fn test_decision_history_add_and_get() {
        let mut history = DecisionHistory::new(10);

        let trace = create_test_trace(
            "dec_1",
            "peer_a",
            0.8,
            DecisionOutcome::Success {
                actual_latency_ms: 50.0,
                actual_quality: 0.9,
            },
        );

        history.add_trace(trace);

        assert_eq!(history.len(), 1);
        assert!(!history.is_empty());

        let all = history.get_all();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].decision_id, "dec_1");
    }

    #[test]
    fn test_decision_history_max_size() {
        let mut history = DecisionHistory::new(3);

        for i in 0..5 {
            let trace = create_test_trace(
                &format!("dec_{}", i),
                "peer_a",
                0.5,
                DecisionOutcome::Pending,
            );
            history.add_trace(trace);
        }

        assert_eq!(history.len(), 3);

        let all = history.get_all();
        assert_eq!(all[0].decision_id, "dec_2"); // Первые 2 удалены
        assert_eq!(all[2].decision_id, "dec_4");
    }

    #[test]
    fn test_decision_history_filter_by_peer() {
        let mut history = DecisionHistory::new(10);

        history.add_trace(create_test_trace(
            "dec_1",
            "peer_a",
            0.8,
            DecisionOutcome::Pending,
        ));
        history.add_trace(create_test_trace(
            "dec_2",
            "peer_b",
            0.7,
            DecisionOutcome::Pending,
        ));
        history.add_trace(create_test_trace(
            "dec_3",
            "peer_a",
            0.9,
            DecisionOutcome::Pending,
        ));

        let peer_a_traces = history.get_by_peer("peer_a");
        assert_eq!(peer_a_traces.len(), 2);
        assert_eq!(peer_a_traces[0].decision_id, "dec_1");
        assert_eq!(peer_a_traces[1].decision_id, "dec_3");
    }

    #[test]
    fn test_decision_history_update_outcome() {
        let mut history = DecisionHistory::new(10);

        history.add_trace(create_test_trace(
            "dec_1",
            "peer_a",
            0.8,
            DecisionOutcome::Pending,
        ));

        let updated = history.update_outcome(
            "dec_1",
            DecisionOutcome::Success {
                actual_latency_ms: 100.0,
                actual_quality: 0.95,
            },
        );

        assert!(updated);

        let traces = history.get_all();
        assert!(traces[0].outcome.is_success());
    }

    #[test]
    fn test_decision_stats() {
        let mut history = DecisionHistory::new(10);

        history.add_trace(create_test_trace(
            "dec_1",
            "peer_a",
            0.8,
            DecisionOutcome::Success {
                actual_latency_ms: 50.0,
                actual_quality: 0.9,
            },
        ));

        history.add_trace(create_test_trace(
            "dec_2",
            "peer_b",
            0.3,
            DecisionOutcome::Failure {
                reason: "timeout".to_string(),
            },
        ));

        history.add_trace(create_test_trace(
            "dec_3",
            "peer_a",
            0.9,
            DecisionOutcome::Success {
                actual_latency_ms: 40.0,
                actual_quality: 0.95,
            },
        ));

        let stats = history.get_success_stats();

        assert_eq!(stats.total_decisions, 3);
        assert_eq!(stats.successful_decisions, 2);
        assert!((stats.success_rate - 0.666).abs() < 0.01);
        assert_eq!(stats.lucky_decisions, 2); // dec_1 and dec_3
        assert_eq!(stats.unlucky_decisions, 1); // dec_2
    }

    #[test]
    fn test_was_lucky_unlucky() {
        let lucky_trace = create_test_trace(
            "dec_1",
            "peer_a",
            0.8,
            DecisionOutcome::Success {
                actual_latency_ms: 50.0,
                actual_quality: 0.9,
            },
        );
        assert!(lucky_trace.was_lucky());
        assert!(!lucky_trace.was_unlucky());

        let unlucky_trace = create_test_trace(
            "dec_2",
            "peer_b",
            0.3,
            DecisionOutcome::Failure {
                reason: "timeout".to_string(),
            },
        );
        assert!(!unlucky_trace.was_lucky());
        assert!(unlucky_trace.was_unlucky());
    }
}
