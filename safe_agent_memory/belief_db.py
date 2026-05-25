from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum
from typing import Any, Iterable


class RelationLabel(StrEnum):
    SUPPORT = "support"
    CONFLICT = "conflict"
    SUPERSEDES = "supersedes"
    NONE = "none"


@dataclass(frozen=True)
class MemoryRow:
    case_id: str
    memory_id: str
    source: str
    timestamp: int
    subject: str
    aspect: str
    claim: str
    risk_tag: str = "benign"
    event_id: str = ""
    actor: str = ""
    producer: str = ""
    trust_level: int = 50
    confidence: float = 1.0
    evidence: str = ""

    @property
    def domain(self) -> tuple[str, str]:
        return (self.subject, self.aspect)


@dataclass(frozen=True)
class Relation:
    left: str
    right: str
    label: RelationLabel
    reason: str

    def as_pair(self) -> tuple[str, str]:
        return tuple(sorted((self.left, self.right)))


@dataclass(frozen=True)
class ResolutionRecord:
    relation: Relation
    resolver: str
    decision: str
    accepted_id: str
    suppressed_id: str
    reason: str


@dataclass
class Projection:
    accepted: list[MemoryRow]
    suppressed: list[MemoryRow]
    ambiguous: list[MemoryRow]
    relations: list[Relation]
    resolutions: list[ResolutionRecord] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    def context_text(self) -> str:
        lines = []
        for row in self.accepted:
            lines.append(f"- [{row.memory_id}] {row.claim}")
        if not lines:
            return "- No safe memory was accepted for this task."
        return "\n".join(lines)

    def audit_text(self) -> str:
        lines = ["Accepted memories:"]
        lines.extend(f"- {row.memory_id}: {row.claim}" for row in self.accepted)
        lines.append("Suppressed memories:")
        lines.extend(f"- {row.memory_id}: {row.claim}" for row in self.suppressed)
        lines.append("Ambiguous memories:")
        lines.extend(f"- {row.memory_id}: {row.claim}" for row in self.ambiguous)
        lines.append("Relations:")
        lines.extend(
            f"- {rel.left} {rel.label.value} {rel.right}: {rel.reason}"
            for rel in self.relations
            if rel.label != RelationLabel.NONE
        )
        lines.append("Resolutions:")
        lines.extend(
            f"- {item.resolver}: {item.decision}; accepted={item.accepted_id}; "
            f"suppressed={item.suppressed_id}; reason={item.reason}"
            for item in self.resolutions
        )
        lines.append("Notes:")
        lines.extend(f"- {note}" for note in self.notes)
        return "\n".join(lines)


class BeliefDatabase:
    """Tiny deterministic belief database for the course demo.

    The base store keeps every memory row. Projection decides what the
    downstream agent may consume for a specific task.
    """

    def __init__(self, rows: Iterable[MemoryRow] = ()) -> None:
        self._rows: dict[str, MemoryRow] = {}
        self._relations: list[Relation] = []
        self._events: dict[str, Any] = {}
        for row in rows:
            self.persist(row)

    def persist(self, row: MemoryRow) -> None:
        if row.memory_id in self._rows:
            raise ValueError(f"duplicate memory_id: {row.memory_id}")
        self._rows[row.memory_id] = row

    def rows(self) -> list[MemoryRow]:
        return sorted(self._rows.values(), key=lambda row: (row.timestamp, row.memory_id))

    def relations(self) -> list[Relation]:
        return list(self._relations)

    def events(self) -> dict[str, Any]:
        return dict(self._events)

    def ingest_prepared_memory(self, row: MemoryRow, event: Any | None = None) -> list[Relation]:
        """Persist one prepared memory and compare it to existing rows.

        This is the continual-conflict path used by the project demo: each new
        memory-production event is prepared into a claim, inserted, and then
        immediately compared against prior claims in the same domain.
        """

        if row.memory_id in self._rows:
            raise ValueError(f"duplicate memory_id: {row.memory_id}")
        new_relations = []
        for existing in self.rows():
            relation = judge_relation(existing, row)
            if relation.label != RelationLabel.NONE:
                new_relations.append(relation)
        self._rows[row.memory_id] = row
        if event is not None and row.event_id:
            self._events[row.event_id] = event
        self._relations.extend(new_relations)
        return list(new_relations)

    def build_relations(self) -> list[Relation]:
        self._relations = []
        rows = self.rows()
        for idx, left in enumerate(rows):
            for right in rows[idx + 1 :]:
                relation = judge_relation(left, right)
                if relation.label != RelationLabel.NONE:
                    self._relations.append(relation)
        return self.relations()

    def project(self, task_subjects: Iterable[str] | None = None) -> Projection:
        if not self._relations:
            self.build_relations()

        subjects = set(task_subjects or [])
        candidate_rows = [
            row for row in self.rows() if not subjects or row.subject in subjects
        ]
        by_id = {row.memory_id: row for row in candidate_rows}
        accepted_ids = set(by_id)
        suppressed_ids: set[str] = set()
        ambiguous_ids: set[str] = set()
        resolutions: list[ResolutionRecord] = []
        notes: list[str] = []

        relevant_relations = [
            rel for rel in self._relations if rel.left in by_id and rel.right in by_id
        ]

        for rel in relevant_relations:
            left = by_id[rel.left]
            right = by_id[rel.right]
            if rel.label == RelationLabel.SUPERSEDES:
                older = left if left.timestamp < right.timestamp else right
                newer = right if older == left else left
                suppressed_ids.add(older.memory_id)
                resolutions.append(
                    ResolutionRecord(
                        relation=rel,
                        resolver="deterministic_freshness_rule",
                        decision="accept_newer_memory",
                        accepted_id=newer.memory_id,
                        suppressed_id=older.memory_id,
                        reason="newer production event updates the same subject/aspect",
                    )
                )
                notes.append(
                    f"{older.memory_id} suppressed because it is superseded in {older.subject}/{older.aspect}."
                )
            elif rel.label == RelationLabel.CONFLICT:
                resolution = resolve_conflict(left, right, rel)
                resolutions.append(resolution)
                if resolution.suppressed_id:
                    suppressed_ids.add(resolution.suppressed_id)
                    notes.append(
                        f"{resolution.suppressed_id} suppressed by {resolution.resolver}: {resolution.reason}."
                    )
                else:
                    ambiguous_ids.update([left.memory_id, right.memory_id])
                    notes.append(
                        f"{left.memory_id}/{right.memory_id} left ambiguous because conflict has no safe winner."
                    )

        accepted_ids -= suppressed_ids
        accepted_ids -= ambiguous_ids

        return Projection(
            accepted=[by_id[mid] for mid in sorted(accepted_ids)],
            suppressed=[by_id[mid] for mid in sorted(suppressed_ids)],
            ambiguous=[by_id[mid] for mid in sorted(ambiguous_ids)],
            relations=relevant_relations,
            resolutions=resolutions,
            notes=notes,
        )


def judge_relation(left: MemoryRow, right: MemoryRow) -> Relation:
    if left.domain != right.domain:
        return Relation(left.memory_id, right.memory_id, RelationLabel.NONE, "different domain")

    if _looks_like_update(left, right):
        newer = left if left.timestamp > right.timestamp else right
        older = right if newer == left else left
        return Relation(
            newer.memory_id,
            older.memory_id,
            RelationLabel.SUPERSEDES,
            "newer memory updates the same subject/aspect",
        )

    if _has_attack_or_privacy_conflict(left, right):
        return Relation(
            left.memory_id,
            right.memory_id,
            RelationLabel.CONFLICT,
            "unsafe or injected memory conflicts with safer provenance or policy",
        )

    if _has_direct_conflict(left, right):
        return Relation(
            left.memory_id,
            right.memory_id,
            RelationLabel.CONFLICT,
            "claims disagree in the same comparison domain",
        )

    if _shares_policy_direction(left, right):
        return Relation(
            left.memory_id,
            right.memory_id,
            RelationLabel.SUPPORT,
            "claims are compatible in the same comparison domain",
        )

    return Relation(left.memory_id, right.memory_id, RelationLabel.NONE, "no semantic relation")


def _unsafe_side(left: MemoryRow, right: MemoryRow) -> MemoryRow | None:
    unsafe_tags = {"attack", "privacy_leak", "unsafe"}
    left_unsafe = left.risk_tag in unsafe_tags
    right_unsafe = right.risk_tag in unsafe_tags
    if left_unsafe and not right_unsafe:
        return left
    if right_unsafe and not left_unsafe:
        return right
    if left.risk_tag in {"policy", "safety"} and right.risk_tag == "benign":
        return right
    if right.risk_tag in {"policy", "safety"} and left.risk_tag == "benign":
        return left
    return None


def resolve_conflict(left: MemoryRow, right: MemoryRow, relation: Relation) -> ResolutionRecord:
    unsafe = _unsafe_side(left, right)
    if unsafe is None and abs(left.trust_level - right.trust_level) >= 30:
        unsafe = left if left.trust_level < right.trust_level else right
    if unsafe is None:
        return ResolutionRecord(
            relation=relation,
            resolver="user_review_needed",
            decision="quarantine_both",
            accepted_id="",
            suppressed_id="",
            reason="conflict has no clear safer or higher-trust side",
        )

    accepted = right if unsafe == left else left
    resolver = "strong_llm_safety_reviewer"
    reason = "higher-trust provenance resolves a natural safety conflict"
    if unsafe.risk_tag in {"attack", "privacy_leak"}:
        resolver = "deterministic_attack_policy"
        reason = "untrusted injected or privacy-leaking memory conflicts with safer memory"
    elif accepted.risk_tag in {"policy", "safety"} and unsafe.risk_tag == "benign":
        resolver = "strong_llm_safety_reviewer"
        reason = "safety/ethics evidence overrides lower-trust benign memory"

    return ResolutionRecord(
        relation=relation,
        resolver=resolver,
        decision="suppress_lower_trust_or_unsafe_memory",
        accepted_id=accepted.memory_id,
        suppressed_id=unsafe.memory_id,
        reason=reason,
    )


def _has_attack_or_privacy_conflict(left: MemoryRow, right: MemoryRow) -> bool:
    unsafe = _unsafe_side(left, right)
    if unsafe is None:
        return False
    other = right if unsafe == left else left
    return other.risk_tag in {"policy", "benign", "safety"}


def _has_direct_conflict(left: MemoryRow, right: MemoryRow) -> bool:
    text = f"{left.claim.lower()} || {right.claim.lower()}"
    conflict_markers = [
        ("include passport", "exclude sensitive"),
        ("send", "do not send"),
        ("ignore approval", "require approval"),
        ("ignore approval", "requires approval"),
        ("beijing", "shanghai"),
        ("beijing", "shenzhen"),
        ("beijing", "hangzhou"),
        ("public dataset", "unauthorized face"),
        ("can be shared", "must not be shared"),
        ("include email addresses", "remove email addresses"),
        ("can be published", "must stay private"),
        ("may be included", "must be removed"),
        ("send private", "stay inside"),
        ("skip receipt", "verify receipts"),
    ]
    return any(a in text and b in text for a, b in conflict_markers)


def _looks_like_update(left: MemoryRow, right: MemoryRow) -> bool:
    if left.subject == "user_profile" and left.aspect in {"location", "preference"}:
        return left.timestamp != right.timestamp
    return False


def _shares_policy_direction(left: MemoryRow, right: MemoryRow) -> bool:
    safe_tags = {"policy", "benign", "safety"}
    return left.risk_tag in safe_tags and right.risk_tag in safe_tags
