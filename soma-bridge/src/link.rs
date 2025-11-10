use crate::Signal;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Канал связи между нейронами/узлами
///
/// Link обеспечивает буферизованную передачу сигналов между компонентами.
/// Использует Arc<Mutex<>> для безопасного совместного доступа из разных потоков.
#[derive(Clone)]
pub struct Link {
    /// Буфер сигналов
    buffer: Arc<Mutex<VecDeque<Signal>>>,
    /// Максимальный размер буфера (0 = без ограничений)
    max_size: usize,
}

impl Link {
    /// Создать новый канал связи без ограничения размера
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            max_size: 0,
        }
    }

    /// Создать канал с ограничением размера буфера
    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
        }
    }

    /// Отправить сигнал в канал
    ///
    /// Если буфер полон (при установленном max_size), старые сигналы удаляются
    pub fn send(&self, signal: Signal) {
        let mut buf = self.buffer.lock().unwrap();

        // Если есть ограничение и буфер полон, удаляем старый сигнал
        if self.max_size > 0 && buf.len() >= self.max_size {
            buf.pop_front();
        }

        buf.push_back(signal);
    }

    /// Получить следующий сигнал из канала
    ///
    /// Возвращает None, если канал пуст
    pub fn receive(&self) -> Option<Signal> {
        let mut buf = self.buffer.lock().unwrap();
        buf.pop_front()
    }

    /// Проверить, пуст ли канал
    pub fn is_empty(&self) -> bool {
        let buf = self.buffer.lock().unwrap();
        buf.is_empty()
    }

    /// Получить количество сигналов в буфере
    pub fn len(&self) -> usize {
        let buf = self.buffer.lock().unwrap();
        buf.len()
    }

    /// Очистить буфер
    pub fn clear(&self) {
        let mut buf = self.buffer.lock().unwrap();
        buf.clear();
    }

    /// Получить все доступные сигналы
    pub fn drain(&self) -> Vec<Signal> {
        let mut buf = self.buffer.lock().unwrap();
        buf.drain(..).collect()
    }
}

impl Default for Link {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_send_receive() {
        let link = Link::new();
        let signal = Signal::new("test", 0.75);

        link.send(signal.clone());
        let received = link.receive().unwrap();

        assert_eq!(received.id, "test");
        assert_eq!(received.value, 0.75);
    }

    #[test]
    fn test_link_empty() {
        let link = Link::new();
        assert!(link.is_empty());
        assert_eq!(link.len(), 0);

        link.send(Signal::new("test", 0.5));
        assert!(!link.is_empty());
        assert_eq!(link.len(), 1);

        link.receive();
        assert!(link.is_empty());
    }

    #[test]
    fn test_link_capacity() {
        let link = Link::with_capacity(2);

        link.send(Signal::new("1", 0.1));
        link.send(Signal::new("2", 0.2));
        link.send(Signal::new("3", 0.3)); // Должен вытеснить "1"

        assert_eq!(link.len(), 2);
        let first = link.receive().unwrap();
        assert_eq!(first.id, "2"); // "1" был удалён
    }

    #[test]
    fn test_link_clone() {
        let link1 = Link::new();
        let link2 = link1.clone();

        link1.send(Signal::new("shared", 0.8));

        // Оба link указывают на один буфер
        let received = link2.receive().unwrap();
        assert_eq!(received.id, "shared");
    }

    #[test]
    fn test_link_drain() {
        let link = Link::new();

        link.send(Signal::new("1", 0.1));
        link.send(Signal::new("2", 0.2));
        link.send(Signal::new("3", 0.3));

        let signals = link.drain();
        assert_eq!(signals.len(), 3);
        assert!(link.is_empty());
    }
}
