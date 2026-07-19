
from readers import load_scenario_results

if __name__ == "__main__":
    print("test load big scenario output...")
    # game_data is no longer needed: the run's manifest.json carries the
    # dimension tables. Run the scenario with --record first.
    res = load_scenario_results(output_dir="tmp/multi_region")
    print(f"  prices={res.prices.shape} buildings={res.buildings.shape} "
          f"pops={res.pops.shape} inventories={res.inventories.shape}")
    print("all good!")
