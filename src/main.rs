use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::thread::sleep;

const RAPL_BASE_PATH: &str = "/sys/class/powercap/intel-rapl/";
const UPDATE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(1);

struct IntelRapl {
    name: String,
    path: PathBuf,
    last_energy: u64,
    last_time: std::time::Instant
}

impl IntelRapl {
    fn new(path: PathBuf) -> Result<Self, io::Error> {
        let name = Self::read_name(&path)?;
        Ok(Self {
            name,
            path,
            last_energy: 0,
            last_time: std::time::Instant::now(),
        })
    }

    fn read_name(path: &std::path::Path) -> Result<String, io::Error> {
        let mut rapl_name = String::new();
        File::open(path.join("name"))?.read_to_string(&mut rapl_name)?;
        Ok(rapl_name.trim().into())
    }

    fn read_energy(&self) -> Result<(u64, std::time::Instant), io::Error> {
        let mut energy_uj_string = String::new();
        File::open(self.path.join("energy_uj"))?.read_to_string(&mut energy_uj_string)?;
        Ok((energy_uj_string.trim().parse::<u64>().unwrap(),
            std::time::Instant::now()))
    }

    fn read_power(&mut self) -> Result<f64, io::Error> {
        let (energy_uj, updated_time) = self.read_energy()?;
        let delta_energy = if energy_uj >= self.last_energy {
            energy_uj - self.last_energy
        } else {
            0
        };
        let delta_time = (updated_time - self.last_time).as_secs_f64();
        let power = (delta_energy as f64) / delta_time * 1e-6; // in watts

        self.last_energy = energy_uj;
        self.last_time = updated_time;
        Ok(power)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_path = std::path::Path::new(RAPL_BASE_PATH);

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
        let mut printed_line = 0;

        for entry in &mut entries {
            if let Ok(power)= entry.read_power() {
                println!("Domain: {:8} Power: {:8.3} W        ", entry.name, power);
                printed_line += 1;
            }
        }

        sleep(UPDATE_INTERVAL);
        let cursor_return = "\x1b[A".repeat(printed_line);
        print!("{}\r", cursor_return);
    }

}
