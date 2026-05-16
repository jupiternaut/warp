## 1. OpenSpec And Protocol Boundary

- [x] 1.1 Capture multi-agent V4 scope: safe app-server text/reasoning streaming, no Codex tool double execution
- [x] 1.2 Add V4 proposal, design, and specs for safe transcript, streaming renderer, and hardening evidence
- [x] 1.3 Validate `warp-codex-app-server-bridge-v4` after implementation changes

## 2. Safe Streaming Renderer

- [x] 2.1 Add a renderable local Codex event translator that creates native `AgentOutput` once and appends text deltas
- [x] 2.2 Add native `AgentReasoning` append support for reasoning deltas
- [x] 2.3 Render command, diff, tool, and unsupported app-server-shaped events as assistant transcript text
- [x] 2.4 Add tests proving command/diff/tool transcript events do not create executable Warp `ToolCall`s

## 3. Hardening Evidence

- [x] 3.1 Add or update a Windows Codex validation report path that records source-ready vs host-validated status without tokens
- [x] 3.2 Add a lightweight local AI route classification artifact for fail-closed audits
- [x] 3.3 Document invalid local conversation quarantine criteria without deleting or migrating user history

## 4. Verification

- [x] 4.1 Run `cargo fmt --check`
- [x] 4.2 Run `cargo test -p warp local_codex --lib`
- [x] 4.3 Run `cargo test -p warp codex --lib`
