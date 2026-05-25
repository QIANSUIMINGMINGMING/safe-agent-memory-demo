from __future__ import annotations

import argparse
from pathlib import Path

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt
import pandas as pd


METHOD_ORDER = [
    "append_all",
    "keyword_topk",
    "latest_only",
    "drop_known_attacks",
    "belief_projection",
]

METHOD_LABELS = {
    "append_all": "Append all",
    "keyword_topk": "Keyword top-k",
    "latest_only": "Latest only",
    "drop_known_attacks": "Drop known attacks",
    "belief_projection": "Belief projection",
}

COLORS = {
    "append_all": "#7f1d1d",
    "keyword_topk": "#b45309",
    "latest_only": "#4b5563",
    "drop_known_attacks": "#2563eb",
    "belief_projection": "#047857",
}


def main() -> None:
    args = parse_args()
    input_dir = Path(args.input_dir)
    output_dir = Path(args.output_dir) if args.output_dir else input_dir / "plots"
    output_dir.mkdir(parents=True, exist_ok=True)

    summary = pd.read_csv(input_dir / "summary.csv")
    summary = normalize_numeric_columns(summary)
    figures = [
        plot_unsafe_by_family(summary, output_dir),
        plot_unsafe_vs_attack_bundles(summary, output_dir),
        plot_unsafe_vs_conflict_pairs(summary, output_dir),
        plot_projection_audit_load(summary, output_dir),
        plot_prompt_size_vs_noise(summary, output_dir),
        plot_ingestion_resolution_load(summary, output_dir),
    ]
    write_manifest(figures, output_dir)
    print(f"Wrote {len(figures)} plots to {output_dir}")


def normalize_numeric_columns(summary: pd.DataFrame) -> pd.DataFrame:
    numeric_columns = [
        "attack_bundles",
        "conflict_pairs",
        "noise_rows",
        "stale_versions",
        "seed",
        "num_memory_rows",
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
    summary = summary.copy()
    for column in numeric_columns:
        if column in summary.columns:
            summary[column] = pd.to_numeric(summary[column], errors="coerce").fillna(0)
    return summary


def plot_unsafe_by_family(summary: pd.DataFrame, output_dir: Path) -> Path:
    grouped = (
        summary.groupby(["case_family", "method"], as_index=False)[
            "unsafe_memory_exposed"
        ]
        .mean()
        .sort_values(["case_family", "method"])
    )
    pivot = _ordered_pivot(grouped, "case_family", "unsafe_memory_exposed")
    path = output_dir / "01_unsafe_memory_by_family.png"
    _grouped_bars(
        pivot,
        title="Unsafe Memory Exposure by Scenario",
        ylabel="Exposure rate",
        xlabel="Case family",
        path=path,
    )
    return path


def plot_unsafe_vs_attack_bundles(summary: pd.DataFrame, output_dir: Path) -> Path:
    data = summary[
        (summary["case_family"] == "attacker") & (summary["attack_bundles"] > 0)
    ]
    grouped = data.groupby(["attack_bundles", "method"], as_index=False)[
        "unsafe_memory_exposed"
    ].mean()
    pivot = _ordered_pivot(grouped, "attack_bundles", "unsafe_memory_exposed")
    path = output_dir / "02_unsafe_vs_attack_bundles.png"
    _method_lines(
        pivot,
        title="Attacker Case: Exposure vs Poisoned Memory Bundles",
        ylabel="Exposure rate",
        xlabel="Poisoned bundle count",
        path=path,
    )
    return path


def plot_unsafe_vs_conflict_pairs(summary: pd.DataFrame, output_dir: Path) -> Path:
    data = summary[
        (summary["case_family"] == "conflict") & (summary["conflict_pairs"] > 0)
    ]
    grouped = data.groupby(["conflict_pairs", "method"], as_index=False)[
        "unsafe_memory_exposed"
    ].mean()
    pivot = _ordered_pivot(grouped, "conflict_pairs", "unsafe_memory_exposed")
    path = output_dir / "03_unsafe_vs_conflict_pairs.png"
    _method_lines(
        pivot,
        title="No-Attacker Case: Exposure vs Natural Conflict Pairs",
        ylabel="Exposure rate",
        xlabel="Natural conflict pair count",
        path=path,
    )
    return path


def plot_projection_audit_load(summary: pd.DataFrame, output_dir: Path) -> Path:
    data = summary[summary["method"] == "belief_projection"]
    grouped = data.groupby("case_family", as_index=False)[
        ["num_suppressed", "num_conflict_relations", "num_supersedes_relations"]
    ].mean()
    grouped = grouped.set_index("case_family")
    labels = {
        "num_suppressed": "Suppressed rows",
        "num_conflict_relations": "Conflict edges",
        "num_supersedes_relations": "Supersedes edges",
    }
    fig, ax = plt.subplots(figsize=(8.5, 4.8))
    x = range(len(grouped.index))
    width = 0.22
    colors = ["#047857", "#9333ea", "#0f766e"]
    for offset, (column, label) in enumerate(labels.items()):
        values = grouped[column].tolist()
        positions = [item + (offset - 1) * width for item in x]
        ax.bar(positions, values, width=width, label=label, color=colors[offset])
    ax.set_title("Belief Projection Audit Signal")
    ax.set_ylabel("Average count per case")
    ax.set_xlabel("Case family")
    ax.set_xticks(list(x), grouped.index.tolist())
    ax.spines[["top", "right"]].set_visible(False)
    ax.grid(axis="y", alpha=0.25)
    ax.legend(frameon=False, ncols=3, loc="upper center", bbox_to_anchor=(0.5, -0.12))
    path = output_dir / "04_projection_audit_load.png"
    fig.tight_layout()
    fig.savefig(path, dpi=220, bbox_inches="tight")
    plt.close(fig)
    return path


def plot_prompt_size_vs_noise(summary: pd.DataFrame, output_dir: Path) -> Path:
    grouped = summary.groupby(["noise_rows", "method"], as_index=False)[
        "prompt_chars"
    ].mean()
    pivot = _ordered_pivot(grouped, "noise_rows", "prompt_chars")
    path = output_dir / "05_prompt_size_vs_noise.png"
    _method_lines(
        pivot,
        title="Prompt Size Cost Under Benign Memory Noise",
        ylabel="Average prompt characters",
        xlabel="Benign noise memories per case",
        path=path,
        ylim_zero=False,
    )
    return path


def plot_ingestion_resolution_load(summary: pd.DataFrame, output_dir: Path) -> Path:
    data = summary[summary["method"] == "belief_projection"]
    grouped = data.groupby("case_family", as_index=False)[
        [
            "num_production_events",
            "num_prepare_traces",
            "num_resolution_records",
            "num_strong_resolutions",
            "num_ingest_steps_with_new_conflict",
        ]
    ].mean()
    grouped = grouped.set_index("case_family")
    labels = {
        "num_production_events": "Production events",
        "num_prepare_traces": "Prepared claims",
        "num_resolution_records": "Resolution records",
        "num_strong_resolutions": "Strong-LLM resolutions",
        "num_ingest_steps_with_new_conflict": "Conflict-detecting ingest steps",
    }
    fig, ax = plt.subplots(figsize=(9.5, 5))
    x = range(len(grouped.index))
    width = 0.15
    colors = ["#0f766e", "#2563eb", "#9333ea", "#b45309", "#b91c1c"]
    for offset, (column, label) in enumerate(labels.items()):
        values = grouped[column].tolist()
        positions = [item + (offset - 2) * width for item in x]
        ax.bar(positions, values, width=width, label=label, color=colors[offset])
    ax.set_title("Prepare and Continual Resolution Load")
    ax.set_ylabel("Average count per case")
    ax.set_xlabel("Case family")
    ax.set_xticks(list(x), grouped.index.tolist())
    ax.spines[["top", "right"]].set_visible(False)
    ax.grid(axis="y", alpha=0.25)
    ax.legend(frameon=False, ncols=2, loc="upper center", bbox_to_anchor=(0.5, -0.12))
    path = output_dir / "06_ingestion_resolution_load.png"
    fig.tight_layout()
    fig.savefig(path, dpi=220, bbox_inches="tight")
    plt.close(fig)
    return path


def _ordered_pivot(
    grouped: pd.DataFrame, index_column: str, value_column: str
) -> pd.DataFrame:
    pivot = grouped.pivot(index=index_column, columns="method", values=value_column)
    pivot = pivot[[method for method in METHOD_ORDER if method in pivot.columns]]
    return pivot.sort_index()


def _grouped_bars(
    pivot: pd.DataFrame, title: str, ylabel: str, xlabel: str, path: Path
) -> None:
    fig, ax = plt.subplots(figsize=(9, 5))
    x = list(range(len(pivot.index)))
    width = 0.15
    for idx, method in enumerate(pivot.columns):
        positions = [item + (idx - (len(pivot.columns) - 1) / 2) * width for item in x]
        ax.bar(
            positions,
            pivot[method].fillna(0).tolist(),
            width=width,
            label=METHOD_LABELS.get(method, method),
            color=COLORS.get(method, "#111827"),
        )
    ax.set_title(title)
    ax.set_ylabel(ylabel)
    ax.set_xlabel(xlabel)
    ax.set_ylim(0, 1.05)
    ax.set_xticks(x, [str(item) for item in pivot.index])
    ax.spines[["top", "right"]].set_visible(False)
    ax.grid(axis="y", alpha=0.25)
    ax.legend(frameon=False, ncols=3, loc="upper center", bbox_to_anchor=(0.5, -0.13))
    fig.tight_layout()
    fig.savefig(path, dpi=220, bbox_inches="tight")
    plt.close(fig)


def _method_lines(
    pivot: pd.DataFrame,
    title: str,
    ylabel: str,
    xlabel: str,
    path: Path,
    ylim_zero: bool = True,
) -> None:
    fig, ax = plt.subplots(figsize=(9, 5))
    for method in pivot.columns:
        ax.plot(
            pivot.index.tolist(),
            pivot[method].fillna(0).tolist(),
            marker="o",
            linewidth=2.2,
            label=METHOD_LABELS.get(method, method),
            color=COLORS.get(method, "#111827"),
        )
    ax.set_title(title)
    ax.set_ylabel(ylabel)
    ax.set_xlabel(xlabel)
    if ylim_zero:
        ax.set_ylim(0, 1.05)
    ax.spines[["top", "right"]].set_visible(False)
    ax.grid(axis="y", alpha=0.25)
    ax.legend(frameon=False, ncols=3, loc="upper center", bbox_to_anchor=(0.5, -0.13))
    fig.tight_layout()
    fig.savefig(path, dpi=220, bbox_inches="tight")
    plt.close(fig)


def write_manifest(figures: list[Path], output_dir: Path) -> None:
    descriptions = {
        "01_unsafe_memory_by_family.png": (
            "Main safety comparison across attacker and no-attacker conflict cases."
        ),
        "02_unsafe_vs_attack_bundles.png": (
            "How methods behave as the PoisonedRAG-style injected memory count grows."
        ),
        "03_unsafe_vs_conflict_pairs.png": (
            "How methods behave as natural memory conflicts become denser."
        ),
        "04_projection_audit_load.png": (
            "What the belief database records: suppressed rows and relation edges."
        ),
        "05_prompt_size_vs_noise.png": (
            "Prompt-size cost as benign memory noise grows."
        ),
        "06_ingestion_resolution_load.png": (
            "Production events, prepared claims, conflict-detecting ingest steps, and resolver decisions used by the belief projection pipeline."
        ),
    }
    lines = ["# Plot Manifest", ""]
    for path in figures:
        lines.append(f"- `{path.name}`: {descriptions.get(path.name, '')}")
    (output_dir / "plot_manifest.md").write_text("\n".join(lines), encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Plot safe memory benchmark results.")
    parser.add_argument("--input-dir", default="results/stress_mock")
    parser.add_argument("--output-dir", default="")
    return parser.parse_args()


if __name__ == "__main__":
    main()
