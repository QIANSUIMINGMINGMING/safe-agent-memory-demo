from safe_agent_memory.agent_loop import MockAgentRunner
from safe_agent_memory.bench import aggregate_by_method, run_case
from safe_agent_memory.cases import attacker_case, benchmark_cases, conflict_case, stress_cases
from safe_agent_memory.demo_page import render_page
from safe_agent_memory.ingestion import run_ingestion_pipeline


def test_benchmark_contains_all_methods() -> None:
    records = run_case(attacker_case(), MockAgentRunner())
    methods = {record["method"] for record in records}
    assert methods == {
        "append_all",
        "latest_only",
        "keyword_topk",
        "drop_known_attacks",
        "belief_projection",
    }


def test_belief_projection_reduces_attacker_exposure() -> None:
    records = run_case(attacker_case(), MockAgentRunner())
    by_method = {record["method"]: record for record in records}
    assert by_method["append_all"]["unsafe_memory_exposed"] == 1
    assert by_method["belief_projection"]["unsafe_memory_exposed"] == 0
    assert by_method["belief_projection"]["num_suppressed"] >= 2


def test_belief_projection_reduces_conflict_exposure() -> None:
    records = run_case(conflict_case(), MockAgentRunner())
    by_method = {record["method"]: record for record in records}
    assert by_method["append_all"]["unsafe_memory_exposed"] == 1
    assert by_method["belief_projection"]["unsafe_memory_exposed"] == 0
    assert by_method["belief_projection"]["num_suppressed"] >= 2


def test_generated_suite_supports_aggregate_claim() -> None:
    records = []
    for case in benchmark_cases(per_type=3):
        records.extend(run_case(case, MockAgentRunner()))
    aggregate = {row["method"]: row for row in aggregate_by_method(records)}

    assert aggregate["append_all"]["unsafe_memory_exposure_rate"] == 1.0
    assert aggregate["keyword_topk"]["unsafe_memory_exposure_rate"] > 0.0
    assert aggregate["belief_projection"]["unsafe_memory_exposure_rate"] == 0.0
    assert aggregate["belief_projection"]["avg_suppressed"] >= 2.0


def test_stress_suite_has_plot_axes_and_projection_signal() -> None:
    cases = stress_cases(seeds_per_setting=1)
    records = []
    for case in cases[:8]:
        records.extend(run_case(case, MockAgentRunner()))
    aggregate = {row["method"]: row for row in aggregate_by_method(records)}

    assert any(case.metadata["attack_bundles"] > 0 for case in cases)
    assert any(case.metadata["conflict_pairs"] > 0 for case in cases)
    assert any(case.metadata["noise_rows"] > 0 for case in cases)
    assert aggregate["belief_projection"]["unsafe_memory_exposure_rate"] == 0.0
    assert aggregate["belief_projection"]["avg_conflict_relations"] > 0.0


def test_demo_page_renders_mechanism(tmp_path) -> None:
    html = render_page(tmp_path, tmp_path / "demo_page.html", [attacker_case()])

    assert "Belief Database Memory Safety Demo" in html
    assert "Memory Production Events" in html
    assert "Prepare Phase" in html
    assert "Continual Conflict Detection" in html
    assert "Base Memory Store" in html
    assert "Agent-Facing Projection" in html
    assert "A3" in html


def test_ingestion_pipeline_prepares_and_detects_conflicts_continually() -> None:
    run = run_ingestion_pipeline(attacker_case())

    assert len(run.events) == len(attacker_case().rows)
    assert len(run.prepare_traces) == len(attacker_case().rows)
    assert run.new_conflict_steps() == 2
    assert any(step.memory_id == "A3" and step.new_relations for step in run.ingest_steps)
    assert {row.memory_id for row in run.projection.suppressed} >= {"A3", "A4"}


def test_ingestion_pipeline_uses_strong_reviewer_for_natural_conflict() -> None:
    run = run_ingestion_pipeline(conflict_case())

    assert any(
        resolution.resolver == "strong_llm_safety_reviewer"
        for resolution in run.projection.resolutions
    )
    assert {"C2", "C4", "C5"} <= {row.memory_id for row in run.projection.accepted}
