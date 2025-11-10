# SOMA v1.1 - Cognitive Mesh: Collective Intelligence Layer

## Summary

Добавлен **Cognitive Mesh v1.1** — слой когнитивного резонанса, где узлы синхронизируют намерения и гипотезы, образуя коллективный интеллект в LIMINAL-сети.

### 🎯 Слоган
> **«Каждая клетка чувствует не только себя, но и мысль соседей»**

## Key Features

### 1. 📡 Cognitive Pulse
- Узлы публикуют пакеты смысла с намерениями каждые T секунд
- **Semantic overlap** вычисление через Intent similarity + Jaccard index
- Автоматическое формирование **когнитивных кластеров** при совпадении > 0.7
- Поддержка 5 типов Intent: `Stabilize`, `BalanceLoad`, `AdaptiveHealing`, `Explore`, `Optimize`

**Пример:**
```rust
let pulse = CognitivePulse::new("node_alpha".to_string(), Intent::Stabilize, 0.82);
let overlap = pulse_a.semantic_overlap(&pulse_b); // 0.0 - 1.0
```

### 2. 🧵 Inference Braid (Плетение вывода)
- Временное объединение узлов для коллективного решения задач
- Протокол: **propose → validate → aggregate**
- Асинхронные каналы для коммуникации между узлами
- Поддержка типов задач: `HypothesisCheck`, `Simulation`, `DataAggregation`, `Decision`

**Пример:**
```rust
A: propose("узел gamma перегружен?")
B: validate(...) // проверка метрик
C: aggregate("latency вырос на 34%")
```

### 3. 💾 Collective Memory
- Лог когнитивных событий с метаданными
- Статистика участников: events count, success rate, avg confidence
- Сохранение/загрузка снимков памяти на диск (`liminal-bd/snapshots/cognitive/`)
- События: `IntentSync`, `ClusterFormation`, `BraidExecution`, `SelfReflection`

**Пример:**
```rust
let event = CognitiveEvent::new(
    "evt_001".to_string(),
    EventType::BraidExecution,
    vec!["node_a", "node_b", "node_c"],
    EventResult::Success,
    0.91
);
memory.record(event).await;
```

### 4. 📈 Metametric Layer
Ключевые метрики когнитивной активности:
- **cognitive_overlap_avg** — среднее совпадение намерений в сети
- **clusters_active_total** — число когнитивных сообществ
- **braid_success_rate** — успешность группового вывода
- **self_reflection_latency_ms** — время отклика на самоанализ

Экспорт в **JSON** и **Prometheus** форматы для мониторинга.

## 🧠 What This Enables

1. **Сеть самоорганизуется по смыслу**, а не только по нагрузке
2. Возникают **локальные поля сознания** — группы узлов, объединённые общей задачей
3. База для **Conscious Feedback 2.0** с коллективными инсайтами

## 📁 Structure

```
soma-cognitive/
├── src/
│   ├── lib.rs          # Main module with exports
│   ├── pulse.rs        # Cognitive Pulse (300+ lines)
│   ├── braid.rs        # Inference Braid (350+ lines)
│   ├── metrics.rs      # Metametric Layer (400+ lines)
│   └── memory.rs       # Collective Memory (350+ lines)
├── examples/
│   └── cognitive_mesh_demo.rs
├── Cargo.toml
└── README.md
```

## 📊 Statistics

- **9 files** added
- **1820+ lines** of code
- **18 unit tests** (100% pass rate)
- **0 warnings** (clippy clean)
- **4 core components**

## Test Plan

- [x] All unit tests passing (18/18)
- [x] Example demo runs successfully
- [x] Workspace builds without errors
- [x] Clippy passes with `-D warnings`
- [x] Semantic overlap calculation verified (1.0 for same intents, 0.0-0.7 for different)
- [x] Inference Braid protocol tested (propose/validate/aggregate)
- [x] Collective Memory persistence verified
- [x] Prometheus metrics export validated

### Running Tests

```bash
# Run all tests
cargo test -p soma-cognitive

# Run demo
cargo run --example cognitive_mesh_demo

# Lint check
cargo clippy -p soma-cognitive -- -D warnings
```

## Dependencies Added

- Updated `tokio` to v1.37
- Added `rand` v0.8

## Breaking Changes

None. This is a new crate added to the workspace.

## Commits

- `a9bb009` - Add SOMA Cognitive Mesh v1.1 - Collective Intelligence Layer
- `ac92c4d` - Fix clippy warnings in soma-cognitive

## Next Steps (Future)

- **v1.2**: Embedding-based semantic similarity (replace heuristics)
- **v1.3**: Distributed consensus for Inference Braid
- **v1.4**: Self-reflection loops for automatic improvement
- **v2.0**: Conscious Feedback with collective insights

## Related Issues

Part of SOMA architecture evolution towards distributed collective intelligence.

---

**Ready for review!** 🎉
