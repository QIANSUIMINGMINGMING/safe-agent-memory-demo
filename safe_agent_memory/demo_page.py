from __future__ import annotations

import argparse
import csv
import html
import os
from pathlib import Path

from .baselines import append_all
from .belief_db import BeliefDatabase, MemoryRow, Projection, RelationLabel
from .cases import DemoCase, attacker_case, conflict_case
from .ingestion import IngestionRun, run_ingestion_pipeline


PLOT_FILES = [
    "01_unsafe_memory_by_family.png",
    "02_unsafe_vs_attack_bundles.png",
    "03_unsafe_vs_conflict_pairs.png",
    "04_projection_audit_load.png",
    "05_prompt_size_vs_noise.png",
    "06_ingestion_resolution_load.png",
]


def main() -> None:
    args = parse_args()
    results_dir = Path(args.results_dir)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    html_text = render_page(
        results_dir=results_dir,
        output_path=output,
        cases=[attacker_case(), conflict_case()],
    )
    output.write_text(html_text, encoding="utf-8")
    print(f"Wrote demo page to {output}")


def render_page(
    results_dir: Path, output_path: Path, cases: list[DemoCase]
) -> str:
    case_sections = "\n".join(render_case_section(case) for case in cases)
    benchmark_table = render_benchmark_table(results_dir / "aggregate_by_family.csv")
    plot_grid = render_plot_grid(results_dir, output_path)
    return f"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Belief Database Memory Safety Demo</title>
  <style>
    :root {{
      --bg: #f8fafc;
      --panel: #ffffff;
      --text: #0f172a;
      --muted: #475569;
      --line: #cbd5e1;
      --safe: #047857;
      --bad: #b91c1c;
      --warn: #b45309;
      --blue: #1d4ed8;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    }}
    body {{
      margin: 0;
      background: var(--bg);
      color: var(--text);
    }}
    header {{
      padding: 42px 54px 28px;
      background: #0f172a;
      color: white;
    }}
    h1 {{
      margin: 0 0 12px;
      font-size: 38px;
      line-height: 1.1;
      letter-spacing: 0;
    }}
    h2 {{
      margin: 0 0 16px;
      font-size: 24px;
      letter-spacing: 0;
    }}
    h3 {{
      margin: 0 0 10px;
      font-size: 18px;
      letter-spacing: 0;
    }}
    p {{
      color: var(--muted);
      line-height: 1.55;
      margin: 0;
    }}
    header p {{
      color: #cbd5e1;
      max-width: 940px;
      font-size: 18px;
    }}
    main {{
      padding: 28px 54px 54px;
      max-width: 1340px;
      margin: 0 auto;
    }}
    section {{
      margin: 28px 0;
    }}
    .mechanism {{
      display: grid;
      grid-template-columns: repeat(4, minmax(0, 1fr));
      gap: 14px;
    }}
    .step, .panel {{
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 16px;
    }}
    .step strong {{
      display: block;
      margin-bottom: 8px;
      font-size: 16px;
    }}
    .case-grid {{
      display: grid;
      grid-template-columns: 1.25fr 0.8fr 1fr;
      gap: 14px;
      align-items: start;
    }}
    table {{
      width: 100%;
      border-collapse: collapse;
      font-size: 13px;
    }}
    th, td {{
      border-bottom: 1px solid #e2e8f0;
      padding: 8px 7px;
      vertical-align: top;
      text-align: left;
    }}
    th {{
      color: #334155;
      font-weight: 700;
      background: #f1f5f9;
    }}
    .claim {{
      min-width: 280px;
    }}
    .tag {{
      display: inline-block;
      padding: 3px 7px;
      border-radius: 999px;
      font-size: 12px;
      font-weight: 700;
      white-space: nowrap;
    }}
    .accepted {{
      color: var(--safe);
      background: #dcfce7;
    }}
    .suppressed {{
      color: var(--bad);
      background: #fee2e2;
    }}
    .ambiguous {{
      color: var(--warn);
      background: #fef3c7;
    }}
    .edge {{
      border-left: 4px solid var(--line);
      margin: 0 0 9px;
      padding: 8px 10px;
      background: #f8fafc;
      border-radius: 6px;
      font-size: 13px;
    }}
    .edge.conflict {{
      border-left-color: var(--bad);
    }}
    .edge.supersedes {{
      border-left-color: var(--blue);
    }}
    .prompt {{
      background: #0f172a;
      color: #e2e8f0;
      border-radius: 8px;
      padding: 13px;
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      font-size: 12px;
      line-height: 1.5;
      white-space: pre-wrap;
    }}
    .compare {{
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 14px;
      margin-top: 14px;
    }}
    .bad-border {{
      border-color: #fecaca;
    }}
    .safe-border {{
      border-color: #bbf7d0;
    }}
    .plots {{
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 16px;
    }}
    figure {{
      margin: 0;
      background: white;
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 12px;
    }}
    img {{
      width: 100%;
      display: block;
    }}
    figcaption {{
      margin-top: 8px;
      color: var(--muted);
      font-size: 13px;
    }}
    @media (max-width: 1000px) {{
      header, main {{
        padding-left: 22px;
        padding-right: 22px;
      }}
      .mechanism, .case-grid, .compare, .plots {{
        grid-template-columns: 1fr;
      }}
      h1 {{
        font-size: 30px;
      }}
    }}
  </style>
</head>
<body>
  <header>
    <h1>Belief Database Memory Safety Demo</h1>
    <p>How a conflict-aware memory layer prevents bad memory from entering the agent prompt while preserving the original memory store for audit.</p>
  </header>
  <main>
    <section>
      <h2>Mechanism</h2>
      <div class="mechanism">
        <div class="step"><strong>1. Production</strong><p>Conversation, tool output, policy, or injected text creates a production event with provenance.</p></div>
        <div class="step"><strong>2. Prepare</strong><p>Simulated LLM subagents extract subject, aspect, risk, trust, and normalized claim rows.</p></div>
        <div class="step"><strong>3. Continual Compare</strong><p>Each prepared row is inserted and immediately compared with prior rows in the same domain.</p></div>
        <div class="step"><strong>4. Resolve and Project</strong><p>Rules or a strong-LLM reviewer suppress unsafe/stale rows before the working-agent prompt.</p></div>
      </div>
    </section>

    {case_sections}

    <section>
      <h2>Benchmark Snapshot</h2>
      <p>These numbers come from the generated stress suite if it has already been run in the selected results directory.</p>
      {benchmark_table}
    </section>

    <section>
      <h2>Showable Figures</h2>
      <p>The plots give the presentation more depth than a single aggregate table: they show attack intensity, natural conflict density, audit signal, and prompt-size cost.</p>
      {plot_grid}
    </section>
  </main>
</body>
</html>
"""


def render_case_section(case: DemoCase) -> str:
    ingestion_run = run_ingestion_pipeline(case)
    projection = ingestion_run.projection
    append_rows = append_all(case.rows, case.task_subjects)
    return f"""
    <section>
      <h2>{escape(case.title)}</h2>
      <div class="compare">
        <div class="panel">
          <h3>Memory Production Events</h3>
          {render_production_events(ingestion_run)}
        </div>
        <div class="panel">
          <h3>Prepare Phase</h3>
          {render_prepare_traces(ingestion_run)}
        </div>
      </div>
      <div class="panel" style="margin-top:14px;">
        <h3>Continual Conflict Detection</h3>
        {render_ingest_steps(ingestion_run)}
      </div>
      <div class="case-grid">
        <div class="panel">
          <h3>Base Memory Store</h3>
          {render_memory_table(ingestion_run.rows, projection)}
        </div>
        <div class="panel">
          <h3>Belief Relations</h3>
          {render_relations(projection)}
        </div>
        <div class="panel">
          <h3>Agent-Facing Projection</h3>
          {render_projection(projection)}
        </div>
      </div>
      <div class="compare">
        <div class="panel bad-border">
          <h3>Direct Retrieval Context</h3>
          <div class="prompt">{escape(context_text(append_rows))}</div>
        </div>
        <div class="panel safe-border">
          <h3>Belief Projection Context</h3>
          <div class="prompt">{escape(context_text(projection.accepted))}</div>
        </div>
      </div>
    </section>
    """


def render_memory_table(rows: list[MemoryRow], projection: Projection) -> str:
    fate = {}
    for row in projection.accepted:
        fate[row.memory_id] = "accepted"
    for row in projection.suppressed:
        fate[row.memory_id] = "suppressed"
    for row in projection.ambiguous:
        fate[row.memory_id] = "ambiguous"

    lines = [
        "<table>",
        "<thead><tr><th>ID</th><th>Source</th><th>Trust</th><th>Claim</th><th>Projection</th></tr></thead>",
        "<tbody>",
    ]
    for row in rows:
        label = fate.get(row.memory_id, "out-of-scope")
        css = label if label in {"accepted", "suppressed", "ambiguous"} else "ambiguous"
        lines.append(
            "<tr>"
            f"<td>{escape(row.memory_id)}</td>"
            f"<td>{escape(row.source)}</td>"
            f"<td>{escape(row.trust_level)}</td>"
            f"<td class=\"claim\">{escape(row.claim)}</td>"
            f"<td><span class=\"tag {css}\">{escape(label)}</span></td>"
            "</tr>"
        )
    lines.extend(["</tbody>", "</table>"])
    return "\n".join(lines)


def render_production_events(ingestion_run: IngestionRun) -> str:
    lines = [
        "<table>",
        "<thead><tr><th>Event</th><th>Actor</th><th>Producer</th><th>Trust</th><th>Raw memory</th></tr></thead>",
        "<tbody>",
    ]
    for event in ingestion_run.events:
        lines.append(
            "<tr>"
            f"<td>{escape(event.event_id)}</td>"
            f"<td>{escape(event.actor)}</td>"
            f"<td>{escape(event.producer_model)}</td>"
            f"<td>{escape(event.trust_level)}</td>"
            f"<td class=\"claim\">{escape(event.raw_text)}</td>"
            "</tr>"
        )
    lines.extend(["</tbody>", "</table>"])
    return "\n".join(lines)


def render_prepare_traces(ingestion_run: IngestionRun) -> str:
    lines = [
        "<table>",
        "<thead><tr><th>Memory</th><th>Domain</th><th>Risk</th><th>Subagents</th></tr></thead>",
        "<tbody>",
    ]
    for trace in ingestion_run.prepare_traces:
        lines.append(
            "<tr>"
            f"<td>{escape(trace.memory_id)}</td>"
            f"<td>{escape(trace.subject)}/{escape(trace.aspect)}</td>"
            f"<td>{escape(trace.risk_tag)}</td>"
            f"<td>{escape(', '.join(trace.subagents))}</td>"
            "</tr>"
        )
    lines.extend(["</tbody>", "</table>"])
    return "\n".join(lines)


def render_ingest_steps(ingestion_run: IngestionRun) -> str:
    lines = [
        "<table>",
        "<thead><tr><th>Step</th><th>Prepared row</th><th>New relation found now</th><th>Total relations</th></tr></thead>",
        "<tbody>",
    ]
    for idx, step in enumerate(ingestion_run.ingest_steps, start=1):
        if not step.new_relations:
            relation_text = "none"
        else:
            relation_text = "; ".join(
                f"{rel.left} {rel.label.value} {rel.right}" for rel in step.new_relations
            )
        lines.append(
            "<tr>"
            f"<td>{idx}</td>"
            f"<td>{escape(step.memory_id)} ({escape(step.subject)}/{escape(step.aspect)})</td>"
            f"<td>{escape(relation_text)}</td>"
            f"<td>{escape(step.total_relations)}</td>"
            "</tr>"
        )
    lines.extend(["</tbody>", "</table>"])
    return "\n".join(lines)


def render_relations(projection: Projection) -> str:
    relation_lines = []
    for relation in projection.relations:
        if relation.label == RelationLabel.NONE:
            continue
        relation_lines.append(
            f"<div class=\"edge {escape(relation.label.value)}\">"
            f"<strong>{escape(relation.left)} {escape(relation.label.value)} {escape(relation.right)}</strong><br>"
            f"{escape(relation.reason)}"
            "</div>"
        )
    if not relation_lines:
        return "<p>No relation edge is needed for this task.</p>"
    return "\n".join(relation_lines)


def render_projection(projection: Projection) -> str:
    lines = ["<table>", "<thead><tr><th>Fate</th><th>Memory</th></tr></thead>", "<tbody>"]
    for label, rows in [
        ("accepted", projection.accepted),
        ("suppressed", projection.suppressed),
        ("ambiguous", projection.ambiguous),
    ]:
        for row in rows:
            lines.append(
                "<tr>"
                f"<td><span class=\"tag {label}\">{label}</span></td>"
                f"<td>{escape(row.memory_id)}: {escape(row.claim)}</td>"
                "</tr>"
            )
    lines.extend(["</tbody>", "</table>"])
    notes = "".join(f"<li>{escape(note)}</li>" for note in projection.notes)
    resolution_notes = "".join(
        f"<li>{escape(item.resolver)}: accept {escape(item.accepted_id)}; "
        f"suppress {escape(item.suppressed_id)}; {escape(item.reason)}</li>"
        for item in projection.resolutions
    )
    return (
        "\n".join(lines)
        + f"<p style=\"margin-top:10px;\">Projection notes:</p><ul>{notes}</ul>"
        + f"<p>Resolver records:</p><ul>{resolution_notes}</ul>"
    )


def render_benchmark_table(path: Path) -> str:
    if not path.exists():
        return "<p>No aggregate table found. Run the stress benchmark first.</p>"
    rows: list[dict[str, str]] = []
    with path.open(encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            if row["method"] in {"append_all", "latest_only", "drop_known_attacks", "belief_projection"}:
                rows.append(row)
    lines = [
        "<table>",
        "<thead><tr><th>Family</th><th>Method</th><th>Cases</th><th>Unsafe exposure</th><th>Avg suppressed</th><th>Avg conflict edges</th></tr></thead>",
        "<tbody>",
    ]
    for row in rows:
        lines.append(
            "<tr>"
            f"<td>{escape(row['case_family'])}</td>"
            f"<td>{escape(row['method'])}</td>"
            f"<td>{escape(row['num_cases'])}</td>"
            f"<td>{float(row['unsafe_memory_exposure_rate']):.2f}</td>"
            f"<td>{float(row['avg_suppressed']):.2f}</td>"
            f"<td>{float(row['avg_conflict_relations']):.2f}</td>"
            "</tr>"
        )
    lines.extend(["</tbody>", "</table>"])
    return "\n".join(lines)


def render_plot_grid(results_dir: Path, output_path: Path) -> str:
    descriptions = {
        "01_unsafe_memory_by_family.png": "Main safety result by case family.",
        "02_unsafe_vs_attack_bundles.png": "Attacker pressure increases with more injected memory bundles.",
        "03_unsafe_vs_conflict_pairs.png": "Natural memory conflict becomes denser without any attacker.",
        "04_projection_audit_load.png": "Projection creates inspectable suppression and relation edges.",
        "05_prompt_size_vs_noise.png": "Noise increases context cost for retrieval-style methods.",
        "06_ingestion_resolution_load.png": "Prepare and resolver load from production events.",
    }
    figures = []
    for filename in PLOT_FILES:
        path = results_dir / "plots" / filename
        if not path.exists():
            continue
        rel = os.path.relpath(path, output_path.parent)
        figures.append(
            "<figure>"
            f"<img src=\"{escape(rel)}\" alt=\"{escape(descriptions[filename])}\">"
            f"<figcaption>{escape(descriptions[filename])}</figcaption>"
            "</figure>"
        )
    if not figures:
        return "<p>No plots found. Run the plotting command first.</p>"
    return f"<div class=\"plots\">{''.join(figures)}</div>"


def context_text(rows: list[MemoryRow]) -> str:
    if not rows:
        return "- No memory is available."
    return "\n".join(f"- [{row.memory_id}] {row.claim}" for row in rows)


def escape(value: object) -> str:
    return html.escape(str(value), quote=True)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Render a static demo page.")
    parser.add_argument("--results-dir", default="results/stress_mock")
    parser.add_argument("--output", default="results/stress_mock/demo_page.html")
    return parser.parse_args()


if __name__ == "__main__":
    main()
