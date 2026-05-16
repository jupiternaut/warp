## Context

Warp treats `run_id` as a server task identifier when it parses as an ambient task UUID. In local Codex mode this is wrong: the execution is local, the Warp cloud task may not exist, and failed `UpdateAgentTask` calls can leave confusing UI state and logs.

The current local stream already generates a local conversation token when one is missing, but it still prefers request metadata `ambient_agent_task_id` for `run_id`. V8 removes that leak.

## Goals / Non-Goals

**Goals:**

- Ensure local Codex streams emit local run ids that cannot be parsed as cloud ambient task ids.
- Ensure local Codex conversations do not call Warp cloud task-status sync.
- Log app-server notification method names so missing native cards can be diagnosed from real UI runs.
- Log local app-server completion/cancellation/error reason.

**Non-Goals:**

- Do not parse or log full app-server payloads.
- Do not change model selection or credits UI.
- Do not add a broad file watcher fallback yet.
- Do not remove existing server task sync for upstream Warp AI.

## Decisions

1. Use a `local-codex-run-*` run id.
   - Rationale: it remains visibly local and cannot parse as an ambient task UUID.

2. Skip task sync by local server conversation token.
   - Rationale: local Codex stream init assigns local conversation tokens; this is the least invasive ownership boundary.

3. Log method metadata only.
   - Rationale: method, item id, and item type are enough for event-shape debugging without leaking prompt/file content.

## Risks / Trade-offs

- [Risk] Existing local conversations with old UUID run ids may still have stale persisted task ids -> Mitigation: new turns stop creating those ids, and local token guard prevents status sync once the local token is known.
- [Risk] App-server method logs are noisy -> Mitigation: log only method/item metadata, no payload.

## Migration Plan

1. Add V8 OpenSpec artifacts.
2. Add local stream id helper and tests.
3. Change local stream init to use local run ids.
4. Add local conversation task-sync skip guard and tests.
5. Add app-server method and completion logs.
6. Run focused tests, formatting, compile check, and UI smoke.
