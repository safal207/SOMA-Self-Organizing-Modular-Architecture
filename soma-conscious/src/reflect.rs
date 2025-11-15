//! # Reflection Module - Pattern Analysis and Insight Generation
//!
//! Analyzes causal chains and generates insights about system state.

use crate::{CausalTrace, ConsciousState, Insight, DominoDecisionTrace, DecisionOutcome};
use std::collections::HashMap;

/// Pattern analyzer for causal chains
pub struct ReflectionAnalyzer {
    /// Threshold for determining significant change
    significance_threshold: f64,
}

impl ReflectionAnalyzer {
    pub fn new() -> Self {
        Self {
            significance_threshold: 0.05,
        }
    }

    /// Set significance threshold
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.significance_threshold = threshold;
        self
    }

    /// Analyze traces and generate insights
    pub fn analyze(&self, state: &ConsciousState, window_ms: i64) -> Vec<Insight> {
        let traces = state.get_traces_window(window_ms);
        let mut insights = Vec::new();

        // Analysis 1: Weight change frequency
        if let Some(insight) = self.analyze_weight_change_frequency(&traces) {
            insights.push(insight);
        }

        // Analysis 2: Network stability
        if let Some(insight) = self.analyze_network_stability(&traces) {
            insights.push(insight);
        }

        // Analysis 3: Active nodes
        if let Some(insight) = self.analyze_active_nodes(&traces) {
            insights.push(insight);
        }

        // Analysis 4: Trends (growth/decline of weights)
        if let Some(insight) = self.analyze_trends(&traces) {
            insights.push(insight);
        }

        insights
    }

    /// Analyze weight change frequency
    fn analyze_weight_change_frequency(&self, traces: &[CausalTrace]) -> Option<Insight> {
        if traces.is_empty() {
            return None;
        }

        let weight_changes: Vec<&CausalTrace> = traces
            .iter()
            .filter(|t| t.effect.contains("weight"))
            .collect();

        let frequency = weight_changes.len() as f64 / traces.len() as f64;

        if frequency > 0.5 {
            Some(Insight::new(
                format!(
                    "High learning activity detected: {:.1}% of events involve weight changes",
                    frequency * 100.0
                ),
                "learning".to_string(),
                0.8,
            ))
        } else if frequency < 0.1 {
            Some(Insight::new(
                "Low learning activity: network weights are stable".to_string(),
                "stability".to_string(),
                0.6,
            ))
        } else {
            None
        }
    }

    /// Analyze network stability
    fn analyze_network_stability(&self, traces: &[CausalTrace]) -> Option<Insight> {
        if traces.is_empty() {
            return None;
        }

        // Count large changes (|delta| > threshold)
        let large_changes = traces
            .iter()
            .filter(|t| t.delta.abs() > self.significance_threshold)
            .count();

        let instability_ratio = large_changes as f64 / traces.len() as f64;

        if instability_ratio > 0.3 {
            Some(Insight::new(
                format!(
                    "Network instability detected: {:.1}% of changes are significant",
                    instability_ratio * 100.0
                ),
                "stability".to_string(),
                0.9,
            ))
        } else if instability_ratio < 0.05 {
            Some(Insight::new(
                "Network is highly stable with minimal fluctuations".to_string(),
                "stability".to_string(),
                0.7,
            ))
        } else {
            None
        }
    }

    /// Analyze active nodes
    fn analyze_active_nodes(&self, traces: &[CausalTrace]) -> Option<Insight> {
        if traces.is_empty() {
            return None;
        }

        // Count mentions of each node
        let mut node_mentions: HashMap<String, usize> = HashMap::new();

        for trace in traces {
            // Extract node_id from cause/effect (assuming format "node_X_action")
            if let Some(node) = self.extract_node_id(&trace.cause) {
                *node_mentions.entry(node).or_insert(0) += 1;
            }
            if let Some(node) = self.extract_node_id(&trace.effect) {
                *node_mentions.entry(node).or_insert(0) += 1;
            }
        }

        if let Some((most_active, count)) = node_mentions.iter().max_by_key(|(_, &count)| count) {
            let activity_ratio = *count as f64 / (traces.len() * 2) as f64;

            if activity_ratio > 0.3 {
                Some(Insight::new(
                    format!(
                        "Node {} is highly active, involved in {:.1}% of network events",
                        most_active,
                        activity_ratio * 100.0
                    ),
                    "performance".to_string(),
                    0.7,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Analyze trends (growth/decline of weights)
    fn analyze_trends(&self, traces: &[CausalTrace]) -> Option<Insight> {
        if traces.len() < 5 {
            return None;
        }

        let weight_deltas: Vec<f64> = traces
            .iter()
            .filter(|t| t.effect.contains("weight"))
            .map(|t| t.delta)
            .collect();

        if weight_deltas.is_empty() {
            return None;
        }

        let avg_delta = weight_deltas.iter().sum::<f64>() / weight_deltas.len() as f64;

        if avg_delta > 0.02 {
            Some(Insight::new(
                format!(
                    "Network is strengthening: average weight change +{:.3}",
                    avg_delta
                ),
                "learning".to_string(),
                0.75,
            ))
        } else if avg_delta < -0.02 {
            Some(Insight::new(
                format!(
                    "Network is weakening: average weight change {:.3}",
                    avg_delta
                ),
                "learning".to_string(),
                0.8,
            ))
        } else {
            Some(Insight::new(
                "Network weights are in equilibrium".to_string(),
                "stability".to_string(),
                0.5,
            ))
        }
    }

    /// Extract node_id from string (e.g., "node_alpha_fire" -> "node_alpha")
    fn extract_node_id(&self, s: &str) -> Option<String> {
        // Simple heuristic: take first two words separated by _
        let parts: Vec<&str> = s.split('_').collect();
        if parts.len() >= 2 {
            Some(format!("{}_{}", parts[0], parts[1]))
        } else {
            None
        }
    }

    // === Domino Decision Analysis (v1.2 Week 2) ===

    /// Analyze routing decisions and generate insights
    pub fn analyze_routing_decisions(&self, state: &ConsciousState) -> Vec<Insight> {
        let decisions = state.get_decisions();
        let mut insights = Vec::new();

        if decisions.is_empty() {
            return insights;
        }

        // Analysis 1: Per-peer success rates
        if let Some(insight) = self.analyze_peer_success_rates(&decisions) {
            insights.push(insight);
        }

        // Analysis 2: Luck correlation
        if let Some(insight) = self.analyze_luck_correlation(&decisions) {
            insights.push(insight);
        }

        // Analysis 3: Intent-specific patterns
        if let Some(insight) = self.analyze_intent_patterns(&decisions) {
            insights.push(insight);
        }

        // Analysis 4: Anomaly detection (high luck but failed)
        if let Some(insight) = self.analyze_decision_anomalies(&decisions) {
            insights.push(insight);
        }

        insights
    }

    /// Calculate per-peer success rates and generate insights
    fn analyze_peer_success_rates(&self, decisions: &[DominoDecisionTrace]) -> Option<Insight> {
        if decisions.is_empty() {
            return None;
        }

        let mut peer_stats: HashMap<String, (usize, usize)> = HashMap::new();

        for decision in decisions {
            let entry = peer_stats.entry(decision.chosen_peer.clone()).or_insert((0, 0));
            entry.0 += 1; // Total

            match &decision.outcome {
                DecisionOutcome::Success { .. } => entry.1 += 1, // Successful
                _ => {}
            }
        }

        // Find best performing peer
        let best_peer = peer_stats
            .iter()
            .filter(|(_, (total, _))| *total >= 3) // At least 3 decisions
            .max_by(|(_, (total_a, success_a)), (_, (total_b, success_b))| {
                let rate_a = *success_a as f64 / *total_a as f64;
                let rate_b = *success_b as f64 / *total_b as f64;
                rate_a.partial_cmp(&rate_b).unwrap()
            });

        if let Some((peer_id, (total, successful))) = best_peer {
            let success_rate = *successful as f64 / *total as f64;

            if success_rate >= 0.7 {
                Some(Insight::new(
                    format!(
                        "Peer '{}' has high success rate: {:.1}% ({}/{})",
                        peer_id,
                        success_rate * 100.0,
                        successful,
                        total
                    ),
                    "routing_performance".to_string(),
                    0.85,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Analyze correlation between luck score and actual outcomes
    fn analyze_luck_correlation(&self, decisions: &[DominoDecisionTrace]) -> Option<Insight> {
        // Filter decisions with outcomes (not Pending)
        let completed: Vec<&DominoDecisionTrace> = decisions
            .iter()
            .filter(|d| !matches!(d.outcome, DecisionOutcome::Pending))
            .collect();

        if completed.len() < 5 {
            return None;
        }

        // Calculate success rate for high luck decisions (>= 0.8)
        let high_luck: Vec<&DominoDecisionTrace> = completed
            .iter()
            .filter(|d| d.luck_score >= 0.8)
            .copied()
            .collect();

        if high_luck.is_empty() {
            return None;
        }

        let high_luck_success = high_luck
            .iter()
            .filter(|d| matches!(d.outcome, DecisionOutcome::Success { .. }))
            .count();

        let success_rate = high_luck_success as f64 / high_luck.len() as f64;

        // Generate insight based on correlation strength
        if success_rate >= 0.8 {
            Some(Insight::new(
                format!(
                    "Strong luck correlation: {:.1}% of high-luck decisions succeed ({}/{})",
                    success_rate * 100.0,
                    high_luck_success,
                    high_luck.len()
                ),
                "prediction_accuracy".to_string(),
                0.8,
            ))
        } else if success_rate < 0.5 {
            Some(Insight::new(
                format!(
                    "Weak luck correlation: only {:.1}% of high-luck decisions succeed - consider recalibration",
                    success_rate * 100.0
                ),
                "prediction_accuracy".to_string(),
                0.9, // High importance - needs attention
            ))
        } else {
            Some(Insight::new(
                format!(
                    "Moderate luck correlation: {:.1}% success rate for high-luck decisions",
                    success_rate * 100.0
                ),
                "prediction_accuracy".to_string(),
                0.6,
            ))
        }
    }

    /// Analyze success patterns by intent type
    fn analyze_intent_patterns(&self, decisions: &[DominoDecisionTrace]) -> Option<Insight> {
        if decisions.is_empty() {
            return None;
        }

        let mut intent_stats: HashMap<String, (usize, usize)> = HashMap::new();

        for decision in decisions {
            let entry = intent_stats.entry(decision.intent_kind.clone()).or_insert((0, 0));
            entry.0 += 1; // Total

            match &decision.outcome {
                DecisionOutcome::Success { .. } => entry.1 += 1,
                _ => {}
            }
        }

        // Find best performing intent type
        let best_intent = intent_stats
            .iter()
            .filter(|(_, (total, _))| *total >= 3)
            .max_by(|(_, (total_a, success_a)), (_, (total_b, success_b))| {
                let rate_a = *success_a as f64 / *total_a as f64;
                let rate_b = *success_b as f64 / *total_b as f64;
                rate_a.partial_cmp(&rate_b).unwrap()
            });

        if let Some((intent, (total, successful))) = best_intent {
            let success_rate = *successful as f64 / *total as f64;

            if success_rate >= 0.7 {
                Some(Insight::new(
                    format!(
                        "Intent '{}' performs well: {:.1}% success rate ({}/{})",
                        intent,
                        success_rate * 100.0,
                        successful,
                        total
                    ),
                    "intent_performance".to_string(),
                    0.7,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Detect anomalies (high luck but failure, low luck but success)
    fn analyze_decision_anomalies(&self, decisions: &[DominoDecisionTrace]) -> Option<Insight> {
        // Find high-luck failures
        let high_luck_failures: Vec<&DominoDecisionTrace> = decisions
            .iter()
            .filter(|d| d.luck_score >= 0.85 && matches!(d.outcome, DecisionOutcome::Failure { .. }))
            .collect();

        if !high_luck_failures.is_empty() {
            let peers: Vec<String> = high_luck_failures
                .iter()
                .map(|d| d.chosen_peer.clone())
                .collect();
            let unique_peers: Vec<&String> = peers.iter().collect();

            return Some(Insight::new(
                format!(
                    "Anomaly detected: {} high-luck decisions failed - investigate peers: {:?}",
                    high_luck_failures.len(),
                    unique_peers
                ),
                "anomaly".to_string(),
                0.95, // Very high importance
            ));
        }

        // Find low-luck successes (lucky outcomes)
        let lucky_successes: Vec<&DominoDecisionTrace> = decisions
            .iter()
            .filter(|d| d.luck_score < 0.5 && matches!(d.outcome, DecisionOutcome::Success { .. }))
            .collect();

        if lucky_successes.len() >= 2 {
            Some(Insight::new(
                format!(
                    "Lucky outcomes: {} low-luck decisions succeeded - peer conditions improved",
                    lucky_successes.len()
                ),
                "anomaly".to_string(),
                0.6,
            ))
        } else {
            None
        }
    }
}

impl Default for ReflectionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflection_analyzer() {
        let analyzer = ReflectionAnalyzer::new();
        let mut state = ConsciousState::new();

        // Add traces with weight changes
        for i in 0..10 {
            let trace = CausalTrace::new(
                "node_alpha_fire".to_string(),
                "node_beta_weight_increase".to_string(),
                0.05 + (i as f64 * 0.01),
            );
            state.record_trace(trace);
        }

        let insights = analyzer.analyze(&state, 60000);
        assert!(!insights.is_empty());

        // Should have insight about high learning activity
        let learning_insights: Vec<&Insight> = insights
            .iter()
            .filter(|i| i.category == "learning")
            .collect();
        assert!(!learning_insights.is_empty());
    }

    #[test]
    fn test_stability_analysis() {
        let analyzer = ReflectionAnalyzer::new();
        let mut state = ConsciousState::new();

        // Add traces with stable changes
        for _ in 0..20 {
            let trace = CausalTrace::new(
                "event_a".to_string(),
                "event_b".to_string(),
                0.001, // Small changes
            );
            state.record_trace(trace);
        }

        let insights = analyzer.analyze(&state, 60000);

        // Should have insight about stability
        let stability_insights: Vec<&Insight> = insights
            .iter()
            .filter(|i| i.category == "stability")
            .collect();
        assert!(!stability_insights.is_empty());
    }

    #[test]
    fn test_extract_node_id() {
        let analyzer = ReflectionAnalyzer::new();

        assert_eq!(
            analyzer.extract_node_id("node_alpha_fire"),
            Some("node_alpha".to_string())
        );
        assert_eq!(
            analyzer.extract_node_id("node_beta_weight_increase"),
            Some("node_beta".to_string())
        );
        assert_eq!(analyzer.extract_node_id("invalid"), None);
    }

    // === Domino Decision Analysis Tests (v1.2 Week 2) ===

    #[test]
    fn test_routing_decision_analysis() {
        use crate::decision_tracker::DominoDecisionTrace;

        let analyzer = ReflectionAnalyzer::new();
        let mut state = ConsciousState::new();

        // Create decisions with various outcomes
        let decisions = vec![
            create_test_decision("decision_1", "peer_alpha", 0.9, DecisionOutcome::Success {
                actual_latency_ms: 45.0,
                actual_quality: 0.95,
            }),
            create_test_decision("decision_2", "peer_alpha", 0.85, DecisionOutcome::Success {
                actual_latency_ms: 50.0,
                actual_quality: 0.90,
            }),
            create_test_decision("decision_3", "peer_alpha", 0.88, DecisionOutcome::Success {
                actual_latency_ms: 48.0,
                actual_quality: 0.92,
            }),
            create_test_decision("decision_4", "peer_beta", 0.95, DecisionOutcome::Failure {
                reason: "timeout".to_string(),
            }),
        ];

        for decision in decisions {
            state.record_decision(decision);
        }

        let insights = analyzer.analyze_routing_decisions(&state);
        assert!(!insights.is_empty(), "Should generate insights from decisions");

        // Should have routing performance insight
        let routing_insights: Vec<&Insight> = insights
            .iter()
            .filter(|i| i.category == "routing_performance")
            .collect();
        assert!(!routing_insights.is_empty(), "Should have routing performance insight");
    }

    #[test]
    fn test_peer_success_rate_analysis() {
        use crate::decision_tracker::DominoDecisionTrace;

        let analyzer = ReflectionAnalyzer::new();

        let decisions = vec![
            create_test_decision("d1", "peer_alpha", 0.9, DecisionOutcome::Success {
                actual_latency_ms: 45.0,
                actual_quality: 0.95,
            }),
            create_test_decision("d2", "peer_alpha", 0.85, DecisionOutcome::Success {
                actual_latency_ms: 50.0,
                actual_quality: 0.90,
            }),
            create_test_decision("d3", "peer_alpha", 0.88, DecisionOutcome::Success {
                actual_latency_ms: 48.0,
                actual_quality: 0.92,
            }),
            create_test_decision("d4", "peer_beta", 0.75, DecisionOutcome::Failure {
                reason: "timeout".to_string(),
            }),
        ];

        let insight = analyzer.analyze_peer_success_rates(&decisions);
        assert!(insight.is_some(), "Should generate peer success insight");

        let insight = insight.unwrap();
        assert!(insight.insight.contains("peer_alpha"), "Should mention best peer");
        assert!(insight.insight.contains("100"), "Should show 100% success rate");
    }

    #[test]
    fn test_luck_correlation_analysis() {
        use crate::decision_tracker::DominoDecisionTrace;

        let analyzer = ReflectionAnalyzer::new();

        // High luck, high success - strong correlation
        let decisions = vec![
            create_test_decision("d1", "peer_a", 0.9, DecisionOutcome::Success {
                actual_latency_ms: 45.0,
                actual_quality: 0.95,
            }),
            create_test_decision("d2", "peer_b", 0.85, DecisionOutcome::Success {
                actual_latency_ms: 50.0,
                actual_quality: 0.90,
            }),
            create_test_decision("d3", "peer_c", 0.92, DecisionOutcome::Success {
                actual_latency_ms: 48.0,
                actual_quality: 0.92,
            }),
            create_test_decision("d4", "peer_d", 0.88, DecisionOutcome::Success {
                actual_latency_ms: 47.0,
                actual_quality: 0.93,
            }),
            create_test_decision("d5", "peer_e", 0.91, DecisionOutcome::Success {
                actual_latency_ms: 46.0,
                actual_quality: 0.94,
            }),
        ];

        let insight = analyzer.analyze_luck_correlation(&decisions);
        assert!(insight.is_some(), "Should generate correlation insight");

        let insight = insight.unwrap();
        assert!(
            insight.insight.contains("Strong luck correlation"),
            "Should detect strong correlation"
        );
        assert_eq!(insight.category, "prediction_accuracy");
    }

    #[test]
    fn test_weak_luck_correlation() {
        use crate::decision_tracker::DominoDecisionTrace;

        let analyzer = ReflectionAnalyzer::new();

        // High luck but failures - weak correlation
        let decisions = vec![
            create_test_decision("d1", "peer_a", 0.9, DecisionOutcome::Failure {
                reason: "timeout".to_string(),
            }),
            create_test_decision("d2", "peer_b", 0.85, DecisionOutcome::Failure {
                reason: "unavailable".to_string(),
            }),
            create_test_decision("d3", "peer_c", 0.92, DecisionOutcome::Failure {
                reason: "error".to_string(),
            }),
            create_test_decision("d4", "peer_d", 0.88, DecisionOutcome::Success {
                actual_latency_ms: 47.0,
                actual_quality: 0.93,
            }),
            create_test_decision("d5", "peer_e", 0.91, DecisionOutcome::Failure {
                reason: "slow".to_string(),
            }),
        ];

        let insight = analyzer.analyze_luck_correlation(&decisions);
        assert!(insight.is_some(), "Should generate weak correlation insight");

        let insight = insight.unwrap();
        assert!(
            insight.insight.contains("Weak luck correlation"),
            "Should detect weak correlation"
        );
        assert!(insight.importance >= 0.8, "Should have high importance");
    }

    #[test]
    fn test_anomaly_detection() {
        use crate::decision_tracker::DominoDecisionTrace;

        let analyzer = ReflectionAnalyzer::new();

        // High luck but failure - anomaly
        let decisions = vec![
            create_test_decision("d1", "peer_alpha", 0.95, DecisionOutcome::Failure {
                reason: "unexpected error".to_string(),
            }),
            create_test_decision("d2", "peer_beta", 0.90, DecisionOutcome::Success {
                actual_latency_ms: 45.0,
                actual_quality: 0.95,
            }),
        ];

        let insight = analyzer.analyze_decision_anomalies(&decisions);
        assert!(insight.is_some(), "Should detect anomaly");

        let insight = insight.unwrap();
        assert!(insight.insight.contains("Anomaly detected"), "Should mention anomaly");
        assert!(insight.insight.contains("peer_alpha"), "Should identify problematic peer");
        assert_eq!(insight.category, "anomaly");
        assert!(insight.importance >= 0.9, "Should have very high importance");
    }

    #[test]
    fn test_lucky_outcomes() {
        use crate::decision_tracker::DominoDecisionTrace;

        let analyzer = ReflectionAnalyzer::new();

        // Low luck but success - lucky outcomes
        let decisions = vec![
            create_test_decision("d1", "peer_a", 0.4, DecisionOutcome::Success {
                actual_latency_ms: 50.0,
                actual_quality: 0.85,
            }),
            create_test_decision("d2", "peer_b", 0.3, DecisionOutcome::Success {
                actual_latency_ms: 55.0,
                actual_quality: 0.80,
            }),
        ];

        let insight = analyzer.analyze_decision_anomalies(&decisions);
        assert!(insight.is_some(), "Should detect lucky outcomes");

        let insight = insight.unwrap();
        assert!(insight.insight.contains("Lucky outcomes"), "Should mention lucky outcomes");
    }

    #[test]
    fn test_intent_pattern_analysis() {
        use crate::decision_tracker::DominoDecisionTrace;

        let analyzer = ReflectionAnalyzer::new();

        let mut decisions = vec![];

        // Routing intent - 100% success
        for i in 0..3 {
            let mut decision = create_test_decision(
                &format!("routing_{}", i),
                "peer_alpha",
                0.9,
                DecisionOutcome::Success {
                    actual_latency_ms: 45.0,
                    actual_quality: 0.95,
                },
            );
            decision.intent_kind = "routing".to_string();
            decisions.push(decision);
        }

        // Task scheduling - 50% success
        for i in 0..4 {
            let outcome = if i % 2 == 0 {
                DecisionOutcome::Success {
                    actual_latency_ms: 50.0,
                    actual_quality: 0.90,
                }
            } else {
                DecisionOutcome::Failure {
                    reason: "timeout".to_string(),
                }
            };

            let mut decision = create_test_decision(&format!("task_{}", i), "peer_beta", 0.8, outcome);
            decision.intent_kind = "task_scheduling".to_string();
            decisions.push(decision);
        }

        let insight = analyzer.analyze_intent_patterns(&decisions);
        assert!(insight.is_some(), "Should generate intent pattern insight");

        let insight = insight.unwrap();
        assert!(
            insight.insight.contains("routing"),
            "Should identify best intent type"
        );
        assert_eq!(insight.category, "intent_performance");
    }

    // Helper function to create test decisions
    fn create_test_decision(
        id: &str,
        peer: &str,
        luck: f32,
        outcome: DecisionOutcome,
    ) -> DominoDecisionTrace {
        use crate::decision_tracker::DominoDecisionTrace;

        DominoDecisionTrace {
            decision_id: id.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            intent_kind: "routing".to_string(),
            context_tags: vec!["test".to_string()],
            candidates: vec![peer.to_string()],
            chosen_peer: peer.to_string(),
            luck_score: luck,
            resistance_score: 1.0 - luck,
            explanation: "Test decision".to_string(),
            outcome,
            node_id: "test_node".to_string(),
        }
    }
}
