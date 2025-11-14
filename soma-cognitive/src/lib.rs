//! # SOMA Cognitive - Cognitive Mesh Layer v1.2
//!
//! Слой когнитивного резонанса: узлы не просто обмениваются данными,
//! а синхронизируют намерения и гипотезы, образуя коллективный интеллект.
//!
//! ## Компоненты
//!
//! - **Pulse**: Cognitive Pulse - узлы публикуют пакеты смысла
//! - **Braid**: Inference Braid - временное объединение для решения задач
//! - **Metrics**: Metametric Layer - метрики когнитивной активности
//! - **Memory**: Collective Memory - лог когнитивных событий
//! - **Embeddings**: Semantic Embeddings - векторное представление намерений (v1.2)

pub mod pulse;
pub mod braid;
pub mod metrics;
pub mod memory;
pub mod embeddings;

pub use pulse::{CognitivePulse, Intent, pulse};
pub use braid::{InferenceBraid, Task, BraidResult};
pub use metrics::{CognitiveMetrics, MetricSnapshot};
pub use memory::{CollectiveMemory, CognitiveEvent};
pub use embeddings::{IntentEmbeddings, cosine_similarity, SemanticClusterer};

/// Версия Cognitive Mesh
pub const COGNITIVE_MESH_VERSION: &str = "1.2.0";

/// Порог семантического совпадения для формирования когнитивных кластеров
pub const SEMANTIC_THRESHOLD: f64 = 0.7;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(COGNITIVE_MESH_VERSION, "1.2.0");
    }

    #[test]
    fn test_threshold() {
        assert_eq!(SEMANTIC_THRESHOLD, 0.7);
    }
}
