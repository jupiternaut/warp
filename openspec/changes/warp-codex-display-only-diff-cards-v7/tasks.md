## 1. OpenSpec

- [x] 1.1 Create V7 proposal, design, specs, and tasks
- [x] 1.2 Validate V7 OpenSpec artifacts

## 2. Display-Only Diff Mapping

- [x] 2.1 Parse app-server structured file patch metadata
- [x] 2.2 Translate structured file patches into display-only `ApplyFileDiffs` tool calls
- [x] 2.3 Keep partial or malformed file patch events as transcript text

## 3. Tests And Verification

- [x] 3.1 Add local Codex translator coverage for display-only diff cards
- [x] 3.2 Extend fake app-server coverage for command and diff display-only cards
- [x] 3.3 Run `openspec validate warp-codex-display-only-diff-cards-v7`
- [x] 3.4 Run `cargo fmt --check`
- [x] 3.5 Run `cargo test -p warp local_codex --lib`
- [x] 3.6 Run `cargo test -p warp codex --lib`
- [x] 3.7 Run `cargo check -p warp --bin warp-oss --features agent_harness`
- [x] 3.8 Run `git diff --check`
