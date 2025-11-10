use soma_core::Resonance;
use soma_vnp::NeuronLayer;
use std::collections::HashMap;

/// Визуализация состояния SOMA
pub struct Visualizer {
    /// История резонансов для отображения
    resonance_history: Vec<ResonanceSnapshot>,
    /// Максимальная длина истории
    max_history: usize,
}

impl Visualizer {
    /// Создать новый визуализатор
    pub fn new() -> Self {
        Self {
            resonance_history: Vec::new(),
            max_history: 100,
        }
    }

    /// Создать визуализатор с заданной длиной истории
    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            resonance_history: Vec::new(),
            max_history,
        }
    }

    /// Записать снимок текущего состояния
    pub fn record_snapshot(&mut self, snapshot: ResonanceSnapshot) {
        self.resonance_history.push(snapshot);

        // Ограничиваем размер истории
        if self.resonance_history.len() > self.max_history {
            self.resonance_history.remove(0);
        }
    }

    /// Получить историю резонансов
    pub fn history(&self) -> &[ResonanceSnapshot] {
        &self.resonance_history
    }

    /// Очистить историю
    pub fn clear(&mut self) {
        self.resonance_history.clear();
    }

    /// Отобразить текущее состояние в консоль (ASCII визуализация)
    pub fn display_ascii(&self) {
        if self.resonance_history.is_empty() {
            println!("No resonance data to display");
            return;
        }

        println!("\n╔═══════════════════════════════════════╗");
        println!("║     SOMA Resonance Visualization     ║");
        println!("╚═══════════════════════════════════════╝");

        // Отображаем последние N записей
        let display_count = self.resonance_history.len().min(10);
        let start = self.resonance_history.len() - display_count;

        for (i, snapshot) in self.resonance_history[start..].iter().enumerate() {
            println!("\nSnapshot {} (t={}ms):", start + i, snapshot.timestamp);

            for (name, value) in &snapshot.values {
                let bar = create_bar(*value, 40);
                println!("  {:<15} │{}│ {:.3}", name, bar, value);
            }
        }

        println!("\n");
    }

    /// Экспортировать данные в JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.resonance_history)
    }

    /// Получить статистику
    pub fn stats(&self) -> VisualizationStats {
        if self.resonance_history.is_empty() {
            return VisualizationStats::default();
        }

        let mut all_values: Vec<f64> = Vec::new();
        for snapshot in &self.resonance_history {
            all_values.extend(snapshot.values.values());
        }

        let sum: f64 = all_values.iter().sum();
        let mean = sum / all_values.len() as f64;

        let min = all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        VisualizationStats {
            snapshot_count: self.resonance_history.len(),
            mean_resonance: mean,
            min_resonance: min,
            max_resonance: max,
        }
    }
}

impl Default for Visualizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Снимок резонансов в определённый момент времени
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResonanceSnapshot {
    /// Временная метка
    pub timestamp: u64,
    /// Значения резонансов различных компонентов
    pub values: HashMap<String, Resonance>,
}

impl ResonanceSnapshot {
    /// Создать новый снимок
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            values: HashMap::new(),
        }
    }

    /// Добавить значение
    pub fn add(&mut self, name: String, value: Resonance) {
        self.values.insert(name, value);
    }

    /// Создать снимок с одним значением
    pub fn with_value(timestamp: u64, name: String, value: Resonance) -> Self {
        let mut snapshot = Self::new(timestamp);
        snapshot.add(name, value);
        snapshot
    }
}

/// Статистика визуализации
#[derive(Debug, Clone)]
pub struct VisualizationStats {
    pub snapshot_count: usize,
    pub mean_resonance: f64,
    pub min_resonance: f64,
    pub max_resonance: f64,
}

impl Default for VisualizationStats {
    fn default() -> Self {
        Self {
            snapshot_count: 0,
            mean_resonance: 0.0,
            min_resonance: 0.0,
            max_resonance: 0.0,
        }
    }
}

/// Создать ASCII бар для значения от 0.0 до 1.0
fn create_bar(value: f64, width: usize) -> String {
    let filled = (value * width as f64) as usize;
    let empty = width - filled;

    let mut bar = String::new();
    for _ in 0..filled {
        bar.push('█');
    }
    for _ in 0..empty {
        bar.push('░');
    }

    bar
}

/// Симулятор SOMA - выполняет пошаговую симуляцию
pub struct Simulator {
    /// Текущий шаг симуляции
    step: u64,
    /// Визуализатор
    visualizer: Visualizer,
}

impl Simulator {
    /// Создать новый симулятор
    pub fn new() -> Self {
        Self {
            step: 0,
            visualizer: Visualizer::new(),
        }
    }

    /// Выполнить один шаг симуляции
    pub fn step(&mut self, neurons: &mut NeuronLayer, inputs: &[f64]) -> ResonanceSnapshot {
        self.step += 1;

        // Обрабатываем входы
        let _outputs = neurons.process(inputs);

        // Создаём снимок состояния
        let mut snapshot = ResonanceSnapshot::new(self.step);

        for i in 0..neurons.len() {
            if let Some(neuron) = neurons.neuron(i) {
                snapshot.add(format!("neuron_{}", i), neuron.potential());
            }
        }

        // Записываем в визуализатор
        self.visualizer.record_snapshot(snapshot.clone());

        snapshot
    }

    /// Получить текущий шаг
    pub fn current_step(&self) -> u64 {
        self.step
    }

    /// Получить визуализатор
    pub fn visualizer(&self) -> &Visualizer {
        &self.visualizer
    }

    /// Получить мутабельный визуализатор
    pub fn visualizer_mut(&mut self) -> &mut Visualizer {
        &mut self.visualizer
    }

    /// Сбросить симуляцию
    pub fn reset(&mut self) {
        self.step = 0;
        self.visualizer.clear();
    }
}

impl Default for Simulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer() {
        let mut viz = Visualizer::new();
        let snapshot = ResonanceSnapshot::with_value(1, "test".to_string(), 0.75);

        viz.record_snapshot(snapshot);

        assert_eq!(viz.history().len(), 1);
        assert_eq!(viz.history()[0].values.get("test"), Some(&0.75));
    }

    #[test]
    fn test_max_history() {
        let mut viz = Visualizer::with_max_history(3);

        for i in 0..5 {
            let snapshot = ResonanceSnapshot::with_value(i, "test".to_string(), 0.5);
            viz.record_snapshot(snapshot);
        }

        assert_eq!(viz.history().len(), 3);
    }

    #[test]
    fn test_simulator() {
        let mut sim = Simulator::new();
        let mut layer = NeuronLayer::new(3);
        let inputs = vec![0.7, 0.8, 0.6];

        let snapshot = sim.step(&mut layer, &inputs);

        assert_eq!(sim.current_step(), 1);
        assert_eq!(snapshot.values.len(), 3);
    }

    #[test]
    fn test_create_bar() {
        let bar = create_bar(0.5, 10);
        assert_eq!(bar.len(), 10);
    }

    #[test]
    fn test_stats() {
        let mut viz = Visualizer::new();

        let mut snapshot1 = ResonanceSnapshot::new(1);
        snapshot1.add("a".to_string(), 0.3);
        snapshot1.add("b".to_string(), 0.7);

        let mut snapshot2 = ResonanceSnapshot::new(2);
        snapshot2.add("a".to_string(), 0.5);
        snapshot2.add("b".to_string(), 0.9);

        viz.record_snapshot(snapshot1);
        viz.record_snapshot(snapshot2);

        let stats = viz.stats();
        assert_eq!(stats.snapshot_count, 2);
        assert_eq!(stats.mean_resonance, 0.6);
        assert_eq!(stats.min_resonance, 0.3);
        assert_eq!(stats.max_resonance, 0.9);
    }
}
