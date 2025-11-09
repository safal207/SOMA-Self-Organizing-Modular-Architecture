//! # SOMA Bridge - Связь с DAO/Garden
//!
//! Мост между SOMA и внешним миром (DAO, Garden, другие лиминальные модули).
//! Обеспечивает транспортный слой для передачи сообщений и синхронизации.
//!
//! ## Компоненты
//!
//! - **Transport**: Абстракция транспортного слоя
//! - **Message**: Структура сообщения
//! - **MessageType**: Типы передаваемых сообщений
//!
//! ## Поддерживаемые транспорты
//!
//! - **LocalTransport**: Локальный транспорт в памяти (для тестов)
//! - WebSocket (планируется)
//! - libp2p (планируется)
//! - NATS (планируется)
//!
//! ## Примеры
//!
//! ```
//! use soma_bridge::{Message, MessageType, LocalTransport, Transport};
//!
//! #[tokio::main]
//! async fn main() {
//!     let transport = LocalTransport::new();
//!
//!     let msg = Message::new(
//!         "msg-1".to_string(),
//!         "soma".to_string(),
//!         "dao".to_string(),
//!         MessageType::Signal,
//!     );
//!
//!     transport.send(msg).await.unwrap();
//! }
//! ```

pub mod transport;

pub use transport::{
    LocalTransport, Message, MessageType, Transport, TransportError,
};

/// Конфигурация моста
#[derive(Debug, Clone)]
pub struct BridgeConfig {
    /// Идентификатор узла
    pub node_id: String,
    /// Адреса для подключения
    pub endpoints: Vec<String>,
    /// Таймаут соединения в миллисекундах
    pub connection_timeout: u64,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            node_id: "soma-node-1".to_string(),
            endpoints: vec![],
            connection_timeout: 5000,
        }
    }
}

/// Статус моста
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeStatus {
    /// Отключен
    Disconnected,
    /// Подключается
    Connecting,
    /// Подключен
    Connected,
    /// Ошибка
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_config() {
        let config = BridgeConfig::default();
        assert_eq!(config.node_id, "soma-node-1");
        assert_eq!(config.connection_timeout, 5000);
    }

    #[test]
    fn test_bridge_status() {
        let status = BridgeStatus::Disconnected;
        assert_eq!(status, BridgeStatus::Disconnected);
    }
}
