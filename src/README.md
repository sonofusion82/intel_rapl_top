# intel_rapl_top

A simple Rust CLI tool to monitor Intel RAPL (Running Average Power Limit) domains for power and energy usage on Linux systems.

## Features

- Displays real-time power (W), cumulative energy (Wh), average power (W), and maximum observed power (W) for each Intel RAPL domain.
- Refreshes output in-place for easy monitoring.

## Usage

Build the project:

```bash
cargo build --release
```

Run the tool (may require root privileges):

```bash
sudo target/release/intel_rapl_top
```

## Example Output

```
Domain                        Power (W)  Energy (Wh)  Avg Pwr (W)  Max Pwr (W)
--------------------------------------------------------------------------------
intel-rapl:0:0/core               1.453        0.055        2.893       23.589
intel-rapl:0/package-0            5.063        0.125        6.529       27.418
intel-rapl:0:1/dram               1.161        0.025        1.308        2.036
```

## Requirements

- Linux system with Intel RAPL support (`/sys/class/powercap/intel-rapl:*`).
- Rust toolchain.

## License

MIT
