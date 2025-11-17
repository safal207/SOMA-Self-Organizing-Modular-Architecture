//! WebSocket обработчики

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use serde_json::json;

use crate::{AppState, errors::lock_arc_mutex, config};

/// WebSocket обработчик
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_task(socket, state))
}

/// Задача WebSocket - отправка сигналов клиенту
async fn websocket_task(mut socket: WebSocket, state: AppState) {
    let mut rx = state.signal_tx.subscribe();

    // Отправляем приветственное сообщение
    let welcome = json!({
        "type": "connected",
        "message": "Connected to SOMA signal stream"
    });

    if socket
        .send(Message::Text(serde_json::to_string(&welcome).unwrap_or_default()))
        .await
        .is_err()
    {
        return;
    }

    // Периодически отправляем состояние системы
    let mut interval = tokio::time::interval(
        tokio::time::Duration::from_secs(config::api::WEBSOCKET_STATE_INTERVAL_SEC)
    );

    loop {
        tokio::select! {
            // Получен сигнал от broadcast канала
            Ok(signal) = rx.recv() => {
                let msg = json!({
                    "type": "signal",
                    "data": signal
                });

                if socket
                    .send(Message::Text(serde_json::to_string(&msg).unwrap_or_default()))
                    .await
                    .is_err()
                {
                    break;
                }
            }

            // Периодическое обновление состояния
            _ = interval.tick() => {
                let state_msg = {
                    if let Ok(stem) = lock_arc_mutex(&state.stem) {
                        json!({
                            "type": "state",
                            "data": {
                                "cells": stem.cell_count(),
                                "generation": stem.generation,
                                "load": stem.load,
                            }
                        })
                    } else {
                        continue;
                    }
                };

                if socket
                    .send(Message::Text(serde_json::to_string(&state_msg).unwrap_or_default()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    }
}

/// Mesh WebSocket обработчик для peer-to-peer соединений
pub async fn mesh_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| mesh_connection_task(socket, state))
}

/// Задача обработки mesh соединения
async fn mesh_connection_task(socket: WebSocket, state: AppState) {
    state.mesh.handle_peer_connection(socket).await;
}

