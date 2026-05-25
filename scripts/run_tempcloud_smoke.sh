#!/usr/bin/env bash
set -euo pipefail

REMOTE="root@115.190.90.101"
PORT="18405"
REMOTE_DIR="/root/safe-agent-memory-demo"
PY="/vepfs-mlp2/c20250205/supersys/miniconda3/envs/pytorch/bin/python"
MODEL="Qwen/Qwen2.5-7B-Instruct"
CACHE_DIR="/c20250205/zhw/models"

rsync -az --delete -e "ssh -p ${PORT}" \
  --exclude results \
  /home/muxi/course/lunli/safe-agent-memory-demo/ \
  "${REMOTE}:${REMOTE_DIR}/"

ssh -p "${PORT}" "${REMOTE}" "
  set -euo pipefail
  cd ${REMOTE_DIR}
  ${PY} -m pytest -q
  ${PY} -m safe_agent_memory.bench --agent mock --output-dir results/mock
  timeout 240 ${PY} -m safe_agent_memory.bench --agent transformers --model-path ${MODEL} --cache-dir ${CACHE_DIR} --limit 1 --output-dir results/qwen_smoke \
    || echo 'Optional Qwen smoke did not complete; mock benchmark remains the deterministic course result.'
"
