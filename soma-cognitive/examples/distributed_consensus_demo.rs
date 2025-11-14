//! # Distributed Consensus Demo (v1.3)
//!
//! Демонстрация distributed consensus в Cognitive Mesh v1.3.
//! Показывает, как узлы голосуют за результаты и достигают консенсуса
//! даже при наличии сбоев и Byzantine узлов.

use soma_cognitive::{
    consensus::{ConsensusManager, ConsensusResult, NodeVote, Vote, ByzantineDetector},
    braid::{InferenceBraid, Task, TaskType},
};

#[tokio::main]
async fn main() {
    println!("🗳️  SOMA v1.3 - Distributed Consensus Demo\n");
    println!("{}\n", "=".repeat(50));

    // === 1. Базовый консенсус ===
    println!("📊 1. Базовый консенсус - Простое голосование\n");

    let consensus_manager = ConsensusManager::new(0.66, 3);

    // Начать раунд консенсуса
    consensus_manager
        .start_round("round_001".to_string(), "task_001".to_string())
        .await
        .unwrap();

    // Узлы голосуют
    println!("Узлы голосуют за результат задачи...\n");

    consensus_manager
        .submit_vote(
            "round_001",
            NodeVote::new("node_alpha".to_string(), Vote::Accept, 0.92)
                .with_reasoning("Результат проверен, данные корректны".to_string()),
        )
        .await
        .unwrap();
    println!("✓ node_alpha: Accept (confidence: 0.92)");

    consensus_manager
        .submit_vote(
            "round_001",
            NodeVote::new("node_beta".to_string(), Vote::Accept, 0.85)
                .with_reasoning("Симуляция подтверждает результат".to_string()),
        )
        .await
        .unwrap();
    println!("✓ node_beta: Accept (confidence: 0.85)");

    consensus_manager
        .submit_vote(
            "round_001",
            NodeVote::new("node_gamma".to_string(), Vote::Accept, 0.90)
                .with_reasoning("Метрики в норме".to_string()),
        )
        .await
        .unwrap();
    println!("✓ node_gamma: Accept (confidence: 0.90)");

    // Вычислить консенсус
    let result = consensus_manager
        .finalize_round("round_001", false)
        .await
        .unwrap();

    println!("\n🎯 Результат консенсуса:");
    match result {
        ConsensusResult::Accepted {
            acceptance_rate,
            participants,
        } => {
            println!("   ✅ ПРИНЯТ");
            println!("   Acceptance rate: {:.1}%", acceptance_rate * 100.0);
            println!("   Participants: {}", participants);
        }
        _ => println!("   ❌ Не принят"),
    }

    // === 2. Weighted Consensus ===
    println!("\n\n⚖️  2. Weighted Consensus - Учет confidence\n");

    consensus_manager
        .start_round("round_002".to_string(), "task_002".to_string())
        .await
        .unwrap();

    println!("Узлы голосуют с разной уверенностью...\n");

    // Два узла с высокой уверенностью "за"
    consensus_manager
        .submit_vote(
            "round_002",
            NodeVote::new("node_alpha".to_string(), Vote::Accept, 0.95),
        )
        .await
        .unwrap();
    println!("✓ node_alpha: Accept (confidence: 0.95) - высокая уверенность");

    consensus_manager
        .submit_vote(
            "round_002",
            NodeVote::new("node_beta".to_string(), Vote::Accept, 0.90),
        )
        .await
        .unwrap();
    println!("✓ node_beta: Accept (confidence: 0.90) - высокая уверенность");

    // Один узел с низкой уверенностью "против"
    consensus_manager
        .submit_vote(
            "round_002",
            NodeVote::new("node_gamma".to_string(), Vote::Reject, 0.25),
        )
        .await
        .unwrap();
    println!("✓ node_gamma: Reject (confidence: 0.25) - низкая уверенность");

    let result_weighted = consensus_manager
        .finalize_round("round_002", true)
        .await
        .unwrap();

    println!("\n🎯 Weighted Consensus:");
    match result_weighted {
        ConsensusResult::Accepted {
            acceptance_rate, ..
        } => {
            println!("   ✅ ПРИНЯТ (weighted)");
            println!("   Weighted acceptance: {:.1}%", acceptance_rate * 100.0);
            println!("   💡 Confidence узлов учтена как вес голоса");
        }
        _ => println!("   ❌ Не принят"),
    }

    // === 3. Отклонение консенсусом ===
    println!("\n\n❌ 3. Отклонение результата - Большинство против\n");

    consensus_manager
        .start_round("round_003".to_string(), "task_003".to_string())
        .await
        .unwrap();

    println!("Узлы обнаружили проблему в результате...\n");

    consensus_manager
        .submit_vote(
            "round_003",
            NodeVote::new("node_alpha".to_string(), Vote::Reject, 0.88)
                .with_reasoning("Данные не согласуются".to_string()),
        )
        .await
        .unwrap();
    println!("✗ node_alpha: Reject - данные не согласуются");

    consensus_manager
        .submit_vote(
            "round_003",
            NodeVote::new("node_beta".to_string(), Vote::Reject, 0.92)
                .with_reasoning("Ошибка в вычислениях".to_string()),
        )
        .await
        .unwrap();
    println!("✗ node_beta: Reject - ошибка в вычислениях");

    consensus_manager
        .submit_vote(
            "round_003",
            NodeVote::new("node_gamma".to_string(), Vote::Accept, 0.65),
        )
        .await
        .unwrap();
    println!("✓ node_gamma: Accept");

    let result_reject = consensus_manager
        .finalize_round("round_003", false)
        .await
        .unwrap();

    println!("\n🎯 Результат:");
    match result_reject {
        ConsensusResult::Rejected {
            rejection_rate, ..
        } => {
            println!("   ❌ ОТКЛОНЕН");
            println!("   Rejection rate: {:.1}%", rejection_rate * 100.0);
            println!("   💡 Сеть самокорректируется!");
        }
        _ => println!("   Другой результат"),
    }

    // === 4. No Consensus ===
    println!("\n\n🤷 4. No Consensus - Мнения разделились\n");

    consensus_manager
        .start_round("round_004".to_string(), "task_004".to_string())
        .await
        .unwrap();

    println!("Узлы не могут прийти к соглашению...\n");

    consensus_manager
        .submit_vote(
            "round_004",
            NodeVote::new("node_alpha".to_string(), Vote::Accept, 0.80),
        )
        .await
        .unwrap();
    println!("✓ node_alpha: Accept");

    consensus_manager
        .submit_vote(
            "round_004",
            NodeVote::new("node_beta".to_string(), Vote::Reject, 0.75),
        )
        .await
        .unwrap();
    println!("✗ node_beta: Reject");

    consensus_manager
        .submit_vote(
            "round_004",
            NodeVote::new("node_gamma".to_string(), Vote::Abstain, 0.50),
        )
        .await
        .unwrap();
    println!("⊝ node_gamma: Abstain - недостаточно данных");

    let result_no_consensus = consensus_manager
        .finalize_round("round_004", false)
        .await
        .unwrap();

    println!("\n🎯 Результат:");
    match result_no_consensus {
        ConsensusResult::NoConsensus {
            vote_distribution,
            participants,
        } => {
            println!("   ⚠️  КОНСЕНСУС НЕ ДОСТИГНУТ");
            println!("   Participants: {}", participants);
            println!("   Распределение голосов:");
            for (vote_type, count) in vote_distribution {
                println!("     - {}: {}", vote_type, count);
            }
            println!("   💡 Требуется дополнительный раунд или данные");
        }
        _ => println!("   Другой результат"),
    }

    // === 5. Byzantine Fault Tolerance ===
    println!("\n\n🛡️  5. Byzantine Fault Tolerance - Детекция недобросовестных узлов\n");

    let byzantine_detector = ByzantineDetector::new(0.6);

    println!("Симуляция поведения узлов...\n");

    // Честный узел - последовательные голоса
    println!("Узел 'honest_node': последовательное поведение");
    for i in 0..10 {
        byzantine_detector
            .record_vote(NodeVote::new(
                "honest_node".to_string(),
                Vote::Accept,
                0.85,
            ))
            .await;
        if i % 3 == 0 {
            print!("✓");
        }
    }
    println!(" (все голоса: Accept)\n");

    // Byzantine узел - часто меняет мнение
    println!("Узел 'byzantine_node': непоследовательное поведение");
    for i in 0..10 {
        let vote = if i % 2 == 0 {
            Vote::Accept
        } else {
            Vote::Reject
        };
        byzantine_detector
            .record_vote(NodeVote::new("byzantine_node".to_string(), vote, 0.80))
            .await;
        print!("{}", if i % 2 == 0 { "✓" } else { "✗" });
    }
    println!(" (чередование Accept/Reject)\n");

    // Проверка
    let is_honest_byzantine = byzantine_detector.is_byzantine("honest_node").await;
    let is_byzantine_byzantine = byzantine_detector.is_byzantine("byzantine_node").await;

    println!("🔍 Анализ детектора:\n");
    println!("   honest_node: {}", if is_honest_byzantine {
        "❌ Byzantine (ошибка!)"
    } else {
        "✅ Честный"
    });

    println!("   byzantine_node: {}", if is_byzantine_byzantine {
        "⚠️  Byzantine detected!"
    } else {
        "✅ Честный (ошибка!)"
    });

    if is_byzantine_byzantine {
        println!("\n   💡 Сеть может исключить Byzantine узлы из консенсуса");
    }

    // === 6. Интеграция с Inference Braid ===
    println!("\n\n🧵 6. Интеграция с Inference Braid\n");

    println!("Создаём задачу Inference Braid...");
    let braid = InferenceBraid::new();
    let task = Task::new(
        "task_network_check".to_string(),
        TaskType::HypothesisCheck("проверить стабильность сети".to_string()),
        "node_alpha".to_string(),
    );

    braid.propose(task).await.unwrap();
    println!("✓ Задача предложена\n");

    println!("Узлы выполняют задачу и голосуют за результат...");

    consensus_manager
        .start_round("braid_round_001".to_string(), "task_network_check".to_string())
        .await
        .unwrap();

    // Результаты от разных узлов
    consensus_manager
        .submit_vote(
            "braid_round_001",
            NodeVote::new("node_alpha".to_string(), Vote::Accept, 0.91)
                .with_reasoning("Latency в норме: 45ms".to_string()),
        )
        .await
        .unwrap();

    consensus_manager
        .submit_vote(
            "braid_round_001",
            NodeVote::new("node_beta".to_string(), Vote::Accept, 0.88)
                .with_reasoning("Throughput стабилен".to_string()),
        )
        .await
        .unwrap();

    consensus_manager
        .submit_vote(
            "braid_round_001",
            NodeVote::new("node_gamma".to_string(), Vote::Accept, 0.94)
                .with_reasoning("Нет потерь пакетов".to_string()),
        )
        .await
        .unwrap();

    let braid_result = consensus_manager
        .finalize_round("braid_round_001", true)
        .await
        .unwrap();

    println!("\n🎯 Коллективное решение:");
    match braid_result {
        ConsensusResult::Accepted {
            acceptance_rate, ..
        } => {
            println!("   ✅ Сеть стабильна (консенсус: {:.1}%)", acceptance_rate * 100.0);
            println!("   💡 Inference Braid + Consensus = Надежное коллективное решение!");
        }
        _ => println!("   Консенсус не достигнут"),
    }

    println!("\n\n🎉 Demo завершено!");
    println!("SOMA v1.3: Узлы умеют договариваться даже при сбоях! 🗳️\n");
}
