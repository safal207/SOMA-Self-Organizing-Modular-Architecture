//! # String Resonance - Струнный резонанс
//!
//! Вычисление "резонанса" кандидатов на основе их метрик.
//! Модель: взвешенная комбинация health, quality, intent_match с добавлением
//! фазового коэффициента (зависит от времени).

use std::time::{SystemTime, UNIX_EPOCH};

/// Представляет кандидата-пира для оценки
#[derive(Debug, Clone)]
pub struct PeerCandidate {
    pub peer_id: String,
    /// Здоровье пира (0.0-1.0)
    pub health: f32,
    /// Качество/latency score (0.0-1.0)
    pub quality: f32,
    /// Семантическое совпадение с намерением (0.0-1.0)
    pub intent_match: f32,
}

/// Веса для вычисления резонанса
pub struct ResonanceWeights {
    pub health_weight: f32,
    pub quality_weight: f32,
    pub intent_weight: f32,
}

impl Default for ResonanceWeights {
    fn default() -> Self {
        Self {
            health_weight: 0.5,
            quality_weight: 0.3,
            intent_weight: 0.2,
        }
    }
}

/// Вычислить струнный резонанс кандидата
///
/// Формула: resonance = (health * w_health + quality * w_quality + intent * w_intent) * phase_coeff
///
/// # Arguments
/// * `candidate` - Кандидат для оценки
///
/// # Returns
/// Значение резонанса от 0.0 до 1.0
pub fn compute_resonance(candidate: &PeerCandidate) -> f32 {
    compute_resonance_with_weights(candidate, &ResonanceWeights::default())
}

/// Вычислить резонанс с кастомными весами
pub fn compute_resonance_with_weights(
    candidate: &PeerCandidate,
    weights: &ResonanceWeights,
) -> f32 {
    let base_resonance = candidate.health * weights.health_weight
        + candidate.quality * weights.quality_weight
        + candidate.intent_match * weights.intent_weight;

    let phase_coeff = compute_phase_coefficient();

    (base_resonance * phase_coeff).min(1.0)
}

/// Вычислить фазовый коэффициент на основе текущего времени
///
/// Фаза зависит от времени суток и колеблется между 0.8 и 1.0
/// Это имитирует "ритмы сети" — в разное время удача может быть выше/ниже
fn compute_phase_coefficient() -> f32 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Простая синусоида: колебание с периодом ~24 часа
    // Амплитуда: 0.8 - 1.0
    let period = 86400.0; // 24 часа в секундах
    let phase = (now as f64 % period) / period * 2.0 * std::f64::consts::PI;
    let sine = phase.sin();

    // Нормализуем от -1..1 к 0.8..1.0
    0.9 + (sine * 0.1) as f32
}

/// Вычислить резонанс для массива кандидатов
pub fn compute_resonances(candidates: &[PeerCandidate]) -> Vec<(String, f32)> {
    candidates
        .iter()
        .map(|c| (c.peer_id.clone(), compute_resonance(c)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_resonance() {
        let candidate = PeerCandidate {
            peer_id: "test_peer".to_string(),
            health: 0.9,
            quality: 0.8,
            intent_match: 0.7,
        };

        let resonance = compute_resonance(&candidate);

        // Ожидаем где-то в диапазоне 0.7-0.9 с учетом фазы
        assert!(resonance > 0.6 && resonance <= 1.0);
    }

    #[test]
    fn test_perfect_candidate() {
        let candidate = PeerCandidate {
            peer_id: "perfect".to_string(),
            health: 1.0,
            quality: 1.0,
            intent_match: 1.0,
        };

        let resonance = compute_resonance(&candidate);

        // Должен быть близок к 1.0 (с учетом фазы)
        assert!(resonance > 0.8);
    }

    #[test]
    fn test_poor_candidate() {
        let candidate = PeerCandidate {
            peer_id: "poor".to_string(),
            health: 0.2,
            quality: 0.1,
            intent_match: 0.1,
        };

        let resonance = compute_resonance(&candidate);

        // Должен быть низким
        assert!(resonance < 0.3);
    }

    #[test]
    fn test_phase_coefficient_range() {
        let phase = compute_phase_coefficient();

        // Фаза должна быть в диапазоне 0.8 - 1.0
        assert!(phase >= 0.8 && phase <= 1.0);
    }

    #[test]
    fn test_custom_weights() {
        let candidate = PeerCandidate {
            peer_id: "test".to_string(),
            health: 0.5,
            quality: 0.5,
            intent_match: 0.5,
        };

        let weights = ResonanceWeights {
            health_weight: 1.0,
            quality_weight: 0.0,
            intent_weight: 0.0,
        };

        let resonance = compute_resonance_with_weights(&candidate, &weights);

        // С такими весами, resonance должен зависеть только от health * phase
        assert!(resonance > 0.3 && resonance < 0.6);
    }
}
