//! # SOMA Conscious - Self-Awareness and Reflection
//!
//! Модуль осознанности для SOMA. Отслеживает причинно-следственные связи,
//! генерирует инсайты и обеспечивает самонаблюдение сети.
//!
//! ## Компоненты
//!
//! - **CausalTrace**: Запись причинно-следственных цепей
//! - **Insight**: Сгенерированные инсайты о состоянии системы
//! - **AttentionMap**: Карта внимания - топ активных узлов
//! - **ConsciousState**: Текущее состояние осознанности
//! - **ReflectionAnalyzer**: Анализ паттернов и генерация инсайтов
//! - **FeedbackController**: Осознанное вмешательство и коррекция
//!
//! ## Цикл осознанности
//!
//! observe → record → analyze → generate → apply
//!
//! ## Примеры
//!
//! ```
//! use soma_conscious::{ConsciousState, CausalTrace};
//!
//! let mut state = ConsciousState::new();
//!
//! // Записать причинную связь
//! let trace = CausalTrace::new(
//!     "node_alpha_fire".to_string(),
//!     "node_beta_weight_increase".to_string(),
//!     0.05,
//! );
//! state.record_trace(trace);
//! ```

pub mod reflect;
pub mod feedback;

pub use reflect::ReflectionAnalyzer;
pub use feedback::{FeedbackController, FeedbackAction, FeedbackActionType};

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use chrono::Utc;

/// Причинно-следственная цепь (cause → effect)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalTrace {
    /// Причина (событие A)
    pub cause: String,

    /// Следствие (событие B)
    pub effect: String,

    /// Величина изменения
    pub delta: f64,

    /// Время записи (timestamp ms)
    pub timestamp: i64,
}

impl CausalTrace {
    pub fn new(cause: String, effect: String, delta: f64) -> Self {
        Self {
            cause,
            effect,
            delta,
            timestamp: Utc::now().timestamp_millis(),
        }
    }
}

/// Инсайт о состоянии системы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    /// Описание инсайта
    pub insight: String,

    /// Категория (stability, performance, learning, etc.)
    pub category: String,

    /// Уровень важности (0.0-1.0)
    pub importance: f64,

    /// Время генерации
    pub timestamp: i64,
}

impl Insight {
    pub fn new(insight: String, category: String, importance: f64) -> Self {
        Self {
            insight,
            category,
            importance,
            timestamp: Utc::now().timestamp_millis(),
        }
    }
}

/// Узел в карте внимания
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionNode {
    /// ID узла
    pub node_id: String,

    /// Оценка активности (0.0-1.0)
    pub activity: f64,

    /// Количество изменений весов
    pub weight_changes: usize,

    /// Средняя величина изменений
    pub avg_delta: f64,
}

/// Карта внимания - топ активных узлов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionMap {
    /// Топ активных узлов (по умолчанию 10)
    pub top_nodes: Vec<AttentionNode>,

    /// Время обновления
    pub updated_at: i64,
}

impl AttentionMap {
    pub fn new() -> Self {
        Self {
            top_nodes: Vec::new(),
            updated_at: Utc::now().timestamp_millis(),
        }
    }

    pub fn update(&mut self, nodes: Vec<AttentionNode>) {
        self.top_nodes = nodes;
        self.updated_at = Utc::now().timestamp_millis();
    }
}

impl Default for AttentionMap {
    fn default() -> Self {
        Self::new()
    }
}

/// Состояние осознанности системы
#[derive(Debug, Clone)]
pub struct ConsciousState {
    /// История причинных цепей (rolling window)
    traces: VecDeque<CausalTrace>,

    /// Максимальный размер окна traces
    max_traces: usize,

    /// Сгенерированные инсайты
    insights: VecDeque<Insight>,

    /// Максимальный размер окна insights
    max_insights: usize,

    /// Карта внимания
    attention_map: AttentionMap,

    /// Счётчик циклов осознанности
    pub cycle_count: u64,

    /// Время последнего цикла
    pub last_cycle: i64,
}

impl ConsciousState {
    /// Создать новое состояние осознанности
    pub fn new() -> Self {
        Self {
            traces: VecDeque::new(),
            max_traces: 1000,
            insights: VecDeque::new(),
            max_insights: 100,
            attention_map: AttentionMap::new(),
            cycle_count: 0,
            last_cycle: Utc::now().timestamp_millis(),
        }
    }

    /// Записать причинную цепь
    pub fn record_trace(&mut self, trace: CausalTrace) {
        if self.traces.len() >= self.max_traces {
            self.traces.pop_front();
        }
        self.traces.push_back(trace);
    }

    /// Получить последние N traces
    pub fn get_traces(&self, limit: usize) -> Vec<CausalTrace> {
        self.traces
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Получить traces за окно времени (в миллисекундах)
    pub fn get_traces_window(&self, window_ms: i64) -> Vec<CausalTrace> {
        let now = Utc::now().timestamp_millis();
        let cutoff = now - window_ms;

        self.traces
            .iter()
            .filter(|t| t.timestamp >= cutoff)
            .cloned()
            .collect()
    }

    /// Добавить инсайт
    pub fn add_insight(&mut self, insight: Insight) {
        if self.insights.len() >= self.max_insights {
            self.insights.pop_front();
        }
        self.insights.push_back(insight);
    }

    /// Получить последние N инсайтов
    pub fn get_insights(&self, limit: usize) -> Vec<Insight> {
        self.insights
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Обновить карту внимания
    pub fn update_attention_map(&mut self, map: AttentionMap) {
        self.attention_map = map;
    }

    /// Получить текущую карту внимания
    pub fn get_attention_map(&self) -> &AttentionMap {
        &self.attention_map
    }

    /// Завершить цикл осознанности
    pub fn complete_cycle(&mut self) {
        self.cycle_count += 1;
        self.last_cycle = Utc::now().timestamp_millis();
    }

    /// Получить количество traces
    pub fn traces_count(&self) -> usize {
        self.traces.len()
    }

    /// Получить количество insights
    pub fn insights_count(&self) -> usize {
        self.insights.len()
    }
}

impl Default for ConsciousState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_trace() {
        let trace = CausalTrace::new(
            "node_alpha_fire".to_string(),
            "node_beta_weight_up".to_string(),
            0.05,
        );

        assert_eq!(trace.cause, "node_alpha_fire");
        assert_eq!(trace.effect, "node_beta_weight_up");
        assert_eq!(trace.delta, 0.05);
    }

    #[test]
    fn test_conscious_state() {
        let mut state = ConsciousState::new();

        let trace = CausalTrace::new(
            "test_cause".to_string(),
            "test_effect".to_string(),
            0.1,
        );

        state.record_trace(trace);
        assert_eq!(state.traces_count(), 1);

        let traces = state.get_traces(10);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].cause, "test_cause");
    }

    #[test]
    fn test_insights() {
        let mut state = ConsciousState::new();

        let insight = Insight::new(
            "Network is stable".to_string(),
            "stability".to_string(),
            0.8,
        );

        state.add_insight(insight);
        assert_eq!(state.insights_count(), 1);

        let insights = state.get_insights(10);
        assert_eq!(insights.len(), 1);
        assert_eq!(insights[0].category, "stability");
    }

    #[test]
    fn test_attention_map() {
        let mut state = ConsciousState::new();

        let node = AttentionNode {
            node_id: "node_alpha".to_string(),
            activity: 0.9,
            weight_changes: 5,
            avg_delta: 0.05,
        };

        let mut map = AttentionMap::new();
        map.update(vec![node]);

        state.update_attention_map(map);

        let attention = state.get_attention_map();
        assert_eq!(attention.top_nodes.len(), 1);
        assert_eq!(attention.top_nodes[0].node_id, "node_alpha");
    }

    #[test]
    fn test_trace_window() {
        let mut state = ConsciousState::new();

        // Добавить traces
        for i in 0..5 {
            let trace = CausalTrace::new(
                format!("cause_{}", i),
                format!("effect_{}", i),
                0.1,
            );
            state.record_trace(trace);
        }

        // Получить все traces за последние 10 секунд
        let traces = state.get_traces_window(10000);
        assert_eq!(traces.len(), 5);
    }
}
