use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use axum::extract::ws::{WebSocket, Message};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use chrono::Utc;
use futures::{StreamExt, SinkExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as TungsteniteMessage};

/// Здоровье соединения с peer
#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    pub failures: u32,
    pub successes: u32,
    pub last_success: Instant,
    pub last_failure: Option<Instant>,
    pub quality: f64, // 0.0-1.0
}

impl ConnectionHealth {
    fn new() -> Self {
        Self {
            failures: 0,
            successes: 0,
            last_success: Instant::now(),
            last_failure: None,
            quality: 1.0,
        }
    }

    fn record_success(&mut self) {
        self.successes += 1;
        self.last_success = Instant::now();
        // Плавное восстановление quality
        self.quality = (self.quality + 0.1).min(1.0);
    }

    fn record_failure(&mut self) {
        self.failures += 1;
        self.last_failure = Some(Instant::now());
        // Быстрая деградация quality
        self.quality = (self.quality - 0.2).max(0.0);
    }

    pub fn is_healthy(&self) -> bool {
        self.quality > 0.5
    }

    pub fn failure_rate(&self) -> f64 {
        let total = self.failures + self.successes;
        if total == 0 {
            return 0.0;
        }
        self.failures as f64 / total as f64
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ResonanceStats {
    pub peer_count: usize,
    pub avg_load: f64,
    pub min_load: f64,
    pub max_load: f64,
    pub resonance: f64,
    pub variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MeshMessage {
    Handshake {
        node_id: String,
        timestamp: i64,
    },
    Heartbeat {
        node_id: String,
        timestamp: i64,
    },
    StateSync {
        node_id: String,
        cells: usize,
        generation: u32,
        load: f64,
        timestamp: i64,
    },
    Ack {
        node_id: String,
        ack_to: String,
        timestamp: i64,
    },
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub id: String,
    pub last_seen: i64,
    pub cells: usize,
    pub generation: u32,
    pub load: f64,
    pub health: ConnectionHealth,
    pub url: Option<String>, // URL для переподключения
    pub connected: bool,      // Активно ли соединение
}

impl PeerInfo {
    fn new(id: String) -> Self {
        Self {
            id,
            last_seen: Utc::now().timestamp_millis(),
            cells: 0,
            generation: 0,
            load: 0.0,
            health: ConnectionHealth::new(),
            url: None,
            connected: true,
        }
    }

    fn with_url(id: String, url: String) -> Self {
        Self {
            id,
            last_seen: Utc::now().timestamp_millis(),
            cells: 0,
            generation: 0,
            load: 0.0,
            health: ConnectionHealth::new(),
            url: Some(url),
            connected: false,
        }
    }

    fn update_heartbeat(&mut self) {
        self.last_seen = Utc::now().timestamp_millis();
        self.health.record_success();
    }

    fn update_state(&mut self, cells: usize, generation: u32, load: f64) {
        self.cells = cells;
        self.generation = generation;
        self.load = load;
        self.last_seen = Utc::now().timestamp_millis();
        self.health.record_success();
    }

    fn record_failure(&mut self) {
        self.health.record_failure();
    }

    pub fn is_alive(&self, timeout_ms: i64) -> bool {
        let now = Utc::now().timestamp_millis();
        (now - self.last_seen) < timeout_ms
    }
}

pub struct MeshNode {
    pub id: String,
    pub peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    pub message_tx: Arc<Mutex<Option<mpsc::UnboundedSender<MeshMessage>>>>,
}

impl MeshNode {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            peers: Arc::new(Mutex::new(HashMap::new())),
            message_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn handle_peer_connection(&self, socket: WebSocket) {
        let node_id = self.id.clone();
        let peers = self.peers.clone();

        let (mut ws_sender, mut ws_receiver) = socket.split();
        let (msg_tx, mut msg_rx) = mpsc::unbounded_channel::<MeshMessage>();

        // Сохраняем канал для отправки сообщений
        {
            let mut tx = self.message_tx.lock().unwrap();
            *tx = Some(msg_tx.clone());
        }

        // Отправляем handshake при подключении
        let handshake = MeshMessage::Handshake {
            node_id: node_id.clone(),
            timestamp: Utc::now().timestamp_millis(),
        };

        if let Ok(json) = serde_json::to_string(&handshake) {
            let _ = ws_sender.send(Message::Text(json)).await;
        }

        // Задача для отправки исходящих сообщений
        let peers_for_send = peers.clone();
        let send_task = tokio::spawn(async move {
            while let Some(msg) = msg_rx.recv().await {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if ws_sender.send(Message::Text(json)).await.is_err() {
                        // Отмечаем failure для всех peers при ошибке отправки
                        let mut peers_map = peers_for_send.lock().unwrap();
                        for peer in peers_map.values_mut() {
                            peer.record_failure();
                        }
                        break;
                    }
                }
            }
        });

        // Обработка входящих сообщений
        let recv_task = tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_receiver.next().await {
                if let Message::Text(txt) = msg {
                    if let Ok(parsed) = serde_json::from_str::<MeshMessage>(&txt) {
                        match &parsed {
                            MeshMessage::Handshake { node_id: peer_id, .. } => {
                                let mut peers_map = peers.lock().unwrap();
                                peers_map.insert(peer_id.clone(), PeerInfo::new(peer_id.clone()));
                                println!("🤝 Handshake from peer: {}", peer_id);

                                // Отправляем Ack
                                let ack = MeshMessage::Ack {
                                    node_id: node_id.clone(),
                                    ack_to: peer_id.clone(),
                                    timestamp: Utc::now().timestamp_millis(),
                                };
                                msg_tx.send(ack).ok();
                            }
                            MeshMessage::Heartbeat { node_id: peer_id, .. } => {
                                let mut peers_map = peers.lock().unwrap();
                                if let Some(peer) = peers_map.get_mut(peer_id) {
                                    peer.update_heartbeat();
                                }
                            }
                            MeshMessage::StateSync { node_id: peer_id, cells, generation, load, .. } => {
                                let mut peers_map = peers.lock().unwrap();
                                if let Some(peer) = peers_map.get_mut(peer_id) {
                                    peer.update_state(*cells, *generation, *load);
                                    println!("📊 State sync from {}: {} cells, gen {}, load {:.2}",
                                             peer_id, cells, generation, load);
                                }
                            }
                            MeshMessage::Ack { ack_to, .. } => {
                                println!("✅ Ack received for: {}", ack_to);
                            }
                        }
                    }
                }
            }
        });

        // Ждём завершения любой из задач
        tokio::select! {
            _ = send_task => {},
            _ = recv_task => {},
        }

        // Очищаем канал
        let mut tx = self.message_tx.lock().unwrap();
        *tx = None;
    }

    pub fn send_message(&self, msg: MeshMessage) {
        let tx = self.message_tx.lock().unwrap();
        if let Some(sender) = tx.as_ref() {
            let _ = sender.send(msg);
        }
    }

    pub fn broadcast_heartbeat(&self) {
        let msg = MeshMessage::Heartbeat {
            node_id: self.id.clone(),
            timestamp: Utc::now().timestamp_millis(),
        };
        self.send_message(msg);
    }

    pub fn broadcast_state(&self, cells: usize, generation: u32, load: f64) {
        let msg = MeshMessage::StateSync {
            node_id: self.id.clone(),
            cells,
            generation,
            load,
            timestamp: Utc::now().timestamp_millis(),
        };
        self.send_message(msg);
    }

    pub fn get_alive_peers(&self, timeout_ms: i64) -> Vec<PeerInfo> {
        let peers = self.peers.lock().unwrap();
        peers.values()
            .filter(|p| p.is_alive(timeout_ms))
            .cloned()
            .collect()
    }

    pub fn get_peer_count(&self) -> usize {
        self.peers.lock().unwrap().len()
    }

    /// Вычислить резонанс сети - среднее отклонение от текущей нагрузки
    pub fn compute_network_resonance(&self, current_load: f64) -> f64 {
        let peers = self.peers.lock().unwrap();

        if peers.is_empty() {
            return 1.0; // Полный резонанс если нет peers
        }

        let peer_loads: Vec<f64> = peers.values()
            .filter(|p| p.is_alive(15000))
            .map(|p| p.load)
            .collect();

        if peer_loads.is_empty() {
            return 1.0;
        }

        // Вычисляем среднюю нагрузку сети
        let avg_load: f64 = peer_loads.iter().sum::<f64>() / peer_loads.len() as f64;

        // Резонанс = 1.0 - нормализованная разница
        let diff = (current_load - avg_load).abs();
        (1.0 - diff.min(1.0)).max(0.0)
    }

    /// Вычислить корректировку нагрузки для достижения резонанса
    pub fn compute_resonance_correction(&self, current_load: f64, strength: f64) -> f64 {
        let peers = self.peers.lock().unwrap();

        if peers.is_empty() {
            return 0.0;
        }

        let peer_loads: Vec<f64> = peers.values()
            .filter(|p| p.is_alive(15000))
            .map(|p| p.load)
            .collect();

        if peer_loads.is_empty() {
            return 0.0;
        }

        // Средняя нагрузка сети
        let avg_load: f64 = peer_loads.iter().sum::<f64>() / peer_loads.len() as f64;

        // Корректировка = разница * сила (0.0-1.0)
        let delta = (avg_load - current_load) * strength;

        delta
    }

    /// Вычислить адаптивную силу резонанса на основе здоровья сети
    /// Возвращает значение от 0.05 (слабая сеть) до 0.2 (здоровая сеть)
    pub fn compute_adaptive_strength(&self) -> f64 {
        let peers = self.peers.lock().unwrap();

        if peers.is_empty() {
            return 0.1; // Базовая сила без peers
        }

        let alive_peers: Vec<&PeerInfo> = peers.values()
            .filter(|p| p.is_alive(15000))
            .collect();

        if alive_peers.is_empty() {
            return 0.05; // Минимальная сила при отсутствии живых peers
        }

        // Средняя качество соединений
        let avg_quality = alive_peers.iter()
            .map(|p| p.health.quality)
            .sum::<f64>() / alive_peers.len() as f64;

        // Маппинг quality (0.0-1.0) -> strength (0.05-0.2)
        0.05 + (avg_quality * 0.15)
    }

    /// Получить статистику резонанса сети
    pub fn get_resonance_stats(&self, current_load: f64) -> ResonanceStats {
        let peers = self.peers.lock().unwrap();

        let alive_peers: Vec<&PeerInfo> = peers.values()
            .filter(|p| p.is_alive(15000))
            .collect();

        if alive_peers.is_empty() {
            return ResonanceStats {
                peer_count: 0,
                avg_load: current_load,
                min_load: current_load,
                max_load: current_load,
                resonance: 1.0,
                variance: 0.0,
            };
        }

        let loads: Vec<f64> = alive_peers.iter().map(|p| p.load).collect();
        let avg_load = loads.iter().sum::<f64>() / loads.len() as f64;
        let min_load = loads.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_load = loads.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Variance
        let variance = loads.iter()
            .map(|l| (l - avg_load).powi(2))
            .sum::<f64>() / loads.len() as f64;

        // Resonance
        let diff = (current_load - avg_load).abs();
        let resonance = (1.0 - diff.min(1.0)).max(0.0);

        ResonanceStats {
            peer_count: alive_peers.len(),
            avg_load,
            min_load,
            max_load,
            resonance,
            variance,
        }
    }

    pub async fn start_heartbeat_loop(self: Arc<Self>) {
        let mut tick = interval(Duration::from_secs(3));
        loop {
            tick.tick().await;
            self.broadcast_heartbeat();
        }
    }

    pub async fn start_cleanup_loop(self: Arc<Self>, timeout_ms: i64) {
        let mut tick = interval(Duration::from_secs(10));
        loop {
            tick.tick().await;
            let mut peers = self.peers.lock().unwrap();
            let now = Utc::now().timestamp_millis();

            // Отмечаем мертвые peers как disconnected, но сохраняем их для переподключения
            for (id, peer) in peers.iter_mut() {
                let alive = (now - peer.last_seen) < timeout_ms;
                if !alive && peer.connected {
                    println!("💀 Peer {} timed out (will attempt reconnect)", id);
                    peer.connected = false;
                    peer.record_failure();
                }
            }
        }
    }

    /// Зарегистрировать peer URL для автоматического переподключения
    pub fn register_peer(&self, peer_id: String, url: String) {
        let mut peers = self.peers.lock().unwrap();
        peers.insert(peer_id.clone(), PeerInfo::with_url(peer_id, url));
    }

    /// Попытаться подключиться к peer как клиент
    pub async fn attempt_connect_to_peer(self: Arc<Self>, peer_id: String, url: String) -> bool {
        println!("🔄 Attempting to connect to peer {} at {}", peer_id, url);

        match connect_async(&url).await {
            Ok((ws_stream, _)) => {
                println!("✅ Connected to peer {}", peer_id);

                // Отмечаем peer как подключенный
                {
                    let mut peers = self.peers.lock().unwrap();
                    if let Some(peer) = peers.get_mut(&peer_id) {
                        peer.connected = true;
                        peer.health.record_success();
                    }
                }

                // Обрабатываем соединение (конвертируем tungstenite WebSocket в axum WebSocket)
                // TODO: Это требует более сложной интеграции, пока просто держим соединение
                let (mut write, mut read) = ws_stream.split();
                let node_id = self.id.clone();
                let peers = self.peers.clone();

                // Отправляем handshake
                let handshake = MeshMessage::Handshake {
                    node_id: node_id.clone(),
                    timestamp: Utc::now().timestamp_millis(),
                };

                if let Ok(json) = serde_json::to_string(&handshake) {
                    let _ = write.send(TungsteniteMessage::Text(json)).await;
                }

                // Обрабатываем входящие сообщения
                tokio::spawn(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        if let TungsteniteMessage::Text(txt) = msg {
                            if let Ok(parsed) = serde_json::from_str::<MeshMessage>(&txt) {
                                match &parsed {
                                    MeshMessage::Handshake { node_id: peer_id, .. } => {
                                        let mut peers_map = peers.lock().unwrap();
                                        if let Some(peer) = peers_map.get_mut(peer_id) {
                                            peer.connected = true;
                                            peer.health.record_success();
                                        }
                                        println!("🤝 Handshake from peer: {}", peer_id);
                                    }
                                    MeshMessage::Heartbeat { node_id: peer_id, .. } => {
                                        let mut peers_map = peers.lock().unwrap();
                                        if let Some(peer) = peers_map.get_mut(peer_id) {
                                            peer.update_heartbeat();
                                        }
                                    }
                                    MeshMessage::StateSync { node_id: peer_id, cells, generation, load, .. } => {
                                        let mut peers_map = peers.lock().unwrap();
                                        if let Some(peer) = peers_map.get_mut(peer_id) {
                                            peer.update_state(*cells, *generation, *load);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    // Соединение закрылось
                    println!("🔌 Connection to peer {} closed", peer_id);
                    let mut peers_map = peers.lock().unwrap();
                    if let Some(peer) = peers_map.get_mut(&peer_id) {
                        peer.connected = false;
                        peer.record_failure();
                    }
                });

                true
            }
            Err(e) => {
                println!("❌ Failed to connect to peer {}: {}", peer_id, e);

                let mut peers = self.peers.lock().unwrap();
                if let Some(peer) = peers.get_mut(&peer_id) {
                    peer.health.record_failure();
                }

                false
            }
        }
    }

    /// Запустить loop автоматического переподключения
    pub async fn start_reconnect_loop(self: Arc<Self>) {
        let mut tick = interval(Duration::from_secs(30)); // Попытка переподключения каждые 30 секунд

        loop {
            tick.tick().await;

            // Найти disconnected peers с URL
            let peers_to_reconnect: Vec<(String, String)> = {
                let peers = self.peers.lock().unwrap();
                peers.values()
                    .filter(|p| !p.connected && p.url.is_some() && p.health.quality > 0.0)
                    .map(|p| (p.id.clone(), p.url.clone().unwrap()))
                    .collect()
            };

            // Попытаться переподключиться
            for (peer_id, url) in peers_to_reconnect {
                let self_clone = self.clone();
                tokio::spawn(async move {
                    self_clone.attempt_connect_to_peer(peer_id, url).await;
                });
            }
        }
    }
}
