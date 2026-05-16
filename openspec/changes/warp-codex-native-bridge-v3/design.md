## Context

WarpCodexOss already has a local Codex bridge that validates the `codex` CLI through `codex login status`, routes several synchronous AI entry points to `codex exec --json`, and labels Agent output as `Local Codex / 0 Warp credits`. The remaining product gap is that Agent generation is still launched inside the server API adapter as a batch operation, so Warp waits for `codex exec` to finish before receiving a stream and user cancellation cannot terminate the running child process.

The codebase also has a separate Warp server-backed AI path. In local Codex mode, silently falling through to that path is a product bug because it can consume Warp credits while the UI implies local execution.

## Goals / Non-Goals

**Goals:**

- Route local Agent generation before Warp server-backed AI calls are made.
- Return a stream immediately for local Codex Agent runs.
- Terminate the Codex child process when the Warp Agent request is cancelled.
- Keep `codex exec --json` as the reliable V3 runner.
- Define an `app-server` runner boundary for a later protocol-native phase.
- Translate local Codex runner events into Warp response events with safe text fallbacks.
- Keep all OAuth handling delegated to the official Codex CLI login state.
- Keep Windows packaging source-ready and explicitly host-validated later.

**Non-Goals:**

- Replacing Warp login, sync, voice transcription, usage pages, model catalog services, or cloud-only features.
- Copying Codex OAuth secrets from any local file.
- Completing full `codex app-server` tool-card and file-diff card parity in this increment.
- Marking Windows as verified without running on a real Windows x64 host.

## Decisions

1. Local Agent routing moves closer to `app/src/ai/agent/api/impl.rs`.
   - Rationale: this layer already owns request construction and cancellation signals.
   - Alternative considered: keep routing in `ServerApi.generate_multi_agent_output`. That preserves a smaller diff but cannot kill a running local child process because the stream is only returned after batch completion.

2. The V3 default runner remains `codex exec --json`.
   - Rationale: it is available today, already proven on this machine, and has simple stdin/stdout semantics.
   - Alternative considered: switch directly to `codex app-server`. The protocol is experimental and needs a larger mapping phase for tool calls, diffs, commands, and cancellation.

3. Runner events are translated through a local intermediate enum.
   - Rationale: Warp response events and Codex protocol events evolve independently. A small adapter layer lets unsupported Codex events degrade into assistant text instead of corrupting conversation state.
   - Alternative considered: construct Warp response events directly in runner code. That couples process management to UI/session semantics and makes app-server support harder.

4. Local mode is fail-closed.
   - Rationale: the product promise is `Local Codex / 0 Warp credits`. If Codex is unavailable, the user should see a local error rather than unknowingly burn Warp credits.
   - Alternative considered: silent upstream fallback. That improves availability but breaks the cost boundary.

5. Windows support remains script and source complete until a Windows x64 host verifies it.
   - Rationale: macOS can prepare PowerShell/Inno scripts but cannot prove PATH lookup, installer behavior, process path, or UI rendering on Windows.

## Risks / Trade-offs

- Batch `codex exec` does not provide token-level streaming -> the stream starts immediately but assistant text may still arrive as one final chunk. Mitigation: expose cancellation now and keep `app-server` as the protocol-native next step.
- Killing a child process can leave partial stdout/stderr -> translator emits a cancelled finish event and avoids treating partial data as a completed answer.
- Warp event shapes may require task bootstrap actions for new conversations -> the local translator creates or reuses a task before adding assistant output.
- Some old local sessions may contain bad persisted state from previous builds -> V3 avoids writing new bad state, while cleanup remains an operator task.

## Migration Plan

1. Create OpenSpec artifacts for V3 requirements.
2. Move local Agent interception to the cancellable Agent API layer.
3. Add a cancellable Codex exec runner and event translator tests.
4. Keep existing server API local guards as defense in depth for non-Agent callers.
5. Run focused Rust tests and format checks.
6. Rebuild/reopen the Mac app after tests pass.
7. Defer Windows completion until a real Windows x64 host runs the documented packaging and UI checks.

## Open Questions

- Which exact `codex app-server` protocol version should be pinned once tool-card/diff-card parity becomes the priority?
- Should old invalid local conversations be automatically quarantined, or should cleanup stay manual to avoid deleting user-visible history?
