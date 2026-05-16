## 1. OpenSpec Product Contract

- [x] 1.1 Initialize OpenSpec for this WarpCodexOss fork.
- [x] 1.2 Create proposal, design, and capability specs for `warp-codex-native-bridge-v3`.
- [x] 1.3 Keep task status updated as implementation lands.

## 2. Cancellable Local Agent Stream

- [x] 2.1 Inspect Warp Agent response-stream construction and cancellation ownership.
- [x] 2.2 Move local Codex Agent routing before Warp server-backed AI stream creation.
- [x] 2.3 Add a cancellable `codex exec --json` runner that starts a stream immediately.
- [x] 2.4 Ensure cancellation terminates the running Codex child process and emits a cancelled finish event.

## 3. Runner And Event Translation

- [x] 3.1 Introduce a local Codex runner/event abstraction that can support `exec` and later `app-server`.
- [x] 3.2 Translate local Codex text/reasoning/tool/diff/command/finish/unsupported events to valid Warp response events.
- [x] 3.3 Keep unsupported structured events as readable assistant-text fallbacks.

## 4. Fail-Closed Routing And Cost Boundary

- [x] 4.1 Verify all current user-visible generative entry points use local Codex when enabled.
- [x] 4.2 Add tests or guards proving local mode does not silently call Warp AI endpoints.
- [x] 4.3 Preserve explicit `Local Codex / 0 Warp credits` model and response labels.

## 5. Windows Source Readiness

- [x] 5.1 Verify Windows packaging scripts still expose `-LOCAL_CODEX` and `WarpCodexOssSetup.exe`.
- [x] 5.2 Document that Windows installer/UI acceptance requires a real Windows x64 host.

## 6. Verification

- [x] 6.1 Run `cargo fmt --check`.
- [x] 6.2 Run `cargo test -p warp local_codex --lib`.
- [x] 6.3 Run `cargo test -p warp codex --lib`.
- [x] 6.4 Validate the OpenSpec change after implementation updates.
