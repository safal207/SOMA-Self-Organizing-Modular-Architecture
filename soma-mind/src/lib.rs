//! # SOMA Mind - Высокоуровневый контроль и Inner Council
//!
//! Модуль сознания SOMA - координирует работу всей системы через Inner Council.
//!
//! ## Архитектура Inner Council
//!
//! Совет состоит из трёх архетипических модулей:
//!
//! - **Пифия** - интуиция и предсказание (Oracle)
//! - **Морфей** - сны и воображение (Dreamer)
//! - **Архитектор** - структура и планирование (Builder)
//!
//! ## Режимы работы
//!
//! - **Balanced**: Все модули работают равномерно
//! - **Intuitive**: Доминирует интуиция (Пифия)
//! - **Creative**: Доминирует творчество (Морфей)
//! - **Structured**: Доминирует планирование (Архитектор)
//!
//! ## Примеры
//!
//! ```
//! use soma_mind::{InnerCouncil, CouncilMode};
//! use std::collections::HashMap;
//!
//! let mut council = InnerCouncil::new();
//! council.set_mode(CouncilMode::Intuitive);
//!
//! let inputs = HashMap::from([
//!     ("signal1".to_string(), 0.7),
//!     ("signal2".to_string(), 0.8),
//! ]);
//!
//! let decision = council.decide(&inputs);
//! println!("Decision confidence: {}", decision.confidence);
//! ```

pub mod council;

pub use council::{
    Architect, CouncilMode, Decision, InnerCouncil, Morpheus, Opinion, Pythia,
};

/// Конфигурация модуля разума
#[derive(Debug, Clone)]
pub struct MindConfig {
    /// Режим работы по умолчанию
    pub default_mode: CouncilMode,
    /// Минимальная уверенность для принятия решения
    pub min_confidence: f64,
    /// Включить адаптивное переключение режимов
    pub adaptive_mode: bool,
}

impl Default for MindConfig {
    fn default() -> Self {
        Self {
            default_mode: CouncilMode::Balanced,
            min_confidence: 0.5,
            adaptive_mode: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mind_config() {
        let config = MindConfig::default();
        assert_eq!(config.default_mode, CouncilMode::Balanced);
        assert_eq!(config.min_confidence, 0.5);
    }
}
