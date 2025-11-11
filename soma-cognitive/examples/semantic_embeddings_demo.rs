//! # Semantic Embeddings Demo (v1.2)
//!
//! Демонстрация embedding-based semantic similarity в Cognitive Mesh v1.2.
//! Показывает, как узлы понимают смысл намерений через векторные представления.

use soma_cognitive::{
    pulse::{CognitivePulse, Intent},
    embeddings::{IntentEmbeddings, SemanticClusterer, cosine_similarity},
};

fn main() {
    println!("🧠 SOMA v1.2 - Semantic Embeddings Demo\n");
    println!("{}\n", "=".repeat(50));

    // === 1. Создание Intent Embeddings ===
    println!("📊 1. Intent Embeddings - Векторное пространство намерений\n");

    let embeddings = IntentEmbeddings::new();

    // Получить embeddings для разных Intent
    let stabilize_emb = embeddings.get_embedding(&Intent::Stabilize);
    let healing_emb = embeddings.get_embedding(&Intent::AdaptiveHealing);
    let balance_emb = embeddings.get_embedding(&Intent::BalanceLoad);
    let optimize_emb = embeddings.get_embedding(&Intent::Optimize);
    let explore_emb = embeddings.get_embedding(&Intent::Explore);

    println!("Embedding размерность: {} dimensions", stabilize_emb.len());
    println!("Stabilize vector:  {:?}...", &stabilize_emb[..4]);
    println!("Healing vector:    {:?}...", &healing_emb[..4]);
    println!("Explore vector:    {:?}...\n", &explore_emb[..4]);

    // === 2. Cosine Similarity ===
    println!("🔗 2. Cosine Similarity - Измерение близости намерений\n");

    let sim_stable_heal = cosine_similarity(&stabilize_emb, &healing_emb);
    let sim_stable_explore = cosine_similarity(&stabilize_emb, &explore_emb);
    let sim_balance_optimize = cosine_similarity(&balance_emb, &optimize_emb);

    println!("Stabilize <-> AdaptiveHealing:  {:.3}", sim_stable_heal);
    println!("Stabilize <-> Explore:          {:.3}", sim_stable_explore);
    println!("BalanceLoad <-> Optimize:       {:.3}", sim_balance_optimize);

    println!("\n💡 Insight:");
    println!("   • Stabilize и AdaptiveHealing семантически близки ({:.1}%)", sim_stable_heal * 100.0);
    println!("   • Stabilize и Explore семантически далеки ({:.1}%)", sim_stable_explore * 100.0);
    println!("   • BalanceLoad и Optimize близки ({:.1}%)\n", sim_balance_optimize * 100.0);

    // === 3. Embedding-based Semantic Overlap ===
    println!("📡 3. Cognitive Pulse с Embedding-based Overlap\n");

    let pulse_alpha = CognitivePulse::new(
        "node_alpha".to_string(),
        Intent::Stabilize,
        0.85,
    );

    let pulse_beta = CognitivePulse::new(
        "node_beta".to_string(),
        Intent::AdaptiveHealing,
        0.78,
    );

    let pulse_gamma = CognitivePulse::new(
        "node_gamma".to_string(),
        Intent::Explore,
        0.92,
    );

    // Используем новый embedding-based метод
    let overlap_alpha_beta = pulse_alpha.semantic_overlap_embedding(&pulse_beta, &embeddings);
    let overlap_alpha_gamma = pulse_alpha.semantic_overlap_embedding(&pulse_gamma, &embeddings);

    println!("Alpha (Stabilize) <-> Beta (AdaptiveHealing): {:.3}", overlap_alpha_beta);
    println!("Alpha (Stabilize) <-> Gamma (Explore):        {:.3}", overlap_alpha_gamma);

    if overlap_alpha_beta > 0.7 {
        println!("\n✨ Alpha и Beta формируют когнитивный кластер!");
        println!("   Общее намерение: стабильность и восстановление");
    }

    if overlap_alpha_gamma < 0.5 {
        println!("\n🔀 Alpha и Gamma в разных когнитивных пространствах");
        println!("   Alpha: стабильность, Gamma: исследование");
    }

    // === 4. Semantic Clustering ===
    println!("\n\n🧬 4. Semantic Clustering - Автоматическая группировка\n");

    let clusterer = SemanticClusterer::new(0.7);

    // Создаем набор узлов с разными намерениями
    let nodes = vec![
        ("node_1".to_string(), embeddings.get_embedding(&Intent::Stabilize)),
        ("node_2".to_string(), embeddings.get_embedding(&Intent::AdaptiveHealing)),
        ("node_3".to_string(), embeddings.get_embedding(&Intent::Explore)),
        ("node_4".to_string(), embeddings.get_embedding(&Intent::Stabilize)),
        ("node_5".to_string(), embeddings.get_embedding(&Intent::BalanceLoad)),
        ("node_6".to_string(), embeddings.get_embedding(&Intent::Optimize)),
        ("node_7".to_string(), embeddings.get_embedding(&Intent::Explore)),
    ];

    let clusters = clusterer.find_clusters(&nodes);

    println!("Найдено {} когнитивных кластеров:\n", clusters.len());

    for (i, cluster) in clusters.iter().enumerate() {
        println!("Кластер {}:", i + 1);
        println!("  Узлы: {}", cluster.join(", "));
        println!("  Размер: {} узлов\n", cluster.len());
    }

    // === 5. Custom Intent Embeddings ===
    println!("🎨 5. Custom Intent - Динамическая генерация embeddings\n");

    let custom1 = Intent::Custom("stabilize_network_latency".to_string());
    let custom2 = Intent::Custom("optimize_throughput".to_string());

    let custom1_emb = embeddings.get_embedding(&custom1);
    let custom2_emb = embeddings.get_embedding(&custom2);

    let sim_custom = cosine_similarity(&custom1_emb, &custom2_emb);

    println!("Custom Intent 1: 'stabilize_network_latency'");
    println!("Custom Intent 2: 'optimize_throughput'");
    println!("Similarity: {:.3}\n", sim_custom);

    // Сравнение с базовыми Intent
    let sim_custom1_stabilize = cosine_similarity(&custom1_emb, &stabilize_emb);
    let sim_custom2_optimize = cosine_similarity(&custom2_emb, &optimize_emb);

    println!("Custom 1 <-> Stabilize: {:.3}", sim_custom1_stabilize);
    println!("Custom 2 <-> Optimize:  {:.3}\n", sim_custom2_optimize);

    // === 6. Сравнение: эвристика vs embeddings ===
    println!("⚖️  6. Сравнение методов: Heuristic vs Embeddings\n");

    #[allow(deprecated)]
    let heuristic_overlap = pulse_alpha.semantic_overlap(&pulse_beta);
    let embedding_overlap = pulse_alpha.semantic_overlap_embedding(&pulse_beta, &embeddings);

    println!("Stabilize <-> AdaptiveHealing:");
    println!("  Эвристический метод (v1.1):  {:.3}", heuristic_overlap);
    println!("  Embedding-based метод (v1.2): {:.3}", embedding_overlap);
    println!("\n  Разница: {:.3} ({:.1}% изменение)",
        (embedding_overlap - heuristic_overlap).abs(),
        ((embedding_overlap - heuristic_overlap) / heuristic_overlap * 100.0).abs()
    );

    println!("\n💡 Embedding-based подход даёт более точное понимание семантики!");

    println!("\n\n🎉 Demo завершено!");
    println!("SOMA v1.2 теперь понимает смысл, а не только сравнивает строки.\n");
}
