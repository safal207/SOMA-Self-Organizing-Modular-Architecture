use soma_bridge::Signal;
use soma_vnp::Neuron;
use std::thread::sleep;
use std::time::Duration;

/// Запустить симуляцию пульса SOMA
pub fn run_pulse_simulation() {
    let mut neuron = Neuron::with_params(1.0, 0.2, 1.0);

    println!("\n╔═══════════════════════════════════════╗");
    println!("║  🧬 SOMA Pulse Simulation Started   ║");
    println!("╚═══════════════════════════════════════╝");
    println!("\nPress Ctrl+C to stop.\n");

    let mut cycle_count = 0;

    loop {
        // Стимулируем нейрон
        let fired = neuron.stimulate(0.15);

        // Применяем временное затухание
        neuron.time_based_decay();

        // Создаем сигнал с текущим состоянием
        let signal = Signal::new("neuron_01", neuron.get_state());

        // Визуализируем
        render_pulse(&signal, fired, cycle_count);

        // Небольшая задержка для эффекта пульсации
        sleep(Duration::from_millis(150));

        cycle_count += 1;

        // Добавляем вариативность (имитация входящих сигналов)
        if cycle_count % 10 == 0 {
            println!("        ⚡ External stimulus");
        }
    }
}

/// Визуализировать пульс нейрона
fn render_pulse(signal: &Signal, fired: bool, cycle: u64) {
    let bars = (signal.value * 30.0) as usize;
    let visual = "█".repeat(bars);
    let empty = "░".repeat(30 - bars);

    let timestamp_str = format!("[{:04}]", cycle);

    if fired {
        // Красная вспышка при активации
        println!(
            "{} \x1b[91m{}{}\x1b[0m  🔥 FIRED (potential: {:.3})",
            timestamp_str, visual, empty, signal.value
        );
    } else {
        // Зеленый для накопления
        println!(
            "{} \x1b[92m{}{}\x1b[0m  ({:.3})",
            timestamp_str, visual, empty, signal.value
        );
    }
}

/// Расширенная визуализация с дополнительной информацией
pub fn render_pulse_detailed(signal: &Signal, fired: bool, potential: f64, cycle: u64) {
    let bars = (signal.value * 40.0) as usize;
    let visual = "█".repeat(bars);

    // Цветовая шкала в зависимости от уровня
    let color = if fired {
        "\x1b[91m" // Красный
    } else if signal.value > 0.7 {
        "\x1b[93m" // Желтый
    } else if signal.value > 0.4 {
        "\x1b[92m" // Зеленый
    } else {
        "\x1b[36m" // Голубой
    };

    let fire_marker = if fired { " 🔥" } else { "" };
    let age = signal.age_millis();

    println!(
        "[{:04}] {}│{}│\x1b[0m {:.3} (age: {}ms){}",
        cycle, color, visual, potential, age, fire_marker
    );
}
