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
use soma_conscious::{
    ConsciousState, CausalTrace, ReflectionAnalyzer, FeedbackController,
    DominoDecisionTrace, DecisionOutcome, DecisionStats,
};
use soma_domino::{DominoEngine, DominoInput, DominoIntentKind, PeerCandidate};
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
    conscious: Arc<Mutex<ConsciousState>>,
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

// Domino Engine DTOs

/// Запрос оценки Domino Luck Engine
#[derive(Debug, Deserialize)]
struct DominoEvaluateRequest {
    /// Тип намерения
    intent_kind: String,

    /// Список кандидатов
    candidates: Vec<PeerCandidateDto>,

    /// Опциональные контекстные теги
    #[serde(default)]
    context_tags: Vec<String>,
}

/// DTO для PeerCandidate
#[derive(Debug, Deserialize)]
struct PeerCandidateDto {
    peer_id: String,
    health: f32,
    quality: f32,
    intent_match: f32,
}

/// Ответ Domino Luck Engine
#[derive(Debug, Serialize)]
struct DominoEvaluateResponse {
    /// Уникальный ID решения (для последующего обновления outcome)
    decision_id: String,

    /// Отсортированный список лучших пиров
    best_peers: Vec<String>,

    /// Общая оценка удачи (0.0 - 1.0)
    luck_score: f32,

    /// Общая оценка сопротивления (0.0 - 1.0)
    resistance_score: f32,

    /// Человекочитаемое объяснение
    explanation: String,
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
    let conscious = Arc::new(Mutex::new(ConsciousState::new()));

    let state = AppState {
        stem: stem.clone(),
        signal_tx: signal_tx.clone(),
        mesh: mesh.clone(),
        conscious: conscious.clone(),
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
        .route("/peers/register", post(register_peer))
        .route("/resonance", get(get_resonance))
        .route("/mesh/links", get(get_links))
        .route("/mesh/links/tune", post(tune_link))
        .route("/mesh/topology", get(get_topology))
        .route("/mesh/fire", post(fire_event))
        .route("/domino/evaluate", post(domino_evaluate))
        .route("/domino/decisions", get(get_domino_decisions))
        .route("/domino/decisions/recent", get(get_recent_domino_decisions))
        .route("/domino/decisions/stats", get(get_domino_stats))
        .route("/domino/decisions/outcome", post(update_decision_outcome))
        .route("/domino/insights", get(get_domino_insights))
        .route("/conscious/state", get(get_conscious_state))
        .route("/conscious/traces", get(get_conscious_traces))
        .route("/conscious/insights", get(get_conscious_insights))
        .route("/conscious/reflect", post(trigger_reflection))
        .route("/conscious/health", get(get_conscious_health))
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

    let mesh_reconnect = mesh.clone();
    tokio::spawn(async move {
        mesh_reconnect.start_reconnect_loop().await;
    });

    // Запуск state sync процесса
    tokio::spawn(mesh_state_sync(stem.clone(), mesh.clone()));

    // Запуск resonance процесса
    tokio::spawn(mesh_resonance_sync(stem.clone(), mesh.clone()));

    // Запуск Conscious Cycle (v1.0)
    tokio::spawn(conscious_cycle(conscious, mesh, stem));

    // Запуск сервера
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("\n╔═══════════════════════════════════════╗");
    println!("║  🧬 SOMA Conscious Layer v1.0        ║");
    println!("╚═══════════════════════════════════════╝\n");
    println!("Node ID: {}", node_id);
    println!("Listening on: http://{}:{}", addr.ip(), port);
    println!("\nEndpoints:");
    println!("  GET  /              - API information");
    println!("  GET  /state         - System state");
    println!("  GET  /cells         - List all cells");
    println!("  GET  /distribution  - Role distribution");
    println!("  GET  /peers         - Connected peers (with health)");
    println!("  POST /peers/register - Register peer for auto-reconnect");
    println!("  GET  /resonance     - Network resonance stats");
    println!("  GET  /mesh/links    - Link weights and metrics");
    println!("  POST /mesh/links/tune - Tune link weight");
    println!("  GET  /mesh/topology - Top N strongest links");
    println!("  POST /mesh/fire     - Trigger fire event");
    println!("  POST /domino/evaluate - Domino Luck Engine evaluation");
    println!("  GET  /domino/decisions - All Domino decisions history");
    println!("  GET  /domino/decisions/recent - Recent Domino decisions (last 50)");
    println!("  GET  /domino/decisions/stats - Domino decision statistics");
    println!("  POST /domino/decisions/outcome - Update decision outcome");
    println!("  GET  /conscious/state - Conscious state and attention map");
    println!("  GET  /conscious/traces - Causal traces (recent)");
    println!("  GET  /conscious/insights - Generated insights");
    println!("  POST /conscious/reflect - Trigger reflection cycle");
    println!("  GET  /conscious/health - Consciousness metrics");
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
    let conscious_state = state.conscious.lock().unwrap();
    Json(serde_json::json!({
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
                "alive": peer.is_alive(15000),
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
struct RegisterPeerRequest {
    peer_id: String,
    url: String,
}

async fn register_peer(
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

            // Вычисляем адаптивную силу на основе здоровья сети (0.05-0.2)
            let strength = mesh.compute_adaptive_strength();

            // Вычисляем корректировку с адаптивной силой
            let correction = mesh.compute_resonance_correction(current_load, strength);

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
    let adaptive_strength = state.mesh.compute_adaptive_strength();

    Json(serde_json::json!({
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
    }))
}

// Hebbian Learning API Handlers (v0.9)

/// GET /mesh/links - Получить все веса связей с метриками
async fn get_links(State(state): State<AppState>) -> Json<serde_json::Value> {
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
struct TuneLinkRequest {
    peer_id: String,
    weight: f64,
}

/// POST /mesh/links/tune - Ручная подстройка веса связи
async fn tune_link(
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
async fn get_topology(State(state): State<AppState>) -> Json<serde_json::Value> {
    let top_links = state.mesh.get_top_links(10); // Топ-10 связей

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
async fn fire_event(State(state): State<AppState>) -> Json<serde_json::Value> {
    state.mesh.send_fire();

    Json(serde_json::json!({
        "status": "ok",
        "node_id": state.mesh.id,
        "message": "Fire event sent to all peers"
    }))
}

// Domino Engine API Handler

/// POST /domino/evaluate - Оценка "удачи" для выбора лучших пиров
async fn domino_evaluate(
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
    {
        let mut conscious = state.conscious.lock().unwrap();
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
async fn get_domino_decisions(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conscious = state.conscious.lock().unwrap();
    let decisions = conscious.get_decisions();

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "total_decisions": decisions.len(),
        "decisions": decisions
    }))
}

/// GET /domino/decisions/recent?limit=N - Получить последние N решений
async fn get_recent_domino_decisions(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conscious = state.conscious.lock().unwrap();
    let recent = conscious.get_recent_decisions(50); // По умолчанию последние 50

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "count": recent.len(),
        "decisions": recent
    }))
}

/// GET /domino/decisions/stats - Статистика решений
async fn get_domino_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conscious = state.conscious.lock().unwrap();
    let stats = conscious.get_decision_stats();

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "stats": stats
    }))
}

/// Request для обновления outcome решения
#[derive(Deserialize)]
struct UpdateOutcomeRequest {
    decision_id: String,
    outcome_type: String, // "success", "failure", "partial"
    #[serde(default)]
    actual_latency_ms: Option<f64>,
    #[serde(default)]
    actual_quality: Option<f64>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    completed_ratio: Option<f64>,
    #[serde(default)]
    issues: Vec<String>,
}

/// POST /domino/decisions/outcome - Обновить результат решения
async fn update_decision_outcome(
    State(state): State<AppState>,
    Json(req): Json<UpdateOutcomeRequest>,
) -> Json<serde_json::Value> {
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
            return Json(serde_json::json!({
                "status": "error",
                "message": "Invalid outcome_type. Use: success, failure, or partial"
            }));
        }
    };

    let mut conscious = state.conscious.lock().unwrap();
    let updated = conscious.update_decision_outcome(&req.decision_id, outcome);

    if updated {
        Json(serde_json::json!({
            "status": "ok",
            "decision_id": req.decision_id,
            "message": "Decision outcome updated"
        }))
    } else {
        Json(serde_json::json!({
            "status": "error",
            "message": "Decision ID not found"
        }))
    }
}

/// GET /domino/insights - Dashboard with routing insights and analysis
async fn get_domino_insights(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conscious = state.conscious.lock().unwrap();

    // Create analyzer and generate insights
    let analyzer = ReflectionAnalyzer::new();
    let insights = analyzer.analyze_routing_decisions(&conscious);

    // Get basic stats for context
    let stats = conscious.get_decision_stats();
    let decisions_count = conscious.decisions_count();

    Json(serde_json::json!({
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
    }))
}

// Conscious API Handlers (v1.0)

/// GET /conscious/state - Текущее состояние осознанности
async fn get_conscious_state(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conscious = state.conscious.lock().unwrap();
    let attention_map = conscious.get_attention_map();

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "cycle_count": conscious.cycle_count,
        "last_cycle_ms": conscious.last_cycle,
        "traces_count": conscious.traces_count(),
        "insights_count": conscious.insights_count(),
        "attention_map": {
            "top_nodes": attention_map.top_nodes,
            "updated_at": attention_map.updated_at
        }
    }))
}

/// GET /conscious/traces - Получить последние причинные цепи
async fn get_conscious_traces(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conscious = state.conscious.lock().unwrap();
    let traces = conscious.get_traces(50); // Последние 50

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "traces": traces,
        "count": traces.len()
    }))
}

/// GET /conscious/insights - Получить сгенерированные инсайты
async fn get_conscious_insights(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conscious = state.conscious.lock().unwrap();
    let insights = conscious.get_insights(20); // Последние 20

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "insights": insights,
        "count": insights.len()
    }))
}

/// POST /conscious/reflect - Триггер немедленной рефлексии
async fn trigger_reflection(State(state): State<AppState>) -> Json<serde_json::Value> {
    let mut conscious = state.conscious.lock().unwrap();

    // Запуск анализа
    let analyzer = ReflectionAnalyzer::new();
    let insights = analyzer.analyze(&conscious, 60000); // Окно 60 секунд

    // Добавить инсайты
    for insight in &insights {
        conscious.add_insight(insight.clone());
    }

    Json(serde_json::json!({
        "status": "ok",
        "node_id": state.mesh.id,
        "insights_generated": insights.len(),
        "insights": insights
    }))
}

/// GET /conscious/health - Метрики осознанности
async fn get_conscious_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let conscious = state.conscious.lock().unwrap();

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

    Json(serde_json::json!({
        "node_id": state.mesh.id,
        "cycle_count": conscious.cycle_count,
        "traces_per_cycle": traces_rate,
        "insights_per_cycle": insights_rate,
        "health_status": if traces_rate > 0.5 { "active" } else { "quiet" }
    }))
}

/// Conscious Cycle - observe → record → analyze → generate → apply
async fn conscious_cycle(
    conscious: Arc<Mutex<ConsciousState>>,
    mesh: Arc<MeshNode>,
    _stem: Arc<Mutex<StemProcessor>>,
) {
    use tokio::time::{interval, Duration};

    let mut tick = interval(Duration::from_secs(5)); // Каждые 5 секунд
    let analyzer = ReflectionAnalyzer::new();
    let feedback = FeedbackController::new();

    loop {
        tick.tick().await;

        // OBSERVE: Наблюдаем за состоянием mesh
        let link_weights = mesh.get_link_weights();

        // RECORD: Записываем причинные цепи
        {
            let mut state = conscious.lock().unwrap();

            // Для каждого изменения веса создаём trace
            for (peer_id, weight, quality) in &link_weights {
                if *weight != 0.3 { // Изменён от дефолта
                    let trace = CausalTrace::new(
                        format!("network_activity"),
                        format!("{}_weight_{:.3}", peer_id, weight),
                        weight - 0.3,
                    );
                    state.record_trace(trace);
                }
            }
        }

        // ANALYZE: Анализируем паттерны (окно 60 секунд)
        let insights = {
            let state = conscious.lock().unwrap();
            analyzer.analyze(&state, 60000)
        };

        // GENERATE: Генерируем инсайты
        {
            let mut state = conscious.lock().unwrap();
            for insight in &insights {
                state.add_insight(insight.clone());
                println!("💭 Insight: {} ({})", insight.insight, insight.category);
            }
        }

        // APPLY: Применяем feedback
        let actions = feedback.generate_actions(&insights);
        if !actions.is_empty() {
            println!("🔧 Feedback: {} actions generated", actions.len());
            for action in &actions {
                println!("   → {:?}: {} = {:.3}", action.action_type, action.target, action.value);
            }
        }

        // Завершаем цикл
        {
            let mut state = conscious.lock().unwrap();
            state.complete_cycle();
        }
    }
}
