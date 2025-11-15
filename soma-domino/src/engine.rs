//! # Domino Engine - Главная точка входа
//!
//! Высокоуровневый интерфейс для оценки "удачи" и выбора лучших пиров.

use crate::qstar_loop::{evaluate_candidates, CandidateScore};
use serde::{Deserialize, Serialize};

// Re-export PeerCandidate для удобства
pub use crate::string_resonance::PeerCandidate;

/// Тип намерения/запроса для оценки
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DominoIntentKind {
    /// Маршрутизация запроса
    Routing,
    /// Планирование задачи
    TaskScheduling,
    /// Пользовательский запрос
    UserRequest,
    /// Кастомный тип
    Custom(String),
}

impl DominoIntentKind {
    /// Получить строковое представление
    pub fn as_str(&self) -> &str {
        match self {
            DominoIntentKind::Routing => "routing",
            DominoIntentKind::TaskScheduling => "task_scheduling",
            DominoIntentKind::UserRequest => "user_request",
            DominoIntentKind::Custom(s) => s,
        }
    }
}

/// Входные данные для оценки "удачи"
#[derive(Debug, Clone)]
pub struct DominoInput {
    /// Тип намерения
    pub intent_kind: DominoIntentKind,

    /// Список кандидатов-пиров для выбора
    pub candidates: Vec<PeerCandidate>,

    /// Контекстные теги из когнитивного слоя
    pub context_tags: Vec<String>,
}

impl DominoInput {
    /// Создать новый input
    pub fn new(
        intent_kind: DominoIntentKind,
        candidates: Vec<PeerCandidate>,
        context_tags: Vec<String>,
    ) -> Self {
        Self {
            intent_kind,
            candidates,
            context_tags,
        }
    }

    /// Создать routing input
    pub fn routing(candidates: Vec<PeerCandidate>) -> Self {
        Self::new(DominoIntentKind::Routing, candidates, vec![])
    }

    /// Добавить контекстные теги
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.context_tags = tags;
        self
    }
}

/// Результат оценки Domino Engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DominoDecision {
    /// Отсортированный список лучших пиров (peer_id)
    pub best_peers: Vec<String>,

    /// Общая оценка удачи (0.0 - 1.0)
    pub luck_score: f32,

    /// Общая оценка сопротивления (0.0 - 1.0)
    pub resistance_score: f32,

    /// Человекочитаемое объяснение
    pub explanation: String,
}

impl DominoDecision {
    /// Создать новое решение
    pub fn new(
        best_peers: Vec<String>,
        luck_score: f32,
        resistance_score: f32,
        explanation: String,
    ) -> Self {
        Self {
            best_peers,
            luck_score,
            resistance_score,
            explanation,
        }
    }

    /// Создать пустое решение (нет подходящих кандидатов)
    pub fn empty(reason: &str) -> Self {
        Self {
            best_peers: vec![],
            luck_score: 0.0,
            resistance_score: 1.0,
            explanation: format!("No suitable candidates: {}", reason),
        }
    }
}

/// Domino Engine - главный движок оценки удачи
pub struct DominoEngine;

impl DominoEngine {
    /// Оценить input и вернуть решение
    ///
    /// Процесс:
    /// 1. Вызвать qstar_loop::evaluate_candidates
    /// 2. Отсортировать кандидатов по score
    /// 3. Выбрать top-N peer_id
    /// 4. Вычислить общий luck_score / resistance_score
    /// 5. Сгенерировать explanation
    ///
    /// # Arguments
    /// * `input` - Входные данные для оценки
    ///
    /// # Returns
    /// Решение с лучшими пирами и оценками
    pub fn evaluate(input: DominoInput) -> DominoDecision {
        // Проверка на пустой список кандидатов
        if input.candidates.is_empty() {
            return DominoDecision::empty("no candidates provided");
        }

        // 1. Оценка всех кандидатов
        let scored = evaluate_candidates(&input.candidates);

        // 2. Выбрать top-N (по умолчанию все, отсортированные)
        let best_peers: Vec<String> = scored.iter().map(|s| s.peer_id.clone()).collect();

        // 3. Вычислить общие метрики (средние по top-3 или всем, если меньше)
        let top_count = scored.len().min(3);
        let top_scores = &scored[..top_count];

        let avg_final_score = top_scores.iter().map(|s| s.final_score).sum::<f32>()
            / top_count as f32;

        let avg_resonance =
            top_scores.iter().map(|s| s.resonance).sum::<f32>() / top_count as f32;

        // luck_score: используем средний final_score top кандидатов
        let luck_score = avg_final_score.min(1.0);

        // resistance_score: обратно пропорционален резонансу
        let resistance_score = (1.0 - avg_resonance).max(0.0);

        // 4. Генерация explanation
        let explanation = Self::generate_explanation(
            &input,
            &scored,
            luck_score,
            resistance_score,
        );

        DominoDecision::new(best_peers, luck_score, resistance_score, explanation)
    }

    /// Сгенерировать человекочитаемое объяснение
    fn generate_explanation(
        input: &DominoInput,
        scored: &[CandidateScore],
        luck_score: f32,
        resistance_score: f32,
    ) -> String {
        let intent_str = input.intent_kind.as_str();

        let luck_desc = if luck_score >= 0.7 {
            "High resonance"
        } else if luck_score >= 0.4 {
            "Medium resonance"
        } else {
            "Low resonance"
        };

        let resistance_desc = if resistance_score <= 0.3 {
            "low resistance"
        } else if resistance_score <= 0.6 {
            "moderate resistance"
        } else {
            "high resistance"
        };

        let best_peer = if !scored.is_empty() {
            &scored[0].peer_id
        } else {
            "none"
        };

        // Добавим совет на основе оценок
        let recommendation = if luck_score >= 0.6 && resistance_score <= 0.4 {
            "Recommended to proceed now."
        } else if luck_score >= 0.4 {
            "Acceptable conditions, monitor for changes."
        } else {
            "Consider waiting or alternative routes."
        };

        format!(
            "{} detected for {} intent. Best candidate: {}. {} Path: {}. {}",
            luck_desc, intent_str, best_peer, resistance_desc,
            if scored.len() > 1 {
                format!("with {} alternatives", scored.len() - 1)
            } else {
                "single option".to_string()
            },
            recommendation
        )
    }

    /// Оценить и вернуть только top-N лучших пиров
    pub fn evaluate_top_n(input: DominoInput, n: usize) -> DominoDecision {
        let mut decision = Self::evaluate(input);
        decision.best_peers.truncate(n);
        decision
    }

    /// Оценить с минимальным порогом score
    pub fn evaluate_with_threshold(input: DominoInput, min_score: f32) -> DominoDecision {
        if input.candidates.is_empty() {
            return DominoDecision::empty("no candidates provided");
        }

        let scored = evaluate_candidates(&input.candidates);

        // Фильтрация по порогу
        let filtered: Vec<&CandidateScore> = scored
            .iter()
            .filter(|s| s.final_score >= min_score)
            .collect();

        if filtered.is_empty() {
            return DominoDecision::empty(&format!(
                "no candidates above threshold {}",
                min_score
            ));
        }

        let best_peers: Vec<String> = filtered.iter().map(|s| s.peer_id.clone()).collect();

        let avg_score =
            filtered.iter().map(|s| s.final_score).sum::<f32>() / filtered.len() as f32;
        let avg_resonance =
            filtered.iter().map(|s| s.resonance).sum::<f32>() / filtered.len() as f32;

        let luck_score = avg_score.min(1.0);
        let resistance_score = (1.0 - avg_resonance).max(0.0);

        let explanation = format!(
            "Filtered {} candidates above threshold {:.2}. Average luck: {:.2}",
            filtered.len(),
            min_score,
            luck_score
        );

        DominoDecision::new(best_peers, luck_score, resistance_score, explanation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_input() -> DominoInput {
        DominoInput::new(
            DominoIntentKind::Routing,
            vec![
                PeerCandidate {
                    peer_id: "alpha".to_string(),
                    health: 0.9,
                    quality: 0.8,
                    intent_match: 0.7,
                },
                PeerCandidate {
                    peer_id: "beta".to_string(),
                    health: 0.6,
                    quality: 0.5,
                    intent_match: 0.4,
                },
            ],
            vec!["low_latency".to_string()],
        )
    }

    #[test]
    fn test_domino_engine_evaluate() {
        let input = create_test_input();
        let decision = DominoEngine::evaluate(input);

        // Должны получить 2 кандидата
        assert_eq!(decision.best_peers.len(), 2);

        // Лучший должен быть alpha (более высокие метрики)
        assert_eq!(decision.best_peers[0], "alpha");

        // luck_score должен быть > 0
        assert!(decision.luck_score > 0.0);

        // explanation не пустой
        assert!(!decision.explanation.is_empty());
    }

    #[test]
    fn test_domino_engine_empty_candidates() {
        let input = DominoInput::new(
            DominoIntentKind::Routing,
            vec![],
            vec![],
        );

        let decision = DominoEngine::evaluate(input);

        assert_eq!(decision.best_peers.len(), 0);
        assert_eq!(decision.luck_score, 0.0);
        assert_eq!(decision.resistance_score, 1.0);
    }

    #[test]
    fn test_domino_engine_top_n() {
        let input = create_test_input();
        let decision = DominoEngine::evaluate_top_n(input, 1);

        assert_eq!(decision.best_peers.len(), 1);
        assert_eq!(decision.best_peers[0], "alpha");
    }

    #[test]
    fn test_domino_engine_with_threshold() {
        let input = create_test_input();
        let decision = DominoEngine::evaluate_with_threshold(input, 0.7);

        // Должен пройти хотя бы alpha
        assert!(decision.best_peers.len() >= 1);
        assert!(decision.luck_score >= 0.7);
    }

    #[test]
    fn test_intent_kind_serialization() {
        let intent = DominoIntentKind::Routing;
        assert_eq!(intent.as_str(), "routing");

        let custom = DominoIntentKind::Custom("special".to_string());
        assert_eq!(custom.as_str(), "special");
    }

    #[test]
    fn test_domino_input_builder() {
        let input = DominoInput::routing(vec![])
            .with_tags(vec!["tag1".to_string(), "tag2".to_string()]);

        assert_eq!(input.intent_kind, DominoIntentKind::Routing);
        assert_eq!(input.context_tags.len(), 2);
    }
}
