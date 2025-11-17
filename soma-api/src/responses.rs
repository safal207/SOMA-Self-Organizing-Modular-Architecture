//! Типы ответов API для SOMA
//!
//! Стандартизированные структуры ответов для всех эндпоинтов

use serde::Serialize;
use serde_json::Value;

/// Стандартный успешный ответ
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub status: String,
    pub message: String,
}

impl SuccessResponse {
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            status: "ok".to_string(),
            message: message.into(),
        }
    }
}

/// Ответ с данными
#[derive(Debug, Serialize)]
pub struct DataResponse<T> {
    pub status: String,
    pub data: T,
}

impl<T: Serialize> DataResponse<T> {
    pub fn new(data: T) -> Self {
        Self {
            status: "ok".to_string(),
            data,
        }
    }
}

/// Ответ со списком элементов
#[derive(Debug, Serialize)]
pub struct ListResponse<T> {
    pub status: String,
    pub count: usize,
    pub items: Vec<T>,
}

impl<T> ListResponse<T> {
    pub fn new(items: Vec<T>) -> Self {
        let count = items.len();
        Self {
            status: "ok".to_string(),
            count,
            items,
        }
    }
}

/// Ответ с метаданными
#[derive(Debug, Serialize)]
pub struct MetaResponse<T> {
    pub status: String,
    #[serde(flatten)]
    pub data: T,
    pub meta: Value,
}

impl<T: Serialize> MetaResponse<T> {
    pub fn new(data: T, meta: Value) -> Self {
        Self {
            status: "ok".to_string(),
            data,
            meta,
        }
    }
}

