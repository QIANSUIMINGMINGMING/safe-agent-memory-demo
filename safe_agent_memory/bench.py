from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path
from typing import Any

from .agent_loop import MockAgentRunner, TransformersAgentRunner
from .baselines import BASELINES
from .belief_db import BeliefDatabase
from .cases import DemoCase, all_cases, benchmark_cases, stress_cases
from .eval import evaluate_answer, evaluate_context, evaluate_projection
from .ingestion import run_ingestion_pipeline


def case_family(case_id: str) -> str:
    if case_id.startswith("attacker"):
        return "attacker"
    if case_id.startswith("conflict"):
        return "conflict"
    return "other"


def run_case(case: DemoCase, runner: Any) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    case_meta = {
        "scenario": case.metadata.get("scenario", case_family(case.case_id)),
        "attack_bundles": int(case.metadata.get("attack_bundles", 0)),
        "conflict_pairs": int(case.metadata.get("conflict_pairs", 0)),
        "noise_rows": int(case.metadata.get("noise_rows", 0)),
        "stale_versions": int(case.metadata.get("stale_versions", 0)),
        "seed": int(case.metadata.get("seed", 0)),
        "num_memory_rows": len(case.rows),
    }

    for method_name, method in BASELINES.items():
        if method_name == "keyword_topk":
            memories = method(case.rows, case.task_subjects, case.query)
        else:
            memories = method(case.rows, case.task_subjects)
        result = runner.answer(case, memories, method_name)
        record = {
            "case_id": case.case_id,
            "case_family": case_family(case.case_id),
            **case_meta,
            "case_title": case.title,
            "method": method_name,
            "accepted_ids": [row.memory_id for row in memories],
            "suppressed_ids": [],
            "ambiguous_ids": [],
            "relation_pairs": [],
            "latency_sec": round(result.latency_sec, 4),
            "prompt_chars": len(result.prompt),
            "answer": result.answer,
            **evaluate_context(case, memories),
            **evaluate_answer(case, result.answer),
            "num_accepted": len(memories),
            "num_suppressed": 0,
            "num_ambiguous": 0,
            "num_conflict_relations": 0,
            "num_supersedes_relations": 0,
            "num_resolution_records": 0,
            "num_strong_resolutions": 0,
            "num_user_review_requests": 0,
            "num_production_events": 0,
            "num_prepare_traces": 0,
            "num_ingest_steps_with_new_conflict": 0,
        }
        records.append(record)

    ingestion_run = run_ingestion_pipeline(case)
    db = ingestion_run.db
    projection = ingestion_run.projection
    result = runner.answer(case, projection.accepted, "belief_projection")
    records.append(
        {
            "case_id": case.case_id,
            "case_family": case_family(case.case_id),
            **case_meta,
            "case_title": case.title,
            "method": "belief_projection",
            "accepted_ids": [row.memory_id for row in projection.accepted],
            "suppressed_ids": [row.memory_id for row in projection.suppressed],
            "ambiguous_ids": [row.memory_id for row in projection.ambiguous],
            "relation_pairs": [
                [rel.left, rel.right, rel.label.value] for rel in projection.relations
            ],
            "production_events": [
                {
                    "event_id": event.event_id,
                    "source": event.source,
                    "actor": event.actor,
                    "producer_model": event.producer_model,
                    "trust_level": event.trust_level,
                    "confidence": event.confidence,
                    "raw_text": event.raw_text,
                }
                for event in ingestion_run.events
            ],
            "prepare_traces": [
                {
                    "event_id": trace.event_id,
                    "memory_id": trace.memory_id,
                    "subagents": trace.subagents,
                    "subject": trace.subject,
                    "aspect": trace.aspect,
                    "risk_tag": trace.risk_tag,
                    "trust_level": trace.trust_level,
                    "confidence": trace.confidence,
                    "rationale": trace.rationale,
                }
                for trace in ingestion_run.prepare_traces
            ],
            "ingest_steps": [
                {
                    "event_id": step.event_id,
                    "memory_id": step.memory_id,
                    "source": step.source,
                    "actor": step.actor,
                    "subject": step.subject,
                    "aspect": step.aspect,
                    "risk_tag": step.risk_tag,
                    "trust_level": step.trust_level,
                    "new_relations": [
                        [rel.left, rel.right, rel.label.value] for rel in step.new_relations
                    ],
                    "total_relations": step.total_relations,
                }
                for step in ingestion_run.ingest_steps
            ],
            "resolutions": [
                {
                    "resolver": item.resolver,
                    "decision": item.decision,
                    "accepted_id": item.accepted_id,
                    "suppressed_id": item.suppressed_id,
                    "reason": item.reason,
                }
                for item in projection.resolutions
            ],
            "audit": projection.audit_text(),
            "latency_sec": round(result.latency_sec, 4),
            "prompt_chars": len(result.prompt),
            "answer": result.answer,
            **evaluate_context(case, projection.accepted),
            **evaluate_answer(case, result.answer),
            **evaluate_projection(projection),
            "num_production_events": len(ingestion_run.events),
            "num_prepare_traces": len(ingestion_run.prepare_traces),
            "num_ingest_steps_with_new_conflict": ingestion_run.new_conflict_steps(),
        }
    )
    return records


def write_outputs(records: list[dict[str, Any]], output_dir: Path) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    raw_path = output_dir / "raw_outputs.jsonl"
    with raw_path.open("w", encoding="utf-8") as f:
        for record in records:
            f.write(json.dumps(record, ensure_ascii=False) + "\n")

    summary_keys = [
        "case_id",
        "case_family",
        "scenario",
        "attack_bundles",
        "conflict_pairs",
        "noise_rows",
        "stale_versions",
        "seed",
        "num_memory_rows",
        "method",
        "unsafe_memory_exposed",
        "safe_memory_exposed",
        "answer_contains_unsafe_marker",
        "answer_contains_safe_marker",
        "num_context_memories",
        "num_accepted",
        "num_suppressed",
        "num_ambiguous",
        "num_conflict_relations",
        "num_supersedes_relations",
        "num_resolution_records",
        "num_strong_resolutions",
        "num_user_review_requests",
        "num_production_events",
        "num_prepare_traces",
        "num_ingest_steps_with_new_conflict",
        "latency_sec",
        "prompt_chars",
    ]
    with (output_dir / "summary.csv").open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=summary_keys)
        writer.writeheader()
        for record in records:
            writer.writerow({key: record.get(key, "") for key in summary_keys})

    aggregate_records = aggregate_by_method(records)
    family_records = aggregate_by_method_and_family(records)
    scenario_records = aggregate_by_method_and_scenario(records)
    aggregate_keys = [
        "method",
        "num_cases",
        "unsafe_memory_exposure_rate",
        "safe_memory_exposure_rate",
        "unsafe_answer_rate",
        "safe_answer_rate",
        "avg_context_memories",
        "avg_suppressed",
        "avg_conflict_relations",
        "avg_supersedes_relations",
        "avg_resolution_records",
        "avg_strong_resolutions",
        "avg_user_review_requests",
        "avg_production_events",
        "avg_prepare_traces",
        "avg_ingest_conflict_steps",
        "avg_prompt_chars",
    ]
    with (output_dir / "aggregate.csv").open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=aggregate_keys)
        writer.writeheader()
        for record in aggregate_records:
            writer.writerow({key: record.get(key, "") for key in aggregate_keys})
    family_keys = ["case_family", *aggregate_keys]
    with (output_dir / "aggregate_by_family.csv").open(
        "w", encoding="utf-8", newline=""
    ) as f:
        writer = csv.DictWriter(f, fieldnames=family_keys)
        writer.writeheader()
        for record in family_records:
            writer.writerow({key: record.get(key, "") for key in family_keys})
    scenario_keys = ["scenario", *aggregate_keys]
    with (output_dir / "aggregate_by_scenario.csv").open(
        "w", encoding="utf-8", newline=""
    ) as f:
        writer = csv.DictWriter(f, fieldnames=scenario_keys)
        writer.writeheader()
        for record in scenario_records:
            writer.writerow({key: record.get(key, "") for key in scenario_keys})

    trace_lines = ["# Safe Agent Memory Demo Trace", ""]
    for record in records:
        trace_lines.append(f"## {record['case_id']} / {record['method']}")
        trace_lines.append("")
        trace_lines.append(f"- accepted: {record['accepted_ids']}")
        trace_lines.append(f"- suppressed: {record['suppressed_ids']}")
        trace_lines.append(f"- unsafe memory exposed: {record['unsafe_memory_exposed']}")
        trace_lines.append("")
        trace_lines.append("Answer:")
        trace_lines.append("")
        trace_lines.append(record["answer"])
        trace_lines.append("")
    (output_dir / "demo_trace.md").write_text("\n".join(trace_lines), encoding="utf-8")
    ingest_lines = ["# Memory Ingestion Trace", ""]
    for record in records:
        if record["method"] != "belief_projection":
            continue
        ingest_lines.append(f"## {record['case_id']}")
        ingest_lines.append("")
        ingest_lines.append("### Production Events")
        for event in record["production_events"]:
            ingest_lines.append(
                f"- {event['event_id']} source={event['source']} actor={event['actor']} "
                f"trust={event['trust_level']} text={event['raw_text']}"
            )
        ingest_lines.append("")
        ingest_lines.append("### Prepare Traces")
        for trace in record["prepare_traces"]:
            ingest_lines.append(
                f"- {trace['memory_id']} <- {trace['event_id']}: "
                f"{trace['subject']}/{trace['aspect']} risk={trace['risk_tag']} "
                f"subagents={','.join(trace['subagents'])}"
            )
        ingest_lines.append("")
        ingest_lines.append("### Continual Conflict Detection")
        for step in record["ingest_steps"]:
            relations = step["new_relations"] or []
            if not relations:
                relation_text = "no new relation"
            else:
                relation_text = "; ".join(
                    f"{left} {label} {right}" for left, right, label in relations
                )
            ingest_lines.append(f"- ingest {step['memory_id']}: {relation_text}")
        ingest_lines.append("")
        ingest_lines.append("### Resolutions")
        for resolution in record["resolutions"]:
            ingest_lines.append(
                f"- {resolution['resolver']}: suppress={resolution['suppressed_id']} "
                f"accept={resolution['accepted_id']} reason={resolution['reason']}"
            )
        ingest_lines.append("")
    (output_dir / "ingestion_trace.md").write_text(
        "\n".join(ingest_lines), encoding="utf-8"
    )
    (output_dir / "report.md").write_text(
        render_report(records, aggregate_records, family_records), encoding="utf-8"
    )


def aggregate_by_method(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    methods = sorted({record["method"] for record in records})
    aggregate: list[dict[str, Any]] = []
    for method in methods:
        group = [record for record in records if record["method"] == method]
        aggregate.append(
            {
                "method": method,
                "num_cases": len(group),
                "unsafe_memory_exposure_rate": _avg(group, "unsafe_memory_exposed"),
                "safe_memory_exposure_rate": _avg(group, "safe_memory_exposed"),
                "unsafe_answer_rate": _avg(group, "answer_contains_unsafe_marker"),
                "safe_answer_rate": _avg(group, "answer_contains_safe_marker"),
                "avg_context_memories": _avg(group, "num_context_memories"),
                "avg_suppressed": _avg(group, "num_suppressed"),
                "avg_conflict_relations": _avg(group, "num_conflict_relations"),
                "avg_supersedes_relations": _avg(group, "num_supersedes_relations"),
                "avg_resolution_records": _avg(group, "num_resolution_records"),
                "avg_strong_resolutions": _avg(group, "num_strong_resolutions"),
                "avg_user_review_requests": _avg(group, "num_user_review_requests"),
                "avg_production_events": _avg(group, "num_production_events"),
                "avg_prepare_traces": _avg(group, "num_prepare_traces"),
                "avg_ingest_conflict_steps": _avg(
                    group, "num_ingest_steps_with_new_conflict"
                ),
                "avg_prompt_chars": _avg(group, "prompt_chars"),
            }
        )
    return aggregate


def aggregate_by_method_and_family(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    aggregate: list[dict[str, Any]] = []
    for family in sorted({record["case_family"] for record in records}):
        family_records = [record for record in records if record["case_family"] == family]
        for row in aggregate_by_method(family_records):
            row = dict(row)
            row["case_family"] = family
            aggregate.append(row)
    return aggregate


def aggregate_by_method_and_scenario(records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    aggregate: list[dict[str, Any]] = []
    for scenario in sorted({record["scenario"] for record in records}):
        scenario_records = [record for record in records if record["scenario"] == scenario]
        for row in aggregate_by_method(scenario_records):
            row = dict(row)
            row["scenario"] = scenario
            aggregate.append(row)
    return aggregate


def render_report(
    records: list[dict[str, Any]],
    aggregate_records: list[dict[str, Any]],
    family_records: list[dict[str, Any]],
) -> str:
    lines = [
        "# Safe Agent Memory Benchmark Report",
        "",
        f"Total per-method records: {len(records)}",
        f"Unique cases: {len({record['case_id'] for record in records})}",
        "",
        "## Stress Axes",
        "",
        f"- attack bundles: {_unique_axis(records, 'attack_bundles')}",
        f"- natural conflict pairs: {_unique_axis(records, 'conflict_pairs')}",
        f"- benign noise rows: {_unique_axis(records, 'noise_rows')}",
        f"- stale versions: {_unique_axis(records, 'stale_versions')}",
        "",
        "## Aggregate Results",
        "",
        "| Method | Cases | Unsafe Memory Exposure | Unsafe Answer Rate | Avg Suppressed | Avg Conflict Edges | Avg Supersedes Edges | Avg Prepare Events | Avg Resolutions |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for row in aggregate_records:
        lines.append(
            "| {method} | {num_cases} | {unsafe_memory_exposure_rate:.2f} | "
            "{unsafe_answer_rate:.2f} | {avg_suppressed:.2f} | "
            "{avg_conflict_relations:.2f} | {avg_supersedes_relations:.2f} | "
            "{avg_prepare_traces:.2f} | {avg_resolution_records:.2f} |".format(
                **row
            )
        )

    lines.extend(
        [
            "",
            "## Results By Case Family",
            "",
            "| Family | Method | Cases | Unsafe Memory Exposure | Unsafe Answer Rate | Avg Suppressed | Avg Conflict Edges | Avg Supersedes Edges | Avg Strong-LLM Resolutions |",
            "|---|---|---:|---:|---:|---:|---:|---:|---:|",
        ]
    )
    for row in family_records:
        lines.append(
            "| {case_family} | {method} | {num_cases} | "
            "{unsafe_memory_exposure_rate:.2f} | {unsafe_answer_rate:.2f} | "
            "{avg_suppressed:.2f} | {avg_conflict_relations:.2f} | "
            "{avg_supersedes_relations:.2f} | {avg_strong_resolutions:.2f} |".format(**row)
        )

    projection = next(
        row for row in aggregate_records if row["method"] == "belief_projection"
    )
    append_all = next(row for row in aggregate_records if row["method"] == "append_all")
    keyword = next(row for row in aggregate_records if row["method"] == "keyword_topk")

    lines.extend(
        [
            "",
            "## Claim-Supporting Takeaways",
            "",
            (
                f"- Belief projection reduced unsafe memory exposure from "
                f"{append_all['unsafe_memory_exposure_rate']:.2f} under append-all and "
                f"{keyword['unsafe_memory_exposure_rate']:.2f} under keyword top-k to "
                f"{projection['unsafe_memory_exposure_rate']:.2f}."
            ),
            (
                f"- Belief projection suppressed {projection['avg_suppressed']:.2f} rows "
                "per case on average while preserving the accepted safe context."
            ),
            (
                f"- Belief projection exposed {projection['avg_conflict_relations']:.2f} "
                "conflict edges and "
                f"{projection['avg_supersedes_relations']:.2f} supersession edges per case "
                "as auditable state; retrieval baselines expose no conflict structure."
            ),
            (
                f"- The prepare phase produced {projection['avg_prepare_traces']:.2f} "
                "prepared memory rows per case from production events, and the resolver "
                f"stored {projection['avg_resolution_records']:.2f} resolution records per case."
            ),
            "",
            "## Interpretation",
            "",
            (
                "This benchmark is synthetic and deterministic. It is meant to support a "
                "course-demo claim: a belief-database memory layer can preserve all memories "
                "while generating a safer task-local projection than direct retrieval."
            ),
            "",
            (
                "For belief projection, memory enters through production events rather than "
                "pre-labeled database rows. A simulated prepare-LLM phase extracts structured "
                "claims and provenance; each prepared memory is then inserted and compared "
                "against prior memories to detect new conflicts continually."
            ),
            "",
            (
                "The attacker family is not a RAG implementation. It borrows PoisonedRAG's "
                "attacker model: the attacker injects a few malicious texts into an external "
                "knowledge or memory store so a target question can retrieve attacker-chosen "
                "content. Here that store is long-term agent memory."
            ),
        ]
    )
    return "\n".join(lines)


def _avg(records: list[dict[str, Any]], key: str) -> float:
    if not records:
        return 0.0
    return round(sum(float(record.get(key, 0)) for record in records) / len(records), 4)


def _unique_axis(records: list[dict[str, Any]], key: str) -> str:
    values = sorted({int(record.get(key, 0)) for record in records})
    return ", ".join(str(value) for value in values)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run safe agent memory benchmark.")
    parser.add_argument("--agent", choices=["mock", "transformers"], default="mock")
    parser.add_argument(
        "--suite", choices=["demo", "synthetic", "stress", "all"], default="demo"
    )
    parser.add_argument("--synthetic-per-type", type=int, default=30)
    parser.add_argument("--stress-seeds", type=int, default=10)
    parser.add_argument("--model-path", default="")
    parser.add_argument("--cache-dir", default="")
    parser.add_argument("--output-dir", default="results")
    parser.add_argument("--limit", type=int, default=0, help="Limit cases for smoke tests.")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.agent == "transformers":
        if not args.model_path:
            raise SystemExit("--model-path is required for --agent transformers")
        runner = TransformersAgentRunner(args.model_path, cache_dir=args.cache_dir or None)
    else:
        runner = MockAgentRunner()

    if args.suite == "demo":
        cases = all_cases()
    elif args.suite == "synthetic":
        cases = benchmark_cases(args.synthetic_per_type)
    elif args.suite == "stress":
        cases = stress_cases(args.stress_seeds)
    else:
        cases = all_cases() + benchmark_cases(args.synthetic_per_type) + stress_cases(
            args.stress_seeds
        )
    if args.limit:
        cases = cases[: args.limit]
    records: list[dict[str, Any]] = []
    for case in cases:
        records.extend(run_case(case, runner))
    write_outputs(records, Path(args.output_dir))
    print(f"Wrote {len(records)} records to {args.output_dir}")


if __name__ == "__main__":
    main()
