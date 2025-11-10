//! # Inference Braid - Плетение вывода
//!
//! Узлы временно объединяются для решения задачи:
//! один генерирует гипотезу, второй проверяет, третий сводит результат.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Тип задачи для коллективного решения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// Проверка гипотезы
    HypothesisCheck(String),
    /// Симуляция сценария
    Simulation(String),
    /// Агрегация данных
    DataAggregation(String),
    /// Принятие решения
    Decision(String),
}

/// Задача для Inference Braid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// ID задачи
    pub id: String,

    /// Тип задачи
    pub task_type: TaskType,

    /// Узел-инициатор
    pub initiator: String,

    /// Участники (node_ids)
    pub participants: Vec<String>,

    /// Данные задачи
    pub data: HashMap<String, String>,

    /// Статус выполнения
    pub status: TaskStatus,
}

/// Статус выполнения задачи
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// Инициализирована
    Initialized,
    /// В процессе
    InProgress,
    /// Завершена успешно
    Completed,
    /// Ошибка
    Failed(String),
}

impl Task {
    /// Создать новую задачу
    pub fn new(id: String, task_type: TaskType, initiator: String) -> Self {
        Self {
            id,
            task_type,
            initiator,
            participants: Vec::new(),
            data: HashMap::new(),
            status: TaskStatus::Initialized,
        }
    }

    /// Добавить участника
    pub fn add_participant(&mut self, node_id: String) {
        if !self.participants.contains(&node_id) {
            self.participants.push(node_id);
        }
    }

    /// Установить статус
    pub fn set_status(&mut self, status: TaskStatus) {
        self.status = status;
    }

    /// Добавить данные
    pub fn add_data(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }
}

/// Результат выполнения Inference Braid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraidResult {
    /// ID задачи
    pub task_id: String,

    /// Успех выполнения
    pub success: bool,

    /// Уверенность в результате (0.0 - 1.0)
    pub confidence: f64,

    /// Результат
    pub result: String,

    /// Участники
    pub participants: Vec<String>,

    /// Временные метки
    pub started_at: u64,
    pub completed_at: u64,
}

impl BraidResult {
    /// Создать успешный результат
    pub fn success(
        task_id: String,
        confidence: f64,
        result: String,
        participants: Vec<String>,
    ) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            task_id,
            success: true,
            confidence,
            result,
            participants,
            started_at: now,
            completed_at: now,
        }
    }

    /// Создать неудачный результат
    pub fn failure(task_id: String, error: String, participants: Vec<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            task_id,
            success: false,
            confidence: 0.0,
            result: error,
            participants,
            started_at: now,
            completed_at: now,
        }
    }

    /// Вычислить длительность выполнения
    pub fn duration_secs(&self) -> u64 {
        self.completed_at.saturating_sub(self.started_at)
    }
}

/// Роль узла в Inference Braid
#[derive(Debug, Clone, PartialEq)]
pub enum BraidRole {
    /// Генератор гипотез
    Proposer,
    /// Проверяющий/симулятор
    Validator,
    /// Агрегатор результатов
    Aggregator,
}

/// Менеджер Inference Braid
pub struct InferenceBraid {
    /// Активные задачи
    tasks: Arc<RwLock<HashMap<String, Task>>>,

    /// Канал для коммуникации
    tx: mpsc::Sender<BraidMessage>,
    rx: Arc<RwLock<mpsc::Receiver<BraidMessage>>>,
}

/// Сообщение в Braid-канале
#[derive(Debug, Clone)]
pub enum BraidMessage {
    /// Предложение задачи
    Propose(Task),
    /// Проверка задачи
    Validate(String, String), // task_id, node_id
    /// Агрегация результата
    Aggregate(String, String), // task_id, result
    /// Завершение задачи
    Complete(BraidResult),
}

impl InferenceBraid {
    /// Создать новый Inference Braid
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            tx,
            rx: Arc::new(RwLock::new(rx)),
        }
    }

    /// Предложить задачу
    pub async fn propose(&self, task: Task) -> Result<(), String> {
        let task_id = task.id.clone();
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id.clone(), task.clone());
        }

        self.tx
            .send(BraidMessage::Propose(task))
            .await
            .map_err(|e| format!("Failed to send propose message: {}", e))
    }

    /// Проверить задачу
    pub async fn validate(&self, task_id: String, node_id: String) -> Result<(), String> {
        self.tx
            .send(BraidMessage::Validate(task_id, node_id))
            .await
            .map_err(|e| format!("Failed to send validate message: {}", e))
    }

    /// Агрегировать результат
    pub async fn aggregate(&self, task_id: String, result: String) -> Result<(), String> {
        self.tx
            .send(BraidMessage::Aggregate(task_id, result))
            .await
            .map_err(|e| format!("Failed to send aggregate message: {}", e))
    }

    /// Получить задачу по ID
    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(task_id).cloned()
    }

    /// Получить все активные задачи
    pub async fn active_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        tasks
            .values()
            .filter(|t| t.status == TaskStatus::InProgress)
            .cloned()
            .collect()
    }

    /// Обработать сообщения (должно запускаться в фоне)
    pub async fn process_messages(&self) {
        let mut rx = self.rx.write().await;
        while let Some(msg) = rx.recv().await {
            match msg {
                BraidMessage::Propose(task) => {
                    println!("🧵 Braid: Task proposed - {}", task.id);
                }
                BraidMessage::Validate(task_id, node_id) => {
                    println!("🔍 Braid: Validating task {} by {}", task_id, node_id);
                }
                BraidMessage::Aggregate(task_id, result) => {
                    println!("📊 Braid: Aggregating task {} - {}", task_id, result);
                }
                BraidMessage::Complete(result) => {
                    println!("✅ Braid: Task {} completed - confidence: {}",
                        result.task_id, result.confidence);

                    // Удалить из активных задач
                    let mut tasks = self.tasks.write().await;
                    tasks.remove(&result.task_id);
                }
            }
        }
    }
}

impl Default for InferenceBraid {
    fn default() -> Self {
        Self::new()
    }
}

/// Пример протокола: A -> propose, B -> simulate, C -> summarize
pub async fn example_braid_protocol() {
    println!("🧵 Example Inference Braid Protocol:");
    println!("A: propose('узел gamma перегружен?')");
    println!("B: simulate(...)");
    println!("C: summarize('да, latency вырос на 34%')");
    println!("A: update_memory(...)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "task_001".to_string(),
            TaskType::HypothesisCheck("test".to_string()),
            "node_a".to_string(),
        );

        assert_eq!(task.id, "task_001");
        assert_eq!(task.initiator, "node_a");
        assert_eq!(task.status, TaskStatus::Initialized);
    }

    #[test]
    fn test_task_participants() {
        let mut task = Task::new(
            "task_001".to_string(),
            TaskType::Simulation("load_test".to_string()),
            "node_a".to_string(),
        );

        task.add_participant("node_b".to_string());
        task.add_participant("node_c".to_string());

        assert_eq!(task.participants.len(), 2);
        assert!(task.participants.contains(&"node_b".to_string()));
    }

    #[tokio::test]
    async fn test_inference_braid() {
        let braid = InferenceBraid::new();

        let task = Task::new(
            "task_001".to_string(),
            TaskType::Decision("route_traffic".to_string()),
            "node_alpha".to_string(),
        );

        let result = braid.propose(task).await;
        assert!(result.is_ok());

        let retrieved = braid.get_task("task_001").await;
        assert!(retrieved.is_some());
    }
}
