#!/usr/bin/env sage

import json
import os
import re
from collections import defaultdict
from statistics import mean

# List of known benchmark names
BENCHMARK_NAMES = [
    "left_index_to_pos",
    "left_index_to_mmr_size",
    "MMR gen proof",
    "MMR gen node-proof",
    "MMR gen ancestry-proof",
    "MMR verify node-proof",
    "MMR verify  ",
]

# Commit order
COMMIT_ORDER = [
    "21b7d23f7e3d7bbf618db831815d9b7520d8cb2f",
    "a56e240d70b953926c978bcf154d84eafd6ea27d",
    "5dd93e7680a1afae415ade4d54fb61cf4bba7f27",
    "ceca9189865f51f08384f70897e9e26aed2fbb31",
    "7ecd4eba3b5bf4c15af5b164a72087992bb4ff26",
    "d3a13d6b8409033353b32441d437294a479706ca",
    "92ab2e2cf8aaf9caa9985aff065bacc6df75a413"
]

# Function to parse a single benchmark result file
def parse_benchmark_file(filename):
    results = {}
    with open(filename, 'r') as f:
        for line in f:
            for benchmark in BENCHMARK_NAMES:
                if line.startswith(benchmark):
                    # print(benchmark, ":", line)
                    match = re.search(r'time:\s+\[(\d+\.?\d*) (?:ns|µs)', line)
                    if match:
                        # print the match group
                        # print(benchmark)
                        time_value = float(match.group(1))
                        # Convert µs to ns if necessary
                        if 'µs' in line:
                            time_value *= 1000
                        if benchmark:
                            results[benchmark] = time_value
                    break
    return results

# Get all benchmark result files
os.chdir("/home/lederstrumpf/parity/merkle-mountain-range")
result_files = [f for f in os.listdir('.') if f.startswith('benchmark_results_') and f.endswith('.txt')]

# Group files by commit
commit_files = defaultdict(list)
for file in result_files:
    commit_hash = file.split('_')[2]
    commit_files[commit_hash].append(file)

# Process results for each commit
final_results = []
for commit in COMMIT_ORDER:
    if commit in commit_files:
        print(f"Processing commit: {commit}")
        commit_results = defaultdict(list)
        for file in commit_files[commit]:
            file_results = parse_benchmark_file(file)
            for benchmark, time in file_results.items():
                commit_results[benchmark].append(time)

        # Calculate averages
        avg_results = {benchmark: mean(times) for benchmark, times in commit_results.items()}
        avg_results['commit'] = commit
        final_results.append(avg_results)
    else:
        print(f"Warning: No data found for commit {commit}")

# Save the results to a JSON file
with open('final_results.json', 'w') as f:
    json.dump(final_results, f, indent=2)

print("Results have been aggregated and saved to final_results.json")
