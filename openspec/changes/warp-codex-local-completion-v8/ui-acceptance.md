# V8 UI Acceptance

Checked at: 2026-05-11 11:27:50 CST

## Build Installed

- Built with `./script/run --features agent_harness --dont-open`.
- Installed to `/Applications/WarpCodexOss.app`.
- `codesign --verify --deep --strict --verbose=2 /Applications/WarpCodexOss.app` passed.
- `codex login status` returned `Logged in using ChatGPT`.

## Automated Verification

- `openspec validate warp-codex-local-completion-v8` passed.
- `cargo fmt --check` passed.
- `cargo test -p warp_core paths --lib` passed.
- `cargo test -p warp local_codex --lib` passed.
- `cargo check -p warp --bin warp-oss --features agent_harness` passed.
- `git diff --check` passed.

## Launch Evidence

- Process path: `/Applications/WarpCodexOss.app/Contents/MacOS/warp-oss`.
- Launch env included:
  - `WARP_LOCAL_CODEX_RUNNER=app-server`
  - `WARP_LOCAL_CODEX_TIMEOUT_MS=180000`
- Startup log included:
  - `Local Codex AI: enabled`
  - `login_status=Logged in using ChatGPT`
- Default debug local Codex data profile created:
  - `/Users/gengrf/Library/Application Support/dev.warp.WarpOss-codex-local/warp.sqlite`
- WindowServer saw an onscreen WarpCodexOss window:
  - owner `WarpCodexOss`
  - bounds `X=279 Y=134 Width=1152 Height=721`
  - owner PID matched the launched `warp-oss` process.

## Fixes Added During Acceptance

- Default debug local Codex launches now use an isolated `codex-local` data profile unless `WARP_LOCAL_CODEX_AI=0`.
- macOS secure state now avoids App Group probing for OSS/profiled launches, preventing the local unsigned OSS app from blocking during startup.
- Local Codex mode skips Warp Agent onboarding so the app can enter the main workspace directly.
- The local Codex app keeps the `WarpCodexOss` app name while registering the OSS login URL scheme `warposs`.
  This fixed the browser auth callback; Safari had rejected the previous `warpcodexoss` scheme while Warp's login service returned `warposs://...`.

## UI Acceptance Result

The earlier macOS `Display 1 Shield` / `SecurityAgent` overlay was gone on the rerun, so Computer Use could read and operate WarpCodexOss.

- Browser login completed and returned to `/Applications/WarpCodexOss.app`.
- The first-run model selector showed `Local Codex (ChatGPT OAuth)`.
- The Agent input bar showed `Local Codex (ChatGPT OAuth) (local codex app-server)`.
- A first fixed prompt was submitted from the native Agent panel:

```text
/agent warp-local-codex-v8-ok
```

Observed UI evidence:

- UI shows `Local Codex / 0 Warp credits`.
- The response rendered in the Agent panel:
  - `warp-local-codex-v8-ok received.`
  - `Native bridge v1 is responding from /Users/gengrf. No file changes or shell actions performed.`
- The footer remained on `Local Codex (ChatGPT OAuth) (local codex app-server)`.

Observed log evidence:

- Log shows `generate_multi_agent_output: routing to local Codex; using 0 Warp AI credits`.
- Log shows `Local Codex AI: running codex app-server --listen stdio://; cwd=/Users/gengrf`.
- Log shows `Local Codex app-server event: method=item/agentMessage/delta`.
- Log shows `Local Codex AI: app-server completed; sending StreamFinished Done`.
- Follow-up generation also stayed local: `Local Codex AI: running cancellable codex exec --json; cwd=/Users/gengrf`.

## Correction After Review

The first fixed-prompt acceptance above proved the route and credits boundary, but it was not sufficient proof of real Codex answer quality. The normal Agent prompt path was still wrapping user input with:

```text
You are running inside Warp's Agent panel through local Codex...
perform only safe textual planning in this native bridge v1...
```

That wrapper biased the model into returning bridge-status text such as `Native bridge v1 is responding...`.

Fix applied:

- Normal Agent turns now pass the extracted Warp request/context directly to Codex.
- Passive suggestion prompts still keep their JSON-only wrapper because that endpoint requires structured suggestion output.
- Added unit coverage: `agent_turn_prompt_does_not_inject_bridge_answer_template`.

Re-test prompt submitted from the native Agent panel:

```text
/agent Write one sentence about a blue cup. Do not mention native bridge Codex or Warp.
```

Observed UI evidence after the fix:

- The response was content-specific, not bridge-status text:
  - `A blue cup sat quietly on the windowsill, catching the morning light.`
- UI still showed `Local Codex / 0 Warp credits`.
- The footer still showed `Local Codex (ChatGPT OAuth) (local codex app-server)`.

Observed log evidence after the fix:

- Log shows `generate_multi_agent_output: routing to local Codex; using 0 Warp AI credits`.
- Log shows `Local Codex AI: running codex app-server --listen stdio://; cwd=/Users/gengrf`.
- Log shows `Local Codex app-server event: method=item/agentMessage/delta`.
- Log shows `Local Codex AI: app-server completed; sending StreamFinished Done`.
- No `native bridge` or `safe textual planning` prompt wrapper remains in the normal Agent path.

## Remaining Gaps

- The reply is still the V8 native bridge text path, not full Warp-native tool card / diff card rendering.
- The prompt used for this pass was text-only; file edit / command execution card acceptance still belongs to the next V9/V10 protocol-bridge pass.
