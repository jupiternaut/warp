## Why

V5 proves that `codex app-server` can stream into Warp, but command/tool/file events still render as plain transcript text. V6 improves native feel by letting already-executed Codex events appear as Warp action cards while preserving the no-double-execution boundary.

## What Changes

- Introduce display-only local Codex action IDs for already-executed app-server events.
- Render command execution start events as native Warp command cards when the app-server payload includes the command.
- Keep command output, file diffs, MCP progress, and unsupported events readable as transcript text.
- Prevent display-only local Codex cards from entering Warp's action execution queue.
- Keep `codex exec` as the default runner and keep app-server opt-in through `WARP_LOCAL_CODEX_RUNNER=app-server`.

## Capabilities

### New Capabilities

- `local-codex-display-only-cards`: Local Codex app-server events can create native-looking Warp cards that are explicitly display-only and cannot be executed by Warp.

### Modified Capabilities

- `local-codex-agent-bridge`: App-server mode may emit display-only native action cards for already-executed Codex work while retaining fail-closed local routing.

## Impact

- Affected Rust modules:
  - `app/src/ai/local_codex.rs`
  - `app/src/ai/agent/api/convert_from.rs`
  - `app/src/ai/blocklist/controller.rs`
  - `app/src/ai/blocklist/controller/shared_session.rs`
- Affected OpenSpec artifacts:
  - `openspec/changes/warp-codex-native-display-cards-v6/`
- Runtime dependencies:
  - No new dependency. V6 still uses the local `codex` CLI and ChatGPT OAuth login state.
- Product boundary:
  - V6 does not let Warp execute Codex app-server tool requests.
  - V6 does not apply file diffs through Warp.
  - V6 does not make app-server the default runner.
