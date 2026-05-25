from safe_agent_memory.belief_db import BeliefDatabase, RelationLabel
from safe_agent_memory.cases import attacker_case, conflict_case


def test_attacker_projection_suppresses_poisoned_memory() -> None:
    case = attacker_case()
    db = BeliefDatabase(case.rows)
    relations = db.build_relations()
    projection = db.project(case.task_subjects)

    relation_labels = {relation.label for relation in relations}
    suppressed = {row.memory_id for row in projection.suppressed}
    accepted = {row.memory_id for row in projection.accepted}

    assert RelationLabel.CONFLICT in relation_labels
    assert {"A3", "A4"} <= suppressed
    assert {"A1", "A2", "A5"} <= accepted


def test_conflict_projection_handles_stale_memory() -> None:
    case = conflict_case()
    db = BeliefDatabase(case.rows)
    relations = db.build_relations()
    projection = db.project(case.task_subjects)

    relation_labels = {relation.label for relation in relations}
    suppressed = {row.memory_id for row in projection.suppressed}
    accepted = {row.memory_id for row in projection.accepted}

    assert RelationLabel.SUPERSEDES in relation_labels
    assert "C1" in suppressed
    assert "C2" in accepted
    assert "C3" in suppressed
    assert {"C4", "C5"} <= accepted


def test_projection_keeps_base_rows() -> None:
    case = attacker_case()
    db = BeliefDatabase(case.rows)
    projection = db.project(case.task_subjects)

    assert len(db.rows()) == len(case.rows)
    assert len(projection.accepted) < len(db.rows())
