use soma_bridge::{Link, Signal};
use soma_vnp::Neuron;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    // Создаём два нейрона с разными параметрами
    let mut neuron_a = Neuron::with_params(1.0, 0.2, 1.0);
    let mut neuron_b = Neuron::with_params(1.0, 0.25, 1.0);

    // Создаём каналы связи
    let link_a_to_b = Link::new();
    let link_b_to_a = Link::new();

    println!("\n╔═══════════════════════════════════════╗");
    println!("║  🌐 SOMA Resonance Simulation       ║");
    println!("╚═══════════════════════════════════════╝");
    println!("\nTwo neurons discovering each other...\n");
    println!("Press Ctrl+C to stop.\n");

    let mut cycle = 0;

    loop {
        // Нейрон A: собственная активность + сигналы от B
        let fired_a = neuron_a.stimulate(0.2);
        neuron_a.time_based_decay();

        // Если A активировался, отправляем сигнал к B
        if fired_a {
            link_a_to_b.send(Signal::new("neuron_a", 1.0));
        }

        // Проверяем сигналы от B к A
        if let Some(signal) = link_b_to_a.receive() {
            // B влияет на A с затуханием
            neuron_a.stimulate(signal.value * 0.5);
        }

        // Нейрон B: собственная активность + сигналы от A
        let fired_b = neuron_b.stimulate(0.1);
        neuron_b.time_based_decay();

        // Если B активировался, отправляем сигнал к A
        if fired_b {
            link_b_to_a.send(Signal::new("neuron_b", 1.0));
        }

        // Проверяем сигналы от A к B
        if let Some(signal) = link_a_to_b.receive() {
            // A влияет на B с затуханием
            neuron_b.stimulate(signal.value * 0.5);
        }

        // Визуализация
        render_resonance(&neuron_a, &neuron_b, fired_a, fired_b, cycle);

        sleep(Duration::from_millis(150));
        cycle += 1;

        // Периодические волны внешнего стимула
        if cycle % 15 == 0 {
            println!("        ⚡ External wave");
        }
    }
}

/// Визуализация состояния двух нейронов
fn render_resonance(
    neuron_a: &Neuron,
    neuron_b: &Neuron,
    fired_a: bool,
    fired_b: bool,
    cycle: u64,
) {
    let state_a = neuron_a.get_state();
    let state_b = neuron_b.get_state();

    let bars_a = (state_a * 25.0) as usize;
    let bars_b = (state_b * 25.0) as usize;

    let vis_a = "█".repeat(bars_a);
    let vis_b = "█".repeat(bars_b);

    let empty_a = "░".repeat(25 - bars_a);
    let empty_b = "░".repeat(25 - bars_b);

    let color_a = if fired_a {
        "\x1b[91m" // Красный при активации
    } else if state_a > 0.7 {
        "\x1b[93m" // Желтый близко к порогу
    } else {
        "\x1b[96m" // Голубой
    };

    let color_b = if fired_b {
        "\x1b[95m" // Фиолетовый при активации
    } else if state_b > 0.7 {
        "\x1b[93m" // Желтый близко к порогу
    } else {
        "\x1b[92m" // Зеленый
    };

    let marker_a = if fired_a { " 🔥" } else { "   " };
    let marker_b = if fired_b { " ✨" } else { "   " };

    let resonance_info = format_resonance(state_a, state_b);

    println!(
        "[{:04}] {}A: {}{}\x1b[0m{}  {}B: {}{}\x1b[0m{}{}",
        cycle,
        color_a,
        vis_a,
        empty_a,
        marker_a,
        color_b,
        vis_b,
        empty_b,
        marker_b,
        resonance_info
    );
}

/// Вычислить резонанс между двумя нейронами
fn format_resonance(state_a: f64, state_b: f64) -> String {
    let diff = (state_a - state_b).abs();
    let resonance = 1.0 - diff;

    if resonance > 0.8 {
        format!(" \x1b[93m⚡ Resonance: {:.2}\x1b[0m", resonance)
    } else {
        String::new()
    }
}
