/// Базовый трейт клетки в SOMA - фундаментальная единица обработки
///
/// Реализует паттерн Sense-Align-Flow:
/// - Sense: восприятие входящего сигнала
/// - Align: внутреннее выравнивание/обработка
/// - Flow: генерация выходящего потока
pub trait Cell {
    /// Воспринять входящий сигнал
    fn sense(&mut self, input: f64);

    /// Выровнять внутреннее состояние
    fn align(&mut self);

    /// Сгенерировать выходной поток
    fn flow(&self) -> f64;
}

/// Стволовая клетка - универсальная недифференцированная клетка
///
/// Может трансформироваться в любой специализированный тип клетки,
/// передавая свой резонанс как начальное состояние
pub struct StemCell {
    /// Базовый резонанс клетки (0.0 - 1.0)
    pub resonance: f64,
}

impl StemCell {
    /// Создать новую стволовую клетку с нейтральным резонансом
    pub fn new() -> Self {
        Self { resonance: 0.5 }
    }

    /// Создать стволовую клетку с заданным резонансом
    pub fn with_resonance(resonance: f64) -> Self {
        Self {
            resonance: resonance.clamp(0.0, 1.0)
        }
    }

    /// Дифференцировать стволовую клетку в специализированный тип
    ///
    /// Передаёт свой резонанс новой клетке и инициализирует её
    pub fn differentiate<T: Cell>(self, mut new_cell: T) -> T {
        // Передаём резонанс через sense
        new_cell.sense(self.resonance);
        // Инициализируем внутреннее состояние
        new_cell.align();
        new_cell
    }
}

impl Default for StemCell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCell {
        value: f64,
    }

    impl Cell for TestCell {
        fn sense(&mut self, input: f64) {
            self.value = input;
        }

        fn align(&mut self) {
            self.value *= 2.0;
        }

        fn flow(&self) -> f64 {
            self.value
        }
    }

    #[test]
    fn test_stem_cell_differentiation() {
        let stem = StemCell::with_resonance(0.3);
        let test_cell = TestCell { value: 0.0 };

        let differentiated = stem.differentiate(test_cell);

        // После дифференциации: sense(0.3) -> value=0.3, align() -> value=0.6
        assert_eq!(differentiated.flow(), 0.6);
    }
}
