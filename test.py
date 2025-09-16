import pathlib
import time
import random
import shutil

def main():
    """ Simulate the behavior of /sys/class/powercap/intel-rapl """
    base_path = pathlib.Path("sys/class/powercap/intel-rapl")
    
    # Remove and recreate the directories
    if base_path.exists():
        shutil.rmtree(base_path, ignore_errors=True)
    base_path.mkdir(parents=True)

    domain_power = [ 15.0, 30.0, 45.0 ]  # Example power values for domains
    t = 0.0
    time_step = 1.0  # Time step in seconds

    energy_uj = [0, 0, 0]
    max_energy_range_uj = 2**28

    try:
        while True:
            for i in range(3):  # Simulate 3 domains
                current_power = domain_power[i] + (random.gauss(1.0, 0.1) * domain_power[i] / 10.0)
                energy_uj[i] += int((current_power * time_step) * 1e6)
                if energy_uj[i] >= max_energy_range_uj:
                    energy_uj[i] -= max_energy_range_uj

                domain_path = base_path / f"intel-rapl:{i}"
                if not domain_path.exists():
                    domain_path.mkdir(exist_ok=True)

                    (domain_path / "name").write_text(f"Domain {i}\n")
                    (domain_path / "max_energy_range_uj").write_text(f"{max_energy_range_uj}\n")

                (domain_path / "energy_uj").write_text("{}\n".format(energy_uj[i]))

            t += time_step
            print(f"\rt={t:.1f}s, energy_uj={energy_uj}       ", end="")
            time.sleep(time_step)

    except KeyboardInterrupt:
        print("\nSimulation stopped.")



if __name__ == "__main__":
    main()