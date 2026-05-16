## Why

V7 UI acceptance proved the local Codex request path and `0 Warp credits` UI, but it also exposed two product gaps:

- Local Codex turns can still inherit a server `ambient_agent_task_id`, causing `TaskStatusSyncModel` to call Warp cloud `UpdateAgentTask` for a task the local fork does not own.
- Real app-server file turns did not produce enough log evidence to tell which Codex event methods arrived, so the missing diff card cannot be diagnosed from the UI alone.

V8 tightens the local boundary before adding more UI mapping.

## What Changes

- Force local Codex response streams to use a `local-codex-*` run id instead of reusing request `ambient_agent_task_id`.
- Skip cloud task-status updates for conversations whose server token is a local Codex token.
- Add app-server event method logging with method, item id, and item type only.
- Add explicit local completion logs when app-server sends a `StreamFinished` reason.
- Keep display-only tool calls non-executable.

## Capabilities

### New Capabilities

- `local-codex-ui-completion`: Local Codex Agent turns complete locally without reporting cloud task status, and app-server events are visible enough for UI acceptance debugging.

### Modified Capabilities

- `local-codex-display-only-diff-cards`: V8 adds observability needed to determine whether real app-server turns emit structured patch events.

## Impact

- Affected Rust modules:
  - `app/src/ai/local_codex.rs`
  - `app/src/ai/blocklist/task_status_sync_model.rs`
  - `app/src/ai/blocklist/task_status_sync_model_tests.rs`
- Affected OpenSpec artifacts:
  - `openspec/changes/warp-codex-local-completion-v8/`
- Runtime dependencies:
  - No new dependency.
- Product boundary:
  - V8 still does not let Warp execute or apply Codex-owned work.
  - V8 does not make app-server the default runner outside the current local launch environment.
