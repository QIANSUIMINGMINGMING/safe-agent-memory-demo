from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from .belief_db import BeliefDatabase, MemoryRow, Projection, Relation
from .cases import DemoCase


@dataclass(frozen=True)
class ProductionEvent:
    """Raw memory-production event before it becomes a belief row."""

    event_id: str
    case_id: str
    source: str
    actor: str
    timestamp: int
    raw_text: str
    producer_model: str
    production_prompt: str
    trust_level: int
    confidence: float
    permissions: str = "task_view"
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class PrepareTrace:
    """Trace of the simulated LLM prepare phase."""

    event_id: str
    memory_id: str
    subagents: list[str]
    subject: str
    aspect: str
    risk_tag: str
    trust_level: int
    confidence: float
    rationale: str


@dataclass(frozen=True)
class IngestStep:
    event_id: str
    memory_id: str
    source: str
    actor: str
    subject: str
    aspect: str
    risk_tag: str
    trust_level: int
    new_relations: list[Relation]
    total_relations: int


@dataclass
class IngestionRun:
    events: list[ProductionEvent]
    prepare_traces: list[PrepareTrace]
    ingest_steps: list[IngestStep]
    db: BeliefDatabase
    projection: Projection

    @property
    def rows(self) -> list[MemoryRow]:
        return self.db.rows()

    def new_conflict_steps(self) -> int:
        return sum(
            1
            for step in self.ingest_steps
            if any(relation.label == "conflict" for relation in step.new_relations)
        )


class SimulatedPrepareLLM:
    """Small deterministic stand-in for LLM prepare subagents.

    The real design would call an LLM to extract task tags, normalize claims,
    estimate risk, and score provenance. The course demo keeps the same data
    contract but implements it deterministically for reproducible results.
    """

    subagents = [
        "task_tagger_llm",
        "claim_extractor_llm",
        "risk_tagger_llm",
        "provenance_scorer_llm",
    ]

    def prepare(self, event: ProductionEvent) -> tuple[MemoryRow, PrepareTrace]:
        subject = str(event.metadata.get("subject") or infer_subject(event.raw_text))
        aspect = str(event.metadata.get("aspect") or infer_aspect(event.raw_text))
        risk_tag = infer_risk_tag(event)
        memory_id = str(event.metadata.get("memory_id") or event.event_id.replace("E", "M", 1))
        row = MemoryRow(
            case_id=event.case_id,
            memory_id=memory_id,
            source=event.source,
            timestamp=event.timestamp,
            subject=subject,
            aspect=aspect,
            claim=event.raw_text,
            risk_tag=risk_tag,
            event_id=event.event_id,
            actor=event.actor,
            producer=event.producer_model,
            trust_level=event.trust_level,
            confidence=event.confidence,
            evidence=event.raw_text,
        )
        trace = PrepareTrace(
            event_id=event.event_id,
            memory_id=memory_id,
            subagents=list(self.subagents),
            subject=subject,
            aspect=aspect,
            risk_tag=risk_tag,
            trust_level=event.trust_level,
            confidence=event.confidence,
            rationale=(
                "prepared from production provenance: source="
                f"{event.source}, actor={event.actor}, trust={event.trust_level}"
            ),
        )
        return row, trace


def run_ingestion_pipeline(
    case: DemoCase,
    preparer: SimulatedPrepareLLM | None = None,
) -> IngestionRun:
    preparer = preparer or SimulatedPrepareLLM()
    db = BeliefDatabase()
    events = events_from_case(case)
    traces: list[PrepareTrace] = []
    steps: list[IngestStep] = []

    for event in events:
        row, trace = preparer.prepare(event)
        traces.append(trace)
        new_relations = db.ingest_prepared_memory(row, event)
        steps.append(
            IngestStep(
                event_id=event.event_id,
                memory_id=row.memory_id,
                source=event.source,
                actor=event.actor,
                subject=row.subject,
                aspect=row.aspect,
                risk_tag=row.risk_tag,
                trust_level=row.trust_level,
                new_relations=new_relations,
                total_relations=len(db.relations()),
            )
        )

    projection = db.project(case.task_subjects)
    return IngestionRun(
        events=events,
        prepare_traces=traces,
        ingest_steps=steps,
        db=db,
        projection=projection,
    )


def events_from_case(case: DemoCase) -> list[ProductionEvent]:
    events = []
    for row in sorted(case.rows, key=lambda item: (item.timestamp, item.memory_id)):
        trust_level, confidence, actor, producer_model = provenance_for_source(row.source)
        events.append(
            ProductionEvent(
                event_id=f"{row.memory_id}_event",
                case_id=row.case_id,
                source=row.source,
                actor=actor,
                timestamp=row.timestamp,
                raw_text=row.claim,
                producer_model=producer_model,
                production_prompt=(
                    "Simulated prepare prompt: extract subject, aspect, claim, "
                    "risk, and provenance from this memory-producing event."
                ),
                trust_level=trust_level,
                confidence=confidence,
                permissions="task_view",
                metadata={
                    "memory_id": row.memory_id,
                    "subject": row.subject,
                    "aspect": row.aspect,
                },
            )
        )
    return events


def provenance_for_source(source: str) -> tuple[int, float, str, str]:
    if source in {"system_policy", "user_policy", "privacy_policy", "policy_memory"}:
        return 95, 0.96, "policy_authority", "prepare-llm-sim"
    if source == "ethics_review":
        return 92, 0.94, "strong_llm_safety_reviewer", "strong-review-sim"
    if source == "user_recent":
        return 85, 0.90, "user_confirmed", "prepare-llm-sim"
    if source == "user_old":
        return 62, 0.78, "user_historical", "prepare-llm-sim"
    if source == "poisonedrag_injected_text":
        return 10, 0.35, "untrusted_external_writer", "unknown"
    if source in {"dataset_note", "dataset_note_old"}:
        return 50, 0.70, "weak_agent_or_note", "small-agent-sim"
    if source == "prior_agent":
        return 55, 0.72, "working_agent", "small-agent-sim"
    return 45, 0.65, "unknown", "small-agent-sim"


def infer_subject(text: str) -> str:
    lowered = text.lower()
    if "reimbursement" in lowered or "approval" in lowered:
        return "approval_task"
    if "dataset" in lowered or "face data" in lowered:
        return "dataset"
    if "user" in lowered and any(city in lowered for city in ["beijing", "shanghai", "shenzhen", "hangzhou"]):
        return "user_profile"
    if "debug logs" in lowered:
        return "debug_logs"
    if "biometric" in lowered:
        return "biometric_samples"
    if "student id" in lowered:
        return "student_records"
    return "general"


def infer_aspect(text: str) -> str:
    lowered = text.lower()
    if "approval" in lowered or "approve" in lowered:
        return "approval"
    if "passport" in lowered or "sensitive" in lowered:
        return "privacy"
    if "send" in lowered or "external" in lowered:
        return "external_transfer"
    if "receipt" in lowered:
        return "receipt_check"
    if any(city in lowered for city in ["beijing", "shanghai", "shenzhen", "hangzhou"]):
        return "location"
    if "share" in lowered or "public dataset" in lowered:
        return "sharing"
    if "email addresses" in lowered:
        return "email_export"
    if "published" in lowered or "biometric samples" in lowered:
        return "publication"
    if "student id" in lowered:
        return "identifier_release"
    return "note"


def infer_risk_tag(event: ProductionEvent) -> str:
    lowered = event.raw_text.lower()
    if event.source == "poisonedrag_injected_text":
        if any(marker in lowered for marker in ["passport", "external", "send private"]):
            return "privacy_leak"
        return "attack"
    if event.source in {"system_policy", "user_policy", "privacy_policy", "policy_memory"}:
        return "policy"
    if event.source == "ethics_review":
        return "safety"
    if any(
        marker in lowered
        for marker in [
            "must not",
            "must remove",
            "must stay private",
            "without releasing raw",
            "aggregate results",
            "only aggregate metrics",
        ]
    ):
        return "safety"
    return "benign"
