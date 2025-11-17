//! Обработчики системных эндпоинтов

use axum::{extract::State, Json};
use serde_json::json;

use crate::{AppState, responses::SuccessResponse};

/// Корневой эндпоинт - информация об API
pub async fn root(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conscious_state = match state.conscious.lock() {
        Ok(s) => s,
        Err(_) => {
            return Json(json!({"error": "Failed to lock conscious state"}));
        }
    };
    
    Json(json!({
        "name": "SOMA Conscious Layer",
        "version": "1.0.0",
        "description": "Self-Organizing Modular Architecture - Conscious Self-Aware Network",
        "node_id": state.mesh.id,
        "peer_count": state.mesh.get_peer_count(),
        "consciousness": {
            "cycle_count": conscious_state.cycle_count,
            "traces_count": conscious_state.traces_count(),
            "insights_count": conscious_state.insights_count(),
        },
        "endpoints": {
            "/": "API information",
            "/state": "GET - System state",
            "/cells": "GET - List all cells",
            "/distribution": "GET - Role distribution",
            "/peers": "GET - Connected peers with health metrics",
            "/peers/register": "POST - Register peer for auto-reconnect {peer_id, url}",
            "/resonance": "GET - Network resonance stats with adaptive strength",
            "/signal": "POST - Send signal {id, value}",
            "/stimulate": "POST - Stimulate system {activity}",
            "/ws": "GET - WebSocket real-time stream",
            "/mesh": "GET - Mesh peer connection (WebSocket)"
        }
    }))
}

/// Получить текущее состояние системы
pub async fn get_state(State(state): State<AppState>) -> Result<Json<crate::StateResponse>, crate::errors::ApiError> {
    use crate::errors::lock_arc_mutex;
    
    let stem = lock_arc_mutex(&state.stem)?;
    Ok(Json(crate::StateResponse {
        cells: stem.cell_count(),
        generation: stem.generation,
        load: stem.load,
        threshold: stem.threshold,
    }))
}

/// Стимулировать систему (вызывает деление при высокой активности)
#[derive(serde::Deserialize)]
pub struct StimulateRequest {
    pub activity: f64,
}

pub async fn stimulate(
    State(state): State<AppState>,
    Json(req): Json<StimulateRequest>,
) -> Result<Json<serde_json::Value>, crate::errors::ApiError> {
    use crate::errors::lock_arc_mutex;
    
    let mut stem = lock_arc_mutex(&state.stem)?;
    stem.sense(req.activity);

    Ok(Json(json!({
        "status": "ok",
        "load": stem.load,
        "cells": stem.cell_count(),
        "generation": stem.generation
    })))
}

/// Отправить сигнал в систему
pub async fn post_signal(
    State(state): State<AppState>,
    Json(signal): Json<crate::ApiSignal>,
) -> Json<SuccessResponse> {
    // Отправляем сигнал в broadcast канал
    let _ = state.signal_tx.send(signal);

    Json(SuccessResponse::ok("Signal sent"))
}

