use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Роли клеток - разные специализации процессоров
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CellRole {
    /// Сенсорная клетка - обработка входных данных
    Sensor,
    /// Логическая клетка - вычисления и обработка
    Logic,
    /// Моторная клетка - управление и выход
    Motor,
}

impl CellRole {
    /// Получить описание роли
    pub fn description(&self) -> &str {
        match self {
            CellRole::Sensor => "Sensor - processes input data",
            CellRole::Logic => "Logic - performs computations",
            CellRole::Motor => "Motor - controls output and actions",
        }
    }
}

/// Информация о клетке в системе
#[derive(Debug, Clone)]
pub struct CellInfo {
    /// Уникальный идентификатор клетки
    pub id: String,
    /// Роль клетки
    pub role: CellRole,
    /// Время создания (timestamp в миллисекундах)
    pub birth_time: u64,
    /// Поколение (генерация от стволовой клетки)
    pub generation: u32,
    /// Текущая активность
    pub activity: f64,
}

impl CellInfo {
    /// Создать новую информацию о клетке
    pub fn new(id: String, role: CellRole, generation: u32) -> Self {
        Self {
            id,
            role,
            birth_time: current_timestamp_millis(),
            generation,
            activity: 0.0,
        }
    }

    /// Получить возраст клетки в миллисекундах
    pub fn age_millis(&self) -> u64 {
        current_timestamp_millis() - self.birth_time
    }
}

/// Стволовой процессор - ядро системы, порождающее новые клетки
///
/// StemProcessor наблюдает за нагрузкой системы и создаёт новые
/// специализированные клетки когда это необходимо.
#[derive(Debug)]
pub struct StemProcessor {
    /// Уникальный идентификатор стволового процессора
    pub id: String,
    /// Текущее поколение (счётчик делений)
    pub generation: u32,
    /// Реестр всех клеток в системе
    pub cells: HashMap<String, CellInfo>,
    /// Текущая нагрузка системы (0.0 - 1.0)
    pub load: f64,
    /// Порог нагрузки для деления
    pub threshold: f64,
    /// Коэффициент сглаживания нагрузки
    pub smoothing: f64,
    /// Счётчик статистики по ролям
    role_stats: HashMap<CellRole, usize>,
}

impl StemProcessor {
    /// Создать новый стволовой процессор
    pub fn new() -> Self {
        Self {
            id: format!("stem_{}", current_timestamp_millis()),
            generation: 0,
            cells: HashMap::new(),
            load: 0.0,
            threshold: 0.7,
            smoothing: 0.9,
            role_stats: HashMap::new(),
        }
    }

    /// Создать процессор с настраиваемыми параметрами
    pub fn with_params(threshold: f64, smoothing: f64) -> Self {
        Self {
            id: format!("stem_{}", current_timestamp_millis()),
            generation: 0,
            cells: HashMap::new(),
            load: 0.0,
            threshold: threshold.clamp(0.0, 1.0),
            smoothing: smoothing.clamp(0.0, 1.0),
            role_stats: HashMap::new(),
        }
    }

    /// Воспринять активность сети (Sense)
    ///
    /// Обновляет нагрузку системы и инициирует деление при необходимости
    pub fn sense(&mut self, activity: f64) {
        // Экспоненциальное сглаживание нагрузки
        self.load = (self.load * self.smoothing) + (activity * (1.0 - self.smoothing));

        // Если нагрузка превышает порог - делимся
        if self.load > self.threshold {
            self.divide();
        }
    }

    /// Деление - создание новой клетки (Align)
    fn divide(&mut self) {
        self.generation += 1;

        // Выбираем роль для новой клетки
        let role = self.choose_role();

        // Создаём уникальный ID
        let id = format!("cell_{}_{}", self.generation, self.cells.len() + 1);

        // Создаём информацию о клетке
        let cell_info = CellInfo::new(id.clone(), role, self.generation);

        // Добавляем в реестр
        self.cells.insert(id, cell_info);

        // Обновляем статистику
        *self.role_stats.entry(role).or_insert(0) += 1;

        // Сбрасываем нагрузку после деления
        self.load *= 0.5;
    }

    /// Выбрать роль для новой клетки на основе текущего состояния
    fn choose_role(&self) -> CellRole {
        // Простая эвристика: циклическое распределение с учётом баланса
        let sensor_count = self.role_stats.get(&CellRole::Sensor).unwrap_or(&0);
        let logic_count = self.role_stats.get(&CellRole::Logic).unwrap_or(&0);
        let motor_count = self.role_stats.get(&CellRole::Motor).unwrap_or(&0);

        // Выбираем наименее представленную роль
        if sensor_count <= logic_count && sensor_count <= motor_count {
            CellRole::Sensor
        } else if logic_count <= motor_count {
            CellRole::Logic
        } else {
            CellRole::Motor
        }
    }

    /// Тик системы - обновление состояния всех клеток (Flow)
    pub fn tick(&mut self) {
        // Обновляем активность каждой клетки
        for cell in self.cells.values_mut() {
            // Простое затухание активности
            cell.activity *= 0.95;
        }
    }

    /// Получить количество клеток
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Получить статистику по ролям
    pub fn role_distribution(&self) -> HashMap<CellRole, usize> {
        self.role_stats.clone()
    }

    /// Удалить клетку (апоптоз)
    pub fn remove_cell(&mut self, id: &str) -> Option<CellInfo> {
        if let Some(cell) = self.cells.remove(id) {
            if let Some(count) = self.role_stats.get_mut(&cell.role) {
                *count = count.saturating_sub(1);
            }
            Some(cell)
        } else {
            None
        }
    }

    /// Получить список всех клеток
    pub fn cells(&self) -> &HashMap<String, CellInfo> {
        &self.cells
    }
}

impl Default for StemProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Получить текущую временную метку в миллисекундах
fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stem_processor_creation() {
        let stem = StemProcessor::new();
        assert_eq!(stem.generation, 0);
        assert_eq!(stem.cell_count(), 0);
        assert_eq!(stem.load, 0.0);
    }

    #[test]
    fn test_cell_division() {
        let mut stem = StemProcessor::with_params(0.3, 0.5);

        // Стимулируем деление несколько раз для накопления нагрузки
        stem.sense(0.6);
        stem.sense(0.6);

        assert_eq!(stem.generation, 1);
        assert_eq!(stem.cell_count(), 1);
        assert!(stem.load < 0.6); // Нагрузка должна снизиться после деления
    }

    #[test]
    fn test_role_distribution() {
        let mut stem = StemProcessor::with_params(0.3, 0.5);

        // Создаём несколько клеток
        for _ in 0..6 {
            stem.sense(0.5);
        }

        let distribution = stem.role_distribution();
        let total: usize = distribution.values().sum();
        assert_eq!(total, stem.cell_count());
    }

    #[test]
    fn test_cell_removal() {
        let mut stem = StemProcessor::with_params(0.3, 0.5);

        // Создаём клетку
        stem.sense(0.6);
        stem.sense(0.6);

        assert!(stem.cell_count() > 0);
        let initial_count = stem.cell_count();

        let cell_id = stem.cells.keys().next().unwrap().clone();
        let removed = stem.remove_cell(&cell_id);

        assert!(removed.is_some());
        assert_eq!(stem.cell_count(), initial_count - 1);
    }

    #[test]
    fn test_cell_info_age() {
        let cell = CellInfo::new("test".to_string(), CellRole::Sensor, 1);
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(cell.age_millis() >= 10);
    }
}
