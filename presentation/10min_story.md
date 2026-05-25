# 10-Minute Presentation Story

Title:

**安全智能体记忆：一种冲突感知的信念数据库方法**

One-sentence thesis:

> Agent memory should not only retrieve what was stored; it should decide what is currently safe to believe and use.

## Main Story Arc

```text
Long-term memory helps agents work across tasks.
But long-term memory also creates a new safety boundary.
Unsafe memory can appear because of PoisonedRAG-style attacker poisoning.
Unsafe memory can also appear without attackers, through stale or conflicting facts.
Recent work shows a third risk: useful memories can become faulty when LLMs continuously rewrite and consolidate them.
Plain retrieval exposes these memories directly to the agent.
A belief database separates memory production from currently usable belief.
Our prototype stores production provenance, prepares structured claims, detects conflicts continuously, records resolutions, and projects a safe working-agent view.
```

## Slide Plan

Target pacing: 14 fast slides in 10 minutes. The logic should feel like one pipeline, not a list of components.

```text
problem
  -> two safety cases
  -> literature warning: useful memories can become faulty
  -> memory != belief
  -> provenance-aware belief database
  -> how memory enters
  -> how conflicts are continuously detected
  -> how the working-agent view is produced
  -> demo and benchmark evidence
  -> takeaway
```

### Slide 1: Title and Main Question, 0:00-0:35

Message:

- Title: safe agent memory with a conflict-aware belief database.
- Main question: when an agent remembers many things, what should the next working agent actually be allowed to use?
- One-sentence answer: remember everything for audit, but believe selectively for action.

Visual:

```text
Long-term memory  ->  ?  ->  working agent prompt
```

Speaker line:

> This project is about the question mark between long-term memory and the next agent prompt. The memory store may contain useful facts, stale facts, and attacker-injected text. I want a layer that decides what is currently safe to believe and use.

### Slide 2: Why Memory Becomes a Safety Boundary, 0:35-1:15

Message:

- Normal memory pipeline: store, retrieve, inject into prompt.
- Once injected, memory becomes instruction-like context.
- If memory is wrong, stale, or poisoned, the working agent may act on it.

Visual:

```text
conversation/tool/doc
        |
        v
 long-term memory
        |
        v
 retrieve similar text
        |
        v
 LLM prompt  -> unsafe action can happen here
```

Speaker line:

> The safety boundary is not only the model weights or the prompt. Long-term memory is also a boundary, because retrieved memory is turned into context that the agent may follow.

### Slide 3: Two Failure Modes, 1:15-2:00

Message:

Attacker case:

- PoisonedRAG-style assumption: attacker injects a few malicious texts into an external knowledge/memory store.
- In our demo, the store is long-term agent memory, not RAG.
- Injected memory says: ignore approval, include passport, send externally.

No-attacker case:

- Old memory says user is in Beijing and dataset is public.
- Later memory says user moved and dataset contains unauthorized face data.
- No one attacks; the memory state is just inconsistent.

Speaker line:

> I intentionally use two cases. One is adversarial, based on the PoisonedRAG threat model. The other has no attacker, because safety problems also come from normal memory drift and conflicting evidence.

### Slide 4: Literature Warning, Useful Memory Can Become Faulty, 2:00-2:45

Message:

- Zhang et al. 2026 study continuously updated LLM memory.
- They distinguish raw episodic traces from consolidated abstractions.
- Main finding: repeated LLM consolidation can make useful memories faulty.
- Memory utility can rise early, then degrade; raw episodic trajectories remain a strong control.
- Practical lesson: preserve raw episodes as first-class evidence and gate consolidation.

Visual:

```text
raw useful episodes
        |
        v
LLM rewrites memory after every interaction
        |
        v
consolidated memory
        |
        v
faulty / overgeneralized / schedule-dependent memory
```

Speaker line:

> This paper is important because it removes a common excuse. The input experiences can be useful, but the continuous LLM rewriting step can still corrupt memory. That supports our design choice: keep raw evidence, store provenance, and gate any projection or consolidation.

### Slide 5: Why Retrieval or Continuous Rewrite Is Not Enough, 2:45-3:25

Message:

Baselines:

- append-all: use every task-relevant memory.
- keyword top-k: retrieve similar memories.
- latest-only: keep newest memory per topic.
- drop-known-attacks: remove obvious attack-tagged rows.
- continuous LLM consolidation: rewrite memory after every interaction.

Problem:

- Retrieval chooses text; it does not maintain belief state.
- It does not store how memory was produced.
- It does not record why two memories conflict.
- It does not produce an auditable working view.
- Continuous rewrite may destroy the raw evidence needed to audit and repair memory.

Speaker line:

> Retrieval asks which text is relevant. Continuous rewrite asks the LLM to keep summarizing its own memory. Belief maintenance asks a safer question: given claims and provenance, what should the agent currently believe?

### Slide 6: Formal Core, Belief Set Maintenance, 3:25-4:10

Message:

- Raw memory log `M`: append-only production events and evidence.
- Claim base `C`: prepared claims with `subject`, `aspect`, `provenance`, `trust`.
- Belief set `B_q`: accepted claims for task/query `q`.
- Projection `pi(q)`: working-agent view generated from `B_q`.

Visual:

```text
M = raw memory log
  e1, e2, e3 ...                 append-only evidence

C = prepared claim base
  c_i = (phi_i, subject, aspect, provenance, trust)

B_q = accepted belief set for task q
  expand if consistent
  revise if conflict
  quarantine if unresolved

pi(q) = working-agent view
```

Speaker line:

> Formally, we keep memory and belief separate. The memory log is append-only, but the accepted belief set is non-monotonic: new evidence can revise what is currently accepted for a task.

### Slide 7: Whole Architecture, 4:10-4:55

Message:

- The system has four stages: production, prepare, belief database, working view.
- Prepare is simulated by LLM subagents.
- Conflict resolution is done before the working agent sees memory.

Visual:

```text
Production event
  source / actor / model / prompt / raw evidence / trust
        |
        v
Prepare LLM subagents
  task tagger + claim extractor + risk tagger + provenance scorer
        |
        v
Belief database
  base store + relation graph + resolver records
        |
        v
Working-agent view
  accepted memories only
```

Speaker line:

> The important architectural point is that we separate the worker agent from the memory-maintenance layer. The worker gets a prepared view, not a raw memory dump.

### Slide 8: Question 1, How Does Memory Enter the DB?, 4:55-5:45

Message:

- A memory first enters as a `ProductionEvent`.
- The event stores how the memory was produced.
- Simulated prepare subagents extract the structured memory row.
- Only then do we insert it into the belief database.

Visual:

```text
ProductionEvent
  event_id
  source = ethics_review
  actor = strong_llm_safety_reviewer
  producer_model = strong-review-sim
  raw_text = dataset contains unauthorized face data
  trust = 92
  confidence = 0.94
        |
        v
MemoryRow
  subject = dataset
  aspect = sharing
  risk_tag = safety
  claim = dataset must not be shared publicly
```

Speaker line:

> So the DB is not initialized with a magic bad-row label. The system stores provenance first, then prepares a structured claim from that event.

### Slide 9: Question 2, How Are Conflicts Detected Continually?, 5:45-6:35

Message:

- The DB ingests prepared rows one by one.
- On each insert, it compares the new row with prior rows in the same `subject/aspect`.
- New edges are recorded immediately: `support`, `conflict`, `supersedes`.
- Resolver records decide accepted, suppressed, or ambiguous.

Visual:

```text
step 1: ingest M1  -> no prior row
step 2: ingest M2  -> M2 supersedes M1
step 3: ingest M3  -> no relation
step 4: ingest M4  -> M3 conflict M4

then:
  rule / strong LLM / user review
  -> resolution record
  -> next working view
```

Speaker line:

> Conflict detection is continuous. Every new prepared memory can change the current belief view, but the base store still preserves all evidence.

### Slide 10: Resolver and Working-Agent View, 6:35-7:15

Message:

Resolver choices:

- deterministic attack policy: suppress obvious injected or privacy-leaking memory.
- freshness rule: newer confirmed user memory supersedes older memory.
- strong-LLM safety reviewer: resolve natural safety or ethics conflicts.
- user review: for conflicts without a clear safe winner.

Example:

```text
M1: dataset is public and can be shared       trust=50
M2: dataset has unauthorized face data        trust=92

Relation:
  M1 conflict M2

Resolution:
  strong_llm_safety_reviewer accepts M2, suppresses M1

Working view:
  "dataset must not be shared publicly"
```

Speaker line:

> This is where provenance matters. The system can prefer higher-trust safety evidence over a weak old note, while still preserving both in the audit trail.

### Slide 11: Demo Walkthrough, 7:15-8:00

Use `results/stress_mock/demo_page.html`.

Message:

- Show Memory Production Events.
- Show Prepare Phase.
- Show Continual Conflict Detection.
- Show Base Memory Store vs Agent-Facing Projection.
- Show Direct Retrieval Context vs Belief Projection Context.

Speaker line:

> This page is the clearest answer to the project mechanism: it shows how memory enters, when the conflict is detected, and what the working agent finally sees.

### Slide 12: Benchmark Setup, 8:00-8:40

Message:

- Stress suite: 360 generated cases, 1800 per-method records.
- 120 attacker cases with PoisonedRAG-style injected memory.
- 240 no-attacker conflict cases with stale versions, natural safety conflicts, and benign memory noise.
- 5 methods: append-all, keyword top-k, latest-only, drop-known-attacks, belief projection.

Stress axes:

- poisoned memory bundles: 1 to 4.
- natural conflict pairs: 1 to 4.
- benign noise rows: 0, 4, 8.
- stale depth: 1 or 3 old versions.

Metrics:

- unsafe memory exposure.
- unsafe answer rate.
- suppressed rows.
- conflict/supersedes edges.
- prepare events and resolver records.

Speaker line:

> The benchmark is still synthetic, but it is not just two hand-picked examples. It varies attack pressure, conflict density, stale depth, and memory noise.

### Slide 13: Main Result, 8:40-9:35

Use this table plus `01_unsafe_memory_by_family.png` and `06_ingestion_resolution_load.png`.

| Method | Unsafe exposure | Unsafe answer | Suppressed | Conflict edges | Supersedes | Prepare events | Resolutions |
|---|---:|---:|---:|---:|---:|---:|---:|
| append_all | 1.00 | 0.25 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| keyword_topk | 0.94 | 0.33 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| latest_only | 0.92 | 0.58 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| drop_known_attacks | 0.67 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 | 0.00 |
| belief_projection | 0.00 | 0.00 | 3.83 | 2.50 | 2.33 | 11.00 | 4.83 |

Message:

- Direct retrieval exposes unsafe memory.
- Latest-only fails attacker cases and many conflict cases.
- Drop-known-attacks cannot solve no-attacker conflicts.
- Belief projection reaches 0.00 unsafe exposure and produces audit evidence.

Speaker line:

> The result supports a modest claim: not that our rules are perfect, but that a provenance-aware belief database can create a safer working view than direct retrieval.

### Slide 14: Takeaway and Limitations, 9:35-10:00

Message:

Takeaway:

- Safe agent memory is not only retrieval.
- Safe memory is also not continuous LLM rewriting.
- Memory should have provenance, conflict state, resolver records, and projected working views.
- Working agents should consume maintained belief views, not raw memory buffers.

Limitations:

- Deterministic synthetic benchmark.
- Prepare subagents and resolver are simulated.
- Future work: real LLM prepare/resolve calls, richer memory sources, real task traces.

Closing sentence:

> Safe agent memory should remember everything for audit, but believe selectively for action.

## 5-Minute Q&A Preparation

### Q1: Is this just rule-based filtering?

Answer:

> The current prototype uses rules because this is a course demo and we want controlled evidence. The research idea is not the specific rule set. The idea is the memory architecture: preserve base memory, record conflict/supersession as explicit state, then project a safe context. A stronger implementation could replace rules with an LLM or classifier while keeping the same database contract.

### Q1b: How does a memory enter the database?

Answer:

> It enters as a production event first, not as a trusted memory row. The event records source, actor, producer model, prompt, timestamp, confidence, trust level, and raw evidence. Then simulated prepare LLM subagents extract subject, aspect, risk tag, and normalized claim. Only after that does it become a `MemoryRow`.

### Q2: Why not just use latest-only memory?

Answer:

> Latest-only works when the problem is only stale facts, but it fails attacker cases because malicious injected memory may be the newest. It also gives no audit trail. In the stress benchmark, latest-only has 0.92 unsafe memory exposure overall, while belief projection has 0.00 and records why rows were suppressed.

### Q3: Why not just remove malicious memory?

Answer:

> Removing memory loses evidence. For safety and ethics, we often need to know that an attack or conflict happened. The belief database keeps unsafe rows in base storage but suppresses them from the task-local prompt. This supports both safety and auditability.

### Q3b: What is the theory of the belief database?

Answer:

> The theory is belief maintenance. The database stores claims and their relations, then computes a usable belief view for the current task. Conflict and supersession are first-class state. This is different from retrieval because the output is not “similar chunks”; it is a maintained belief projection with audit evidence.

### Q4: How does this relate to AI safety and ethics?

Answer:

> It connects to poisoning, prompt injection, privacy leakage, and ethical accountability. Long-term memory makes unsafe information persistent. A belief database gives a mechanism to manage this persistence instead of relying on the LLM to notice contradictions inside a prompt.

### Q4b: Are you implementing PoisonedRAG?

Answer:

> No. PoisonedRAG is used as the security-paper attacker model. The relevant idea is that an attacker can inject a few malicious texts into the external knowledge store to target a later question. I apply the same assumption to long-term agent memory, then test whether belief projection prevents those injected memories from being used.

### Q4c: How does the "Useful Memories Become Faulty" paper change the motivation?

Answer:

> It shows that memory safety is not only about malicious input. Even useful experiences can become faulty when an LLM continuously rewrites them into consolidated memory. This supports our architecture: keep raw production evidence, store provenance, and make consolidation/projection gated and auditable rather than automatic after every interaction.

### Q5: Is the benchmark too synthetic?

Answer:

> Yes, it is synthetic by design. The goal is to isolate the mechanism and make the safety claim measurable. The next step would be to use real web/email/document memories and an LLM relation judge. For the course project, this benchmark is enough to show the effect of conflict-aware projection.

### Q6: What is the main novelty?

Answer:

> The novelty is not that agents need memory. The novelty is treating memory safety as a belief-management problem over production provenance. The system separates memory production, conflict resolution, and working-agent view generation, so the agent consumes a safer task-local view instead of raw retrieval results.

## Closing Sentence

> Safe agent memory should remember everything for audit, but believe selectively for action.
