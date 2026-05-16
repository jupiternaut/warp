## Context

Warp's native file edit UI is driven by `ApplyFileDiffs` tool calls converted into `RequestFileEdits` actions. V6 made display-only command cards safe by using a local tool call ID prefix and marking those actions as `requires_result = false`, then filtering non-result actions out of execution queues.

The same mechanism can render Codex file patch metadata: app-server owns the actual edit, while Warp receives a synthetic tool call only so the existing diff card UI has something to display.

## Goals / Non-Goals

**Goals:**

- Render structured `item/fileChange/patchUpdated` events as display-only file diff cards.
- Preserve the no-double-application boundary by reusing `local-codex-display-` IDs and `requires_result = false`.
- Fall back to transcript text when patch metadata is missing or malformed.
- Cover the behavior with translator and fake app-server tests.

**Non-Goals:**

- Do not call Warp's file edit executor for Codex-generated diffs.
- Do not claim the diff card means Warp applied the patch.
- Do not parse every possible unified diff shape into perfect semantic hunks in this version.
- Do not change the default runner from `codex exec` to `codex app-server`.

## Decisions

1. Reuse `ApplyFileDiffs` for display.
   - Rationale: This is the narrowest path into Warp's existing diff UI.
   - Alternative considered: Build a separate custom Codex diff component. That would duplicate UI and expand the fork.

2. Represent Codex unified diff text as a lossy V4A hunk body.
   - Rationale: app-server patch metadata currently provides path plus diff text; V4A lets us associate that text with a file path without reading or rewriting files.
   - Trade-off: The card is a display card, not a perfect re-application patch.

3. Keep output deltas as transcript.
   - Rationale: `outputDelta` is streaming text and may contain partial hunks. Creating cards from partial text would be noisy and unstable.

## Risks / Trade-offs

- [Risk] The diff card may not be as semantically rich as a Warp-owned diff -> Mitigation: the full diff text remains visible in the card body, and malformed data falls back to transcript text.
- [Risk] A user could try to accept the card -> Mitigation: display-only cards are marked `requires_result = false` and excluded from action queueing; V7 does not register them as work Warp should perform.
- [Risk] Codex app-server payloads may change -> Mitigation: parse loosely and fall back to transcript text.

## Migration Plan

1. Add V7 OpenSpec artifacts.
2. Add a display-only file diff event shape and parser for patch updates.
3. Translate file diff events into display-only `ApplyFileDiffs` tool call messages.
4. Add translator and fake app-server coverage.
5. Run OpenSpec validation, formatting, local Codex tests, Codex harness tests, compile check, and whitespace check.
