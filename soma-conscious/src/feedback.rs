//! # Feedback Module - Adaptive Intervention
//!
//! A>7=0==>5 2<5H0B5;LAB2> A8AB5<K =0 >A=>25 0=0;870 ?0BB5@=>2.
//! "-<>F8>=0;L=K5 @5D;5:AK" A5B8 4;O A0<>:>@@5:F88.

use crate::{ConsciousState, Insight};
use serde::{Deserialize, Serialize};

///  5:><5=40F8O ?> :>@@5:B8@>2:5 ?0@0<5B@>2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackAction {
    /// "8? 459AB28O
    pub action_type: FeedbackActionType,

    /// &5;52>9 ?0@0<5B@
    pub target: String,

    ///  5:><5=4>20==>5 7=0G5=85 8;8 45;LB0
    pub value: f64,

    /// 1>A=>20=85
    pub reason: String,
}

/// "8?K 459AB289 >1@0B=>9 A2O78
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedbackActionType {
    /// >4AB@>9:0 A:>@>AB8 >1CG5=8O
    AdjustLearningRate,

    /// 2545=85 <8:@>?0C7K
    IntroducePause,

    /// #A8;5=85 :>@@5:F88
    IncreaseCorrection,

    /// A;01;5=85 :>@@5:F88
    DecreaseCorrection,

    /// !1@>A 25A>2
    ResetWeights,
}

/// >=B@>;;5@ >1@0B=>9 A2O78
pub struct FeedbackController {
    /// >@>3 =5AB018;L=>AB8 4;O 2<5H0B5;LAB20
    instability_threshold: f64,

    /// >@>3 =87:>9 0:B82=>AB8
    low_activity_threshold: f64,

    /// :;NGQ= ;8 02B><0B8G5A:89 feedback
    auto_feedback_enabled: bool,
}

impl FeedbackController {
    pub fn new() -> Self {
        Self {
            instability_threshold: 0.7,
            low_activity_threshold: 0.2,
            auto_feedback_enabled: true,
        }
    }

    /// :;NG8BL/2K:;NG8BL 02B><0B8G5A:89 feedback
    pub fn set_auto_feedback(&mut self, enabled: bool) {
        self.auto_feedback_enabled = enabled;
    }

    /// @>0=0;878@>20BL 8=A09BK 8 A35=5@8@>20BL 459AB28O
    pub fn generate_actions(&self, insights: &[Insight]) -> Vec<FeedbackAction> {
        if !self.auto_feedback_enabled {
            return Vec::new();
        }

        let mut actions = Vec::new();

        for insight in insights {
            //  5038@>20BL =0 =5AB018;L=>ABL
            if insight.category == "stability"
                && insight.insight.contains("instability")
                && insight.importance >= self.instability_threshold
            {
                actions.push(FeedbackAction {
                    action_type: FeedbackActionType::DecreaseCorrection,
                    target: "eta_pos".to_string(),
                    value: 0.03, // !=878BL A 0.06 4> 0.03
                    reason: format!("High instability detected: {}", insight.insight),
                });

                actions.push(FeedbackAction {
                    action_type: FeedbackActionType::IntroducePause,
                    target: "learning_cycle".to_string(),
                    value: 1000.0, // 1 A5:C=40 ?0C7K
                    reason: "Pause learning to allow network stabilization".to_string(),
                });
            }

            //  5038@>20BL =0 =87:CN 0:B82=>ABL
            if insight.category == "learning"
                && insight.insight.contains("Low learning activity")
            {
                actions.push(FeedbackAction {
                    action_type: FeedbackActionType::AdjustLearningRate,
                    target: "eta_pos".to_string(),
                    value: 0.09, // #25;8G8BL A 0.06 4> 0.09
                    reason: "Low learning activity: increase learning rate".to_string(),
                });
            }

            //  5038@>20BL =0 2KA>:CN 0:B82=>ABL
            if insight.category == "learning"
                && insight.insight.contains("High learning activity")
                && insight.importance > 0.8
            {
                // AQ E>@>H>, <>6=> =5<=>3> CA8;8BL
                actions.push(FeedbackAction {
                    action_type: FeedbackActionType::IncreaseCorrection,
                    target: "resonance_strength".to_string(),
                    value: 0.25, // #25;8G8BL adaptive strength
                    reason: "High learning activity: network is responsive".to_string(),
                });
            }

            //  5038@>20BL =0 @02=>25A85
            if insight.category == "stability"
                && insight.insight.contains("equilibrium")
            {
                // !5BL AB018;L=0, <>6=> A;53:0 C25;8G8BL 8AA;54>20=85
                actions.push(FeedbackAction {
                    action_type: FeedbackActionType::AdjustLearningRate,
                    target: "eta_pos".to_string(),
                    value: 0.07,
                    reason: "Network equilibrium: slight exploration boost".to_string(),
                });
            }
        }

        actions
    }

    /// @8<5=8BL 459AB28O : A>AB>O=8N (A8<C;OF8O)
    pub fn apply_actions(
        &self,
        _state: &mut ConsciousState,
        actions: &[FeedbackAction],
    ) -> usize {
        //  @50;L=>9 @50;870F88 745AL 1C45B >1=>2;5=85 ?0@0<5B@>2 mesh
        // >:0 ?@>AB> 2>72@0I05< :>;8G5AB2> ?@8<5=5==KE 459AB289
        actions.len()
    }
}

impl Default for FeedbackController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Insight;

    #[test]
    fn test_feedback_controller() {
        let controller = FeedbackController::new();

        let insights = vec![
            Insight::new(
                "Network instability detected: 45.0% of changes are significant".to_string(),
                "stability".to_string(),
                0.9,
            ),
            Insight::new(
                "High learning activity detected: 60.0% of events involve weight changes"
                    .to_string(),
                "learning".to_string(),
                0.8,
            ),
        ];

        let actions = controller.generate_actions(&insights);

        // >;6=K 1KBL A35=5@8@>20=K 459AB28O
        assert!(!actions.is_empty());

        // >;6=> 1KBL 459AB285 =0 A=865=85 :>@@5:F88
        let decrease_actions: Vec<&FeedbackAction> = actions
            .iter()
            .filter(|a| matches!(a.action_type, FeedbackActionType::DecreaseCorrection))
            .collect();
        assert!(!decrease_actions.is_empty());

        // >;6=0 1KBL @5:><5=40F8O ?0C7K
        let pause_actions: Vec<&FeedbackAction> = actions
            .iter()
            .filter(|a| matches!(a.action_type, FeedbackActionType::IntroducePause))
            .collect();
        assert!(!pause_actions.is_empty());
    }

    #[test]
    fn test_low_activity_response() {
        let controller = FeedbackController::new();

        let insights = vec![Insight::new(
            "Low learning activity: network weights are stable".to_string(),
            "learning".to_string(),
            0.6,
        )];

        let actions = controller.generate_actions(&insights);

        // >;6=0 1KBL @5:><5=40F8O C25;8G8BL learning rate
        let adjust_actions: Vec<&FeedbackAction> = actions
            .iter()
            .filter(|a| matches!(a.action_type, FeedbackActionType::AdjustLearningRate))
            .collect();
        assert!(!adjust_actions.is_empty());
    }

    #[test]
    fn test_auto_feedback_disabled() {
        let mut controller = FeedbackController::new();
        controller.set_auto_feedback(false);

        let insights = vec![Insight::new(
            "Network instability detected".to_string(),
            "stability".to_string(),
            0.9,
        )];

        let actions = controller.generate_actions(&insights);

        // 5 4>;6=> 1KBL 459AB289 ?@8 >B:;NGQ==>< auto_feedback
        assert!(actions.is_empty());
    }
}
