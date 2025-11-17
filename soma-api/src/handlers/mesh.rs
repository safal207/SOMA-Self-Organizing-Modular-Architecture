//! Обработчики для Mesh сети

use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use crate::{AppState, errors::ApiError, errors::lock_arc_mutex, config};

/// Получить список подключенных peers
pub async fn get_peers(State(state): State<AppState>) -> Json<serde_json::Value> {
    let alive_peers = state.mesh.get_alive_peers(config::timeouts::PEER_ALIVE_TIMEOUT_MS);

    let peers_json: Vec<serde_json::Value> = alive_peers
        .iter()
        .map(|peer| {
            serde_json::json!({
                "id": peer.id,
                "last_seen_ms": peer.last_seen,
                "cells": peer.cells,
                "generation": peer.generation,
                "load": peer.load,
                "alive": peer.is_alive(config::timeouts::PEER_ALIVE_TIMEOUT_MS),
                "health": {
                    "quality": peer.health.quality,
                    "failures": peer.health.failures,
                    "successes": peer.health.successes,
                    "failure_rate": peer.health.failure_rate(),
                    "is_healthy": peer.health.is_healthy()
                }
            })
        })
        .collect();

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "peer_count": state.mesh.get_peer_count(),
        "peers": peers_json
    }))
}

/// Зарегистрировать peer для автоматического переподключения
#[derive(Deserialize)]
pub struct RegisterPeerRequest {
    pub peer_id: String,
    pub url: String,
}

pub async fn register_peer(
    State(state): State<AppState>,
    Json(req): Json<RegisterPeerRequest>,
) -> Json<serde_json::Value> {
    state.mesh.register_peer(req.peer_id.clone(), req.url.clone());

    // Попытаться подключиться сразу
    let mesh = state.mesh.clone();
    tokio::spawn(async move {
        mesh.attempt_connect_to_peer(req.peer_id, req.url).await;
    });

    Json(serde_json::json!({
        "status": "ok",
        "message": "Peer registered and connection initiated"
    }))
}

/// Получить статистику резонанса сети
pub async fn get_resonance(State(state): State<AppState>) -> Result<Json<serde_json::Value>, ApiError> {
    let current_load = {
        let stem = lock_arc_mutex(&state.stem)?;
        stem.load
    };

    let stats = state.mesh.get_resonance_stats(current_load);
    let adaptive_strength = state.mesh.compute_adaptive_strength();

    Ok(Json(serde_json::json!({
        "node_id": state.mesh.id,
        "current_load": current_load,
        "resonance": stats.resonance,
        "adaptive_strength": adaptive_strength,
        "peer_count": stats.peer_count,
        "network": {
            "avg_load": stats.avg_load,
            "min_load": stats.min_load,
            "max_load": stats.max_load,
            "variance": stats.variance
        }
    })))
}

/// GET /mesh/links - Получить все веса связей с метриками
pub async fn get_links(State(state): State<AppState>) -> Json<serde_json::Value> {
    let links = state.mesh.get_link_weights();

    let links_json: Vec<serde_json::Value> = links
        .into_iter()
        .map(|(peer_id, weight, quality)| {
            serde_json::json!({
                "peer_id": peer_id,
                "weight": weight,
                "health_quality": quality,
                "score": weight * quality
            })
        })
        .collect();

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "links": links_json,
        "count": links_json.len()
    }))
}

#[derive(Deserialize)]
pub struct TuneLinkRequest {
    pub peer_id: String,
    pub weight: f64,
}

/// POST /mesh/links/tune - Ручная подстройка веса связи
pub async fn tune_link(
    State(state): State<AppState>,
    Json(req): Json<TuneLinkRequest>,
) -> Json<serde_json::Value> {
    state.mesh.set_link_weight(&req.peer_id, req.weight);

    Json(serde_json::json!({
        "status": "ok",
        "peer_id": req.peer_id,
        "new_weight": req.weight,
        "message": "Link weight updated"
    }))
}

/// GET /mesh/topology - Получить топ-N самых сильных связей
pub async fn get_topology(State(state): State<AppState>) -> Json<serde_json::Value> {
    let top_links = state.mesh.get_top_links(config::api::DEFAULT_TOP_LINKS_COUNT);

    let topology: Vec<serde_json::Value> = top_links
        .into_iter()
        .map(|(peer_id, weight, quality)| {
            serde_json::json!({
                "peer_id": peer_id,
                "weight": weight,
                "health_quality": quality,
                "score": weight * quality
            })
        })
        .collect();

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "top_links": topology,
        "count": topology.len()
    }))
}

/// POST /mesh/fire - Триггер Fire события
pub async fn fire_event(State(state): State<AppState>) -> Json<serde_json::Value> {
    state.mesh.send_fire();

    Json(serde_json::json!({
        "status": "ok",
        "node_id": state.mesh.id,
        "message": "Fire event sent to all peers"
    }))
}

