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
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

mod mesh;
use mesh::MeshNode;

/// Состояние приложения, разделяемое между обработчиками
#[derive(Clone)]
struct AppState {
    stem: Arc<Mutex<StemProcessor>>,
    signal_tx: broadcast::Sender<ApiSignal>,
    mesh: Arc<MeshNode>,
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
    // Получить ID узла из переменной окружения или сгенерировать
    let node_id = env::var("NODE_ID").unwrap_or_else(|_| {
        format!("node_{}", chrono::Utc::now().timestamp_millis() % 10000)
    });

    // Получить порт из переменной окружения или использовать 8080
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    // Инициализация состояния
    let stem = Arc::new(Mutex::new(StemProcessor::new()));
    let (signal_tx, _) = broadcast::channel::<ApiSignal>(100);
    let mesh = Arc::new(MeshNode::new(&node_id));

    let state = AppState {
        stem: stem.clone(),
        signal_tx: signal_tx.clone(),
        mesh: mesh.clone(),
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
        .route("/mesh", get(mesh_handler))
        .route("/peers", get(get_peers))
        .route("/resonance", get(get_resonance))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Запуск фонового процесса обновления
    tokio::spawn(background_update(stem.clone(), signal_tx));

    // Запуск mesh фоновых процессов
    let mesh_heartbeat = mesh.clone();
    tokio::spawn(async move {
        mesh_heartbeat.start_heartbeat_loop().await;
    });

    let mesh_cleanup = mesh.clone();
    tokio::spawn(async move {
        mesh_cleanup.start_cleanup_loop(15000).await; // 15 секунд timeout
    });

    // Запуск state sync процесса
    tokio::spawn(mesh_state_sync(stem.clone(), mesh.clone()));

    // Запуск resonance процесса
    tokio::spawn(mesh_resonance_sync(stem, mesh));

    // Запуск сервера
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("\n╔═══════════════════════════════════════╗");
    println!("║  🌊 SOMA Resonance Mesh v0.7         ║");
    println!("╚═══════════════════════════════════════╝\n");
    println!("Node ID: {}", node_id);
    println!("Listening on: http://{}:{}", addr.ip(), port);
    println!("\nEndpoints:");
    println!("  GET  /              - API information");
    println!("  GET  /state         - System state");
    println!("  GET  /cells         - List all cells");
    println!("  GET  /distribution  - Role distribution");
    println!("  GET  /peers         - Connected peers");
    println!("  GET  /resonance     - Network resonance stats");
    println!("  POST /signal        - Send signal");
    println!("  POST /stimulate     - Stimulate system");
    println!("  GET  /ws            - WebSocket stream");
    println!("  GET  /mesh          - Mesh peer connection");
    println!("\nPress Ctrl+C to stop.\n");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// Корневой эндпоинт - информация об API
async fn root(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "SOMA Resonance Mesh",
        "version": "0.7.0",
        "description": "Self-Organizing Modular Architecture - Node Mesh",
        "node_id": state.mesh.id,
        "peer_count": state.mesh.get_peer_count(),
        "endpoints": {
            "/": "API information",
            "/state": "GET - System state",
            "/cells": "GET - List all cells",
            "/distribution": "GET - Role distribution",
            "/peers": "GET - Connected peers",
            "/resonance": "GET - Network resonance stats",
            "/signal": "POST - Send signal {id, value}",
            "/stimulate": "POST - Stimulate system {activity}",
            "/ws": "GET - WebSocket real-time stream",
            "/mesh": "GET - Mesh peer connection (WebSocket)"
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

/// Mesh WebSocket обработчик для peer-to-peer соединений
async fn mesh_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| mesh_connection_task(socket, state))
}

/// Задача обработки mesh соединения
async fn mesh_connection_task(socket: WebSocket, state: AppState) {
    state.mesh.handle_peer_connection(socket).await;
}

/// Получить список подключенных peers
async fn get_peers(State(state): State<AppState>) -> Json<serde_json::Value> {
    let alive_peers = state.mesh.get_alive_peers(15000); // 15 секунд timeout

    let peers_json: Vec<serde_json::Value> = alive_peers
        .iter()
        .map(|peer| {
            serde_json::json!({
                "id": peer.id,
                "last_seen_ms": peer.last_seen,
                "cells": peer.cells,
                "generation": peer.generation,
                "load": peer.load,
                "alive": peer.is_alive(15000)
            })
        })
        .collect();

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "peer_count": state.mesh.get_peer_count(),
        "peers": peers_json
    }))
}

/// Фоновая задача синхронизации состояния mesh
async fn mesh_state_sync(stem: Arc<Mutex<StemProcessor>>, mesh: Arc<MeshNode>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let (cells, generation, load) = {
            let stem = stem.lock().unwrap();
            (stem.cell_count(), stem.generation, stem.load)
        };

        mesh.broadcast_state(cells, generation, load);
    }
}

/// Фоновая задача применения резонанса
async fn mesh_resonance_sync(stem: Arc<Mutex<StemProcessor>>, mesh: Arc<MeshNode>) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

    loop {
        interval.tick().await;

        // Применяем резонанс только если есть живые peers
        if mesh.get_peer_count() > 0 {
            let mut stem = stem.lock().unwrap();
            let current_load = stem.load;

            // Вычисляем корректировку с силой 0.1 (10% коррекции)
            let correction = mesh.compute_resonance_correction(current_load, 0.1);

            // Применяем корректировку
            stem.load = (stem.load + correction).max(0.0).min(1.0);
        }
    }
}

/// Получить статистику резонанса сети
async fn get_resonance(State(state): State<AppState>) -> Json<serde_json::Value> {
    let current_load = {
        let stem = state.stem.lock().unwrap();
        stem.load
    };

    let stats = state.mesh.get_resonance_stats(current_load);

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "current_load": current_load,
        "resonance": stats.resonance,
        "peer_count": stats.peer_count,
        "network": {
            "avg_load": stats.avg_load,
            "min_load": stats.min_load,
            "max_load": stats.max_load,
            "variance": stats.variance
        }
    }))
}
