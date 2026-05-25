from __future__ import annotations

from collections import defaultdict

from .belief_db import MemoryRow


def append_all(rows: list[MemoryRow], task_subjects: list[str]) -> list[MemoryRow]:
    subjects = set(task_subjects)
    return [row for row in rows if row.subject in subjects]


def latest_only(rows: list[MemoryRow], task_subjects: list[str]) -> list[MemoryRow]:
    subjects = set(task_subjects)
    latest: dict[tuple[str, str], MemoryRow] = {}
    for row in rows:
        if row.subject not in subjects:
            continue
        key = row.domain
        if key not in latest or row.timestamp > latest[key].timestamp:
            latest[key] = row
    return sorted(latest.values(), key=lambda row: row.memory_id)


def keyword_topk(
    rows: list[MemoryRow], task_subjects: list[str], query: str, top_k: int = 4
) -> list[MemoryRow]:
    subjects = set(task_subjects)
    terms = {term.strip(".,?!:;").lower() for term in query.split()}
    scored: list[tuple[int, int, MemoryRow]] = []
    for row in rows:
        if row.subject not in subjects:
            continue
        text = f"{row.subject} {row.aspect} {row.claim}".lower()
        score = sum(1 for term in terms if term and term in text)
        scored.append((score, row.timestamp, row))
    scored.sort(key=lambda item: (item[0], item[1]), reverse=True)
    return [row for _, _, row in scored[:top_k]]


def relation_oracle(rows: list[MemoryRow], task_subjects: list[str]) -> list[MemoryRow]:
    """Upper-bound baseline: drop known attack/privacy rows but ignore conflicts."""
    subjects = set(task_subjects)
    return [
        row
        for row in rows
        if row.subject in subjects and row.risk_tag not in {"attack", "privacy_leak"}
    ]


BASELINES = {
    "append_all": append_all,
    "latest_only": latest_only,
    "keyword_topk": keyword_topk,
    "drop_known_attacks": relation_oracle,
}


def group_by_domain(rows: list[MemoryRow]) -> dict[tuple[str, str], list[MemoryRow]]:
    grouped: dict[tuple[str, str], list[MemoryRow]] = defaultdict(list)
    for row in rows:
        grouped[row.domain].append(row)
    return grouped
