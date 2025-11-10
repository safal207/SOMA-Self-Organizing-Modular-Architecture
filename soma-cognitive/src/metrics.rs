//! # Metametric Layer - Метрики когнитивной активности
//!
//! Отслеживание:
//! - cognitive_overlap_avg - среднее совпадение намерений
//! - clusters_active_total - число когнитивных сообществ
//! - braid_success_rate - успешность группового вывода
//! - self_reflection_latency_ms - время отклика на самоанализ

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Снимок метрик когнитивной активности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    /// Временная метка
    pub timestamp: u64,

    /// Среднее совпадение намерений в сети (0.0 - 1.0)
    pub cognitive_overlap_avg: f64,

    /// Число активных когнитивных кластеров
    pub clusters_active_total: usize,

    /// Успешность группового вывода (0.0 - 1.0)
    pub braid_success_rate: f64,

    /// Время отклика сети на самоанализ (мс)
    pub self_reflection_latency_ms: u64,

    /// Общее число узлов в сети
    pub nodes_total: usize,

    /// Число активных inference braids
    pub braids_active: usize,

    /// Дополнительные метрики
    pub custom_metrics: HashMap<String, f64>,
}

impl MetricSnapshot {
    /// Создать новый снимок метрик
    pub fn new() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            cognitive_overlap_avg: 0.0,
            clusters_active_total: 0,
            braid_success_rate: 0.0,
            self_reflection_latency_ms: 0,
            nodes_total: 0,
            braids_active: 0,
            custom_metrics: HashMap::new(),
        }
    }

    /// Добавить кастомную метрику
    pub fn add_custom(&mut self, key: String, value: f64) {
        self.custom_metrics.insert(key, value);
    }

    /// Экспортировать в JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Экспортировать в Prometheus format
    pub fn to_prometheus(&self) -> String {
        let mut output = String::new();

        output.push_str("# HELP cognitive_overlap_avg Average semantic overlap between nodes\n");
        output.push_str("# TYPE cognitive_overlap_avg gauge\n");
        output.push_str(&format!(
            "cognitive_overlap_avg {}\n",
            self.cognitive_overlap_avg
        ));

        output.push_str("# HELP clusters_active_total Number of active cognitive clusters\n");
        output.push_str("# TYPE clusters_active_total gauge\n");
        output.push_str(&format!(
            "clusters_active_total {}\n",
            self.clusters_active_total
        ));

        output.push_str("# HELP braid_success_rate Success rate of collective inference\n");
        output.push_str("# TYPE braid_success_rate gauge\n");
        output.push_str(&format!(
            "braid_success_rate {}\n",
            self.braid_success_rate
        ));

        output.push_str("# HELP self_reflection_latency_ms Network self-reflection latency in milliseconds\n");
        output.push_str("# TYPE self_reflection_latency_ms gauge\n");
        output.push_str(&format!(
            "self_reflection_latency_ms {}\n",
            self.self_reflection_latency_ms
        ));

        output
    }
}

impl Default for MetricSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Менеджер когнитивных метрик
pub struct CognitiveMetrics {
    /// Текущие метрики
    current: Arc<RwLock<MetricSnapshot>>,

    /// История снимков (последние N)
    history: Arc<RwLock<Vec<MetricSnapshot>>>,

    /// Максимальный размер истории
    max_history_size: usize,
}

impl CognitiveMetrics {
    /// Создать новый менеджер метрик
    pub fn new(max_history_size: usize) -> Self {
        Self {
            current: Arc::new(RwLock::new(MetricSnapshot::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
        }
    }

    /// Обновить метрику cognitive_overlap_avg
    pub async fn update_cognitive_overlap(&self, value: f64) {
        let mut current = self.current.write().await;
        current.cognitive_overlap_avg = value;
    }

    /// Обновить метрику clusters_active_total
    pub async fn update_clusters(&self, count: usize) {
        let mut current = self.current.write().await;
        current.clusters_active_total = count;
    }

    /// Обновить метрику braid_success_rate
    pub async fn update_braid_success_rate(&self, rate: f64) {
        let mut current = self.current.write().await;
        current.braid_success_rate = rate;
    }

    /// Обновить метрику self_reflection_latency_ms
    pub async fn update_reflection_latency(&self, latency_ms: u64) {
        let mut current = self.current.write().await;
        current.self_reflection_latency_ms = latency_ms;
    }

    /// Обновить число узлов
    pub async fn update_nodes_count(&self, count: usize) {
        let mut current = self.current.write().await;
        current.nodes_total = count;
    }

    /// Обновить число активных braids
    pub async fn update_braids_active(&self, count: usize) {
        let mut current = self.current.write().await;
        current.braids_active = count;
    }

    /// Добавить кастомную метрику
    pub async fn add_custom_metric(&self, key: String, value: f64) {
        let mut current = self.current.write().await;
        current.add_custom(key, value);
    }

    /// Получить текущий снимок
    pub async fn snapshot(&self) -> MetricSnapshot {
        self.current.read().await.clone()
    }

    /// Сохранить текущий снимок в историю
    pub async fn save_snapshot(&self) {
        let snapshot = self.current.read().await.clone();
        let mut history = self.history.write().await;

        history.push(snapshot);

        // Ограничить размер истории
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    /// Получить историю снимков
    pub async fn history(&self) -> Vec<MetricSnapshot> {
        self.history.read().await.clone()
    }

    /// Вычислить тренд для метрики
    pub async fn compute_trend(&self, metric_name: &str) -> Option<f64> {
        let history = self.history.read().await;

        if history.len() < 2 {
            return None;
        }

        let values: Vec<f64> = history
            .iter()
            .map(|s| match metric_name {
                "cognitive_overlap_avg" => s.cognitive_overlap_avg,
                "braid_success_rate" => s.braid_success_rate,
                _ => 0.0,
            })
            .collect();

        // Простой линейный тренд (последнее значение - первое)
        Some(values.last().unwrap() - values.first().unwrap())
    }

    /// Экспортировать текущие метрики в JSON
    pub async fn export_json(&self) -> Result<String, serde_json::Error> {
        let snapshot = self.current.read().await;
        snapshot.to_json()
    }

    /// Экспортировать текущие метрики в Prometheus format
    pub async fn export_prometheus(&self) -> String {
        let snapshot = self.current.read().await;
        snapshot.to_prometheus()
    }
}

impl Default for CognitiveMetrics {
    fn default() -> Self {
        Self::new(100) // По умолчанию хранить 100 снимков
    }
}

/// Агрегатор метрик для анализа когнитивной сети
pub struct MetricsAggregator {
    /// Снимки от разных узлов
    node_snapshots: Arc<RwLock<HashMap<String, MetricSnapshot>>>,
}

impl MetricsAggregator {
    /// Создать новый агрегатор
    pub fn new() -> Self {
        Self {
            node_snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Добавить снимок от узла
    pub async fn add_snapshot(&self, node_id: String, snapshot: MetricSnapshot) {
        let mut snapshots = self.node_snapshots.write().await;
        snapshots.insert(node_id, snapshot);
    }

    /// Вычислить глобальную метрику cognitive_overlap_avg
    pub async fn global_cognitive_overlap(&self) -> f64 {
        let snapshots = self.node_snapshots.read().await;

        if snapshots.is_empty() {
            return 0.0;
        }

        let sum: f64 = snapshots.values().map(|s| s.cognitive_overlap_avg).sum();
        sum / snapshots.len() as f64
    }

    /// Вычислить общее число активных кластеров
    pub async fn total_active_clusters(&self) -> usize {
        let snapshots = self.node_snapshots.read().await;
        snapshots.values().map(|s| s.clusters_active_total).sum()
    }

    /// Получить топ N узлов по cognitive overlap
    pub async fn top_nodes_by_overlap(&self, n: usize) -> Vec<(String, f64)> {
        let snapshots = self.node_snapshots.read().await;

        let mut nodes: Vec<(String, f64)> = snapshots
            .iter()
            .map(|(id, s)| (id.clone(), s.cognitive_overlap_avg))
            .collect();

        nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        nodes.truncate(n);
        nodes
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_snapshot() {
        let mut snapshot = MetricSnapshot::new();
        snapshot.cognitive_overlap_avg = 0.85;
        snapshot.clusters_active_total = 5;

        assert_eq!(snapshot.cognitive_overlap_avg, 0.85);
        assert_eq!(snapshot.clusters_active_total, 5);
    }

    #[tokio::test]
    async fn test_cognitive_metrics() {
        let metrics = CognitiveMetrics::new(10);

        metrics.update_cognitive_overlap(0.75).await;
        metrics.update_clusters(3).await;

        let snapshot = metrics.snapshot().await;
        assert_eq!(snapshot.cognitive_overlap_avg, 0.75);
        assert_eq!(snapshot.clusters_active_total, 3);
    }

    #[tokio::test]
    async fn test_metrics_history() {
        let metrics = CognitiveMetrics::new(5);

        for i in 0..7 {
            metrics.update_cognitive_overlap(i as f64 * 0.1).await;
            metrics.save_snapshot().await;
        }

        let history = metrics.history().await;
        assert_eq!(history.len(), 5); // Должно быть не больше max_history_size
    }

    #[test]
    fn test_prometheus_export() {
        let mut snapshot = MetricSnapshot::new();
        snapshot.cognitive_overlap_avg = 0.82;
        snapshot.clusters_active_total = 4;

        let prom = snapshot.to_prometheus();
        assert!(prom.contains("cognitive_overlap_avg 0.82"));
        assert!(prom.contains("clusters_active_total 4"));
    }

    #[tokio::test]
    async fn test_metrics_aggregator() {
        let aggregator = MetricsAggregator::new();

        let mut snapshot1 = MetricSnapshot::new();
        snapshot1.cognitive_overlap_avg = 0.8;

        let mut snapshot2 = MetricSnapshot::new();
        snapshot2.cognitive_overlap_avg = 0.6;

        aggregator.add_snapshot("node_a".to_string(), snapshot1).await;
        aggregator.add_snapshot("node_b".to_string(), snapshot2).await;

        let global_overlap = aggregator.global_cognitive_overlap().await;
        assert!((global_overlap - 0.7).abs() < 0.01);
    }
}
