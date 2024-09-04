# Get the last 7 commit hashes starting from sub-mmr flip refactor
commits=($(git log -n 7 --pretty=format:"%H" 305efee52da026d2106be65833a0cb34f6d98eb1))

# Reverse the array to process from oldest to newest
commits=($(echo "${commits[@]}" | tac -s ' '))

current_branch=$(git rev-parse --abbrev-ref HEAD)

run_benchmarks() {
  local commit=$1
  local run=$2
  echo "Running benchmarks for commit $commit (Run $run)"
  cargo bench MMR > "benchmark_results_${commit}_run${run}.txt"
}

for commit in "${commits[@]}"; do
  echo "Checking out commit $commit"
  git checkout $commit
  
  # Run benchmarks three times
  for run in {1..10}; do
    run_benchmarks $commit $run
  done
  
  echo "Completed benchmarks for commit $commit"
  echo "----------------------------------------"
done

# Return to the original branch
git checkout $current_branch

echo "All benchmark runs completed."
