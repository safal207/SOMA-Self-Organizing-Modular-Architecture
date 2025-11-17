//! Обработчики для Domino Luck Engine

use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use crate::{AppState, errors::ApiError, errors::lock_arc_mutex, config};
use soma_domino::{DominoEngine, DominoInput, DominoIntentKind, PeerCandidate};
use soma_conscious::DominoDecisionTrace;

/// Запрос оценки Domino Luck Engine
#[derive(Debug, Deserialize)]
pub struct DominoEvaluateRequest {
    /// Тип намерения
    pub intent_kind: String,

    /// Список кандидатов
    pub candidates: Vec<PeerCandidateDto>,

    /// Опциональные контекстные теги
    #[serde(default)]
    pub context_tags: Vec<String>,
}

/// DTO для PeerCandidate
#[derive(Debug, Deserialize)]
pub struct PeerCandidateDto {
    pub peer_id: String,
    pub health: f32,
    pub quality: f32,
    pub intent_match: f32,
}

/// Ответ Domino Luck Engine
#[derive(Debug, serde::Serialize)]
pub struct DominoEvaluateResponse {
    /// Уникальный ID решения (для последующего обновления outcome)
    pub decision_id: String,

    /// Отсортированный список лучших пиров
    pub best_peers: Vec<String>,

    /// Общая оценка удачи (0.0 - 1.0)
    pub luck_score: f32,

    /// Общая оценка сопротивления (0.0 - 1.0)
    pub resistance_score: f32,

    /// Человекочитаемое объяснение
    pub explanation: String,
}

/// POST /domino/evaluate - Оценка "удачи" для выбора лучших пиров
pub async fn domino_evaluate(
    State(state): State<AppState>,
    Json(req): Json<DominoEvaluateRequest>,
) -> Json<DominoEvaluateResponse> {
    // Генерируем уникальный ID решения
    let timestamp = chrono::Utc::now().timestamp_millis();
    let decision_id = format!(
        "domino_{}_{}",
        state.mesh.id,
        timestamp
    );

    // Парсим intent_kind из строки
    let intent_kind = match req.intent_kind.to_lowercase().as_str() {
        "routing" => DominoIntentKind::Routing,
        "task_scheduling" => DominoIntentKind::TaskScheduling,
        "user_request" => DominoIntentKind::UserRequest,
        custom => DominoIntentKind::Custom(custom.to_string()),
    };

    // Конвертируем DTOs в PeerCandidate
    let candidates: Vec<PeerCandidate> = req
        .candidates
        .iter()
        .map(|dto| PeerCandidate {
            peer_id: dto.peer_id.clone(),
            health: dto.health,
            quality: dto.quality,
            intent_match: dto.intent_match,
        })
        .collect();

    // Создаём DominoInput
    let input = DominoInput::new(intent_kind.clone(), candidates.clone(), req.context_tags.clone());

    // Выполняем оценку
    let decision = DominoEngine::evaluate(input);

    // Создаём trace для Conscious Layer
    let trace = DominoDecisionTrace::new(
        decision_id.clone(),
        chrono::Utc::now().timestamp_millis() as u64,
        format!("{:?}", intent_kind),
        req.context_tags,
        req.candidates.iter().map(|c| c.peer_id.clone()).collect(),
        decision.best_peers.first().cloned().unwrap_or_default(),
        decision.luck_score,
        decision.resistance_score,
        decision.explanation.clone(),
        state.mesh.id.clone(),
    );

    // Записываем решение в Conscious State
    if let Ok(mut conscious) = state.conscious.lock() {
        conscious.record_decision(trace);
    }

    // Конвертируем в DTO ответа
    Json(DominoEvaluateResponse {
        decision_id,
        best_peers: decision.best_peers,
        luck_score: decision.luck_score,
        resistance_score: decision.resistance_score,
        explanation: decision.explanation,
    })
}

/// GET /domino/decisions - Получить все решения
pub async fn get_domino_decisions(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let conscious = lock_arc_mutex(&state.conscious)?;
    let decisions = conscious.get_decisions();

    Ok(Json(serde_json::json!({
        "node_id": state.mesh.id,
        "total_decisions": decisions.len(),
        "decisions": decisions
    })))
}

/// GET /domino/decisions/recent?limit=N - Получить последние N решений
pub async fn get_recent_domino_decisions(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let conscious = lock_arc_mutex(&state.conscious)?;
    let recent = conscious.get_recent_decisions(config::api::DEFAULT_DECISIONS_LIMIT);

    Ok(Json(serde_json::json!({
        "node_id": state.mesh.id,
        "count": recent.len(),
        "decisions": recent
    })))
}

/// GET /domino/decisions/stats - Статистика решений
pub async fn get_domino_stats(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let conscious = lock_arc_mutex(&state.conscious)?;
    let stats = conscious.get_decision_stats();

    Ok(Json(serde_json::json!({
        "node_id": state.mesh.id,
        "stats": stats
    })))
}

/// Request для обновления outcome решения
#[derive(Deserialize)]
pub struct UpdateOutcomeRequest {
    pub decision_id: String,
    pub outcome_type: String, // "success", "failure", "partial"
    #[serde(default)]
    pub actual_latency_ms: Option<f64>,
    #[serde(default)]
    pub actual_quality: Option<f64>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub completed_ratio: Option<f64>,
    #[serde(default)]
    pub issues: Vec<String>,
}

/// POST /domino/decisions/outcome - Обновить результат решения
pub async fn update_decision_outcome(
    State(state): State<AppState>,
    Json(req): Json<UpdateOutcomeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use soma_conscious::DecisionOutcome;
    
    let outcome = match req.outcome_type.as_str() {
        "success" => DecisionOutcome::Success {
            actual_latency_ms: req.actual_latency_ms.unwrap_or(0.0),
            actual_quality: req.actual_quality.unwrap_or(1.0),
        },
        "failure" => DecisionOutcome::Failure {
            reason: req.reason.unwrap_or_else(|| "unknown".to_string()),
        },
        "partial" => DecisionOutcome::Partial {
            completed_ratio: req.completed_ratio.unwrap_or(0.5),
            issues: req.issues,
        },
        _ => {
            return Err(ApiError::BadRequest(
                "Invalid outcome_type. Use: success, failure, or partial".to_string()
            ));
        }
    };

    let mut conscious = lock_arc_mutex(&state.conscious)?;
    let updated = conscious.update_decision_outcome(&req.decision_id, outcome);

    if updated {
        Ok(Json(serde_json::json!({
            "status": "ok",
            "decision_id": req.decision_id,
            "message": "Decision outcome updated"
        })))
    } else {
        Err(ApiError::NotFound(format!("Decision ID {} not found", req.decision_id)))
    }
}

/// GET /domino/insights - Dashboard with routing insights and analysis
pub async fn get_domino_insights(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    use soma_conscious::ReflectionAnalyzer;
    
    let conscious = lock_arc_mutex(&state.conscious)?;

    // Create analyzer and generate insights
    let analyzer = ReflectionAnalyzer::new();
    let insights = analyzer.analyze_routing_decisions(&conscious);

    // Get basic stats for context
    let stats = conscious.get_decision_stats();
    let decisions_count = conscious.decisions_count();

    Ok(Json(serde_json::json!({
        "node_id": state.mesh.id,
        "timestamp": chrono::Utc::now().timestamp_millis(),
        "total_decisions": decisions_count,
        "stats": stats,
        "insights": insights,
        "insights_count": insights.len(),
        "categories": {
            "routing_performance": insights.iter().filter(|i| i.category == "routing_performance").count(),
            "prediction_accuracy": insights.iter().filter(|i| i.category == "prediction_accuracy").count(),
            "intent_performance": insights.iter().filter(|i| i.category == "intent_performance").count(),
            "anomaly": insights.iter().filter(|i| i.category == "anomaly").count(),
        }
    })))
}

