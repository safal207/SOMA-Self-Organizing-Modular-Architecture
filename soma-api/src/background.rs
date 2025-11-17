//! Фоновые задачи для SOMA API
//!
//! Все асинхронные фоновые процессы системы

use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

use crate::{AppState, ApiSignal, config};
use soma_core::StemProcessor;
use soma_conscious::ConsciousState;
use soma_conscious::{ReflectionAnalyzer, FeedbackController, CausalTrace};

/// Фоновая задача обновления системы
pub async fn background_update(
    stem: Arc<Mutex<StemProcessor>>,
    signal_tx: broadcast::Sender<ApiSignal>,
) {
    let mut tick = interval(Duration::from_millis(config::api::BACKGROUND_UPDATE_INTERVAL_MS));
    let mut cycle = 0u64;

    loop {
        tick.tick().await;

        let mut stem = match stem.lock() {
            Ok(s) => s,
            Err(_) => continue, // Пропускаем цикл при ошибке блокировки
        };

        // Имитация активности (синусоида)
        let activity = ((cycle as f64 * 0.1).sin().abs() * 0.5) + 0.2;

        stem.sense(activity);
        stem.tick();

        // Периодически отправляем сигнал о состоянии
        if cycle % 10 == 0 {
            let signal = ApiSignal {
                id: "system".to_string(),
                value: stem.load,
                timestamp: cycle,
            };
            let _ = signal_tx.send(signal);
        }

        cycle += 1;
    }
}

/// Фоновая задача синхронизации состояния mesh
pub async fn mesh_state_sync(
    stem: Arc<Mutex<StemProcessor>>,
    mesh: Arc<crate::mesh::MeshNode>,
) {
    let mut tick = interval(Duration::from_secs(config::api::MESH_STATE_SYNC_INTERVAL_SEC));

    loop {
        tick.tick().await;

        let (cells, generation, load) = {
            match stem.lock() {
                Ok(s) => (s.cell_count(), s.generation, s.load),
                Err(_) => continue,
            }
        };

        mesh.broadcast_state(cells, generation, load);
    }
}

/// Фоновая задача применения резонанса
pub async fn mesh_resonance_sync(
    stem: Arc<Mutex<StemProcessor>>,
    mesh: Arc<crate::mesh::MeshNode>,
) {
    let mut tick = interval(Duration::from_millis(config::api::MESH_RESONANCE_SYNC_INTERVAL_MS));

    loop {
        tick.tick().await;

        // Применяем резонанс только если есть живые peers
        if mesh.get_peer_count() > 0 {
            let mut stem = match stem.lock() {
                Ok(s) => s,
                Err(_) => continue,
            };
            
            let current_load = stem.load;

            // Вычисляем адаптивную силу на основе здоровья сети (0.05-0.2)
            let strength = mesh.compute_adaptive_strength();

            // Вычисляем корректировку с адаптивной силой
            let correction = mesh.compute_resonance_correction(current_load, strength);

            // Применяем корректировку
            stem.load = (stem.load + correction).max(0.0).min(1.0);
        }
    }
}

/// Conscious Cycle - observe → record → analyze → generate → apply
pub async fn conscious_cycle(
    conscious: Arc<Mutex<ConsciousState>>,
    mesh: Arc<crate::mesh::MeshNode>,
    _stem: Arc<Mutex<StemProcessor>>,
) {
    let mut tick = interval(Duration::from_secs(config::api::CONSCIOUS_CYCLE_INTERVAL_SEC));
    let analyzer = ReflectionAnalyzer::new();
    let feedback = FeedbackController::new();

    loop {
        tick.tick().await;

        // OBSERVE: Наблюдаем за состоянием mesh
        let link_weights = mesh.get_link_weights();

        // RECORD: Записываем причинные цепи
        {
            let mut state = match conscious.lock() {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Для каждого изменения веса создаём trace
            for (peer_id, weight, _quality) in &link_weights {
                if *weight != 0.3 { // Изменён от дефолта
                    let trace = CausalTrace::new(
                        format!("network_activity"),
                        format!("{}_weight_{:.3}", peer_id, weight),
                        weight - 0.3,
                    );
                    state.record_trace(trace);
                }
            }
        }

        // ANALYZE: Анализируем паттерны (окно 60 секунд)
        let insights = {
            let state = match conscious.lock() {
                Ok(s) => s,
                Err(_) => return,
            };
            analyzer.analyze(&state, config::api::REFLECTION_ANALYSIS_WINDOW_MS)
        };

        // GENERATE: Генерируем инсайты
        {
            let mut state = match conscious.lock() {
                Ok(s) => s,
                Err(_) => continue,
            };
            
            for insight in &insights {
                state.add_insight(insight.clone());
                println!("💭 Insight: {} ({})", insight.insight, insight.category);
            }
        }

        // APPLY: Применяем feedback
        let actions = feedback.generate_actions(&insights);
        if !actions.is_empty() {
            println!("🔧 Feedback: {} actions generated", actions.len());
            for action in &actions {
                println!("   → {:?}: {} = {:.3}", action.action_type, action.target, action.value);
            }
        }

        // Завершаем цикл
        {
            let mut state = match conscious.lock() {
                Ok(s) => s,
                Err(_) => continue,
            };
            state.complete_cycle();
        }
    }
}

