//! Обработчики для Conscious Layer

use axum::extract::State;
use axum::Json;

use crate::{AppState, errors::ApiError, errors::lock_arc_mutex, config};
use soma_conscious::{ReflectionAnalyzer, CausalTrace};

/// GET /conscious/state - Текущее состояние осознанности
pub async fn get_conscious_state(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let conscious = lock_arc_mutex(&state.conscious)?;
    let attention_map = conscious.get_attention_map();

    Ok(Json(serde_json::json!({
        "node_id": state.mesh.id,
        "cycle_count": conscious.cycle_count,
        "last_cycle_ms": conscious.last_cycle,
        "traces_count": conscious.traces_count(),
        "insights_count": conscious.insights_count(),
        "attention_map": {
            "top_nodes": attention_map.top_nodes,
            "updated_at": attention_map.updated_at
        }
    })))
}

/// GET /conscious/traces - Получить последние причинные цепи
pub async fn get_conscious_traces(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let conscious = lock_arc_mutex(&state.conscious)?;
    let traces = conscious.get_traces(config::api::DEFAULT_TRACES_LIMIT);

    Ok(Json(serde_json::json!({
        "node_id": state.mesh.id,
        "traces": traces,
        "count": traces.len()
    })))
}

/// GET /conscious/insights - Получить сгенерированные инсайты
pub async fn get_conscious_insights(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let conscious = lock_arc_mutex(&state.conscious)?;
    let insights = conscious.get_insights(config::api::DEFAULT_INSIGHTS_LIMIT);

    Ok(Json(serde_json::json!({
        "node_id": state.mesh.id,
        "insights": insights,
        "count": insights.len()
    })))
}

/// POST /conscious/reflect - Триггер немедленной рефлексии
pub async fn trigger_reflection(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let mut conscious = lock_arc_mutex(&state.conscious)?;

    // Запуск анализа
    let analyzer = ReflectionAnalyzer::new();
    let insights = analyzer.analyze(&conscious, config::api::REFLECTION_ANALYSIS_WINDOW_MS);

    // Добавить инсайты
    for insight in &insights {
        conscious.add_insight(insight.clone());
    }

    Ok(Json(serde_json::json!({
        "status": "ok",
        "node_id": state.mesh.id,
        "insights_generated": insights.len(),
        "insights": insights
    })))
}

/// GET /conscious/health - Метрики осознанности
pub async fn get_conscious_health(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let conscious = lock_arc_mutex(&state.conscious)?;

    // Вычисляем метрики здоровья
    let traces_rate = if conscious.cycle_count > 0 {
        conscious.traces_count() as f64 / conscious.cycle_count as f64
    } else {
        0.0
    };

    let insights_rate = if conscious.cycle_count > 0 {
        conscious.insights_count() as f64 / conscious.cycle_count as f64
    } else {
        0.0
    };

    Ok(Json(serde_json::json!({
        "node_id": state.mesh.id,
        "cycle_count": conscious.cycle_count,
        "traces_per_cycle": traces_rate,
        "insights_per_cycle": insights_rate,
        "health_status": if traces_rate > 0.5 { "active" } else { "quiet" }
    })))
}

