use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use soma_bridge::Signal as BridgeSignal;
use soma_core::{CellRole, StemProcessor};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

/// Состояние приложения, разделяемое между обработчиками
#[derive(Clone)]
struct AppState {
    stem: Arc<Mutex<StemProcessor>>,
    signal_tx: broadcast::Sender<ApiSignal>,
}

/// API-представление сигнала
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiSignal {
    id: String,
    value: f64,
    timestamp: u64,
}

impl From<BridgeSignal> for ApiSignal {
    fn from(sig: BridgeSignal) -> Self {
        Self {
            id: sig.id,
            value: sig.value,
            timestamp: sig.timestamp as u64,
        }
    }
}

/// Ответ с состоянием системы
#[derive(Serialize)]
struct StateResponse {
    cells: usize,
    generation: u32,
    load: f64,
    threshold: f64,
}

/// Информация о клетке для API
#[derive(Serialize)]
struct CellResponse {
    id: String,
    role: String,
    generation: u32,
    age_ms: u64,
    activity: f64,
}

/// Распределение ролей
#[derive(Serialize)]
struct DistributionResponse {
    sensor: usize,
    logic: usize,
    motor: usize,
    total: usize,
}

#[tokio::main]
async fn main() {
    // Инициализация состояния
    let stem = Arc::new(Mutex::new(StemProcessor::new()));
    let (signal_tx, _) = broadcast::channel::<ApiSignal>(100);

    let state = AppState {
        stem: stem.clone(),
        signal_tx: signal_tx.clone(),
    };

    // Построение роутера
    let app = Router::new()
        .route("/", get(root))
        .route("/state", get(get_state))
        .route("/cells", get(get_cells))
        .route("/distribution", get(get_distribution))
        .route("/signal", post(post_signal))
        .route("/stimulate", post(stimulate))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Запуск фонового процесса обновления
    tokio::spawn(background_update(stem, signal_tx));

    // Запуск сервера
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("\n╔═══════════════════════════════════════╗");
    println!("║  🚪 SOMA API Server Started          ║");
    println!("╚═══════════════════════════════════════╝\n");
    println!("Listening on: http://{}", addr);
    println!("\nEndpoints:");
    println!("  GET  /              - API information");
    println!("  GET  /state         - System state");
    println!("  GET  /cells         - List all cells");
    println!("  GET  /distribution  - Role distribution");
    println!("  POST /signal        - Send signal");
    println!("  POST /stimulate     - Stimulate system");
    println!("  GET  /ws            - WebSocket stream");
    println!("\nPress Ctrl+C to stop.\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Корневой эндпоинт - информация об API
async fn root() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "SOMA API",
        "version": "0.5.0",
        "description": "Self-Organizing Modular Architecture API",
        "endpoints": {
            "/": "API information",
            "/state": "GET - System state",
            "/cells": "GET - List all cells",
            "/distribution": "GET - Role distribution",
            "/signal": "POST - Send signal {id, value}",
            "/stimulate": "POST - Stimulate system {activity}",
            "/ws": "GET - WebSocket real-time stream"
        }
    }))
}

/// Получить текущее состояние системы
async fn get_state(State(state): State<AppState>) -> Json<StateResponse> {
    let stem = state.stem.lock().unwrap();
    Json(StateResponse {
        cells: stem.cell_count(),
        generation: stem.generation,
        load: stem.load,
        threshold: stem.threshold,
    })
}

/// Получить список всех клеток
async fn get_cells(State(state): State<AppState>) -> Json<Vec<CellResponse>> {
    let stem = state.stem.lock().unwrap();
    let cells: Vec<CellResponse> = stem
        .cells()
        .values()
        .map(|cell| CellResponse {
            id: cell.id.clone(),
            role: format!("{:?}", cell.role),
            generation: cell.generation,
            age_ms: cell.age_millis(),
            activity: cell.activity,
        })
        .collect();
    Json(cells)
}

/// Получить распределение ролей
async fn get_distribution(State(state): State<AppState>) -> Json<DistributionResponse> {
    let stem = state.stem.lock().unwrap();
    let dist = stem.role_distribution();

    Json(DistributionResponse {
        sensor: *dist.get(&CellRole::Sensor).unwrap_or(&0),
        logic: *dist.get(&CellRole::Logic).unwrap_or(&0),
        motor: *dist.get(&CellRole::Motor).unwrap_or(&0),
        total: stem.cell_count(),
    })
}

/// Отправить сигнал в систему
async fn post_signal(
    State(state): State<AppState>,
    Json(signal): Json<ApiSignal>,
) -> Json<serde_json::Value> {
    // Отправляем сигнал в broadcast канал
    let _ = state.signal_tx.send(signal);

    Json(serde_json::json!({
        "status": "ok",
        "message": "Signal sent"
    }))
}

/// Стимулировать систему (вызывает деление при высокой активности)
#[derive(Deserialize)]
struct StimulateRequest {
    activity: f64,
}

async fn stimulate(
    State(state): State<AppState>,
    Json(req): Json<StimulateRequest>,
) -> Json<serde_json::Value> {
    let mut stem = state.stem.lock().unwrap();
    stem.sense(req.activity);

    Json(serde_json::json!({
        "status": "ok",
        "load": stem.load,
        "cells": stem.cell_count(),
        "generation": stem.generation
    }))
}

/// WebSocket обработчик
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_task(socket, state))
}

/// Задача WebSocket - отправка сигналов клиенту
async fn websocket_task(mut socket: WebSocket, state: AppState) {
    let mut rx = state.signal_tx.subscribe();

    // Отправляем приветственное сообщение
    let welcome = serde_json::json!({
        "type": "connected",
        "message": "Connected to SOMA signal stream"
    });

    if socket
        .send(Message::Text(serde_json::to_string(&welcome).unwrap()))
        .await
        .is_err()
    {
        return;
    }

    // Периодически отправляем состояние системы
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

    loop {
        tokio::select! {
            // Получен сигнал от broadcast канала
            Ok(signal) = rx.recv() => {
                let msg = serde_json::json!({
                    "type": "signal",
                    "data": signal
                });

                if socket
                    .send(Message::Text(serde_json::to_string(&msg).unwrap()))
                    .await
                    .is_err()
                {
                    break;
                }
            }

            // Периодическое обновление состояния
            _ = interval.tick() => {
                let state_msg = {
                    let stem = state.stem.lock().unwrap();
                    serde_json::json!({
                        "type": "state",
                        "data": {
                            "cells": stem.cell_count(),
                            "generation": stem.generation,
                            "load": stem.load,
                        }
                    })
                    // Lock освобождается здесь
                };

                if socket
                    .send(Message::Text(serde_json::to_string(&state_msg).unwrap()))
                    .await
                    .is_err()
                {
                    break;
                }
            }
        }
    }
}

/// Фоновая задача обновления системы
async fn background_update(
    stem: Arc<Mutex<StemProcessor>>,
    signal_tx: broadcast::Sender<ApiSignal>,
) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
    let mut cycle = 0u64;

    loop {
        interval.tick().await;

        let mut stem = stem.lock().unwrap();

        // Имитация активности (синусоида)
        let activity = ((cycle as f64 * 0.1).sin().abs() * 0.5) + 0.2;

        stem.sense(activity);
        stem.tick();

        // Периодически отправляем сигнал о состоянии
        if cycle % 10 == 0 {
            let signal = ApiSignal {
                id: "system".to_string(),
                value: stem.load,
                timestamp: cycle,
            };
            let _ = signal_tx.send(signal);
        }

        cycle += 1;
    }
}
