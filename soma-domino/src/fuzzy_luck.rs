//! # Fuzzy Luck - Нечёткая логика для оценки удачи
//!
//! Преобразование числовых значений резонанса в лингвистические категории:
//! "низкая/средняя/высокая удача" и "низкое/среднее/высокое сопротивление".

/// Уровень удачи (лингвистическая переменная)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuckLevel {
    /// Низкая удача (неблагоприятные условия)
    Low,
    /// Средняя удача (нейтральные условия)
    Medium,
    /// Высокая удача (благоприятные условия)
    High,
}

impl LuckLevel {
    /// Получить строковое представление
    pub fn as_str(&self) -> &'static str {
        match self {
            LuckLevel::Low => "low",
            LuckLevel::Medium => "medium",
            LuckLevel::High => "high",
        }
    }

    /// Получить числовой множитель для расчётов
    pub fn multiplier(&self) -> f32 {
        match self {
            LuckLevel::Low => 0.7,
            LuckLevel::Medium => 1.0,
            LuckLevel::High => 1.3,
        }
    }
}

/// Уровень сопротивления (лингвистическая переменная)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResistanceLevel {
    /// Низкое сопротивление (легкий путь)
    Low,
    /// Среднее сопротивление (обычные препятствия)
    Medium,
    /// Высокое сопротивление (сложный путь)
    High,
}

impl ResistanceLevel {
    /// Получить строковое представление
    pub fn as_str(&self) -> &'static str {
        match self {
            ResistanceLevel::Low => "low",
            ResistanceLevel::Medium => "medium",
            ResistanceLevel::High => "high",
        }
    }

    /// Получить числовой коэффициент сопротивления
    pub fn coefficient(&self) -> f32 {
        match self {
            ResistanceLevel::Low => 0.1,
            ResistanceLevel::Medium => 0.3,
            ResistanceLevel::High => 0.6,
        }
    }
}

/// Результат нечёткой оценки
#[derive(Debug, Clone, Copy)]
pub struct FuzzyLuck {
    /// Уровень удачи
    pub luck_level: LuckLevel,
    /// Уровень сопротивления
    pub resistance_level: ResistanceLevel,
}

impl FuzzyLuck {
    /// Создать новую нечёткую оценку
    pub fn new(luck_level: LuckLevel, resistance_level: ResistanceLevel) -> Self {
        Self {
            luck_level,
            resistance_level,
        }
    }

    /// Вычислить итоговый score с учётом удачи и сопротивления
    pub fn compute_score(&self, base_resonance: f32) -> f32 {
        let luck_mult = self.luck_level.multiplier();
        let resistance_coeff = self.resistance_level.coefficient();

        // Формула: score = base * luck * (1 - resistance)
        base_resonance * luck_mult * (1.0 - resistance_coeff)
    }

    /// Получить текстовое описание
    pub fn description(&self) -> String {
        format!(
            "Luck: {}, Resistance: {}",
            self.luck_level.as_str(),
            self.resistance_level.as_str()
        )
    }
}

/// Оценить нечёткую удачу на основе резонанса
///
/// Использует простые пороги membership functions:
/// - resonance < 0.33 => Low luck, High resistance
/// - 0.33 <= resonance < 0.66 => Medium luck, Medium resistance
/// - resonance >= 0.66 => High luck, Low resistance
///
/// # Arguments
/// * `resonance` - Значение резонанса (0.0-1.0)
///
/// # Returns
/// Нечёткая оценка удачи и сопротивления
pub fn evaluate_fuzzy(resonance: f32) -> FuzzyLuck {
    let (luck_level, resistance_level) = if resonance < 0.33 {
        (LuckLevel::Low, ResistanceLevel::High)
    } else if resonance < 0.66 {
        (LuckLevel::Medium, ResistanceLevel::Medium)
    } else {
        (LuckLevel::High, ResistanceLevel::Low)
    };

    FuzzyLuck::new(luck_level, resistance_level)
}

/// Оценить нечёткую удачу с кастомными порогами
pub fn evaluate_fuzzy_custom(
    resonance: f32,
    low_threshold: f32,
    high_threshold: f32,
) -> FuzzyLuck {
    let (luck_level, resistance_level) = if resonance < low_threshold {
        (LuckLevel::Low, ResistanceLevel::High)
    } else if resonance < high_threshold {
        (LuckLevel::Medium, ResistanceLevel::Medium)
    } else {
        (LuckLevel::High, ResistanceLevel::Low)
    };

    FuzzyLuck::new(luck_level, resistance_level)
}

/// Вычислить степень принадлежности (membership degree) для уровня удачи
///
/// Возвращает значение 0.0-1.0, показывающее насколько сильно
/// данный резонанс принадлежит к категории
pub fn membership_degree(resonance: f32, level: LuckLevel) -> f32 {
    match level {
        LuckLevel::Low => {
            if resonance <= 0.0 {
                1.0
            } else if resonance >= 0.4 {
                0.0
            } else {
                (0.4 - resonance) / 0.4
            }
        }
        LuckLevel::Medium => {
            if resonance < 0.2 {
                0.0
            } else if resonance <= 0.5 {
                (resonance - 0.2) / 0.3
            } else if resonance <= 0.7 {
                (0.7 - resonance) / 0.2
            } else {
                0.0
            }
        }
        LuckLevel::High => {
            if resonance <= 0.6 {
                0.0
            } else if resonance >= 1.0 {
                1.0
            } else {
                (resonance - 0.6) / 0.4
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_fuzzy_low() {
        let fuzzy = evaluate_fuzzy(0.2);

        assert_eq!(fuzzy.luck_level, LuckLevel::Low);
        assert_eq!(fuzzy.resistance_level, ResistanceLevel::High);
    }

    #[test]
    fn test_evaluate_fuzzy_medium() {
        let fuzzy = evaluate_fuzzy(0.5);

        assert_eq!(fuzzy.luck_level, LuckLevel::Medium);
        assert_eq!(fuzzy.resistance_level, ResistanceLevel::Medium);
    }

    #[test]
    fn test_evaluate_fuzzy_high() {
        let fuzzy = evaluate_fuzzy(0.8);

        assert_eq!(fuzzy.luck_level, LuckLevel::High);
        assert_eq!(fuzzy.resistance_level, ResistanceLevel::Low);
    }

    #[test]
    fn test_luck_multiplier() {
        assert_eq!(LuckLevel::Low.multiplier(), 0.7);
        assert_eq!(LuckLevel::Medium.multiplier(), 1.0);
        assert_eq!(LuckLevel::High.multiplier(), 1.3);
    }

    #[test]
    fn test_resistance_coefficient() {
        assert_eq!(ResistanceLevel::Low.coefficient(), 0.1);
        assert_eq!(ResistanceLevel::Medium.coefficient(), 0.3);
        assert_eq!(ResistanceLevel::High.coefficient(), 0.6);
    }

    #[test]
    fn test_compute_score() {
        let fuzzy = FuzzyLuck::new(LuckLevel::High, ResistanceLevel::Low);
        let score = fuzzy.compute_score(0.8);

        // 0.8 * 1.3 * (1 - 0.1) = 0.8 * 1.3 * 0.9 ≈ 0.936
        assert!((score - 0.936).abs() < 0.01);
    }

    #[test]
    fn test_membership_degree_low() {
        // Полная принадлежность к Low при резонансе 0.0
        assert_eq!(membership_degree(0.0, LuckLevel::Low), 1.0);

        // Частичная принадлежность при резонансе 0.2
        assert!(membership_degree(0.2, LuckLevel::Low) > 0.0);

        // Нулевая принадлежность при резонансе 0.5
        assert_eq!(membership_degree(0.5, LuckLevel::Low), 0.0);
    }

    #[test]
    fn test_membership_degree_high() {
        // Нулевая принадлежность к High при низком резонансе
        assert_eq!(membership_degree(0.3, LuckLevel::High), 0.0);

        // Полная принадлежность при резонансе 1.0
        assert_eq!(membership_degree(1.0, LuckLevel::High), 1.0);
    }

    #[test]
    fn test_custom_thresholds() {
        let fuzzy = evaluate_fuzzy_custom(0.5, 0.4, 0.8);

        assert_eq!(fuzzy.luck_level, LuckLevel::Medium);
    }
}
