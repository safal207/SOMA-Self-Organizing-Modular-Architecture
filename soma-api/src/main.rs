//! SOMA API - Главная точка входа
//!
//! Рефакторинг: модульная архитектура с разделением handlers, errors, responses

use axum::{
    routing::{get, post},
    Router,
};
use std::{env, net::SocketAddr};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use soma_api::{
    AppState, ApiSignal, config,
    handlers::{
        system, cells, mesh, domino, conscious, websocket,
    },
    background,
};
use soma_core::StemProcessor;
use soma_conscious::ConsciousState;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    // Получить ID узла из переменной окружения или сгенерировать
    let node_id = env::var("NODE_ID").unwrap_or_else(|_| {
        format!("node_{}", chrono::Utc::now().timestamp_millis() % 10000)
    });

    // Получить порт из переменной окружения или использовать дефолтный
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(config::api::DEFAULT_PORT);

    // Инициализация состояния
    let stem = Arc::new(Mutex::new(StemProcessor::new()));
    let (signal_tx, _) = broadcast::channel::<ApiSignal>(config::api::SIGNAL_CHANNEL_SIZE);
    let mesh = Arc::new(soma_api::mesh::MeshNode::new(&node_id));
    let conscious = Arc::new(Mutex::new(ConsciousState::new()));

    let state = AppState {
        stem: stem.clone(),
        signal_tx: signal_tx.clone(),
        mesh: mesh.clone(),
        conscious: conscious.clone(),
    };

    // Построение роутера с использованием модульных handlers
    let app = Router::new()
        // System endpoints
        .route("/", get(system::root))
        .route("/state", get(system::get_state))
        .route("/signal", post(system::post_signal))
        .route("/stimulate", post(system::stimulate))
        
        // Cell endpoints
        .route("/cells", get(cells::get_cells))
        .route("/distribution", get(cells::get_distribution))
        
        // Mesh endpoints
        .route("/mesh", get(websocket::mesh_handler))
        .route("/peers", get(mesh::get_peers))
        .route("/peers/register", post(mesh::register_peer))
        .route("/resonance", get(mesh::get_resonance))
        .route("/mesh/links", get(mesh::get_links))
        .route("/mesh/links/tune", post(mesh::tune_link))
        .route("/mesh/topology", get(mesh::get_topology))
        .route("/mesh/fire", post(mesh::fire_event))
        
        // Domino endpoints
        .route("/domino/evaluate", post(domino::domino_evaluate))
        .route("/domino/decisions", get(domino::get_domino_decisions))
        .route("/domino/decisions/recent", get(domino::get_recent_domino_decisions))
        .route("/domino/decisions/stats", get(domino::get_domino_stats))
        .route("/domino/decisions/outcome", post(domino::update_decision_outcome))
        .route("/domino/insights", get(domino::get_domino_insights))
        
        // Conscious endpoints
        .route("/conscious/state", get(conscious::get_conscious_state))
        .route("/conscious/traces", get(conscious::get_conscious_traces))
        .route("/conscious/insights", get(conscious::get_conscious_insights))
        .route("/conscious/reflect", post(conscious::trigger_reflection))
        .route("/conscious/health", get(conscious::get_conscious_health))
        
        // WebSocket endpoints
        .route("/ws", get(websocket::websocket_handler))
        
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Запуск фоновых процессов
    start_background_tasks(stem, signal_tx, mesh.clone(), conscious);

    // Запуск сервера
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    print_startup_info(&node_id, &addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

/// Запуск всех фоновых задач
fn start_background_tasks(
    stem: Arc<Mutex<StemProcessor>>,
    signal_tx: broadcast::Sender<ApiSignal>,
    mesh: Arc<soma_api::mesh::MeshNode>,
    conscious: Arc<Mutex<ConsciousState>>,
) {
    // Фоновое обновление системы
    tokio::spawn(background::background_update(stem.clone(), signal_tx));

    // Mesh фоновые процессы
    let mesh_heartbeat = mesh.clone();
    tokio::spawn(async move {
        mesh_heartbeat.start_heartbeat_loop().await;
    });

    let mesh_cleanup = mesh.clone();
    tokio::spawn(async move {
        mesh_cleanup.start_cleanup_loop().await;
    });

    let mesh_reconnect = mesh.clone();
    tokio::spawn(async move {
        mesh_reconnect.start_reconnect_loop().await;
    });

    // State sync процессы
    tokio::spawn(background::mesh_state_sync(stem.clone(), mesh.clone()));
    tokio::spawn(background::mesh_resonance_sync(stem.clone(), mesh.clone()));

    // Conscious Cycle
    tokio::spawn(background::conscious_cycle(conscious, mesh, stem));
}

/// Вывод информации о запуске сервера
fn print_startup_info(node_id: &str, addr: &SocketAddr) {
    println!("\n╔═══════════════════════════════════════╗");
    println!("║  🧬 SOMA Conscious Layer v1.0        ║");
    println!("╚═══════════════════════════════════════╝\n");
    println!("Node ID: {}", node_id);
    println!("Listening on: http://{}:{}", addr.ip(), addr.port());
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
    println!("  GET  /domino/decisions/recent - Recent Domino decisions");
    println!("  GET  /domino/decisions/stats - Domino decision statistics");
    println!("  POST /domino/decisions/outcome - Update decision outcome");
    println!("  GET  /domino/insights - Routing insights dashboard");
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
}
