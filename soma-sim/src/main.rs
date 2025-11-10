mod viz;

use soma_core::StemCell;
use soma_mind::{CouncilMode, InnerCouncil};
use soma_vnp::{Neuron, NeuronLayer};
use std::collections::HashMap;
use viz::Simulator;

fn main() {
    println!("╔════════════════════════════════════════════╗");
    println!("║  SOMA v0.1 - Skeleton Iteration Simulator ║");
    println!("╚════════════════════════════════════════════╝\n");

    // Демонстрация стволовых клеток
    demo_stem_cells();

    // Демонстрация нейронов
    demo_neurons();

    // Демонстрация Inner Council
    demo_inner_council();

    // Запуск симуляции
    run_simulation();

    println!("\n✨ SOMA skeleton demonstration completed!");
}

/// Демонстрация работы стволовых клеток
fn demo_stem_cells() {
    println!("🧬 Demo: Stem Cells");
    println!("═══════════════════\n");

    // Создаём стволовую клетку
    let stem = StemCell::with_resonance(0.6);
    println!("Created stem cell with resonance: {}", stem.resonance);

    // Дифференцируем в нейрон
    let neuron = Neuron::new();
    let differentiated = stem.differentiate(neuron);

    println!("Differentiated into neuron with potential: {:.3}", differentiated.potential());
    println!();
}

/// Демонстрация работы нейронов
fn demo_neurons() {
    println!("🧠 Demo: Neurons");
    println!("════════════════\n");

    // Создаём слой из 5 нейронов
    let mut layer = NeuronLayer::new(5);
    println!("Created neuron layer with {} neurons", layer.len());

    // Подаём сигналы
    let inputs = vec![0.8, 0.6, 0.4, 0.9, 0.3];
    println!("Input signals: {:?}", inputs);

    let outputs = layer.process(&inputs);
    println!("Output signals: {:?}", outputs);

    // Показываем активированные нейроны
    let mut activated = Vec::new();
    for i in 0..layer.len() {
        if let Some(neuron) = layer.neuron(i) {
            if neuron.is_activated() {
                activated.push(i);
            }
        }
    }
    println!("Activated neurons: {:?}", activated);
    println!();
}

/// Демонстрация работы Inner Council
fn demo_inner_council() {
    println!("👁️  Demo: Inner Council");
    println!("═══════════════════════\n");

    let mut council = InnerCouncil::new();

    // Подготавливаем входные данные
    let inputs = HashMap::from([
        ("urgency".to_string(), 0.7),
        ("complexity".to_string(), 0.6),
        ("creativity".to_string(), 0.8),
    ]);

    // Тестируем разные режимы
    let modes = [
        CouncilMode::Balanced,
        CouncilMode::Intuitive,
        CouncilMode::Creative,
        CouncilMode::Structured,
    ];

    for mode in modes {
        council.set_mode(mode);
        let decision = council.decide(&inputs);

        println!("Mode: {:?}", mode);
        println!("  Confidence: {:.3}", decision.confidence);
        println!("  Pythia:     {:.3}", decision.details.get("pythia").unwrap_or(&0.0));
        println!("  Morpheus:   {:.3}", decision.details.get("morpheus").unwrap_or(&0.0));
        println!("  Architect:  {:.3}", decision.details.get("architect").unwrap_or(&0.0));
        println!();
    }
}

/// Запустить полную симуляцию с визуализацией
fn run_simulation() {
    println!("🎬 Demo: Full Simulation");
    println!("════════════════════════\n");

    let mut sim = Simulator::new();
    let mut layer = NeuronLayer::new(3);

    // Симулируем 5 шагов
    println!("Running 5 simulation steps...\n");

    let input_sequences = vec![
        vec![0.3, 0.4, 0.5],
        vec![0.5, 0.6, 0.7],
        vec![0.7, 0.8, 0.9],
        vec![0.6, 0.7, 0.8],
        vec![0.4, 0.5, 0.6],
    ];

    for (i, inputs) in input_sequences.iter().enumerate() {
        println!("Step {}: inputs = {:?}", i + 1, inputs);
        let snapshot = sim.step(&mut layer, inputs);

        for (name, value) in &snapshot.values {
            println!("  {} = {:.3}", name, value);
        }
        println!();
    }

    // Показываем визуализацию
    sim.visualizer().display_ascii();

    // Показываем статистику
    let stats = sim.visualizer().stats();
    println!("Statistics:");
    println!("  Total snapshots: {}", stats.snapshot_count);
    println!("  Mean resonance:  {:.3}", stats.mean_resonance);
    println!("  Min resonance:   {:.3}", stats.min_resonance);
    println!("  Max resonance:   {:.3}", stats.max_resonance);
    println!();
}
