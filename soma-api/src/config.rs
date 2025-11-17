//! Конфигурационные константы для SOMA API
//!
//! Централизованное хранение всех магических чисел и параметров конфигурации

/// Таймауты и интервалы (в миллисекундах)
pub mod timeouts {
    /// Таймаут для определения живого peer (15 секунд)
    pub const PEER_ALIVE_TIMEOUT_MS: i64 = 15_000;

    /// Интервал heartbeat сообщений (3 секунды)
    pub const HEARTBEAT_INTERVAL_SEC: u64 = 3;

    /// Интервал cleanup цикла (10 секунд)
    pub const CLEANUP_INTERVAL_SEC: u64 = 10;

    /// Интервал переподключения (30 секунд)
    pub const RECONNECT_INTERVAL_SEC: u64 = 30;

    /// Окно совпадения вспышек для Hebbian обучения (120 мс)
    pub const HEBBIAN_FIRE_WINDOW_MS: i64 = 120;
}

/// Параметры здоровья соединения
pub mod health {
    /// Минимальное качество для здорового соединения
    pub const MIN_HEALTHY_QUALITY: f64 = 0.5;

    /// Шаг восстановления качества при успехе
    pub const QUALITY_RECOVERY_STEP: f64 = 0.1;

    /// Шаг деградации качества при ошибке
    pub const QUALITY_DEGRADATION_STEP: f64 = 0.2;
}

/// Параметры резонанса
pub mod resonance {
    /// Минимальная сила резонанса (слабая сеть)
    pub const MIN_STRENGTH: f64 = 0.05;

    /// Базовая сила резонанса (без peers)
    pub const BASE_STRENGTH: f64 = 0.1;

    /// Максимальная сила резонанса (здоровая сеть)
    pub const MAX_STRENGTH: f64 = 0.2;

    /// Диапазон силы резонанса
    pub const STRENGTH_RANGE: f64 = MAX_STRENGTH - MIN_STRENGTH;
}

/// Параметры Hebbian обучения
pub mod hebbian {
    /// Минимальный вес связи
    pub const WEIGHT_MIN: f64 = 0.1;

    /// Максимальный вес связи
    pub const WEIGHT_MAX: f64 = 1.0;

    /// Начальный вес связи
    pub const WEIGHT_INITIAL: f64 = 0.3;

    /// Скорость обучения при совпадении (co-fire)
    pub const ETA_POSITIVE: f64 = 0.06;

    /// Скорость наказания при рассинхроне (anti-fire)
    pub const ETA_NEGATIVE: f64 = 0.03;

    /// Скорость забывания (сек^-1)
    pub const DECAY_RATE: f64 = 0.002;
}

/// Параметры API
pub mod api {
    /// Дефолтный порт сервера
    pub const DEFAULT_PORT: u16 = 8080;

    /// Размер broadcast канала для сигналов
    pub const SIGNAL_CHANNEL_SIZE: usize = 100;

    /// Интервал обновления фонового процесса (мс)
    pub const BACKGROUND_UPDATE_INTERVAL_MS: u64 = 100;

    /// Интервал отправки состояния через WebSocket (сек)
    pub const WEBSOCKET_STATE_INTERVAL_SEC: u64 = 1;

    /// Интервал синхронизации состояния mesh (сек)
    pub const MESH_STATE_SYNC_INTERVAL_SEC: u64 = 5;

    /// Интервал синхронизации резонанса mesh (мс)
    pub const MESH_RESONANCE_SYNC_INTERVAL_MS: u64 = 500;

    /// Интервал цикла осознанности (сек)
    pub const CONSCIOUS_CYCLE_INTERVAL_SEC: u64 = 5;

    /// Окно анализа для рефлексии (мс)
    pub const REFLECTION_ANALYSIS_WINDOW_MS: i64 = 60_000;

    /// Количество последних traces для API (по умолчанию)
    pub const DEFAULT_TRACES_LIMIT: usize = 50;

    /// Количество последних insights для API (по умолчанию)
    pub const DEFAULT_INSIGHTS_LIMIT: usize = 20;

    /// Количество последних решений Domino (по умолчанию)
    pub const DEFAULT_DECISIONS_LIMIT: usize = 50;

    /// Количество топ связей для topology endpoint
    pub const DEFAULT_TOP_LINKS_COUNT: usize = 10;
}

