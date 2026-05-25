# Rust Reference Code

This directory preserves a small source-level snapshot of the earlier Rust
AgentDB prototype used as background for the course demo.

- `agentdb-re/` is a Rust crate with the original `persist`, `bind`, semantic
  relation, and projection examples.
- The course project's runnable benchmark is still the Python prototype in
  `safe_agent_memory/`.
- The most relevant Rust example for the presentation story is
  `agentdb-re/examples/step4_projection.rs`, which demonstrates conservative
  projection with accepted, suppressed, ambiguous, and consulted-conflict rows.

Useful checks:

```bash
cd rust_reference/agentdb-re
cargo test
cargo run --example step4_projection
```
