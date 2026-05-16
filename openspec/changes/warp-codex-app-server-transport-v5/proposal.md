## Why

V4 added the safe renderer but still leaves `WARP_LOCAL_CODEX_RUNNER=app-server` as a future boundary. V5 makes that runner real for the safe subset: local Agent turns can start `codex app-server --listen stdio://`, send thread/turn requests, stream text and reasoning into Warp, and fail closed without using Warp AI credits.

## What Changes

- Add a newline JSON-RPC stdio client for `codex app-server`.
- Implement `initialize -> thread/start -> turn/start` for normal Agent prompts.
- Map side-effect-free app-server notifications into the V4 safe renderer:
  - `item/agentMessage/delta` -> `AgentOutput` append
  - `item/reasoning/textDelta` and summary deltas -> `AgentReasoning` append
  - command/file/tool/process notifications -> transcript text only
- Handle cancellation by sending `turn/interrupt` when possible and killing the child process as a fallback.
- Keep passive suggestions and structured JSON short tasks on the existing `codex exec --json` path.
- Preserve fail-closed behavior: app-server failure returns a local Codex error and does not call Warp `/ai/*`.

## Capabilities

### New Capabilities

- `codex-app-server-stdio-runner`: WarpCodexOss can run a safe local Agent turn through Codex app-server stdio.
- `codex-app-server-cancellation`: Local Agent cancellation interrupts the Codex app-server turn and terminates the child when needed.

### Modified Capabilities

- `local-codex-agent-bridge`: Adds an opt-in app-server runner while keeping `exec` as the default reliable runner.

## Impact

- Affected Rust modules:
  - `app/src/ai/local_codex.rs`
- Affected OpenSpec artifacts:
  - `openspec/changes/warp-codex-app-server-transport-v5/`
- Runtime dependencies:
  - `codex app-server --listen stdio://`
  - ChatGPT OAuth validated by `codex login status`
- Product boundary:
  - V5 does not enable native executable tool cards.
  - V5 does not make app-server the default runner until manual testing proves it is stable enough.
