## Context

V5 maps app-server command, diff, and tool notifications to plain assistant transcript text to avoid double execution. Warp's native action cards are normally backed by executable `ToolCall` messages, and the controller queues those actions after a response stream finishes. Creating ordinary command or diff tool calls for Codex-executed work would therefore let Warp execute work that Codex already ran.

V6 keeps Codex as the only execution owner and introduces display-only local Codex action cards. These cards reuse Warp's existing output rendering path but are marked so they do not enter the action execution queue.

## Goals / Non-Goals

**Goals:**

- Render Codex `commandExecution` item start events as native command cards when the command text is available.
- Keep command output, diffs, MCP progress, and unsupported events visible as transcript text.
- Prevent display-only Codex actions from entering Warp action queues in both normal and shared-session stream handling.
- Cover the behavior with fake app-server and conversion tests.

**Non-Goals:**

- Do not execute app-server tool requests in Warp.
- Do not apply Codex file diffs through Warp.
- Do not synthesize full finished status for every native card in this version.
- Do not make `WARP_LOCAL_CODEX_RUNNER=app-server` the default.

## Decisions

1. Use a `local-codex-display-` tool call ID prefix.
   - Rationale: Warp action conversion has access to `tool_call_id`, and the prefix is easy to test without changing the protobuf schema.
   - Alternative considered: Add a new protobuf field. That would be a larger schema change for a private fork and would not help upstream compatibility.

2. Mark display-only local Codex actions with `requires_result = false`.
   - Rationale: These cards represent work Codex already executed, so Warp must not require or generate a result for them.
   - Alternative considered: Emit normal `ToolCall` and a matching `ToolCallResult`. Normal client stream handling still queues output actions and can trigger follow-up logic, so this is not safe enough without broader controller changes.

3. Filter non-result actions before queueing.
   - Rationale: Rendering can still use the output action message, but action execution should only queue actions that require a result.
   - Alternative considered: Filter only by the local Codex prefix. Filtering by `requires_result` matches the field's semantics and is less brittle.

4. Keep transcript fallback for result/output content.
   - Rationale: It preserves exact Codex output without depending on Warp action-model status machinery or risking follow-up loops.
   - Alternative considered: Apply synthetic action results to the action model. This can make Warp think a local display event completed an action and trigger another Agent request.

## Risks / Trade-offs

- [Risk] A display-only card may look less complete than a fully executed Warp card -> Mitigation: keep the transcript immediately below it so output and diffs are still visible.
- [Risk] Upstream uses `requires_result = false` for another action type later -> Mitigation: filtering non-result actions from queueing matches the documented meaning of the field.
- [Risk] app-server item payloads may change -> Mitigation: parse loosely from `serde_json::Value` and fall back to transcript text when command metadata is missing.
- [Risk] Diff cards are still transcript-only -> Mitigation: keep file diff native mapping for a later version once we can represent already-applied diffs without action-model side effects.

## Migration Plan

1. Add V6 OpenSpec artifacts.
2. Add display-only event variants and native command card translation in `local_codex.rs`.
3. Mark local display-only tool calls as `requires_result = false` during API conversion.
4. Filter non-result actions out of normal and shared-session action queueing.
5. Add fake app-server and conversion tests.
6. Run OpenSpec validation, formatting, local Codex tests, Codex harness tests, and a targeted compile check.
