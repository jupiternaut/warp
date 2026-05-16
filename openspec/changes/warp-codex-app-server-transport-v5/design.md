## Context

Codex app-server exposes a JSON-RPC protocol over stdio. Local probing showed the stdio transport uses one JSON object per line, not LSP `Content-Length` framing. The V5 runner must therefore use line-delimited JSON and avoid importing generated schema files into production.

## Goals / Non-Goals

**Goals:**

- Make `WARP_LOCAL_CODEX_RUNNER=app-server` start a real local Codex app-server child.
- Stream safe notification deltas into Warp while the turn runs.
- Preserve the V4 no-double-execution guard.
- Support cancellation with `turn/interrupt` plus process kill fallback.
- Cover the runner with fake app-server tests.

**Non-Goals:**

- Full Codex app-server protocol coverage.
- Native Warp tool execution cards for Codex-executed work.
- Native diff application cards.
- Replacing command search/dialogue/metadata/code review short tasks with app-server.
- Making app-server the default runner.

## Decisions

1. Use newline JSON-RPC over stdio.
   - Probe evidence: Content-Length framing caused app-server JSON deserialize errors; newline JSON returned `initialize` successfully.

2. Keep app-server opt-in.
   - `exec` remains the default because it is simpler and already validated in the UI.
   - App-server is selected only by `WARP_LOCAL_CODEX_RUNNER=app-server`.

3. Map only safe notifications natively.
   - Text and reasoning append to native messages.
   - Command/file/tool/process notifications are transcript text only.
   - Unknown operational notifications such as MCP startup or thread status are ignored unless they represent an error.

4. On server-originated requests, respond with a JSON-RPC error instead of executing work in Warp.
   - This preserves the V4 ownership boundary.
   - Future V6 can add an allowlist for Warp-owned already-reviewed actions.

## Risks / Trade-offs

- [Risk] app-server protocol is experimental -> Mitigation: parse loosely with `serde_json::Value` and fake protocol fixtures.
- [Risk] app-server startup can emit noisy plugin/status events -> Mitigation: ignore operational notifications and only render output events.
- [Risk] app-server may request client-side tool execution -> Mitigation: reject server requests in V5 instead of silently executing.
- [Risk] cancellation can race with turn id discovery -> Mitigation: kill the child when no turn id is available.

## Migration Plan

1. Add V5 OpenSpec artifacts.
2. Add app-server JSON line writer/reader helpers.
3. Add app-server handshake and turn loop.
4. Connect normal Agent stream to app-server when the env var selects it.
5. Add fake app-server tests for streaming, transcript fallback, and cancellation.
6. Run formatting, local Codex tests, Codex harness tests, OpenSpec validation, and a bundle/install pass.
