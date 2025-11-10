use soma_core::Cell;
use std::time::Instant;

/// Виртуальный нейрон - базовая вычислительная единица SOMA
///
/// Реализует паттерн Sense-Align-Flow с внутренним состоянием и памятью.
/// Нейроны могут накапливать энергию и активироваться при достижении порога.
pub struct Neuron {
    /// Текущий потенциал нейрона
    potential: f64,
    /// Порог активации
    threshold: f64,
    /// Коэффициент затухания (leak)
    decay: f64,
    /// Накопленная память/вес
    weight: f64,
    /// Время последнего обновления (для временного затухания)
    last_update: Instant,
}

impl Neuron {
    /// Создать новый нейрон с параметрами по умолчанию
    pub fn new() -> Self {
        Self {
            potential: 0.0,
            threshold: 0.7,
            decay: 0.1,
            weight: 1.0,
            last_update: Instant::now(),
        }
    }

    /// Создать нейрон с заданными параметрами
    pub fn with_params(threshold: f64, decay: f64, weight: f64) -> Self {
        Self {
            potential: 0.0,
            threshold: threshold.clamp(0.0, 1.0),
            decay: decay.clamp(0.0, 1.0),
            weight: weight.clamp(0.0, 10.0),
            last_update: Instant::now(),
        }
    }

    /// Проверить, активирован ли нейрон
    pub fn is_activated(&self) -> bool {
        self.potential >= self.threshold
    }

    /// Получить текущий потенциал
    pub fn potential(&self) -> f64 {
        self.potential
    }

    /// Установить порог активации
    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }

    /// Получить вес нейрона
    pub fn weight(&self) -> f64 {
        self.weight
    }

    /// Обучить нейрон (модифицировать вес)
    pub fn train(&mut self, delta: f64) {
        self.weight = (self.weight + delta).clamp(0.0, 10.0);
    }

    /// Стимулировать нейрон входным сигналом (для pulse-режима)
    ///
    /// Возвращает true, если нейрон сработал (fired)
    pub fn stimulate(&mut self, input: f64) -> bool {
        self.potential += input;
        if self.potential >= self.threshold {
            let fired = true;
            self.potential = 0.0; // Сброс после активации
            fired
        } else {
            false
        }
    }

    /// Применить временное затухание (для pulse-режима)
    ///
    /// Учитывает реальное время, прошедшее с последнего обновления
    pub fn time_based_decay(&mut self) {
        let elapsed = self.last_update.elapsed().as_secs_f64();
        self.potential *= (1.0 - self.decay * elapsed).max(0.0);
        self.last_update = Instant::now();
    }

    /// Получить нормализованное состояние (0.0 - 1.0)
    pub fn get_state(&self) -> f64 {
        self.potential / self.threshold
    }
}

impl Default for Neuron {
    fn default() -> Self {
        Self::new()
    }
}

impl Cell for Neuron {
    /// Воспринять входящий сигнал и добавить к потенциалу
    fn sense(&mut self, input: f64) {
        self.potential += input * self.weight;
    }

    /// Применить затухание и нормализовать потенциал
    fn align(&mut self) {
        // Применяем затухание
        self.potential *= 1.0 - self.decay;
        // Нормализуем в диапазон [0, 1]
        self.potential = self.potential.clamp(0.0, 1.0);
    }

    /// Вернуть выходной сигнал (активация или затухание)
    fn flow(&self) -> f64 {
        if self.is_activated() {
            self.potential
        } else {
            0.0
        }
    }
}

/// Слой нейронов - коллекция связанных нейронов
pub struct NeuronLayer {
    neurons: Vec<Neuron>,
}

impl NeuronLayer {
    /// Создать слой с заданным количеством нейронов
    pub fn new(count: usize) -> Self {
        Self {
            neurons: (0..count).map(|_| Neuron::new()).collect(),
        }
    }

    /// Количество нейронов в слое
    pub fn len(&self) -> usize {
        self.neurons.len()
    }

    /// Проверить, пустой ли слой
    pub fn is_empty(&self) -> bool {
        self.neurons.is_empty()
    }

    /// Обработать входные данные через весь слой
    pub fn process(&mut self, inputs: &[f64]) -> Vec<f64> {
        self.neurons
            .iter_mut()
            .enumerate()
            .map(|(i, neuron)| {
                if i < inputs.len() {
                    neuron.sense(inputs[i]);
                }
                neuron.align();
                neuron.flow()
            })
            .collect()
    }

    /// Получить доступ к нейрону по индексу
    pub fn neuron(&self, index: usize) -> Option<&Neuron> {
        self.neurons.get(index)
    }

    /// Получить мутабельный доступ к нейрону
    pub fn neuron_mut(&mut self, index: usize) -> Option<&mut Neuron> {
        self.neurons.get_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_activation() {
        let mut neuron = Neuron::new();
        neuron.sense(0.8);
        neuron.align();

        assert!(neuron.is_activated());
        assert!(neuron.flow() > 0.0);
    }

    #[test]
    fn test_neuron_below_threshold() {
        let mut neuron = Neuron::new();
        neuron.sense(0.3);
        neuron.align();

        assert!(!neuron.is_activated());
        assert_eq!(neuron.flow(), 0.0);
    }

    #[test]
    fn test_neuron_decay() {
        let mut neuron = Neuron::with_params(0.5, 0.2, 1.0);
        neuron.sense(0.8);
        neuron.align();

        let potential_before = neuron.potential();
        neuron.align(); // Второй раз - только затухание
        let potential_after = neuron.potential();

        assert!(potential_after < potential_before);
    }

    #[test]
    fn test_neuron_layer() {
        let mut layer = NeuronLayer::new(3);
        let inputs = vec![0.8, 0.6, 0.4];

        let outputs = layer.process(&inputs);

        assert_eq!(outputs.len(), 3);
        assert!(outputs[0] > 0.0); // Должен активироваться
    }
}
