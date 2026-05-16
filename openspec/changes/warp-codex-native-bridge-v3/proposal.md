## Why

WarpCodexOss v2 proves that Warp's visible Agent surface can be routed to the local Codex CLI with ChatGPT OAuth and `0 Warp credits`, but the implementation is still a batch text bridge. The next product iteration needs a clearer native-session bridge so cancellation, routing, cost display, and future tool/diff event mapping behave like a real Warp Agent path instead of a hidden terminal call.

## What Changes

- Introduce a formal local Codex Agent bridge that owns the local request path before Warp server-backed AI is invoked.
- Move local Agent generation to a cancellable stream source so Warp can start a response stream immediately and terminate the Codex child process when the user cancels.
- Add a Codex runner abstraction with an `exec` runner now and an `app-server` runner boundary for native Codex protocol events.
- Add an event translator that maps local Codex text, reasoning, tool, diff, command-result, finish, cancel, and unsupported events into Warp response events or safe assistant-text fallbacks.
- Harden fail-closed routing: when `WARP_LOCAL_CODEX_AI=1`, user-visible generative AI entry points must not silently fall back to Warp `/ai/*` credit-consuming endpoints.
- Preserve OAuth safety: never read, copy, parse, or write `~/.codex/auth.json`; readiness is checked through `codex login status`.
- Keep Windows support source-ready while marking real Windows x64 build/install/UI acceptance as host-dependent.

## Capabilities

### New Capabilities

- `local-codex-agent-bridge`: Warp Agent sessions can run through local Codex CLI with explicit OAuth-safe readiness checks, cancellation, local cost labeling, and structured event translation.
- `local-codex-fail-closed-routing`: Local Codex mode prevents silent calls to Warp AI credit endpoints and exposes clear fallback/error behavior.
- `warpcodexoss-windows-packaging`: Windows x64 packaging scripts can produce a `WarpCodexOss` installer that expects a separately installed and logged-in Codex CLI.

### Modified Capabilities

- None. This repository did not have existing OpenSpec capabilities before this change.

## Impact

- Affected Rust modules:
  - `app/src/ai/local_codex.rs`
  - `app/src/ai/agent/api/impl.rs`
  - `app/src/server/server_api.rs`
  - `app/src/server/server_api/ai.rs`
  - `app/src/ai/llms.rs`
  - `app/src/ai/blocklist/block/view_impl/output.rs`
- Affected packaging/scripts:
  - `script/macos/run`
  - `script/windows/bundle.ps1`
  - `script/windows/windows-installer.iss`
  - `script/windows/check_codex.ps1`
  - `script/windows/README.md`
- Runtime dependencies:
  - `codex` CLI available on `PATH`
  - `codex login status` returning a ChatGPT OAuth login state
- Product constraints:
  - V3 prioritizes reliable local Agent text/cancel/cost/routing semantics.
  - Full native tool cards and diff cards may initially degrade to assistant text until the `codex app-server` event mapping is implemented end to end.
