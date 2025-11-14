//! Integration tests for Domino Engine
//!
//! Тесты проверяют:
//! - Выбор превосходного пира из списка
//! - Влияние health на luck_score
//! - Порядок best_peers (отсортирован по score)

use soma_domino::{DominoEngine, DominoInput, DominoIntentKind, PeerCandidate};

#[test]
fn test_selects_superior_peer() {
    // Создаём кандидатов: один явно превосходный, остальные посредственные
    let candidates = vec![
        PeerCandidate {
            peer_id: "superior".to_string(),
            health: 0.98,
            quality: 0.95,
            intent_match: 0.92,
        },
        PeerCandidate {
            peer_id: "mediocre_1".to_string(),
            health: 0.45,
            quality: 0.40,
            intent_match: 0.38,
        },
        PeerCandidate {
            peer_id: "mediocre_2".to_string(),
            health: 0.50,
            quality: 0.48,
            intent_match: 0.42,
        },
    ];

    let input = DominoInput::new(
        DominoIntentKind::Routing,
        candidates,
        vec![],
    );

    let decision = DominoEngine::evaluate(input);

    // Лучший пир должен быть "superior"
    assert_eq!(decision.best_peers[0], "superior");

    // Luck score должен быть высоким (> 0.5, учитывая phase coefficient)
    assert!(
        decision.luck_score > 0.5,
        "Expected luck_score > 0.5, got {}",
        decision.luck_score
    );

    // Resistance score должен быть относительно низким (< 0.5, учитывая phase coefficient)
    assert!(
        decision.resistance_score < 0.5,
        "Expected resistance_score < 0.5, got {}",
        decision.resistance_score
    );
}

#[test]
fn test_high_health_increases_luck_score() {
    // Сценарий 1: Все кандидаты здоровы
    let healthy_candidates = vec![
        PeerCandidate {
            peer_id: "healthy_1".to_string(),
            health: 0.95,
            quality: 0.90,
            intent_match: 0.85,
        },
        PeerCandidate {
            peer_id: "healthy_2".to_string(),
            health: 0.92,
            quality: 0.88,
            intent_match: 0.82,
        },
    ];

    let input_healthy = DominoInput::new(
        DominoIntentKind::Routing,
        healthy_candidates,
        vec![],
    );

    let decision_healthy = DominoEngine::evaluate(input_healthy);

    // Сценарий 2: Все кандидаты нездоровы
    let unhealthy_candidates = vec![
        PeerCandidate {
            peer_id: "unhealthy_1".to_string(),
            health: 0.35,
            quality: 0.30,
            intent_match: 0.25,
        },
        PeerCandidate {
            peer_id: "unhealthy_2".to_string(),
            health: 0.40,
            quality: 0.35,
            intent_match: 0.30,
        },
    ];

    let input_unhealthy = DominoInput::new(
        DominoIntentKind::Routing,
        unhealthy_candidates,
        vec![],
    );

    let decision_unhealthy = DominoEngine::evaluate(input_unhealthy);

    // Здоровые кандидаты должны иметь значительно более высокий luck_score
    assert!(
        decision_healthy.luck_score > decision_unhealthy.luck_score + 0.3,
        "Healthy candidates should have significantly higher luck_score: {} vs {}",
        decision_healthy.luck_score,
        decision_unhealthy.luck_score
    );

    // Сопротивление должно быть выше у нездоровых
    assert!(
        decision_unhealthy.resistance_score > decision_healthy.resistance_score,
        "Unhealthy candidates should have higher resistance: {} vs {}",
        decision_unhealthy.resistance_score,
        decision_healthy.resistance_score
    );
}

#[test]
fn test_best_peers_sorted_by_score() {
    // Создаём кандидатов с явными различиями в метриках
    let candidates = vec![
        PeerCandidate {
            peer_id: "best".to_string(),
            health: 0.95,
            quality: 0.92,
            intent_match: 0.90,
        },
        PeerCandidate {
            peer_id: "good".to_string(),
            health: 0.75,
            quality: 0.70,
            intent_match: 0.65,
        },
        PeerCandidate {
            peer_id: "medium".to_string(),
            health: 0.55,
            quality: 0.50,
            intent_match: 0.45,
        },
        PeerCandidate {
            peer_id: "poor".to_string(),
            health: 0.35,
            quality: 0.30,
            intent_match: 0.25,
        },
    ];

    let input = DominoInput::new(
        DominoIntentKind::TaskScheduling,
        candidates,
        vec![],
    );

    let decision = DominoEngine::evaluate(input);

    // best_peers должен содержать всех кандидатов
    assert_eq!(decision.best_peers.len(), 4);

    // Первый должен быть "best" (наивысшие метрики)
    assert_eq!(decision.best_peers[0], "best");

    // Последний должен быть "poor" (наименьшие метрики)
    assert_eq!(decision.best_peers[3], "poor");

    // Второй и третий могут быть "good" или "medium" в зависимости от phase coefficient
    // но проверим, что они не "best" и не "poor"
    assert_ne!(decision.best_peers[1], "best");
    assert_ne!(decision.best_peers[1], "poor");
    assert_ne!(decision.best_peers[2], "best");
    assert_ne!(decision.best_peers[2], "poor");
}

#[test]
fn test_intent_kinds() {
    let candidate = vec![
        PeerCandidate {
            peer_id: "test_peer".to_string(),
            health: 0.80,
            quality: 0.75,
            intent_match: 0.70,
        },
    ];

    // Тестируем разные типы намерений
    let intents = vec![
        DominoIntentKind::Routing,
        DominoIntentKind::TaskScheduling,
        DominoIntentKind::UserRequest,
        DominoIntentKind::Custom("custom_intent".to_string()),
    ];

    for intent in intents {
        let input = DominoInput::new(
            intent.clone(),
            candidate.clone(),
            vec![],
        );

        let decision = DominoEngine::evaluate(input);

        // Все намерения должны возвращать результат
        assert_eq!(decision.best_peers.len(), 1);
        assert_eq!(decision.best_peers[0], "test_peer");

        // explanation должен быть не пустым
        assert!(!decision.explanation.is_empty());

        // luck_score должен быть в диапазоне [0, 1]
        assert!(decision.luck_score >= 0.0 && decision.luck_score <= 1.0);
    }
}

#[test]
fn test_top_n_functionality() {
    let candidates = vec![
        PeerCandidate {
            peer_id: "peer_1".to_string(),
            health: 0.95,
            quality: 0.90,
            intent_match: 0.88,
        },
        PeerCandidate {
            peer_id: "peer_2".to_string(),
            health: 0.85,
            quality: 0.80,
            intent_match: 0.75,
        },
        PeerCandidate {
            peer_id: "peer_3".to_string(),
            health: 0.75,
            quality: 0.70,
            intent_match: 0.65,
        },
        PeerCandidate {
            peer_id: "peer_4".to_string(),
            health: 0.65,
            quality: 0.60,
            intent_match: 0.55,
        },
        PeerCandidate {
            peer_id: "peer_5".to_string(),
            health: 0.55,
            quality: 0.50,
            intent_match: 0.45,
        },
    ];

    let input = DominoInput::new(
        DominoIntentKind::Routing,
        candidates,
        vec![],
    );

    // Выбрать топ-3
    let decision = DominoEngine::evaluate_top_n(input, 3);

    // Должно быть ровно 3 пира
    assert_eq!(decision.best_peers.len(), 3);

    // Первый должен быть peer_1 (самый лучший)
    assert_eq!(decision.best_peers[0], "peer_1");

    // luck_score должен быть высоким для топ-3
    assert!(decision.luck_score > 0.6);
}

#[test]
fn test_threshold_filtering() {
    let candidates = vec![
        PeerCandidate {
            peer_id: "excellent".to_string(),
            health: 0.98,
            quality: 0.95,
            intent_match: 0.92,
        },
        PeerCandidate {
            peer_id: "good".to_string(),
            health: 0.75,
            quality: 0.70,
            intent_match: 0.68,
        },
        PeerCandidate {
            peer_id: "poor".to_string(),
            health: 0.35,
            quality: 0.30,
            intent_match: 0.25,
        },
    ];

    let input = DominoInput::new(
        DominoIntentKind::TaskScheduling,
        candidates,
        vec![],
    );

    // Фильтрация с высоким порогом (должны пройти только excellent)
    let decision_high = DominoEngine::evaluate_with_threshold(input.clone(), 0.8);

    // С высоким порогом должен пройти только "excellent"
    assert!(
        decision_high.best_peers.len() <= 2,
        "High threshold should filter out poor candidates"
    );

    // "excellent" должен быть в списке
    assert!(decision_high.best_peers.contains(&"excellent".to_string()));

    // Фильтрация с низким порогом (должны пройти все или почти все)
    let input2 = DominoInput::new(
        DominoIntentKind::TaskScheduling,
        vec![
            PeerCandidate {
                peer_id: "excellent".to_string(),
                health: 0.98,
                quality: 0.95,
                intent_match: 0.92,
            },
            PeerCandidate {
                peer_id: "good".to_string(),
                health: 0.75,
                quality: 0.70,
                intent_match: 0.68,
            },
            PeerCandidate {
                peer_id: "poor".to_string(),
                health: 0.35,
                quality: 0.30,
                intent_match: 0.25,
            },
        ],
        vec![],
    );

    let decision_low = DominoEngine::evaluate_with_threshold(input2, 0.2);

    // С низким порогом должно пройти больше кандидатов
    assert!(
        decision_low.best_peers.len() >= decision_high.best_peers.len(),
        "Low threshold should allow more candidates"
    );
}

#[test]
fn test_context_tags() {
    let candidates = vec![
        PeerCandidate {
            peer_id: "tagged_peer".to_string(),
            health: 0.80,
            quality: 0.75,
            intent_match: 0.70,
        },
    ];

    // С тегами
    let input_with_tags = DominoInput::new(
        DominoIntentKind::Routing,
        candidates.clone(),
        vec!["low_latency".to_string(), "high_bandwidth".to_string()],
    );

    let decision_with_tags = DominoEngine::evaluate(input_with_tags);

    // Без тегов
    let input_without_tags = DominoInput::new(
        DominoIntentKind::Routing,
        candidates,
        vec![],
    );

    let decision_without_tags = DominoEngine::evaluate(input_without_tags);

    // Результаты должны быть сопоставимы (теги не влияют на score в текущей версии)
    assert_eq!(decision_with_tags.best_peers, decision_without_tags.best_peers);

    // Оба должны быть валидными
    assert!(!decision_with_tags.explanation.is_empty());
    assert!(!decision_without_tags.explanation.is_empty());
}

#[test]
fn test_builder_pattern() {
    let candidates = vec![
        PeerCandidate {
            peer_id: "test".to_string(),
            health: 0.80,
            quality: 0.75,
            intent_match: 0.70,
        },
    ];

    // Использование builder pattern
    let input = DominoInput::routing(candidates.clone())
        .with_tags(vec!["tag1".to_string(), "tag2".to_string()]);

    let decision = DominoEngine::evaluate(input);

    assert_eq!(decision.best_peers.len(), 1);
    assert_eq!(decision.best_peers[0], "test");
}
