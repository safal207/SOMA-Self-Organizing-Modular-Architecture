use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Сигнал - базовая единица передачи данных в SOMA
///
/// Используется для передачи активности нейронов, резонансов и других событий
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// Уникальный идентификатор источника сигнала
    pub id: String,
    /// Значение сигнала (обычно 0.0 - 1.0)
    pub value: f64,
    /// Временная метка в миллисекундах (Unix timestamp)
    pub timestamp: i64,
}

impl Signal {
    /// Создать новый сигнал
    pub fn new(id: &str, value: f64) -> Self {
        Self {
            id: id.to_string(),
            value,
            timestamp: current_timestamp_millis(),
        }
    }

    /// Создать сигнал с заданной временной меткой
    pub fn with_timestamp(id: &str, value: f64, timestamp: i64) -> Self {
        Self {
            id: id.to_string(),
            value,
            timestamp,
        }
    }

    /// Проверить, старше ли сигнал заданного времени (в миллисекундах)
    pub fn is_older_than(&self, millis: i64) -> bool {
        let now = current_timestamp_millis();
        now - self.timestamp > millis
    }

    /// Получить возраст сигнала в миллисекундах
    pub fn age_millis(&self) -> i64 {
        let now = current_timestamp_millis();
        now - self.timestamp
    }
}

/// Получить текущую временную метку в миллисекундах
fn current_timestamp_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let signal = Signal::new("test_neuron", 0.75);

        assert_eq!(signal.id, "test_neuron");
        assert_eq!(signal.value, 0.75);
        assert!(signal.timestamp > 0);
    }

    #[test]
    fn test_signal_age() {
        let old_timestamp = current_timestamp_millis() - 1000;
        let signal = Signal::with_timestamp("test", 0.5, old_timestamp);

        assert!(signal.is_older_than(500));
        assert!(!signal.is_older_than(2000));
        assert!(signal.age_millis() >= 1000);
    }
}
