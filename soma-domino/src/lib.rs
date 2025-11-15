//! # SOMA Domino Luck Engine
//!
//! "Орган интуиции" сети — оценка удачи и резонанса для принятия решений.
//!
//! ## Концепция
//!
//! Domino Engine оценивает "удачу" и "резонанс" различных вариантов маршрутизации
//! или действий в сети на основе:
//! - Струнного резонанса (string resonance) - взвешенная комбинация health/quality/intent
//! - Нечёткой логики (fuzzy logic) - перевод числовых значений в лингвистические уровни
//! - Q*-подобного цикла - iterative improvement для выбора лучших кандидатов
//!
//! ## Использование
//!
//! ```no_run
//! use soma_domino::{DominoEngine, DominoInput, DominoIntentKind, PeerCandidate};
//!
//! let input = DominoInput {
//!     intent_kind: DominoIntentKind::Routing,
//!     candidates: vec![
//!         PeerCandidate {
//!             peer_id: "alpha".to_string(),
//!             health: 0.9,
//!             quality: 0.8,
//!             intent_match: 0.7,
//!         },
//!     ],
//!     context_tags: vec!["low_latency".to_string()],
//! };
//!
//! let decision = DominoEngine::evaluate(input);
//! println!("Best peers: {:?}", decision.best_peers);
//! println!("Luck score: {}", decision.luck_score);
//! ```

pub mod string_resonance;
pub mod fuzzy_luck;
pub mod qstar_loop;
pub mod engine;

pub use engine::{DominoEngine, DominoDecision, DominoInput, DominoIntentKind, PeerCandidate};
pub use fuzzy_luck::{FuzzyLuck, LuckLevel, ResistanceLevel};
pub use string_resonance::compute_resonance;

/// Версия Domino Engine
pub const DOMINO_VERSION: &str = "0.1.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(DOMINO_VERSION, "0.1.0");
    }
}
