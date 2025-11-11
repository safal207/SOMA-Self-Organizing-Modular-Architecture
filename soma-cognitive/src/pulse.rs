//! # Cognitive Pulse - Пульс когнитивной активности
//!
//! Узлы раз в T секунд публикуют короткий пакет смысла.
//! Соседи вычисляют semantic overlap и усиливают связи при совпадении.
//!
//! v1.2: Добавлена поддержка embedding-based semantic similarity

use crate::embeddings::{cosine_similarity, IntentEmbeddings};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{interval, Duration};

/// Намерение узла - что он пытается достичь
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    /// Стабилизировать систему
    Stabilize,
    /// Балансировать нагрузку
    BalanceLoad,
    /// Адаптивное исцеление
    AdaptiveHealing,
    /// Исследование новых паттернов
    Explore,
    /// Оптимизация ресурсов
    Optimize,
    /// Кастомное намерение
    Custom(String),
}

impl Intent {
    /// Получить строковое представление намерения
    pub fn as_str(&self) -> &str {
        match self {
            Intent::Stabilize => "stabilize",
            Intent::BalanceLoad => "load_balancing",
            Intent::AdaptiveHealing => "adaptive_healing",
            Intent::Explore => "explore",
            Intent::Optimize => "optimize",
            Intent::Custom(s) => s,
        }
    }

    /// Вычислить семантическое совпадение с другим намерением (DEPRECATED)
    /// Простая эвристика: точное совпадение = 1.0, разное = 0.0
    ///
    /// **DEPRECATED**: Используйте `similarity_embedding()` для более точного анализа
    #[deprecated(since = "1.2.0", note = "use similarity_embedding() instead")]
    pub fn similarity(&self, other: &Intent) -> f64 {
        if self == other {
            1.0
        } else {
            // Некоторые намерения близки по смыслу
            match (self, other) {
                (Intent::Stabilize, Intent::AdaptiveHealing) => 0.6,
                (Intent::AdaptiveHealing, Intent::Stabilize) => 0.6,
                (Intent::BalanceLoad, Intent::Optimize) => 0.7,
                (Intent::Optimize, Intent::BalanceLoad) => 0.7,
                _ => 0.0,
            }
        }
    }

    /// Вычислить семантическое совпадение используя embeddings (v1.2)
    /// Возвращает cosine similarity между embedding-векторами намерений
    pub fn similarity_embedding(&self, other: &Intent, embeddings: &IntentEmbeddings) -> f64 {
        let emb_self = embeddings.get_embedding(self);
        let emb_other = embeddings.get_embedding(other);
        cosine_similarity(&emb_self, &emb_other) as f64
    }

    /// Создать список контекстных тегов для намерения
    pub fn context_tags(&self) -> Vec<String> {
        match self {
            Intent::Stabilize => vec!["stability".into(), "homeostasis".into()],
            Intent::BalanceLoad => vec!["load_balancing".into(), "distribution".into()],
            Intent::AdaptiveHealing => vec!["healing".into(), "recovery".into(), "adaptation".into()],
            Intent::Explore => vec!["exploration".into(), "discovery".into()],
            Intent::Optimize => vec!["optimization".into(), "efficiency".into()],
            Intent::Custom(s) => vec![s.clone()],
        }
    }
}

/// Пакет когнитивного пульса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitivePulse {
    /// ID узла-отправителя
    pub node_id: String,

    /// Текущее намерение узла
    pub intent: Intent,

    /// Уверенность в выбранном намерении (0.0 - 1.0)
    pub confidence: f64,

    /// Контекстные теги
    pub context: Vec<String>,

    /// Временная метка (Unix timestamp)
    pub timestamp: u64,

    /// Дополнительные метаданные
    pub metadata: HashMap<String, String>,
}

impl CognitivePulse {
    /// Создать новый когнитивный пульс
    pub fn new(node_id: String, intent: Intent, confidence: f64) -> Self {
        let context = intent.context_tags();
        Self {
            node_id,
            intent,
            confidence,
            context,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Добавить метаданные к пульсу
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Вычислить семантическое перекрытие с другим пульсом (DEPRECATED)
    ///
    /// **DEPRECATED**: Используйте `semantic_overlap_embedding()` для embedding-based анализа
    #[deprecated(since = "1.2.0", note = "use semantic_overlap_embedding() instead")]
    pub fn semantic_overlap(&self, other: &CognitivePulse) -> f64 {
        // Базовая similarity между намерениями
        #[allow(deprecated)]
        let intent_sim = self.intent.similarity(&other.intent);

        // Similarity на основе контекстных тегов (Jaccard index)
        let context_sim = self.context_jaccard(&other.context);

        // Взвешенная комбинация (intent важнее)
        0.7 * intent_sim + 0.3 * context_sim
    }

    /// Вычислить семантическое перекрытие используя embeddings (v1.2)
    pub fn semantic_overlap_embedding(&self, other: &CognitivePulse, embeddings: &IntentEmbeddings) -> f64 {
        // Embedding-based similarity между намерениями
        let intent_sim = self.intent.similarity_embedding(&other.intent, embeddings);

        // Similarity на основе контекстных тегов (Jaccard index)
        let context_sim = self.context_jaccard(&other.context);

        // Взвешенная комбинация (intent важнее, так как использует embeddings)
        0.8 * intent_sim + 0.2 * context_sim
    }

    /// Вычислить Jaccard similarity для контекстных тегов
    fn context_jaccard(&self, other_context: &[String]) -> f64 {
        if self.context.is_empty() && other_context.is_empty() {
            return 1.0;
        }
        if self.context.is_empty() || other_context.is_empty() {
            return 0.0;
        }

        let set_a: std::collections::HashSet<_> = self.context.iter().collect();
        let set_b: std::collections::HashSet<_> = other_context.iter().collect();

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        intersection as f64 / union as f64
    }

    /// Сериализовать пульс в JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Простая функция для отправки когнитивного пульса
pub async fn pulse(node_id: &str, intent: Intent, confidence: f64) {
    let pulse = CognitivePulse::new(node_id.to_string(), intent, confidence);

    match pulse.to_json() {
        Ok(json) => println!("📡 Cognitive pulse: {}", json),
        Err(e) => eprintln!("❌ Failed to serialize pulse: {}", e),
    }
}

/// Менеджер когнитивных пульсов для узла
pub struct PulseManager {
    node_id: String,
    interval_secs: u64,
}

impl PulseManager {
    /// Создать новый менеджер пульсов
    pub fn new(node_id: String, interval_secs: u64) -> Self {
        Self {
            node_id,
            interval_secs,
        }
    }

    /// Запустить периодическую отправку пульсов
    pub async fn start<F>(&self, mut intent_provider: F)
    where
        F: FnMut() -> (Intent, f64),
    {
        let mut ticker = interval(Duration::from_secs(self.interval_secs));

        loop {
            ticker.tick().await;
            let (intent, confidence) = intent_provider();
            pulse(&self.node_id, intent, confidence).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_similarity() {
        assert_eq!(Intent::Stabilize.similarity(&Intent::Stabilize), 1.0);
        assert_eq!(Intent::Stabilize.similarity(&Intent::Explore), 0.0);
        assert_eq!(Intent::Stabilize.similarity(&Intent::AdaptiveHealing), 0.6);
    }

    #[test]
    fn test_cognitive_pulse_creation() {
        let pulse = CognitivePulse::new(
            "node_alpha".to_string(),
            Intent::Stabilize,
            0.82,
        );

        assert_eq!(pulse.node_id, "node_alpha");
        assert_eq!(pulse.intent, Intent::Stabilize);
        assert_eq!(pulse.confidence, 0.82);
        assert!(!pulse.context.is_empty());
    }

    #[test]
    fn test_semantic_overlap() {
        let pulse1 = CognitivePulse::new(
            "node_a".to_string(),
            Intent::Stabilize,
            0.8,
        );

        let pulse2 = CognitivePulse::new(
            "node_b".to_string(),
            Intent::Stabilize,
            0.9,
        );

        let overlap = pulse1.semantic_overlap(&pulse2);
        assert!(overlap > 0.7); // Должно быть высокое совпадение
    }

    #[test]
    fn test_context_jaccard() {
        let pulse1 = CognitivePulse::new(
            "node_a".to_string(),
            Intent::Stabilize,
            0.8,
        );

        let pulse2 = CognitivePulse::new(
            "node_b".to_string(),
            Intent::AdaptiveHealing,
            0.9,
        );

        // Разные namерения, но есть некоторое перекрытие в контексте
        let overlap = pulse1.semantic_overlap(&pulse2);
        assert!(overlap > 0.4);
    }
}
