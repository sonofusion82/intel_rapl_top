use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;

pub const RAPL_BASE_PATH: &str = "/sys/class/powercap/";

pub struct IntelRapl {
    pub name: String,
    path: PathBuf,
    last_energy: u64,
    max_energy_range_uj: u64,
    last_time: std::time::Instant,
    cumulative_energy_uj: u64,
    cumulative_energy_start_time: std::time::Instant,
    max_power: f64, // Add this field
}

/// Represents an Intel RAPL (Running Average Power Limit) device, providing methods to read energy and power usage.
///
/// # Fields
/// - `name`: The name of the RAPL device.
/// - `path`: The filesystem path to the RAPL device.
/// - `last_energy`: The last read energy value in microjoules.
/// - `max_energy_range_uj`: The maximum energy range in microjoules.
/// - `last_time`: The timestamp of the last energy reading.
/// - `cumulative_energy_uj`: The cumulative energy consumed since initialization, in microjoules.
/// - `cumulative_energy_start_time`: The timestamp when cumulative energy measurement started.
///
/// # Methods
/// - `new(path: PathBuf) -> Result<Self, io::Error>`: Constructs a new `IntelRapl` instance from the given path, reading initial device information.
/// - `read_name(path: &std::path::Path) -> Result<String, io::Error>`: Reads the device name from the specified path.
/// - `read_max_energy_range_uj(path: &std::path::Path) -> Result<u64, io::Error>`: Reads the maximum energy range (in microjoules) from the specified path.
/// - `read_energy(&self) -> Result<(u64, std::time::Instant), io::Error>`: Reads the current energy value and timestamp from the device.
/// - `read_power(&mut self) -> Result<f64, io::Error>`: Calculates and returns the instantaneous power usage in watts, updating internal state.
/// - `average_power(&self) -> f64`: Returns the average power usage in watts since cumulative measurement started.
/// - `cumulative_energy_wh(&self) -> f64`: Returns the cumulative energy consumed in watt-hours since cumulative measurement started.
impl IntelRapl {
    fn new(path: PathBuf) -> Result<Self, io::Error> {
        let name_value = Self::read_name(&path)?;
        // Get only the last component of the parent directory
        let parent_dir_name = path.file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or_default();
        let name = format!("{}/{}", parent_dir_name, name_value);

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
            max_power: 0.0, // Initialize
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

    pub fn read_power(&mut self) -> Result<f64, io::Error> {
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

        self.cumulative_energy_uj += delta_energy;
        self.last_energy = energy_uj;
        self.last_time = updated_time;
        if power > self.max_power {
            self.max_power = power;
        }
        Ok(power)
    }

    pub fn average_power(&self) -> f64 {
        let total_time = self.last_time - self.cumulative_energy_start_time;
        self.cumulative_energy_uj as f64 / total_time.as_secs_f64() * 1e-6
    }

    pub fn cumulative_energy_wh(&self) -> f64 {
        self.cumulative_energy_uj as f64 * 1e-6 / 3600.0
    }

    /// Returns the maximum observed power in watts.
    pub fn max_power(&self) -> f64 {
        self.max_power
    }
}


/// Initializes and returns a vector of `IntelRapl` entries found under the specified base path.
///
/// # Arguments
/// * `base_path` - The base path to search for Intel RAPL entries (typically `/sys/class/powercap/`).
///
/// # Returns
/// * `Result<Vec<IntelRapl>, io::Error>` - A vector of initialized `IntelRapl` devices, or an error if initialization fails.
pub fn init_intel_rapl_entries(base_path: &std::path::Path) -> Result<Vec<IntelRapl>, io::Error> {
    std::fs::read_dir(base_path).unwrap()
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
        .collect::<Result<Vec<_>,_>>()
}