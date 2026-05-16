## Why

V3 made local Codex Agent runs cancellable and visibly zero Warp credits, but it still renders Codex output mostly as batch text. V4 moves the product toward a real `codex app-server` protocol bridge while preserving the key safety boundary: do not make Warp re-execute shell/file/tool work that Codex has already executed.

## What Changes

- Add a protocol-level V4 boundary for `codex app-server` events, based on generated app-server schema inspection.
- Introduce a safe renderable-event translator that can stream assistant text, stream reasoning, and preserve tool/diff/command events as transcript text unless Warp explicitly owns execution.
- Add native Warp event support for safe output-only events:
  - `AgentOutput` with `AppendToMessageContent`
  - `AgentReasoning` with `AppendToMessageContent`
  - model/local-cost markers
  - prompt suggestions where already supported
- Keep native `ToolCall` cards behind a future allowlist so Codex-executed work does not run twice inside Warp.
- Add product hardening requirements for old invalid local conversations, endpoint route audits, and Windows validation evidence.
- Preserve V3 fail-closed behavior: local mode failures do not silently fall back to Warp AI credits.

## Capabilities

### New Capabilities

- `codex-app-server-safe-transcript`: The bridge can ingest app-server-shaped events and render safe output/reasoning/transcript events without double execution.
- `local-codex-streaming-renderer`: Local Codex output can use Warp streaming message actions instead of one final text block where safe.
- `local-codex-hardening-evidence`: The fork tracks invalid-history isolation, endpoint route classification, and Windows host-validation evidence as explicit product requirements.

### Modified Capabilities

- `local-codex-agent-bridge`: Extend the V3 local Agent bridge with app-server protocol boundaries and native-safe streaming translation.

## Impact

- Affected Rust modules:
  - `app/src/ai/local_codex.rs`
  - `app/src/ai/agent/api/impl.rs`
  - `app/src/ai/blocklist/history_model/conversation_loader.rs`
  - `app/src/ai/agent/conversation.rs`
  - `app/src/server/server_api.rs`
  - `app/src/server/server_api/ai.rs`
- Affected scripts/docs:
  - `script/windows/check_codex.ps1`
  - `script/windows/README.md`
  - V4 OpenSpec artifacts under `openspec/changes/warp-codex-app-server-bridge-v4/`
- Runtime dependencies:
  - `codex` CLI with `app-server generate-json-schema` support
  - ChatGPT OAuth login validated only by `codex login status`
- Product boundaries:
  - V4 does not claim full native tool/diff card parity.
  - V4 explicitly prevents double execution by rendering Codex-executed tool events as transcript/fallback until a Warp-owned execution allowlist is implemented.
