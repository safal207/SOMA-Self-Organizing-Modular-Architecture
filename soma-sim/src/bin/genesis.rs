use soma_core::{CellRole, StemProcessor};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    println!("\n╔═══════════════════════════════════════╗");
    println!("║  🌍 SOMA Genesis Simulation         ║");
    println!("╚═══════════════════════════════════════╝");
    println!("\nStem processor observing the system...\n");
    println!("Press Ctrl+C to stop.\n");

    let mut stem = StemProcessor::with_params(0.5, 0.85);
    let mut cycle = 0;

    loop {
        // Имитация активности сети (синусоида с пиками для стимуляции деления)
        let activity = ((cycle as f64 * 0.3).sin().abs() * 0.6) + 0.2;

        // Процессор воспринимает активность
        stem.sense(activity);

        // Обновляем состояние всех клеток
        stem.tick();

        // Визуализация
        render_genesis(&stem, activity, cycle);

        sleep(Duration::from_millis(200));
        cycle += 1;

        // Периодически показываем статистику
        if cycle % 20 == 0 {
            print_statistics(&stem);
        }
    }
}

/// Визуализация текущего состояния системы
fn render_genesis(stem: &StemProcessor, activity: f64, cycle: u64) {
    let load_bars = (stem.load * 30.0) as usize;
    let activity_bars = (activity * 30.0) as usize;

    let load_vis = "█".repeat(load_bars);
    let load_empty = "░".repeat(30 - load_bars);

    let activity_vis = "█".repeat(activity_bars);
    let activity_empty = "░".repeat(30 - activity_bars);

    let load_color = if stem.load > stem.threshold {
        "\x1b[91m" // Красный - порог превышен
    } else if stem.load > stem.threshold * 0.7 {
        "\x1b[93m" // Желтый - близко к порогу
    } else {
        "\x1b[92m" // Зеленый - норма
    };

    print!("\r[{:04}] ", cycle);
    print!("Load: {}{}{}\x1b[0m {:.2} | ", load_color, load_vis, load_empty, stem.load);
    print!("Activity: \x1b[96m{}{}\x1b[0m {:.2} | ", activity_vis, activity_empty, activity);
    print!("Cells: \x1b[95m{:3}\x1b[0m Gen: {}", stem.cell_count(), stem.generation);

    std::io::stdout().flush().unwrap();
}

/// Печать статистики по ролям
fn print_statistics(stem: &StemProcessor) {
    let distribution = stem.role_distribution();

    println!("\n\n╔═══════════ Statistics ═══════════╗");
    println!("║ Total Cells: {:3}                ║", stem.cell_count());
    println!("║ Generation:  {:3}                ║", stem.generation);
    println!("╠═══════════════════════════════════╣");

    let sensor_count = distribution.get(&CellRole::Sensor).unwrap_or(&0);
    let logic_count = distribution.get(&CellRole::Logic).unwrap_or(&0);
    let motor_count = distribution.get(&CellRole::Motor).unwrap_or(&0);

    println!("║ 🔵 Sensor cells: {:3}            ║", sensor_count);
    println!("║ 🟢 Logic cells:  {:3}            ║", logic_count);
    println!("║ 🟡 Motor cells:  {:3}            ║", motor_count);
    println!("╚═══════════════════════════════════╝\n");

    // Показываем последние созданные клетки
    if stem.cell_count() > 0 {
        println!("Recent cells:");
        let mut cells: Vec<_> = stem.cells().values().collect();
        cells.sort_by_key(|c| c.birth_time);
        cells.reverse();

        for (i, cell) in cells.iter().take(5).enumerate() {
            let age_sec = cell.age_millis() / 1000;
            let role_icon = match cell.role {
                CellRole::Sensor => "🔵",
                CellRole::Logic => "🟢",
                CellRole::Motor => "🟡",
            };

            println!(
                "  {}. {} {} (gen {}, age {}s)",
                i + 1,
                role_icon,
                cell.id,
                cell.generation,
                age_sec
            );
        }
        println!();
    }
}
