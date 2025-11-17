//! Обработка ошибок для SOMA API
//!
//! Централизованная система обработки ошибок с преобразованием в HTTP ответы

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Типы ошибок API
#[derive(Debug)]
pub enum ApiError {
    /// Внутренняя ошибка сервера
    Internal(String),
    /// Ошибка блокировки (deadlock, poison)
    LockError(String),
    /// Неверный запрос
    BadRequest(String),
    /// Ресурс не найден
    NotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::LockError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Lock error: {}", msg))
            }
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

/// Helper для безопасной работы с мьютексами
pub fn lock_mutex<T>(guard_result: Result<std::sync::MutexGuard<'_, T>, std::sync::PoisonError<std::sync::MutexGuard<'_, T>>>) -> Result<std::sync::MutexGuard<'_, T>, ApiError> {
    guard_result.map_err(|e| ApiError::LockError(format!("Mutex poisoned: {}", e)))
}

/// Helper для безопасной работы с Arc<Mutex<T>>
pub fn lock_arc_mutex<T>(arc: &std::sync::Arc<std::sync::Mutex<T>>) -> Result<std::sync::MutexGuard<'_, T>, ApiError> {
    lock_mutex(arc.lock())
}

