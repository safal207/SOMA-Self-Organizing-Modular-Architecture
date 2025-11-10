# SOMA Cognitive Mesh v1.1

> **«Каждая клетка чувствует не только себя, но и мысль соседей»**

Cognitive Mesh — это слой когнитивного резонанса для SOMA архитектуры, где узлы не просто обмениваются данными, а синхронизируют намерения и гипотезы, образуя коллективный интеллект.

## 🎯 Цель

Добавить слой когнитивного резонанса: узлы не просто обмениваются данными, а синхронизируют намерения и гипотезы. Это начало коллективного интеллекта внутри LIMINAL-сети.

## 🧩 Компоненты

### 1. Cognitive Pulse

Узлы раз в T секунд публикуют короткий пакет смысла:

```rust
{
  "node_id": "alpha",
  "intent": "stabilize",
  "confidence": 0.82,
  "context": ["load_balancing", "adaptive_healing"]
}
```

Соседи вычисляют **semantic overlap** и усиливают связи, где совпадение > τ (0.7):

```rust
if sim(intent_a, intent_b) > 0.7 {
    link.weight += 0.02 * sim;
}
```

Это создаёт **«мысленные кластеры»** — узлы, работающие над схожими идеями, спонтанно образуют когнитивные сообщества.

### 2. Inference Braid (Плетение вывода)

Узлы временно объединяются для решения задачи:
- один генерирует гипотезу
- второй — проверяет
- третий — сводит результат

Пример протокола:

```rust
A: propose("узел gamma перегружен?")
B: simulate(...)
C: summarize("да, latency вырос на 34%")
A: update_memory(...)
```

### 3. Collective Memory

Расширение слоя памяти: теперь сохраняются не только связи, но и **лог когнитивных событий**:

```rust
{
  "task": "stabilize_network",
  "participants": ["A","B","C"],
  "result": "success",
  "confidence": 0.91
}
```

Снимки сохраняются в `liminal-bd/snapshots/cognitive/`.

### 4. Metametric Layer

Ключевые метрики:

- **cognitive_overlap_avg** — среднее совпадение намерений в сети
- **clusters_active_total** — число когнитивных сообществ
- **braid_success_rate** — успешность группового вывода
- **self_reflection_latency_ms** — время отклика сети на самоанализ

## 🧠 Что это даёт

1. **Сеть начинает самоорганизовываться по смыслу**, а не только по нагрузке
2. Возникают **локальные поля сознания** — группы узлов, объединённые общей задачей
3. Это база для **Conscious Feedback 2.0**, где система сможет формировать коллективные инсайты

## 🚀 Использование

### Базовый пример

```rust
use soma_cognitive::{
    pulse::{CognitivePulse, Intent},
    braid::{InferenceBraid, Task, TaskType},
    memory::{CollectiveMemory, CognitiveEvent, EventType, EventResult},
    metrics::CognitiveMetrics,
};

#[tokio::main]
async fn main() {
    // 1. Создать когнитивный пульс
    let pulse = CognitivePulse::new(
        "node_alpha".to_string(),
        Intent::Stabilize,
        0.82,
    );

    // 2. Вычислить semantic overlap с другим узлом
    let other_pulse = CognitivePulse::new(
        "node_beta".to_string(),
        Intent::AdaptiveHealing,
        0.75,
    );

    let overlap = pulse.semantic_overlap(&other_pulse);
    println!("Semantic overlap: {}", overlap);

    // 3. Создать Inference Braid для коллективного решения
    let braid = InferenceBraid::new();
    let task = Task::new(
        "task_001".to_string(),
        TaskType::HypothesisCheck("проверить нагрузку".to_string()),
        "node_alpha".to_string(),
    );

    braid.propose(task).await.unwrap();

    // 4. Записать событие в Collective Memory
    let memory = CollectiveMemory::new(
        PathBuf::from("./snapshots"),
        1000
    );

    let event = CognitiveEvent::new(
        "evt_001".to_string(),
        EventType::IntentSync,
        vec!["node_alpha".to_string(), "node_beta".to_string()],
        EventResult::Success,
        0.95,
    );

    memory.record(event).await;

    // 5. Собрать метрики
    let metrics = CognitiveMetrics::new(100);
    metrics.update_cognitive_overlap(overlap).await;

    let snapshot = metrics.snapshot().await;
    println!("{}", snapshot.to_json().unwrap());
}
```

### Запуск демо

```bash
cargo run --example cognitive_mesh_demo
```

## 📊 Метрики

Метрики можно экспортировать в формате Prometheus:

```rust
let metrics = CognitiveMetrics::new(100);
let prometheus_output = metrics.export_prometheus().await;
```

Пример вывода:

```
# HELP cognitive_overlap_avg Average semantic overlap between nodes
# TYPE cognitive_overlap_avg gauge
cognitive_overlap_avg 0.82

# HELP clusters_active_total Number of active cognitive clusters
# TYPE clusters_active_total gauge
clusters_active_total 3
```

## 🧪 Тестирование

Запустить тесты:

```bash
cargo test -p soma-cognitive
```

## 📁 Структура

```
soma-cognitive/
├── src/
│   ├── lib.rs          # Главный модуль
│   ├── pulse.rs        # Cognitive Pulse
│   ├── braid.rs        # Inference Braid
│   ├── metrics.rs      # Metametric Layer
│   └── memory.rs       # Collective Memory
├── examples/
│   └── cognitive_mesh_demo.rs
├── Cargo.toml
└── README.md
```

## 🔮 Следующие шаги

- **v1.2**: Embedding-based semantic similarity (вместо эвристик)
- **v1.3**: Distributed consensus для Inference Braid
- **v1.4**: Self-reflection loops для автоматического улучшения

## 📄 Лицензия

MIT OR Apache-2.0
