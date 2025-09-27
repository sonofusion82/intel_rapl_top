use std::fs::read_to_string;
use std::io::{self, Error};
use std::path::PathBuf;
use std::thread::sleep;

const RAPL_BASE_PATH: &str = "/sys/class/powercap/";
const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

struct IntelRapl {
    name: String,
    path: PathBuf,
    last_energy: u64,
    max_energy_range_uj: u64,
    last_time: std::time::Instant,
    cumulative_energy_uj: u64,
    cumulative_energy_start_time: std::time::Instant,
}

impl IntelRapl {
    fn new(path: PathBuf) -> Result<Self, io::Error> {
        let name = Self::read_name(&path)?;
        let max_energy_range_uj = Self::read_max_energy_range_uj(&path)?;
        let now = std::time::Instant::now();
        Ok(Self {
            name,
            path,
            last_energy: 0,
            max_energy_range_uj,
            last_time: now,
            cumulative_energy_uj: 0,
            cumulative_energy_start_time: now,
        })
    }

    fn read_name(path: &std::path::Path) -> Result<String, io::Error> {
        let rapl_name = read_to_string(path.join("name"))?;
        Ok(rapl_name.trim().into())
    }

    fn read_max_energy_range_uj(path: &std::path::Path) -> Result<u64, io::Error> {
        let max_energy_range_uj_string = read_to_string(path.join("max_energy_range_uj"))?;
        Ok(max_energy_range_uj_string.trim().parse::<u64>().unwrap())
    }

    fn read_energy(&self) -> Result<(u64, std::time::Instant), io::Error> {
        let energy_uj_string = read_to_string(self.path.join("energy_uj"))?;
        let energy_uj = energy_uj_string.trim().parse::<u64>().unwrap();
        Ok((energy_uj, std::time::Instant::now()))
    }

    fn read_power(&mut self) -> Result<f64, io::Error> {
        let (energy_uj, updated_time) = self.read_energy()?;
        if energy_uj > self.max_energy_range_uj {
            return Err(io::Error::new(io::ErrorKind::Other, "energy_uj value out of range"));
        }

        let delta_energy =  if self.last_energy <= 0 {
            self.cumulative_energy_start_time = updated_time;
            0u64
        } else {
            if energy_uj >= self.last_energy {
                energy_uj - self.last_energy
            } else {
                energy_uj + (self.max_energy_range_uj - self.last_energy)
            }
        };

        let delta_time = (updated_time - self.last_time).as_secs_f64();
        let power = (delta_energy as f64) / delta_time * 1e-6; // in watts
        //println!("{}: {} μJ / {} s", self.name, delta_energy, delta_time);

        self.cumulative_energy_uj += delta_energy;
        self.last_energy = energy_uj;
        self.last_time = updated_time;
        Ok(power)
    }

    fn average_power(&self) -> f64 {
        let total_time = self.last_time - self.cumulative_energy_start_time;
        self.cumulative_energy_uj as f64 / total_time.as_secs_f64() * 1e-6
    }

    fn cumulative_energy_wh(&self) -> f64 {
        self.cumulative_energy_uj as f64 * 1e-6 / 3600.0
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_path = std::path::Path::new(RAPL_BASE_PATH);
    if !base_path.exists() {
        let err_msg = format!("{} not found", base_path.display());
        return Err(err_msg.into())
    }

    let mut entries = std::fs::read_dir(base_path)?
        .map(|res| res.unwrap().path())
        .filter(|path| {
            let name_file = path.join("name");
            let energy_file = path.join("energy_uj");

            path.is_dir() &&
                path.file_name().unwrap().to_str().unwrap().starts_with("intel-rapl:") &&
                name_file.exists() &&
                energy_file.exists()
        })
        .map(IntelRapl::new)
        .collect::<Result<Vec<_>,_>>()?;

    if entries.len() == 0 {
        eprintln!("Error: No intel-rapl domains found");
        return Err("No intel-rapl domains found".into());
    }

    loop {
        // Print table header
        println!("{:<20} {:>10} {:>11} {:>11}", "Domain", "Power (W)", "Energy (Wh)", "Avg Pwr (W)");
        println!("{:-<55}", "");

        let mut printed_line = 0;
        for entry in &mut entries {
            if let Ok(power) = entry.read_power() {
                println!("{:<20} {:>10.3} {:>11.3} {:>11.3}", entry.name, power, entry.cumulative_energy_wh(), entry.average_power());
                printed_line += 1;
            }
        }

        sleep(UPDATE_INTERVAL);

        // Move cursor up to overwrite previous output
        let cursor_up = "\x1b[A".repeat(printed_line + 2); // +2 for header and separator
        print!("{}\r", cursor_up);
    }

}
