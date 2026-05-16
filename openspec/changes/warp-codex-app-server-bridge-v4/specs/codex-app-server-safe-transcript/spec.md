## ADDED Requirements

### Requirement: Codex app-server events avoid double execution
The bridge SHALL NOT convert Codex-executed shell, file, or tool events into Warp-native executable `ToolCall`s unless the event is explicitly classified as Warp-owned before execution.

#### Scenario: Codex command output event
- **WHEN** Codex app-server reports command output for a command Codex already executed
- **THEN** Warp renders the event as transcript or fallback text and does not execute the command again

#### Scenario: Codex diff event
- **WHEN** Codex app-server reports a turn diff that may already reflect local file changes
- **THEN** Warp renders the diff as transcript or fallback text unless a future allowlist routes application through Warp first

### Requirement: Safe app-server events map to native messages
The bridge SHALL map side-effect-free Codex app-server events to native Warp messages when possible.

#### Scenario: Assistant message delta
- **WHEN** Codex app-server emits an assistant message delta
- **THEN** Warp appends that delta to a native `AgentOutput` message

#### Scenario: Reasoning delta
- **WHEN** Codex app-server emits a reasoning delta
- **THEN** Warp appends that delta to a native `AgentReasoning` message

### Requirement: Unsupported protocol events remain inspectable
The bridge SHALL render unsupported app-server events as readable assistant transcript text.

#### Scenario: Unknown notification
- **WHEN** Codex app-server emits a notification with no V4 mapping
- **THEN** Warp keeps the conversation valid and displays a readable fallback
