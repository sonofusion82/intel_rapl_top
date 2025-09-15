use std::fs::File;
use std::io::Read;
use std::thread::sleep;

fn main() {
    let base_path = std::path::Path::new("/sys/class/powercap/intel-rapl/");

    let entries = std::fs::read_dir(base_path).unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| {
            let name_file = path.join("name");
            let energy_file = path.join("energy_uj");

            path.is_dir() &&
                path.file_name().unwrap().to_str().unwrap().starts_with("intel-rapl:") &&
                name_file.exists() &&
                energy_file.exists()
        })
        .collect::<Vec<_>>();

    if entries.len() == 0 {
        println!("Error: No intel-rapl domains found");
        return;
    }

    let rapl_names = entries.iter().map(|path| {
        let name_path = path.join("name");
        let mut name_file = File::open(name_path).unwrap();
        let mut name_string = String::new();
        name_file.read_to_string(&mut name_string).unwrap();
        name_string.trim().to_string()
    }).collect::<Vec<_>>();

    let mut last_energies = vec![0u64; entries.len()];
    let mut last_time = vec![std::time::Instant::now(); entries.len()]; 

    loop {
        let mut printed_line = 0;

        for i in 0.. entries.len() {
            let energy_path = entries[i].join("energy_uj");
            let mut energy_file = File::open(energy_path).unwrap();
            let mut energy_string = String::new();
            energy_file.read_to_string(&mut energy_string).unwrap();
            let now = std::time::Instant::now();

            let energy_uj = energy_string.trim().parse::<u64>().unwrap();
            if last_energies[i] > 0 {
                let delta_energy = if energy_uj >= last_energies[i] {
                    energy_uj - last_energies[i]
                } else {
                    0
                };
                let delta_time = (now - last_time[i]).as_secs_f64();
                let power = (delta_energy as f64) / delta_time * 1e-6; // in watts
                println!("Domain {} Power: {:.3} W        ", rapl_names[i], power);
                printed_line += 1;
            }
            last_energies[i] = energy_uj;
            last_time[i] = now;
        }

        sleep(std::time::Duration::from_secs(1));
        let cursor_return = String::from("\r") + &"\x1b[A".repeat(printed_line);
        print!("{}", cursor_return)
    }

}
