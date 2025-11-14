//! # Semantic Embeddings - Векторное представление намерений
//!
//! Преобразование Intent в векторное пространство для семантического анализа.
//! Использует предвычисленные embeddings для каждого типа намерения.

use crate::pulse::Intent;
use std::collections::HashMap;

/// Размерность embedding-векторов
pub const EMBEDDING_DIM: usize = 16;

/// Embedding-вектор
pub type Embedding = [f32; EMBEDDING_DIM];

/// Менеджер embeddings для Intent
pub struct IntentEmbeddings {
    /// Предвычисленные embeddings для базовых Intent
    embeddings: HashMap<String, Embedding>,
}

impl IntentEmbeddings {
    /// Создать новый менеджер с предвычисленными embeddings
    pub fn new() -> Self {
        let mut embeddings = HashMap::new();

        // Предвычисленные embeddings для базовых Intent
        // Векторы построены так, чтобы семантически близкие Intent были близки в пространстве

        // Stabilize - фокус на стабильности и балансе
        embeddings.insert(
            "stabilize".to_string(),
            [0.8, 0.2, 0.1, 0.9, 0.3, 0.1, 0.7, 0.2, 0.4, 0.1, 0.6, 0.3, 0.2, 0.8, 0.1, 0.5],
        );

        // AdaptiveHealing - близко к Stabilize, но с акцентом на восстановление
        embeddings.insert(
            "adaptive_healing".to_string(),
            [0.7, 0.3, 0.2, 0.8, 0.4, 0.2, 0.6, 0.3, 0.5, 0.2, 0.7, 0.4, 0.3, 0.7, 0.2, 0.6],
        );

        // BalanceLoad - фокус на распределении и оптимизации
        embeddings.insert(
            "load_balancing".to_string(),
            [0.3, 0.7, 0.8, 0.4, 0.9, 0.6, 0.2, 0.5, 0.7, 0.8, 0.3, 0.6, 0.9, 0.4, 0.7, 0.3],
        );

        // Optimize - близко к BalanceLoad
        embeddings.insert(
            "optimize".to_string(),
            [0.4, 0.8, 0.7, 0.5, 0.9, 0.7, 0.3, 0.6, 0.8, 0.7, 0.4, 0.7, 0.8, 0.5, 0.6, 0.4],
        );

        // Explore - фокус на исследовании и новизне
        embeddings.insert(
            "explore".to_string(),
            [0.2, 0.4, 0.3, 0.2, 0.5, 0.9, 0.8, 0.9, 0.2, 0.6, 0.1, 0.8, 0.4, 0.3, 0.9, 0.7],
        );

        Self { embeddings }
    }

    /// Получить embedding для Intent
    pub fn get_embedding(&self, intent: &Intent) -> Embedding {
        let key = match intent {
            Intent::Stabilize => "stabilize",
            Intent::BalanceLoad => "load_balancing",
            Intent::AdaptiveHealing => "adaptive_healing",
            Intent::Explore => "explore",
            Intent::Optimize => "optimize",
            Intent::Custom(s) => {
                // Для кастомных Intent генерируем embedding на основе строки
                return self.generate_custom_embedding(s);
            }
        };

        self.embeddings
            .get(key)
            .copied()
            .unwrap_or_else(|| self.generate_default_embedding())
    }

    /// Генерировать embedding для кастомного Intent
    fn generate_custom_embedding(&self, text: &str) -> Embedding {
        let mut emb = [0.0f32; EMBEDDING_DIM];

        // Простой хеш-based подход для генерации уникальных векторов
        let bytes = text.as_bytes();
        for (i, chunk) in bytes.chunks(EMBEDDING_DIM).enumerate() {
            for (j, &byte) in chunk.iter().enumerate() {
                emb[j] += (byte as f32 / 255.0) * (1.0 / (i as f32 + 1.0));
            }
        }

        // Нормализация
        normalize_embedding(&mut emb);
        emb
    }

    /// Генерировать дефолтный embedding
    fn generate_default_embedding(&self) -> Embedding {
        [0.5; EMBEDDING_DIM]
    }

    /// Добавить кастомный embedding
    pub fn add_custom(&mut self, key: String, embedding: Embedding) {
        self.embeddings.insert(key, embedding);
    }
}

impl Default for IntentEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

/// Вычислить cosine similarity между двумя embeddings
pub fn cosine_similarity(a: &Embedding, b: &Embedding) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// Нормализовать embedding вектор (L2 нормализация)
pub fn normalize_embedding(emb: &mut Embedding) {
    let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in emb.iter_mut() {
            *x /= norm;
        }
    }
}

/// Вычислить евклидово расстояние между embeddings
pub fn euclidean_distance(a: &Embedding, b: &Embedding) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

/// Semantic clustering - группировка Intent по близости в векторном пространстве
pub struct SemanticClusterer {
    /// Порог similarity для объединения в кластер
    threshold: f32,
}

impl SemanticClusterer {
    /// Создать новый кластеризатор
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }

    /// Проверить, принадлежат ли два Intent одному кластеру
    pub fn are_clustered(&self, emb_a: &Embedding, emb_b: &Embedding) -> bool {
        cosine_similarity(emb_a, emb_b) >= self.threshold
    }

    /// Найти кластеры среди набора embeddings
    pub fn find_clusters(&self, embeddings: &[(String, Embedding)]) -> Vec<Vec<String>> {
        let mut clusters: Vec<Vec<String>> = Vec::new();
        let mut assigned = vec![false; embeddings.len()];

        for (i, (id_i, emb_i)) in embeddings.iter().enumerate() {
            if assigned[i] {
                continue;
            }

            let mut cluster = vec![id_i.clone()];
            assigned[i] = true;

            for (j, (id_j, emb_j)) in embeddings.iter().enumerate().skip(i + 1) {
                if !assigned[j] && self.are_clustered(emb_i, emb_j) {
                    cluster.push(id_j.clone());
                    assigned[j] = true;
                }
            }

            clusters.push(cluster);
        }

        clusters
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);

        let c = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &c), 0.0);
    }

    #[test]
    fn test_intent_embeddings() {
        let embeddings = IntentEmbeddings::new();

        let stabilize_emb = embeddings.get_embedding(&Intent::Stabilize);
        let healing_emb = embeddings.get_embedding(&Intent::AdaptiveHealing);
        let explore_emb = embeddings.get_embedding(&Intent::Explore);

        // Stabilize и AdaptiveHealing должны быть близки
        let sim_stable_heal = cosine_similarity(&stabilize_emb, &healing_emb);
        assert!(sim_stable_heal > 0.7);

        // Stabilize и Explore должны быть далеко
        let sim_stable_explore = cosine_similarity(&stabilize_emb, &explore_emb);
        assert!(sim_stable_explore < 0.7);
    }

    #[test]
    fn test_normalize_embedding() {
        let mut emb = [3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        normalize_embedding(&mut emb);

        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_semantic_clusterer() {
        let embeddings = IntentEmbeddings::new();
        let clusterer = SemanticClusterer::new(0.7);

        let items = vec![
            ("node_a".to_string(), embeddings.get_embedding(&Intent::Stabilize)),
            ("node_b".to_string(), embeddings.get_embedding(&Intent::AdaptiveHealing)),
            ("node_c".to_string(), embeddings.get_embedding(&Intent::Explore)),
            ("node_d".to_string(), embeddings.get_embedding(&Intent::Stabilize)),
        ];

        let clusters = clusterer.find_clusters(&items);

        // Должно быть минимум 2 кластера: один для Stabilize/Healing, другой для Explore
        assert!(clusters.len() >= 2);
    }

    #[test]
    fn test_custom_intent_embedding() {
        let embeddings = IntentEmbeddings::new();

        let custom1 = embeddings.get_embedding(&Intent::Custom("task_alpha".to_string()));
        let custom2 = embeddings.get_embedding(&Intent::Custom("task_alpha".to_string()));
        let custom3 = embeddings.get_embedding(&Intent::Custom("task_beta".to_string()));

        // Одинаковые кастомные Intent должны давать одинаковые embeddings
        let sim_same = cosine_similarity(&custom1, &custom2);
        assert!((sim_same - 1.0).abs() < 0.001);

        // Разные кастомные Intent должны давать разные embeddings
        let sim_diff = cosine_similarity(&custom1, &custom3);
        assert!(sim_diff < 0.99);
    }
}
