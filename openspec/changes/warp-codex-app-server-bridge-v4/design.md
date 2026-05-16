## Context

V3 introduced a cancellable local Codex `exec` runner and a fail-closed local Agent path. The remaining product gap is native protocol fidelity: `codex exec --json` is batch-oriented, while `codex app-server` exposes thread/turn notifications for message deltas, reasoning deltas, diffs, command output, status, and cancellation.

The main V4 risk is double execution. Codex app-server events may describe work that Codex has already performed. If Warp renders those events as native `ToolCall`s, Warp can run the same command or apply the same patch a second time. V4 therefore separates "rendered transcript events" from "Warp-owned executable tool calls".

## Goals / Non-Goals

**Goals:**

- Create a V4 protocol bridge boundary based on app-server schema-generated event names.
- Add native-safe streaming translation for assistant text and reasoning.
- Preserve Codex tool/diff/command events as transcript/fallback text unless Warp explicitly owns execution.
- Keep local mode fail-closed with explicit local/Warp route logging.
- Add product hardening tasks for invalid local conversation quarantine and endpoint route auditing.
- Add Windows host-validation evidence requirements without marking Windows acceptance done on macOS.

**Non-Goals:**

- Full `codex app-server` JSON-RPC client implementation in this increment.
- Full native tool-card and diff-card parity.
- Letting Warp execute Codex tool events that were already executed by Codex.
- Deleting old invalid conversation records automatically.
- Marking Windows installer/UI acceptance complete without a real Windows x64 host.

## Decisions

1. V4 implements a renderable-event translator before the app-server transport.
   - Rationale: translator behavior can be tested with fixture events before we bind to a moving experimental protocol.
   - Alternative considered: wire app-server transport first. That risks adding process/protocol complexity before the double-execution boundary is testable.

2. Text and reasoning use native streaming actions.
   - Rationale: Warp already supports `AddMessagesToTask` followed by `AppendToMessageContent`, which matches app-server delta notifications without side effects.
   - Alternative considered: keep emitting final `AgentOutput` blocks. That preserves V3 behavior but does not advance toward native streaming.

3. Tool/diff/command events default to transcript fallback.
   - Rationale: transcript rendering is safe and inspectable. Native `ToolCall` cards should only be produced when a future allowlist routes execution to Warp before Codex performs the action.
   - Alternative considered: convert all app-server tool events to native cards. That creates double execution risk.

4. Hardening stays in the V4 product scope even if not all hardening code lands in the first implementation slice.
   - Rationale: app-server protocol debugging will be much harder if invalid history and unclear route logs keep polluting evidence.

5. Windows remains evidence-driven.
   - Rationale: macOS can prepare scripts and templates but cannot prove Windows PATH, installer, process path, or UI behavior.

## Risks / Trade-offs

- [Risk] app-server protocol changes because it is experimental -> Mitigation: keep generated schemas out of production code and test through stable local event fixtures.
- [Risk] transcript fallback is less native than tool cards -> Mitigation: ship safe streaming/reasoning now, then open tool-card allowlists in later versions.
- [Risk] invalid local conversation records keep logging warnings -> Mitigation: specify quarantine and summary logging before any destructive cleanup.
- [Risk] route audit becomes stale after upstream merges -> Mitigation: introduce an endpoint classification list and tests around `/ai/` usage.

## Migration Plan

1. Add V4 OpenSpec artifacts.
2. Add safe renderable-event translator tests for text deltas, reasoning deltas, command transcript, diff transcript, and unsupported events.
3. Keep production runner on `exec` unless `app-server` transport is deliberately enabled later.
4. Add Windows validation report expectations.
5. Run `cargo fmt --check`, `cargo test -p warp local_codex --lib`, `cargo test -p warp codex --lib`, and `openspec validate`.

## Open Questions

- Should V5 implement app-server transport over `stdio://` first or websocket loopback first?
- Which read-only Codex tool events can safely become Warp-owned native `ToolCall`s without double execution?
- Should invalid local conversations be hidden by default with a manual restore option, or only summarized in logs?
