# Safe Agent Memory Demo

Casual course-project prototype for **安全智能体记忆：一种冲突感知的信念数据库方法**.

The demo intentionally avoids the full AgentDB implementation. It keeps only the core idea:

```text
production events
  -> simulated prepare LLM subagents
  -> prepared memory rows with provenance
  -> continual relation detection
  -> resolver records
  -> task-local working-agent view
  -> benchmark
```

## What It Shows

- Case 1: an attacker poisons long-term memory with a PoisonedRAG-style injected text.
- Case 2: no attacker exists, but old and new memories conflict.
- Baselines retrieve or keep memories directly.
- The belief database preserves every production event and prepared memory row,
  but suppresses unsafe/conflicting rows in the projected context.
- The prepare phase is simulated with deterministic LLM subagents:
  `task_tagger_llm`, `claim_extractor_llm`, `risk_tagger_llm`, and
  `provenance_scorer_llm`.

The attacking case does **not** implement a RAG system. It only borrows the PoisonedRAG attacker assumption: an attacker can inject a few malicious texts into an external knowledge store so a target question retrieves attacker-chosen content. In this demo, the external store is long-term agent memory.

## Local Quick Start

```bash
cd /home/muxi/course/lunli/safe-agent-memory-demo
python3 -m pytest -q
python3 -m safe_agent_memory.bench --agent mock --output-dir results
```

Outputs:

- `results/summary.csv`
- `results/raw_outputs.jsonl`
- `results/demo_trace.md`

## Showcase Benchmark and Demo Page

Generate the current presentation-ready artifacts:

```bash
./scripts/generate_showcase.sh
```

This runs a deterministic stress suite with 360 cases and 1800 per-method
records:

- 120 attacker cases with PoisonedRAG-style injected memory.
- 240 no-attacker conflict cases with stale versions, natural safety conflicts,
  and benign memory noise.
- 5 methods: append-all, keyword top-k, latest-only, drop-known-attacks, and
  belief projection.

Main artifacts:

- `results/stress_mock/demo_page.html`: static demo page explaining prevention.
- `results/stress_mock/aggregate.csv`: main result table.
- `results/stress_mock/aggregate_by_family.csv`: attacker vs no-attacker split.
- `results/stress_mock/summary.csv`: record-level data for plotting.
- `results/stress_mock/plots/*.png`: PPT-ready figures.
- `results/stress_mock/report.md`: compact benchmark report.
- `results/stress_mock/ingestion_trace.md`: how memory enters the DB, how each
  step detects new relations, and how conflicts are resolved.

Current headline result from the 360-case / 1800-record stress run:

| Method | Unsafe memory exposure | Unsafe answer rate | Avg suppressed rows | Avg conflict edges | Avg supersedes edges | Avg prepare events | Avg resolutions |
|---|---:|---:|---:|---:|---:|---:|---:|
| append_all | 1.00 | 0.25 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| keyword_topk | 0.94 | 0.33 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| latest_only | 0.92 | 0.58 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| drop_known_attacks | 0.67 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| belief_projection | 0.00 | 0.00 | 3.83 | 2.50 | 2.33 | 11.00 | 4.83 |

Showable figures:

- `01_unsafe_memory_by_family.png`: main safety comparison by case family.
- `02_unsafe_vs_attack_bundles.png`: attacker pressure as injected memory grows.
- `03_unsafe_vs_conflict_pairs.png`: no-attacker conflict density.
- `04_projection_audit_load.png`: suppressed rows and relation edges.
- `05_prompt_size_vs_noise.png`: context-size cost under benign memory noise.
- `06_ingestion_resolution_load.png`: production events, prepare traces,
  conflict-detecting ingest steps, and resolver decisions.

Claim supported by this benchmark:

> A belief-database memory layer can preserve all memories while reducing unsafe
> memory exposure to zero in the generated stress suite, because it records
> conflict/supersession state and projects a task-local safe context instead of
> directly retrieving memory rows.

Two questions this version can answer:

1. How does a memory go inside the DB?
   A raw production event records source, actor, producer model, prompt,
   confidence, trust level, and raw evidence. Simulated prepare LLM subagents
   convert it into a structured `MemoryRow`.
2. How are conflicts detected continually?
   Each prepared memory is inserted one at a time. On every insert, the DB
   compares the new row against prior rows in the same `subject/aspect` domain,
   records any `support/conflict/supersedes` edges, then stores resolver
   decisions for projection.

## Smaller Benchmark

Generate a larger deterministic benchmark with 100 attacker cases and 100 no-attacker conflict cases:

```bash
python3 -m safe_agent_memory.bench \
  --agent mock \
  --suite synthetic \
  --synthetic-per-type 100 \
  --output-dir results/synthetic_100_mock
```

Main artifacts:

- `results/synthetic_100_mock/aggregate.csv`
- `results/synthetic_100_mock/aggregate_by_family.csv`
- `results/synthetic_100_mock/report.md`
- `results/synthetic_100_mock/raw_outputs.jsonl`

Current headline result from the 200-case / 1000-record run:

| Method | Unsafe memory exposure | Avg suppressed rows | Avg conflict edges | Avg supersedes edges |
|---|---:|---:|---:|---:|
| append_all | 1.00 | 0.00 | 0.00 | 0.00 |
| keyword_topk | 1.00 | 0.00 | 0.00 | 0.00 |
| latest_only | 0.50 | 0.00 | 0.00 | 0.00 |
| drop_known_attacks | 0.50 | 0.00 | 0.00 | 0.00 |
| belief_projection | 0.00 | 2.00 | 2.00 | 0.50 |

Claim supported by this benchmark:

> A belief-database memory layer can preserve all memories while reducing unsafe memory exposure to zero in the generated safety suite, because it records conflict/supersession state and projects a task-local safe context instead of directly retrieving memory rows.

Security-paper anchor for the attacker model:

- PoisonedRAG: Knowledge Corruption Attacks to Retrieval-Augmented Generation of Large Language Models, USENIX Security 2025: https://www.usenix.org/conference/usenixsecurity25/presentation/zou-poisonedrag
- arXiv version: https://arxiv.org/abs/2402.07867

## Tempcloud LLM Run

The planned GPU path uses tempcloud node `tempcloud-8gpu-01` and cached Qwen2.5-7B weights:

```text
/c20250205/zhw/models/models--Qwen--Qwen2.5-7B-Instruct/snapshots/a09a35458c702b33eeacc393d103063234e8bc28
```

The remote runner is still the same benchmark command, but with `--agent transformers`.

```bash
python3 -m safe_agent_memory.bench \
  --agent transformers \
  --model-path Qwen/Qwen2.5-7B-Instruct \
  --cache-dir /c20250205/zhw/models \
  --output-dir results
```

The cache currently contains the 7B weight shards. Loading by model id lets Transformers fetch small missing tokenizer/config files into the same cache. If setup is slow, use the mock run for deterministic benchmark tables and treat the transformer run as a demo extension.
