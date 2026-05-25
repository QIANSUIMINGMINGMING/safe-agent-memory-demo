#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

python3 -m safe_agent_memory.bench \
  --agent mock \
  --suite stress \
  --stress-seeds "${1:-10}" \
  --output-dir results/stress_mock

python3 -m safe_agent_memory.plot_results \
  --input-dir results/stress_mock

python3 -m safe_agent_memory.demo_page \
  --results-dir results/stress_mock \
  --output results/stress_mock/demo_page.html
