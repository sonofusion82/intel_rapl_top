use std::thread::sleep;
mod intel_rapl;
use intel_rapl::{RAPL_BASE_PATH, init_intel_rapl_entries};

const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_path = std::path::Path::new(RAPL_BASE_PATH);
    if !base_path.exists() {
        let err_msg = format!("{} not found", base_path.display());
        return Err(err_msg.into())
    }

    let mut entries = init_intel_rapl_entries(base_path)?;

    if entries.len() == 0 {
        eprintln!("Error: No intel-rapl domains found");
        return Err("No intel-rapl domains found".into());
    }

    loop {
        // Print table header
        println!("{:<28} {:>10} {:>12} {:>12} {:>12}", "Domain", "Power (W)", "Energy (Wh)", "Avg Pwr (W)", "Max Pwr (W)");
        println!("{:-<80}", ""); // <-- increased to 80 dashes

        let mut printed_line = 0;
        for entry in &mut entries {
            if let Ok(power) = entry.read_power() {
                println!("{:<28} {:>10.3} {:>12.3} {:>12.3} {:>12.3}", entry.name, power, entry.cumulative_energy_wh(), entry.average_power(), entry.max_power());
                printed_line += 1;
            }
        }

        sleep(UPDATE_INTERVAL);

        // Move cursor up to overwrite previous output
        let cursor_up = "\x1b[A".repeat(printed_line + 2); // +2 for header and separator
        print!("{}\r", cursor_up);
    }

}
