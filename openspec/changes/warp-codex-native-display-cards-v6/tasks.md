## 1. OpenSpec

- [x] 1.1 Create V6 proposal, design, specs, and tasks
- [x] 1.2 Validate V6 OpenSpec artifacts

## 2. Display-Only Card Mapping

- [x] 2.1 Add a display-only local Codex tool call id prefix
- [x] 2.2 Parse app-server `item/started` command execution payloads into display-only command cards
- [x] 2.3 Keep command output, file diffs, tool progress, and unsupported events as transcript text

## 3. Execution Guard

- [x] 3.1 Mark display-only local Codex tool calls as `requires_result = false`
- [x] 3.2 Filter non-result actions out of normal response-stream action queueing
- [x] 3.3 Filter non-result actions out of shared-session action queueing

## 4. Tests And Verification

- [x] 4.1 Add conversion coverage for display-only tool calls
- [x] 4.2 Add local Codex translator coverage for app-server command cards
- [x] 4.3 Run `openspec validate warp-codex-native-display-cards-v6`
- [x] 4.4 Run `cargo fmt --check`
- [x] 4.5 Run `cargo test -p warp local_codex --lib`
- [x] 4.6 Run `cargo test -p warp codex --lib`
- [x] 4.7 Run `cargo check -p warp --bin warp-oss --features agent_harness`
