//! SOMA API - REST + WebSocket интерфейс для Self-Organizing Modular Architecture
//!
//! Модульная архитектура API с разделением на handlers, errors, responses и config

pub mod mesh;
pub mod config;
pub mod errors;
pub mod responses;
pub mod handlers;
pub mod background;

// Re-export для удобства
pub use errors::ApiError;
pub use responses::{SuccessResponse, DataResponse, ListResponse, MetaResponse};

// Типы состояния и данных
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use soma_core::StemProcessor;
use soma_conscious::ConsciousState;

/// Состояние приложения, разделяемое между обработчиками
#[derive(Clone)]
pub struct AppState {
    pub stem: Arc<Mutex<StemProcessor>>,
    pub signal_tx: broadcast::Sender<ApiSignal>,
    pub mesh: Arc<mesh::MeshNode>,
    pub conscious: Arc<Mutex<ConsciousState>>,
}

/// API-представление сигнала
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiSignal {
    pub id: String,
    pub value: f64,
    pub timestamp: u64,
}

impl From<soma_bridge::Signal> for ApiSignal {
    fn from(sig: soma_bridge::Signal) -> Self {
        Self {
            id: sig.id,
            value: sig.value,
            timestamp: sig.timestamp as u64,
        }
    }
}

/// Ответ с состоянием системы
#[derive(serde::Serialize)]
pub struct StateResponse {
    pub cells: usize,
    pub generation: u32,
    pub load: f64,
    pub threshold: f64,
}

/// Информация о клетке для API
#[derive(serde::Serialize)]
pub struct CellResponse {
    pub id: String,
    pub role: String,
    pub generation: u32,
    pub age_ms: u64,
    pub activity: f64,
}

/// Распределение ролей
#[derive(serde::Serialize)]
pub struct DistributionResponse {
    pub sensor: usize,
    pub logic: usize,
    pub motor: usize,
    pub total: usize,
}
