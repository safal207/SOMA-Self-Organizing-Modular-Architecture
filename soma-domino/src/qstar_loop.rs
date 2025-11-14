//! # Q* Loop - Итеративное улучшение выбора
//!
//! Q*-подобный цикл (без реального Q-learning) для оценки и ранжирования кандидатов.
//! Iterative improvement: для каждого кандидата вычисляем резонанс, fuzzy-оценку,
//! итоговый score и возвращаем отсортированный список.

use crate::fuzzy_luck::{evaluate_fuzzy, FuzzyLuck};
use crate::string_resonance::{compute_resonance, PeerCandidate};

/// Результат оценки одного кандидата
#[derive(Debug, Clone)]
pub struct CandidateScore {
    /// ID кандидата
    pub peer_id: String,
    /// Базовый резонанс (0.0-1.0)
    pub resonance: f32,
    /// Нечёткая оценка
    pub fuzzy: FuzzyLuck,
    /// Итоговый score с учётом удачи и сопротивления
    pub final_score: f32,
}

impl CandidateScore {
    /// Создать новый результат оценки
    pub fn new(peer_id: String, resonance: f32, fuzzy: FuzzyLuck, final_score: f32) -> Self {
        Self {
            peer_id,
            resonance,
            fuzzy,
            final_score,
        }
    }
}

/// Оценить всех кандидатов и вернуть отсортированный список
///
/// Для каждого кандидата:
/// 1. Вычисляем string resonance
/// 2. Передаём в fuzzy-модуль для получения luck/resistance
/// 3. Вычисляем final_score = resonance * luck_factor * (1 - resistance_factor)
/// 4. Сортируем по убыванию final_score
///
/// # Arguments
/// * `candidates` - Список кандидатов для оценки
///
/// # Returns
/// Отсортированный список результатов (лучшие первые)
pub fn evaluate_candidates(candidates: &[PeerCandidate]) -> Vec<CandidateScore> {
    let mut scores: Vec<CandidateScore> = candidates
        .iter()
        .map(|candidate| {
            // 1. Вычислить резонанс
            let resonance = compute_resonance(candidate);

            // 2. Fuzzy-оценка
            let fuzzy = evaluate_fuzzy(resonance);

            // 3. Итоговый score
            let final_score = fuzzy.compute_score(resonance);

            CandidateScore::new(candidate.peer_id.clone(), resonance, fuzzy, final_score)
        })
        .collect();

    // 4. Сортировка по убыванию final_score
    scores.sort_by(|a, b| {
        b.final_score
            .partial_cmp(&a.final_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    scores
}

/// Оценить кандидатов и вернуть top-N лучших
pub fn evaluate_top_n(candidates: &[PeerCandidate], n: usize) -> Vec<CandidateScore> {
    let mut scores = evaluate_candidates(candidates);
    scores.truncate(n);
    scores
}

/// Оценить кандидатов и отфильтровать по минимальному score
pub fn evaluate_with_threshold(
    candidates: &[PeerCandidate],
    min_score: f32,
) -> Vec<CandidateScore> {
    evaluate_candidates(candidates)
        .into_iter()
        .filter(|score| score.final_score >= min_score)
        .collect()
}

/// Итеративное улучшение: несколько раундов оценки с обновлением параметров
///
/// Простая симуляция Q*-подобного подхода:
/// - Делаем несколько раундов оценки
/// - В каждом раунде используем feedback от предыдущих для корректировки
/// - Возвращаем финальные результаты
pub fn qstar_iterate(
    candidates: &[PeerCandidate],
    iterations: usize,
) -> Vec<CandidateScore> {
    if iterations == 0 || candidates.is_empty() {
        return vec![];
    }

    // Первый раунд - базовая оценка
    let mut current_scores = evaluate_candidates(candidates);

    // Последующие итерации
    for iteration in 1..iterations {
        // Вычисляем adjustment factor на основе предыдущих результатов
        let avg_score = current_scores.iter().map(|s| s.final_score).sum::<f32>()
            / current_scores.len() as f32;

        // Создаём adjusted кандидатов (boost для тех, кто был хорош в прошлом раунде)
        let adjusted_candidates: Vec<PeerCandidate> = candidates
            .iter()
            .zip(&current_scores)
            .map(|(candidate, prev_score)| {
                let boost = if prev_score.final_score > avg_score {
                    1.05 // 5% boost
                } else {
                    0.95 // 5% penalty
                };

                PeerCandidate {
                    peer_id: candidate.peer_id.clone(),
                    health: (candidate.health * boost).min(1.0),
                    quality: (candidate.quality * boost).min(1.0),
                    intent_match: (candidate.intent_match * boost).min(1.0),
                }
            })
            .collect();

        // Переоценка
        current_scores = evaluate_candidates(&adjusted_candidates);

        // Логирование (опционально)
        #[cfg(debug_assertions)]
        println!(
            "Q* iteration {}: avg_score = {:.3}",
            iteration, avg_score
        );
    }

    current_scores
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candidates() -> Vec<PeerCandidate> {
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
            PeerCandidate {
                peer_id: "gamma".to_string(),
                health: 0.95,
                quality: 0.9,
                intent_match: 0.85,
            },
        ]
    }

    #[test]
    fn test_evaluate_candidates() {
        let candidates = create_test_candidates();
        let scores = evaluate_candidates(&candidates);

        // Должно быть 3 результата
        assert_eq!(scores.len(), 3);

        // Лучший кандидат должен быть первым (gamma с наивысшими метриками)
        assert_eq!(scores[0].peer_id, "gamma");

        // Все scores должны быть > 0
        for score in &scores {
            assert!(score.final_score > 0.0);
        }
    }

    #[test]
    fn test_evaluate_top_n() {
        let candidates = create_test_candidates();
        let top2 = evaluate_top_n(&candidates, 2);

        assert_eq!(top2.len(), 2);
        assert_eq!(top2[0].peer_id, "gamma");
    }

    #[test]
    fn test_evaluate_with_threshold() {
        let candidates = create_test_candidates();
        let filtered = evaluate_with_threshold(&candidates, 0.5);

        // Должны пройти только кандидаты с высоким score
        assert!(filtered.len() <= 3);
        for score in &filtered {
            assert!(score.final_score >= 0.5);
        }
    }

    #[test]
    fn test_qstar_iterate() {
        let candidates = create_test_candidates();
        let results = qstar_iterate(&candidates, 3);

        // Должны получить результаты после 3 итераций
        assert_eq!(results.len(), 3);

        // Результаты должны быть отсортированы по убыванию score
        for i in 0..results.len() - 1 {
            assert!(
                results[i].final_score >= results[i + 1].final_score,
                "Results should be sorted by score"
            );
        }

        // Лучший кандидат должен быть либо alpha, либо gamma (зависит от phase coefficient)
        assert!(
            results[0].peer_id == "alpha" || results[0].peer_id == "gamma",
            "Best candidate should be alpha or gamma, got {}",
            results[0].peer_id
        );
    }

    #[test]
    fn test_qstar_zero_iterations() {
        let candidates = create_test_candidates();
        let results = qstar_iterate(&candidates, 0);

        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_candidate_score_ordering() {
        let candidates = create_test_candidates();
        let scores = evaluate_candidates(&candidates);

        // Проверяем что сортировка корректна (по убыванию)
        for i in 0..scores.len() - 1 {
            assert!(scores[i].final_score >= scores[i + 1].final_score);
        }
    }
}
