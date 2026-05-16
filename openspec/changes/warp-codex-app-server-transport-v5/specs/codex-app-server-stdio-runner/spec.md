## ADDED Requirements

### Requirement: App-server runner uses newline JSON-RPC
The local Codex app-server runner SHALL communicate with `codex app-server --listen stdio://` using newline-delimited JSON-RPC messages.

#### Scenario: Initialize succeeds
- **WHEN** the runner starts app-server
- **THEN** it sends `initialize` and waits for a matching response before starting a thread

#### Scenario: Thread and turn start
- **WHEN** initialization succeeds
- **THEN** the runner sends `thread/start` and `turn/start` with local cwd, no approval prompts, and danger-full-access sandbox semantics

### Requirement: Safe app-server notifications stream into Warp
The runner SHALL translate safe app-server notifications into Warp client actions while the turn is running.

#### Scenario: Agent message delta
- **WHEN** app-server emits `item/agentMessage/delta`
- **THEN** Warp appends the delta to a native `AgentOutput` message

#### Scenario: Reasoning delta
- **WHEN** app-server emits reasoning text or summary deltas
- **THEN** Warp appends the delta to a native `AgentReasoning` message

#### Scenario: Executed command or file event
- **WHEN** app-server emits command, file, tool, or process events
- **THEN** Warp renders them as transcript text and does not create executable `ToolCall`s

### Requirement: App-server errors fail closed
The runner SHALL return local Codex errors without falling back to Warp AI credits.

#### Scenario: JSON-RPC error
- **WHEN** app-server returns an error response or error notification
- **THEN** Warp shows a local Codex error and finishes the stream as an internal error
