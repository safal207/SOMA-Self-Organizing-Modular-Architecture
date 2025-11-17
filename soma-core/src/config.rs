//! Конфигурационные константы для SOMA Core

/// Параметры стволового процессора
pub mod stem {
    /// Порог нагрузки по умолчанию
    pub const DEFAULT_THRESHOLD: f64 = 0.7;

    /// Коэффициент сглаживания по умолчанию
    pub const DEFAULT_SMOOTHING: f64 = 0.9;

    /// Множитель снижения нагрузки после деления
    pub const LOAD_REDUCTION_FACTOR: f64 = 0.5;
}

/// Параметры активности клеток
pub mod cell {
    /// Коэффициент затухания активности за тик
    pub const ACTIVITY_DECAY: f64 = 0.95;
}

