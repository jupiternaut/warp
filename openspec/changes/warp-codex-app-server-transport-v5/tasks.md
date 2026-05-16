## 1. OpenSpec

- [x] 1.1 Create V5 change artifacts for app-server stdio transport
- [x] 1.2 Validate V5 OpenSpec after implementation

## 2. App-Server Runner

- [x] 2.1 Add newline JSON-RPC request/response helpers
- [x] 2.2 Implement initialize, thread/start, and turn/start handshake
- [x] 2.3 Translate app-server notifications into the V4 safe renderer
- [x] 2.4 Reject server-originated tool/approval requests without executing them in Warp

## 3. Stream And Cancellation

- [x] 3.1 Route normal Agent prompts through app-server when `WARP_LOCAL_CODEX_RUNNER=app-server`
- [x] 3.2 Keep passive suggestions and structured helper tasks on `codex exec`
- [x] 3.3 Send `turn/interrupt` on cancellation and kill the child as fallback

## 4. Tests And Verification

- [x] 4.1 Add fake app-server tests for streaming output and reasoning
- [x] 4.2 Add fake app-server tests proving command/diff/tool events remain transcript text
- [x] 4.3 Add fake app-server cancellation coverage
- [x] 4.4 Run `cargo fmt --check`
- [x] 4.5 Run `cargo test -p warp local_codex --lib`
- [x] 4.6 Run `cargo test -p warp codex --lib`
- [x] 4.7 Run `cargo check -p warp --bin warp-oss --features agent_harness`
