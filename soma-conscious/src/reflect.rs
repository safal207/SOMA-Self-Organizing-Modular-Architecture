//! # Reflection Module - Pattern Analysis and Insight Generation
//!
//! Analyzes causal chains and generates insights about system state.

use crate::{CausalTrace, ConsciousState, Insight};
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
}
