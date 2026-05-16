# V7 UI Acceptance Report

Checked on 2026-05-11 using `/Applications/WarpCodexOss.app` with:

- `WARP_LOCAL_CODEX_AI=1`
- `WARP_LOCAL_CODEX_RUNNER=app-server`
- `WARP_LOCAL_CODEX_TIMEOUT_MS=180000`

## Passed

- App identity: running process path was `/Applications/WarpCodexOss.app/Contents/MacOS/warp-oss`.
- Model UI: prompt chip showed `Local Codex (ChatGPT OAuth) (local codex app-server)`.
- Credits UI: completed responses showed `Local Codex / 0 Warp credits`.
- Request routing: logs showed `generate_multi_agent_output: routing to local Codex; using 0 Warp AI credits`.
- App-server runner: logs showed `Local Codex AI: running codex app-server --listen stdio://; cwd=/Users/gengrf`.
- Basic response: a fixed prompt returned in the Agent panel.
- Command card: a file-write prompt produced a display-only shell command card in the Agent panel.
- File side effect: `/tmp/warpcodex-v7-ui-acceptance.txt` contained `warp-local-codex-v7-diff-ok`.
- Patch prompt side effect: `/Users/gengrf/warp/.codex/v7-ui-diff-card.txt` contained `warp-local-codex-v7-diff-card-ok`.

## Failed Or Partial

- Diff card not visually accepted: the patch-oriented prompt created the file and returned `DONE`, but the Agent panel did not show a native diff card. The visible UI showed assistant text plus `Local Codex / 0 Warp credits`.
- Completion state partial: one file-write turn wrote the file and the Codex child exited, but the Agent panel kept showing `Warping...` until manually moving to a new conversation.
- Task status sync errors appeared in logs:
  - `TaskStatusSyncModel: failed to update task ... to InProgress`
  - `failed to load AI task owner: Not found: no rows in result set`

## Interpretation

V7's synthetic display-only diff card translator is covered by unit and fake app-server tests, but real UI acceptance shows the current live Codex app-server path did not expose a structured `item/fileChange/patchUpdated` event for these file-write prompts, or Warp did not surface it in the visible Agent panel. The command-card path is working in real UI; the diff-card path still needs a real-event fallback or deeper app-server event instrumentation.

## V8 Follow-Up

- Add temporary dev logging for every app-server `method` seen during a turn.
- Add a UI-visible or log-visible completion reason when local app-server finishes.
- Fix local conversation task status so `UpdateAgentTask` failures cannot leave the panel stuck in `Warping...`.
- Add a fallback display-only file-change card when Codex changes files through shell/apply_patch but does not emit `item/fileChange/patchUpdated`.
- Keep all local display cards `requires_result = false` so Warp still never repeats Codex-owned work.
