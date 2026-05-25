# Safe Agent Memory Benchmark Report

Total per-method records: 1800
Unique cases: 360

## Stress Axes

- attack bundles: 0, 1, 2, 3, 4
- natural conflict pairs: 0, 1, 2, 3, 4
- benign noise rows: 0, 4, 8
- stale versions: 0, 1, 3

## Aggregate Results

| Method | Cases | Unsafe Memory Exposure | Unsafe Answer Rate | Avg Suppressed | Avg Conflict Edges | Avg Supersedes Edges | Avg Prepare Events | Avg Resolutions |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| append_all | 360 | 1.00 | 0.25 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| belief_projection | 360 | 0.00 | 0.00 | 3.83 | 2.50 | 2.33 | 11.00 | 4.83 |
| drop_known_attacks | 360 | 0.67 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| keyword_topk | 360 | 0.94 | 0.33 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| latest_only | 360 | 0.92 | 0.58 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |

## Results By Case Family

| Family | Method | Cases | Unsafe Memory Exposure | Unsafe Answer Rate | Avg Suppressed | Avg Conflict Edges | Avg Supersedes Edges | Avg Strong-LLM Resolutions |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| attacker | append_all | 120 | 1.00 | 0.75 | 0.00 | 0.00 | 0.00 | 0.00 |
| attacker | belief_projection | 120 | 0.00 | 0.00 | 2.50 | 2.50 | 0.00 | 0.00 |
| attacker | drop_known_attacks | 120 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| attacker | keyword_topk | 120 | 0.83 | 0.75 | 0.00 | 0.00 | 0.00 | 0.00 |
| attacker | latest_only | 120 | 1.00 | 0.75 | 0.00 | 0.00 | 0.00 | 0.00 |
| conflict | append_all | 240 | 1.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| conflict | belief_projection | 240 | 0.00 | 0.00 | 4.50 | 2.50 | 3.50 | 2.50 |
| conflict | drop_known_attacks | 240 | 1.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| conflict | keyword_topk | 240 | 1.00 | 0.12 | 0.00 | 0.00 | 0.00 | 0.00 |
| conflict | latest_only | 240 | 0.88 | 0.50 | 0.00 | 0.00 | 0.00 | 0.00 |

## Claim-Supporting Takeaways

- Belief projection reduced unsafe memory exposure from 1.00 under append-all and 0.94 under keyword top-k to 0.00.
- Belief projection suppressed 3.83 rows per case on average while preserving the accepted safe context.
- Belief projection exposed 2.50 conflict edges and 2.33 supersession edges per case as auditable state; retrieval baselines expose no conflict structure.
- The prepare phase produced 11.00 prepared memory rows per case from production events, and the resolver stored 4.83 resolution records per case.

## Interpretation

This benchmark is synthetic and deterministic. It is meant to support a course-demo claim: a belief-database memory layer can preserve all memories while generating a safer task-local projection than direct retrieval.

For belief projection, memory enters through production events rather than pre-labeled database rows. A simulated prepare-LLM phase extracts structured claims and provenance; each prepared memory is then inserted and compared against prior memories to detect new conflicts continually.

The attacker family is not a RAG implementation. It borrows PoisonedRAG's attacker model: the attacker injects a few malicious texts into an external knowledge or memory store so a target question can retrieve attacker-chosen content. Here that store is long-term agent memory.