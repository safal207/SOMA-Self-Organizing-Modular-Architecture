//! # Cognitive Mesh Demo
//!
//! Демонстрация работы Cognitive Mesh v1.1:
//! - Cognitive Pulse - синхронизация намерений
//! - Inference Braid - коллективное решение задач
//! - Collective Memory - запись когнитивных событий
//! - Metametric Layer - мониторинг когнитивной активности

use soma_cognitive::{
    pulse::{CognitivePulse, Intent},
    braid::{InferenceBraid, Task, TaskType, BraidResult},
    memory::{CollectiveMemory, CognitiveEvent, EventType, EventResult},
    metrics::{CognitiveMetrics, MetricsAggregator},
};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("🧠 SOMA Cognitive Mesh v1.1 Demo\n");
    println!("{}\n", "=".repeat(40));

    // === 1. Cognitive Pulse Demo ===
    println!("📡 1. Cognitive Pulse - Синхронизация намерений\n");

    let pulse_alpha = CognitivePulse::new(
        "node_alpha".to_string(),
        Intent::Stabilize,
        0.82,
    );

    let pulse_beta = CognitivePulse::new(
        "node_beta".to_string(),
        Intent::AdaptiveHealing,
        0.75,
    );

    let pulse_gamma = CognitivePulse::new(
        "node_gamma".to_string(),
        Intent::Stabilize,
        0.91,
    );

    println!("Alpha pulse: {}", pulse_alpha.to_json().unwrap());
    println!("\nBeta pulse: {}", pulse_beta.to_json().unwrap());
    println!("\nGamma pulse: {}", pulse_gamma.to_json().unwrap());

    // Вычисление semantic overlap
    let overlap_alpha_gamma = pulse_alpha.semantic_overlap(&pulse_gamma);
    let overlap_alpha_beta = pulse_alpha.semantic_overlap(&pulse_beta);

    println!("\n🔗 Semantic Overlap:");
    println!("  Alpha <-> Gamma: {:.2} (одинаковое намерение)", overlap_alpha_gamma);
    println!("  Alpha <-> Beta:  {:.2} (близкие намерения)", overlap_alpha_beta);

    if overlap_alpha_gamma > 0.7 {
        println!("\n✨ Alpha и Gamma формируют когнитивный кластер!");
    }

    sleep(Duration::from_secs(1)).await;

    // === 2. Inference Braid Demo ===
    println!("\n\n🧵 2. Inference Braid - Коллективное решение\n");

    let braid = InferenceBraid::new();

    // Создание задачи
    let mut task = Task::new(
        "task_001".to_string(),
        TaskType::HypothesisCheck("узел gamma перегружен?".to_string()),
        "node_alpha".to_string(),
    );

    task.add_participant("node_beta".to_string());
    task.add_participant("node_gamma".to_string());

    println!("A (Alpha): propose('узел gamma перегружен?')");
    braid.propose(task.clone()).await.unwrap();

    sleep(Duration::from_millis(500)).await;

    println!("B (Beta):  simulate(...) -> проверка метрик");
    braid.validate("task_001".to_string(), "node_beta".to_string()).await.unwrap();

    sleep(Duration::from_millis(500)).await;

    println!("C (Gamma): summarize('да, latency вырос на 34%')");
    braid.aggregate("task_001".to_string(), "latency_increase: 34%".to_string()).await.unwrap();

    sleep(Duration::from_millis(500)).await;

    let result = BraidResult::success(
        "task_001".to_string(),
        0.91,
        "Confirmed: node gamma is overloaded, latency increased by 34%".to_string(),
        vec!["node_alpha".to_string(), "node_beta".to_string(), "node_gamma".to_string()],
    );

    println!("\n✅ Результат Inference Braid:");
    println!("   Task ID: {}", result.task_id);
    println!("   Success: {}", result.success);
    println!("   Confidence: {:.2}", result.confidence);
    println!("   Result: {}", result.result);
    println!("   Participants: {}", result.participants.join(", "));

    sleep(Duration::from_secs(1)).await;

    // === 3. Collective Memory Demo ===
    println!("\n\n💾 3. Collective Memory - Запись событий\n");

    let memory = CollectiveMemory::new(PathBuf::from("./liminal-bd/snapshots/cognitive"), 1000);

    // Записать события
    let event1 = CognitiveEvent::new(
        "evt_001".to_string(),
        EventType::IntentSync,
        vec!["node_alpha".to_string(), "node_gamma".to_string()],
        EventResult::Success,
        0.95,
    )
    .with_task("stabilize_network".to_string());

    let event2 = CognitiveEvent::new(
        "evt_002".to_string(),
        EventType::BraidExecution,
        vec!["node_alpha".to_string(), "node_beta".to_string(), "node_gamma".to_string()],
        EventResult::Success,
        0.91,
    )
    .with_task("check_overload".to_string());

    memory.record(event1.clone()).await;
    memory.record(event2.clone()).await;

    println!("Записано событий: {}", memory.all_events().await.len());
    println!("Success rate: {:.2}%", memory.success_rate().await * 100.0);

    // Статистика участников
    let stats = memory.participant_stats().await;
    println!("\n📊 Статистика участников:");
    for (node, stat) in stats.iter() {
        println!("  {}: {} events, success: {:.0}%, confidence: {:.2}",
            node, stat.total_events, stat.success_rate() * 100.0, stat.avg_confidence);
    }

    sleep(Duration::from_secs(1)).await;

    // === 4. Metametric Layer Demo ===
    println!("\n\n📈 4. Metametric Layer - Мониторинг\n");

    let metrics = CognitiveMetrics::new(100);

    // Обновить метрики
    metrics.update_cognitive_overlap(overlap_alpha_gamma).await;
    metrics.update_clusters(1).await;
    metrics.update_braid_success_rate(1.0).await;
    metrics.update_reflection_latency(45).await;
    metrics.update_nodes_count(3).await;
    metrics.update_braids_active(1).await;

    let snapshot = metrics.snapshot().await;

    println!("Метрики Cognitive Mesh:");
    println!("  cognitive_overlap_avg:       {:.2}", snapshot.cognitive_overlap_avg);
    println!("  clusters_active_total:       {}", snapshot.clusters_active_total);
    println!("  braid_success_rate:          {:.2}", snapshot.braid_success_rate);
    println!("  self_reflection_latency_ms:  {} ms", snapshot.self_reflection_latency_ms);
    println!("  nodes_total:                 {}", snapshot.nodes_total);
    println!("  braids_active:               {}", snapshot.braids_active);

    // Экспорт в Prometheus
    println!("\n📤 Экспорт в Prometheus format:");
    println!("{}", snapshot.to_prometheus());

    // === 5. Агрегация глобальных метрик ===
    println!("\n🌐 5. Глобальная агрегация метрик\n");

    let aggregator = MetricsAggregator::new();
    aggregator.add_snapshot("node_alpha".to_string(), snapshot.clone()).await;
    aggregator.add_snapshot("node_beta".to_string(), snapshot.clone()).await;

    let global_overlap = aggregator.global_cognitive_overlap().await;
    let total_clusters = aggregator.total_active_clusters().await;

    println!("Глобальные метрики сети:");
    println!("  Global cognitive overlap: {:.2}", global_overlap);
    println!("  Total active clusters:    {}", total_clusters);

    println!("\n\n🎉 Demo завершено!");
    println!("Cognitive Mesh v1.1 готов к использованию в LIMINAL-сети.\n");
}
