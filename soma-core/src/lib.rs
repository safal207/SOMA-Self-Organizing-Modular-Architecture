//! # SOMA Core - Стволовые клетки и базовые интерфейсы
//!
//! Ядро SOMA (Self-Organizing Modular Architecture) содержит фундаментальные
//! абстракции и паттерны для построения саморганизующихся систем.
//!
//! ## Основные концепции
//!
//! - **Cell**: Базовая единица обработки (Sense-Align-Flow)
//! - **StemCell**: Универсальная клетка для дифференциации
//! - **StemProcessor**: Стволовой процессор для порождения новых клеток
//! - **Resonance**: Механизм синхронизации и передачи состояния

pub mod cell;
pub mod stem;
pub mod config;

pub use cell::{Cell, StemCell};
pub use stem::{CellInfo, CellRole, StemProcessor};

/// Версия протокола SOMA
pub const SOMA_VERSION: &str = "0.1.0";

/// Тип резонанса - нормализованное значение от 0.0 до 1.0
pub type Resonance = f64;

/// Базовые константы резонанса
pub mod resonance {
    use super::Resonance;

    /// Минимальный резонанс (полное затухание)
    pub const MIN: Resonance = 0.0;

    /// Нейтральный резонанс (баланс)
    pub const NEUTRAL: Resonance = 0.5;

    /// Максимальный резонанс (полная активация)
    pub const MAX: Resonance = 1.0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(SOMA_VERSION, "0.1.0");
    }

    #[test]
    fn test_resonance_constants() {
        assert_eq!(resonance::MIN, 0.0);
        assert_eq!(resonance::NEUTRAL, 0.5);
        assert_eq!(resonance::MAX, 1.0);
    }
}
