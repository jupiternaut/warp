## ADDED Requirements

### Requirement: Local Agent requests use a cancellable Codex stream
When local Codex mode is enabled, Warp Agent requests SHALL be routed to a local Codex stream source before any Warp server-backed AI request is made.

#### Scenario: Agent request starts local stream immediately
- **WHEN** `WARP_LOCAL_CODEX_AI` is enabled and the user sends an Agent prompt
- **THEN** Warp creates a local response stream without waiting for the Codex runner to finish

#### Scenario: Agent cancellation terminates Codex
- **WHEN** the user cancels a running local Codex Agent request
- **THEN** the running Codex child process is terminated and Warp receives a cancelled finish event

### Requirement: Local Codex readiness is OAuth-safe
The system SHALL validate local Codex readiness through CLI commands only and MUST NOT read, copy, parse, or write Codex OAuth token files.

#### Scenario: Codex login is missing
- **WHEN** `codex login status` does not report a ChatGPT login
- **THEN** Warp shows an actionable local Codex login error without reading `~/.codex/auth.json`

### Requirement: Codex runner events are translated to Warp events
The system SHALL translate local Codex runner events into Warp response events and SHALL degrade unsupported events to assistant text.

#### Scenario: Text output
- **WHEN** Codex emits assistant text
- **THEN** Warp appends assistant output to the active Agent task

#### Scenario: Unsupported structured event
- **WHEN** Codex emits an event that the V3 translator cannot render as a native Warp card
- **THEN** Warp appends a readable assistant-text fallback and keeps the conversation valid

### Requirement: App-server runner boundary exists
The system SHALL define a runner boundary that can support both `codex exec` and `codex app-server` without changing the Agent API routing layer.

#### Scenario: Exec runner selected
- **WHEN** V3 runs with the default local Codex runner
- **THEN** the bridge uses `codex exec --json --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check`

#### Scenario: App-server runner is not fully enabled
- **WHEN** app-server protocol mapping is incomplete
- **THEN** the product remains on the exec runner and does not claim native tool-card parity
