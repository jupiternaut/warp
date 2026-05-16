## 1. OpenSpec

- [x] 1.1 Create V8 proposal, design, specs, and tasks
- [x] 1.2 Validate V8 OpenSpec artifacts

## 2. Local Completion Boundary

- [x] 2.1 Add local stream id helper that ignores request ambient task ids
- [x] 2.2 Use local run ids in both local multi-agent response paths
- [x] 2.3 Skip task status sync for local Codex conversation tokens

## 3. App-Server Observability

- [x] 3.1 Log app-server method, item id, and item type for real UI acceptance
- [x] 3.2 Log local app-server completion, cancellation, and error finish paths

## 4. Local App Launch

- [x] 4.1 Use an isolated debug data profile for default local Codex OSS launches
- [x] 4.2 Avoid macOS App Group secure-state probing for OSS/profiled launches
- [x] 4.3 Skip Warp Agent onboarding when local Codex mode is enabled

## 5. Tests And Verification

- [x] 5.1 Add local stream id test for requests with ambient task ids
- [x] 5.2 Add task-status local token guard test
- [x] 5.3 Run `openspec validate warp-codex-local-completion-v8`
- [x] 5.4 Run `cargo fmt --check`
- [x] 5.5 Run `cargo test -p warp local_codex --lib`
- [x] 5.6 Run `cargo test -p warp task_status_sync_model --lib`
- [x] 5.7 Run `cargo check -p warp --bin warp-oss --features agent_harness`
- [x] 5.8 Run `git diff --check`
- [x] 5.9 Complete UI acceptance in `/Applications/WarpCodexOss.app`
- [x] 5.10 Remove normal Agent bridge-answer prompt wrapper and re-test with a non-fixed prompt
