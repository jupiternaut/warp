## Why

V6 renders Codex app-server command executions as native-looking Warp cards without letting Warp execute them again. File changes still appear as plain transcript text, which makes the Agent panel feel split between a native session UI and a raw Codex log.

V7 extends the same display-only boundary to Codex file change events: show a native diff-style card when app-server reports file patch metadata, but keep Codex as the only execution owner.

## What Changes

- Parse `item/fileChange/patchUpdated` app-server payloads into display-only file diff cards when file paths and diff content are available.
- Emit a local `ApplyFileDiffs` tool call with the existing `local-codex-display-` prefix so Warp can render the existing diff UI while marking the action as `requires_result = false`.
- Keep malformed or incomplete file change payloads as transcript text.
- Keep `item/fileChange/outputDelta` as transcript text because it is streaming output, not structured diff metadata.
- Reuse the V6 queue guard so display-only diff cards are never queued for Warp-side application.

## Capabilities

### New Capabilities

- `local-codex-display-only-diff-cards`: Local Codex app-server file patch events can create native-looking Warp diff cards that are explicitly display-only and cannot be applied by Warp.

### Modified Capabilities

- `local-codex-display-only-cards`: Display-only native cards now cover both command starts and structured file patch updates.

## Impact

- Affected Rust modules:
  - `app/src/ai/local_codex.rs`
- Affected OpenSpec artifacts:
  - `openspec/changes/warp-codex-display-only-diff-cards-v7/`
- Runtime dependencies:
  - No new dependency. V7 still uses the local `codex` CLI and ChatGPT OAuth login state.
- Product boundary:
  - V7 does not let Warp apply Codex file changes.
  - V7 does not execute app-server tool requests in Warp.
  - V7 does not make app-server the default runner.
