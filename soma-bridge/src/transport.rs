use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Сообщение, передаваемое через bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Уникальный идентификатор сообщения
    pub id: String,
    /// Источник сообщения
    pub source: String,
    /// Получатель сообщения
    pub destination: String,
    /// Тип сообщения
    pub msg_type: MessageType,
    /// Полезная нагрузка
    pub payload: HashMap<String, serde_json::Value>,
    /// Временная метка
    pub timestamp: u64,
}

/// Типы сообщений в системе
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageType {
    /// Сигнал (нейронная активация)
    Signal,
    /// Команда (выполнение действия)
    Command,
    /// Запрос (получение данных)
    Query,
    /// Ответ на запрос
    Response,
    /// Событие (уведомление)
    Event,
    /// Резонанс (синхронизация)
    Resonance,
}

impl Message {
    /// Создать новое сообщение
    pub fn new(
        id: String,
        source: String,
        destination: String,
        msg_type: MessageType,
    ) -> Self {
        Self {
            id,
            source,
            destination,
            msg_type,
            payload: HashMap::new(),
            timestamp: current_timestamp(),
        }
    }

    /// Добавить данные в payload
    pub fn with_payload(mut self, key: String, value: serde_json::Value) -> Self {
        self.payload.insert(key, value);
        self
    }

    /// Получить значение из payload
    pub fn get_payload(&self, key: &str) -> Option<&serde_json::Value> {
        self.payload.get(key)
    }
}

/// Транспортный слой - абстракция для передачи сообщений
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    /// Отправить сообщение
    async fn send(&self, message: Message) -> Result<(), TransportError>;

    /// Получить следующее сообщение
    async fn receive(&self) -> Result<Message, TransportError>;

    /// Подписаться на определённый тип сообщений
    async fn subscribe(&self, msg_type: MessageType) -> Result<(), TransportError>;

    /// Отписаться от типа сообщений
    async fn unsubscribe(&self, msg_type: MessageType) -> Result<(), TransportError>;
}

/// Ошибки транспортного слоя
#[derive(Debug, Clone)]
pub enum TransportError {
    /// Ошибка подключения
    ConnectionError(String),
    /// Ошибка сериализации
    SerializationError(String),
    /// Ошибка таймаута
    Timeout,
    /// Сообщение не найдено
    NotFound,
    /// Другая ошибка
    Other(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            TransportError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            TransportError::Timeout => write!(f, "Timeout"),
            TransportError::NotFound => write!(f, "Not found"),
            TransportError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for TransportError {}

/// Получить текущую временную метку в миллисекундах
fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Локальный транспорт для тестирования (в памяти)
pub struct LocalTransport {
    messages: std::sync::Arc<tokio::sync::Mutex<Vec<Message>>>,
}

impl LocalTransport {
    /// Создать новый локальный транспорт
    pub fn new() -> Self {
        Self {
            messages: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
}

impl Default for LocalTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Transport for LocalTransport {
    async fn send(&self, message: Message) -> Result<(), TransportError> {
        let mut messages = self.messages.lock().await;
        messages.push(message);
        Ok(())
    }

    async fn receive(&self) -> Result<Message, TransportError> {
        let mut messages = self.messages.lock().await;
        messages.pop().ok_or(TransportError::NotFound)
    }

    async fn subscribe(&self, _msg_type: MessageType) -> Result<(), TransportError> {
        // Локальный транспорт получает все сообщения
        Ok(())
    }

    async fn unsubscribe(&self, _msg_type: MessageType) -> Result<(), TransportError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_transport() {
        let transport = LocalTransport::new();

        let msg = Message::new(
            "test-1".to_string(),
            "sender".to_string(),
            "receiver".to_string(),
            MessageType::Signal,
        );

        transport.send(msg.clone()).await.unwrap();
        let received = transport.receive().await.unwrap();

        assert_eq!(received.id, "test-1");
        assert_eq!(received.msg_type, MessageType::Signal);
    }

    #[test]
    fn test_message_payload() {
        let msg = Message::new(
            "test-2".to_string(),
            "sender".to_string(),
            "receiver".to_string(),
            MessageType::Command,
        )
        .with_payload("key".to_string(), serde_json::json!("value"));

        assert_eq!(
            msg.get_payload("key"),
            Some(&serde_json::json!("value"))
        );
    }
}
