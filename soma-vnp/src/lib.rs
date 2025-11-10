//! # SOMA VNP - Virtual Neuro/Muscle Processors
//!
//! Виртуальные процессоры SOMA для обработки сигналов и вычислений.
//! Реализует нейронные структуры на основе паттерна Sense-Align-Flow.
//!
//! ## Компоненты
//!
//! - **Neuron**: Виртуальный нейрон с порогом активации
//! - **NeuronLayer**: Слой связанных нейронов
//!
//! ## Примеры
//!
//! ```
//! use soma_vnp::Neuron;
//! use soma_core::Cell;
//!
//! let mut neuron = Neuron::new();
//! neuron.sense(0.8);
//! neuron.align();
//! let output = neuron.flow();
//! ```

pub mod neuron;

pub use neuron::{Neuron, NeuronLayer};

/// Тип процессора VNP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorType {
    /// Нейронный процессор (обработка сигналов)
    Neural,
    /// Мышечный процессор (выполнение действий)
    Muscle,
    /// Гибридный процессор
    Hybrid,
}

/// Конфигурация VNP процессора
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Тип процессора
    pub processor_type: ProcessorType,
    /// Количество нейронов/юнитов
    pub unit_count: usize,
    /// Параметры обучения
    pub learning_rate: f64,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            processor_type: ProcessorType::Neural,
            unit_count: 10,
            learning_rate: 0.01,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_config() {
        let config = ProcessorConfig::default();
        assert_eq!(config.processor_type, ProcessorType::Neural);
        assert_eq!(config.unit_count, 10);
    }
}
