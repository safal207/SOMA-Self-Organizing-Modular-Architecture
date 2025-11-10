//! # Collective Memory - Коллективная память
//!
//! Расширение слоя памяти: сохранение не только связей,
//! но и лога когнитивных событий.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::fs;

/// Когнитивное событие
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEvent {
    /// ID события
    pub id: String,

    /// Тип события
    pub event_type: EventType,

    /// Временная метка
    pub timestamp: u64,

    /// Задача (если применимо)
    pub task: Option<String>,

    /// Участники
    pub participants: Vec<String>,

    /// Результат
    pub result: EventResult,

    /// Уверенность (0.0 - 1.0)
    pub confidence: f64,

    /// Дополнительные данные
    pub metadata: HashMap<String, String>,
}

/// Тип когнитивного события
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    /// Синхронизация намерений
    IntentSync,
    /// Формирование кластера
    ClusterFormation,
    /// Inference Braid выполнен
    BraidExecution,
    /// Самоанализ сети
    SelfReflection,
    /// Адаптация структуры
    StructuralAdaptation,
    /// Кастомное событие
    Custom(String),
}

/// Результат события
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventResult {
    /// Успех
    Success,
    /// Частичный успех
    PartialSuccess(String),
    /// Неудача
    Failure(String),
}

impl CognitiveEvent {
    /// Создать новое когнитивное событие
    pub fn new(
        id: String,
        event_type: EventType,
        participants: Vec<String>,
        result: EventResult,
        confidence: f64,
    ) -> Self {
        Self {
            id,
            event_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            task: None,
            participants,
            result,
            confidence,
            metadata: HashMap::new(),
        }
    }

    /// Установить задачу
    pub fn with_task(mut self, task: String) -> Self {
        self.task = Some(task);
        self
    }

    /// Добавить метаданные
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Проверить успешность события
    pub fn is_successful(&self) -> bool {
        matches!(self.result, EventResult::Success | EventResult::PartialSuccess(_))
    }

    /// Экспортировать в JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Менеджер коллективной памяти
pub struct CollectiveMemory {
    /// События в памяти
    events: Arc<RwLock<Vec<CognitiveEvent>>>,

    /// Путь для сохранения снимков
    snapshot_dir: PathBuf,

    /// Максимальное число событий в памяти
    max_events: usize,
}

impl CollectiveMemory {
    /// Создать новый менеджер памяти
    pub fn new(snapshot_dir: PathBuf, max_events: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            snapshot_dir,
            max_events,
        }
    }

    /// Записать событие
    pub async fn record(&self, event: CognitiveEvent) {
        let mut events = self.events.write().await;
        events.push(event);

        // Ограничить размер памяти
        if events.len() > self.max_events {
            events.remove(0);
        }
    }

    /// Получить все события
    pub async fn all_events(&self) -> Vec<CognitiveEvent> {
        self.events.read().await.clone()
    }

    /// Получить события по типу
    pub async fn events_by_type(&self, event_type: &EventType) -> Vec<CognitiveEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| &e.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Получить события с участием узла
    pub async fn events_by_participant(&self, node_id: &str) -> Vec<CognitiveEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.participants.contains(&node_id.to_string()))
            .cloned()
            .collect()
    }

    /// Получить успешные события
    pub async fn successful_events(&self) -> Vec<CognitiveEvent> {
        let events = self.events.read().await;
        events
            .iter()
            .filter(|e| e.is_successful())
            .cloned()
            .collect()
    }

    /// Вычислить статистику успешности
    pub async fn success_rate(&self) -> f64 {
        let events = self.events.read().await;

        if events.is_empty() {
            return 0.0;
        }

        let successful = events.iter().filter(|e| e.is_successful()).count();
        successful as f64 / events.len() as f64
    }

    /// Сохранить снимок памяти на диск
    pub async fn save_snapshot(&self, name: &str) -> Result<PathBuf, std::io::Error> {
        // Создать директорию если не существует
        fs::create_dir_all(&self.snapshot_dir).await?;

        let events = self.events.read().await;
        let snapshot = serde_json::to_string_pretty(&*events)
            .map_err(std::io::Error::other)?;

        let file_path = self.snapshot_dir.join(format!("{}.json", name));
        fs::write(&file_path, snapshot).await?;

        Ok(file_path)
    }

    /// Загрузить снимок памяти с диска
    pub async fn load_snapshot(&self, name: &str) -> Result<(), std::io::Error> {
        let file_path = self.snapshot_dir.join(format!("{}.json", name));
        let content = fs::read_to_string(&file_path).await?;

        let loaded_events: Vec<CognitiveEvent> = serde_json::from_str(&content)
            .map_err(std::io::Error::other)?;

        let mut events = self.events.write().await;
        *events = loaded_events;

        Ok(())
    }

    /// Очистить память
    pub async fn clear(&self) {
        let mut events = self.events.write().await;
        events.clear();
    }

    /// Получить статистику по участникам
    pub async fn participant_stats(&self) -> HashMap<String, ParticipantStats> {
        let events = self.events.read().await;
        let mut stats: HashMap<String, ParticipantStats> = HashMap::new();

        for event in events.iter() {
            for participant in &event.participants {
                let entry = stats.entry(participant.clone()).or_insert(ParticipantStats {
                    total_events: 0,
                    successful_events: 0,
                    avg_confidence: 0.0,
                    total_confidence: 0.0,
                });

                entry.total_events += 1;
                if event.is_successful() {
                    entry.successful_events += 1;
                }
                entry.total_confidence += event.confidence;
            }
        }

        // Вычислить средние значения
        for stat in stats.values_mut() {
            stat.avg_confidence = stat.total_confidence / stat.total_events as f64;
        }

        stats
    }

    /// Получить последние N событий
    pub async fn recent_events(&self, n: usize) -> Vec<CognitiveEvent> {
        let events = self.events.read().await;
        let start = events.len().saturating_sub(n);
        events[start..].to_vec()
    }
}

/// Статистика участника
#[derive(Debug, Clone)]
pub struct ParticipantStats {
    /// Общее число событий
    pub total_events: usize,

    /// Число успешных событий
    pub successful_events: usize,

    /// Средняя уверенность
    pub avg_confidence: f64,

    /// Сумма уверенности (для вычислений)
    total_confidence: f64,
}

impl ParticipantStats {
    /// Вычислить success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_events == 0 {
            return 0.0;
        }
        self.successful_events as f64 / self.total_events as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cognitive_event() {
        let event = CognitiveEvent::new(
            "event_001".to_string(),
            EventType::IntentSync,
            vec!["node_a".to_string(), "node_b".to_string()],
            EventResult::Success,
            0.95,
        );

        assert_eq!(event.id, "event_001");
        assert_eq!(event.participants.len(), 2);
        assert!(event.is_successful());
    }

    #[tokio::test]
    async fn test_collective_memory() {
        let memory = CollectiveMemory::new(PathBuf::from("/tmp/soma-test"), 100);

        let event = CognitiveEvent::new(
            "event_001".to_string(),
            EventType::BraidExecution,
            vec!["node_alpha".to_string()],
            EventResult::Success,
            0.88,
        );

        memory.record(event).await;

        let all_events = memory.all_events().await;
        assert_eq!(all_events.len(), 1);
    }

    #[tokio::test]
    async fn test_success_rate() {
        let memory = CollectiveMemory::new(PathBuf::from("/tmp/soma-test"), 100);

        for i in 0..10 {
            let result = if i < 7 {
                EventResult::Success
            } else {
                EventResult::Failure("test".to_string())
            };

            let event = CognitiveEvent::new(
                format!("event_{}", i),
                EventType::IntentSync,
                vec!["node_a".to_string()],
                result,
                0.8,
            );

            memory.record(event).await;
        }

        let rate = memory.success_rate().await;
        assert!((rate - 0.7).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_events_by_type() {
        let memory = CollectiveMemory::new(PathBuf::from("/tmp/soma-test"), 100);

        let event1 = CognitiveEvent::new(
            "e1".to_string(),
            EventType::IntentSync,
            vec!["node_a".to_string()],
            EventResult::Success,
            0.9,
        );

        let event2 = CognitiveEvent::new(
            "e2".to_string(),
            EventType::BraidExecution,
            vec!["node_b".to_string()],
            EventResult::Success,
            0.8,
        );

        memory.record(event1).await;
        memory.record(event2).await;

        let intent_events = memory.events_by_type(&EventType::IntentSync).await;
        assert_eq!(intent_events.len(), 1);
    }
}
