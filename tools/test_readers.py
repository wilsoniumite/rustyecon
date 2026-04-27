
from readers import load_scenario_results

if __name__ == "__main__":
    print("test load big scenario output...")
    load_scenario_results(output_dir="tmp/multi_region", game_data="data/scenarios/multi_region/game_data.ron")
    print("all good!")