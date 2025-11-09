use soma_core::Resonance;
use std::collections::HashMap;

/// Модуль Inner Council - коллективный разум SOMA
///
/// Координирует работу трёх архетипических модулей:
/// - Пифия (интуиция, предсказание)
/// - Морфей (сны, воображение)
/// - Архитектор (структура, планирование)
pub struct InnerCouncil {
    pythia: Pythia,
    morpheus: Morpheus,
    architect: Architect,
    /// Текущий режим работы совета
    mode: CouncilMode,
}

/// Режимы работы совета
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CouncilMode {
    /// Режим баланса (все модули активны равномерно)
    Balanced,
    /// Режим интуиции (доминирует Пифия)
    Intuitive,
    /// Режим творчества (доминирует Морфей)
    Creative,
    /// Режим планирования (доминирует Архитектор)
    Structured,
}

impl InnerCouncil {
    /// Создать новый совет
    pub fn new() -> Self {
        Self {
            pythia: Pythia::new(),
            morpheus: Morpheus::new(),
            architect: Architect::new(),
            mode: CouncilMode::Balanced,
        }
    }

    /// Установить режим работы
    pub fn set_mode(&mut self, mode: CouncilMode) {
        self.mode = mode;
    }

    /// Получить текущий режим
    pub fn mode(&self) -> CouncilMode {
        self.mode
    }

    /// Принять решение на основе входных данных
    pub fn decide(&mut self, inputs: &HashMap<String, f64>) -> Decision {
        // Получаем мнения от каждого модуля
        let pythia_opinion = self.pythia.predict(inputs);
        let morpheus_opinion = self.morpheus.dream(inputs);
        let architect_opinion = self.architect.plan(inputs);

        // Вычисляем веса в зависимости от режима
        let weights = self.calculate_weights();

        // Взвешенное решение
        let confidence = pythia_opinion.confidence * weights.0
            + morpheus_opinion.confidence * weights.1
            + architect_opinion.confidence * weights.2;

        Decision {
            action: format!(
                "Council decision (mode: {:?})",
                self.mode
            ),
            confidence,
            details: HashMap::from([
                ("pythia".to_string(), pythia_opinion.confidence),
                ("morpheus".to_string(), morpheus_opinion.confidence),
                ("architect".to_string(), architect_opinion.confidence),
            ]),
        }
    }

    /// Рассчитать веса модулей в зависимости от режима
    fn calculate_weights(&self) -> (f64, f64, f64) {
        match self.mode {
            CouncilMode::Balanced => (0.33, 0.33, 0.34),
            CouncilMode::Intuitive => (0.6, 0.2, 0.2),
            CouncilMode::Creative => (0.2, 0.6, 0.2),
            CouncilMode::Structured => (0.2, 0.2, 0.6),
        }
    }

    /// Получить доступ к модулю Пифии
    pub fn pythia(&self) -> &Pythia {
        &self.pythia
    }

    /// Получить доступ к модулю Морфея
    pub fn morpheus(&self) -> &Morpheus {
        &self.morpheus
    }

    /// Получить доступ к модулю Архитектора
    pub fn architect(&self) -> &Architect {
        &self.architect
    }
}

impl Default for InnerCouncil {
    fn default() -> Self {
        Self::new()
    }
}

/// Решение совета
#[derive(Debug, Clone)]
pub struct Decision {
    /// Описание действия
    pub action: String,
    /// Уверенность в решении (0.0 - 1.0)
    pub confidence: f64,
    /// Детали от каждого модуля
    pub details: HashMap<String, f64>,
}

/// Пифия - модуль интуиции и предсказания
pub struct Pythia {
    resonance: Resonance,
}

impl Pythia {
    pub fn new() -> Self {
        Self { resonance: 0.5 }
    }

    /// Предсказать на основе входных данных
    pub fn predict(&mut self, inputs: &HashMap<String, f64>) -> Opinion {
        // Простая эвристика: среднее значение входов
        let avg = if inputs.is_empty() {
            0.5
        } else {
            inputs.values().sum::<f64>() / inputs.len() as f64
        };

        self.resonance = avg;

        Opinion {
            confidence: self.resonance,
            reasoning: "Intuitive prediction based on patterns".to_string(),
        }
    }

    pub fn resonance(&self) -> Resonance {
        self.resonance
    }
}

impl Default for Pythia {
    fn default() -> Self {
        Self::new()
    }
}

/// Морфей - модуль снов и воображения
pub struct Morpheus {
    resonance: Resonance,
}

impl Morpheus {
    pub fn new() -> Self {
        Self { resonance: 0.5 }
    }

    /// Создать творческую интерпретацию
    pub fn dream(&mut self, inputs: &HashMap<String, f64>) -> Opinion {
        // Добавляем элемент случайности/креативности
        let max_value = inputs.values().fold(0.0_f64, |a, &b| a.max(b));
        self.resonance = (max_value + 0.3).min(1.0);

        Opinion {
            confidence: self.resonance,
            reasoning: "Creative exploration of possibilities".to_string(),
        }
    }

    pub fn resonance(&self) -> Resonance {
        self.resonance
    }
}

impl Default for Morpheus {
    fn default() -> Self {
        Self::new()
    }
}

/// Архитектор - модуль структуры и планирования
pub struct Architect {
    resonance: Resonance,
}

impl Architect {
    pub fn new() -> Self {
        Self { resonance: 0.5 }
    }

    /// Создать структурный план
    pub fn plan(&mut self, inputs: &HashMap<String, f64>) -> Opinion {
        // Консервативный подход: минимальное значение
        let min_value = inputs.values().fold(1.0_f64, |a, &b| a.min(b));
        self.resonance = min_value.max(0.4);

        Opinion {
            confidence: self.resonance,
            reasoning: "Structured analysis and planning".to_string(),
        }
    }

    pub fn resonance(&self) -> Resonance {
        self.resonance
    }
}

impl Default for Architect {
    fn default() -> Self {
        Self::new()
    }
}

/// Мнение модуля
#[derive(Debug, Clone)]
pub struct Opinion {
    /// Уверенность (0.0 - 1.0)
    pub confidence: f64,
    /// Обоснование
    pub reasoning: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inner_council_creation() {
        let council = InnerCouncil::new();
        assert_eq!(council.mode(), CouncilMode::Balanced);
    }

    #[test]
    fn test_council_mode_change() {
        let mut council = InnerCouncil::new();
        council.set_mode(CouncilMode::Intuitive);
        assert_eq!(council.mode(), CouncilMode::Intuitive);
    }

    #[test]
    fn test_council_decision() {
        let mut council = InnerCouncil::new();
        let inputs = HashMap::from([
            ("signal1".to_string(), 0.7),
            ("signal2".to_string(), 0.8),
        ]);

        let decision = council.decide(&inputs);
        assert!(decision.confidence > 0.0 && decision.confidence <= 1.0);
        assert_eq!(decision.details.len(), 3);
    }

    #[test]
    fn test_pythia() {
        let mut pythia = Pythia::new();
        let inputs = HashMap::from([("x".to_string(), 0.6)]);

        let opinion = pythia.predict(&inputs);
        assert_eq!(opinion.confidence, 0.6);
    }
}
