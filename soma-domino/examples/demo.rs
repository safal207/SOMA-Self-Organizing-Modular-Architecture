//! # SOMA Domino Engine Demo
//!
//! Демонстрация работы Domino Luck Engine для маршрутизации и выбора пиров.

use soma_domino::{DominoEngine, DominoInput, DominoIntentKind, PeerCandidate};

fn main() {
    println!("🎲 SOMA Domino Luck Engine Demo\n");
    println!("{}\n", "=".repeat(60));

    // === Сценарий 1: Routing - выбор лучшего пира для маршрутизации ===
    println!("📊 Сценарий 1: Routing Intent\n");

    let routing_candidates = vec![
        PeerCandidate {
            peer_id: "node_alpha".to_string(),
            health: 0.95,
            quality: 0.88,
            intent_match: 0.92,
        },
        PeerCandidate {
            peer_id: "node_beta".to_string(),
            health: 0.75,
            quality: 0.70,
            intent_match: 0.65,
        },
        PeerCandidate {
            peer_id: "node_gamma".to_string(),
            health: 0.60,
            quality: 0.55,
            intent_match: 0.50,
        },
        PeerCandidate {
            peer_id: "node_delta".to_string(),
            health: 0.40,
            quality: 0.35,
            intent_match: 0.30,
        },
    ];

    let routing_input = DominoInput::new(
        DominoIntentKind::Routing,
        routing_candidates,
        vec!["low_latency".to_string(), "high_bandwidth".to_string()],
    );

    let routing_decision = DominoEngine::evaluate(routing_input);

    println!("Результат маршрутизации:");
    println!("  🎯 Лучшие пиры: {:?}", routing_decision.best_peers);
    println!("  🍀 Luck score: {:.2}", routing_decision.luck_score);
    println!("  🛡️  Resistance score: {:.2}", routing_decision.resistance_score);
    println!("  💬 {}\n", routing_decision.explanation);

    // === Сценарий 2: Task Scheduling - выбор узлов для выполнения задачи ===
    println!("{}\n", "=".repeat(60));
    println!("📊 Сценарий 2: Task Scheduling Intent\n");

    let task_candidates = vec![
        PeerCandidate {
            peer_id: "worker_001".to_string(),
            health: 0.85,
            quality: 0.90,
            intent_match: 0.80,
        },
        PeerCandidate {
            peer_id: "worker_002".to_string(),
            health: 0.92,
            quality: 0.85,
            intent_match: 0.88,
        },
        PeerCandidate {
            peer_id: "worker_003".to_string(),
            health: 0.70,
            quality: 0.75,
            intent_match: 0.65,
        },
    ];

    let task_input = DominoInput::new(
        DominoIntentKind::TaskScheduling,
        task_candidates,
        vec!["cpu_intensive".to_string(), "memory_available".to_string()],
    );

    let task_decision = DominoEngine::evaluate(task_input);

    println!("Результат планирования задачи:");
    println!("  🎯 Лучшие воркеры: {:?}", task_decision.best_peers);
    println!("  🍀 Luck score: {:.2}", task_decision.luck_score);
    println!("  🛡️  Resistance score: {:.2}", task_decision.resistance_score);
    println!("  💬 {}\n", task_decision.explanation);

    // === Сценарий 3: Top-N - выбор только топ-2 пиров ===
    println!("{}\n", "=".repeat(60));
    println!("📊 Сценарий 3: Top-N Selection (N=2)\n");

    let topn_candidates = vec![
        PeerCandidate {
            peer_id: "peer_A".to_string(),
            health: 0.90,
            quality: 0.85,
            intent_match: 0.88,
        },
        PeerCandidate {
            peer_id: "peer_B".to_string(),
            health: 0.95,
            quality: 0.92,
            intent_match: 0.90,
        },
        PeerCandidate {
            peer_id: "peer_C".to_string(),
            health: 0.75,
            quality: 0.70,
            intent_match: 0.72,
        },
        PeerCandidate {
            peer_id: "peer_D".to_string(),
            health: 0.88,
            quality: 0.82,
            intent_match: 0.85,
        },
        PeerCandidate {
            peer_id: "peer_E".to_string(),
            health: 0.65,
            quality: 0.60,
            intent_match: 0.55,
        },
    ];

    let topn_input = DominoInput::new(
        DominoIntentKind::UserRequest,
        topn_candidates,
        vec!["fast_response".to_string()],
    );

    let topn_decision = DominoEngine::evaluate_top_n(topn_input, 2);

    println!("Результат Top-2 выбора:");
    println!("  🎯 Top-2 пиры: {:?}", topn_decision.best_peers);
    println!("  🍀 Luck score: {:.2}", topn_decision.luck_score);
    println!("  🛡️  Resistance score: {:.2}", topn_decision.resistance_score);
    println!("  💬 {}\n", topn_decision.explanation);

    // === Сценарий 4: Threshold - фильтрация по минимальному score ===
    println!("{}\n", "=".repeat(60));
    println!("📊 Сценарий 4: Threshold Filtering (min_score >= 0.7)\n");

    let threshold_candidates = vec![
        PeerCandidate {
            peer_id: "high_quality".to_string(),
            health: 0.95,
            quality: 0.93,
            intent_match: 0.90,
        },
        PeerCandidate {
            peer_id: "medium_quality".to_string(),
            health: 0.70,
            quality: 0.65,
            intent_match: 0.60,
        },
        PeerCandidate {
            peer_id: "low_quality".to_string(),
            health: 0.40,
            quality: 0.35,
            intent_match: 0.30,
        },
    ];

    let threshold_input = DominoInput::new(
        DominoIntentKind::Custom("critical_operation".to_string()),
        threshold_candidates,
        vec!["high_reliability".to_string()],
    );

    let threshold_decision = DominoEngine::evaluate_with_threshold(threshold_input, 0.7);

    println!("Результат с порогом 0.7:");
    println!("  🎯 Qualified пиры: {:?}", threshold_decision.best_peers);
    println!("  🍀 Luck score: {:.2}", threshold_decision.luck_score);
    println!("  🛡️  Resistance score: {:.2}", threshold_decision.resistance_score);
    println!("  💬 {}\n", threshold_decision.explanation);

    // === Сценарий 5: Empty candidates - обработка пустого списка ===
    println!("{}\n", "=".repeat(60));
    println!("📊 Сценарий 5: Empty Candidates List\n");

    let empty_input = DominoInput::routing(vec![]);
    let empty_decision = DominoEngine::evaluate(empty_input);

    println!("Результат с пустым списком:");
    println!("  🎯 Лучшие пиры: {:?}", empty_decision.best_peers);
    println!("  🍀 Luck score: {:.2}", empty_decision.luck_score);
    println!("  🛡️  Resistance score: {:.2}", empty_decision.resistance_score);
    println!("  💬 {}\n", empty_decision.explanation);

    // === Сценарий 6: Builder pattern - использование with_tags ===
    println!("{}\n", "=".repeat(60));
    println!("📊 Сценарий 6: Builder Pattern - Custom Tags\n");

    let builder_candidates = vec![
        PeerCandidate {
            peer_id: "cognitive_node".to_string(),
            health: 0.88,
            quality: 0.85,
            intent_match: 0.90,
        },
    ];

    let builder_input = DominoInput::routing(builder_candidates).with_tags(vec![
        "cognitive_mesh".to_string(),
        "semantic_analysis".to_string(),
        "distributed_consensus".to_string(),
    ]);

    let builder_decision = DominoEngine::evaluate(builder_input);

    println!("Результат с кастомными тегами:");
    println!("  🎯 Лучшие пиры: {:?}", builder_decision.best_peers);
    println!("  🍀 Luck score: {:.2}", builder_decision.luck_score);
    println!("  🛡️  Resistance score: {:.2}", builder_decision.resistance_score);
    println!("  💬 {}\n", builder_decision.explanation);

    println!("{}\n", "=".repeat(60));
    println!("✅ Demo completed!");
    println!("\n💡 Domino Engine помогает выбирать лучших пиров на основе:");
    println!("   - String resonance (health, quality, intent_match)");
    println!("   - Fuzzy logic (luck/resistance levels)");
    println!("   - Q* loop (iterative score optimization)");
    println!("   - Phase coefficient (time-based oscillation)\n");
}
