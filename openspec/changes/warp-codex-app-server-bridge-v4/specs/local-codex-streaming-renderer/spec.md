## ADDED Requirements

### Requirement: Local Codex text can stream through Warp append actions
The local Codex renderer SHALL be able to create a message once and append future text deltas using `AppendToMessageContent`.

#### Scenario: First text delta
- **WHEN** a local Codex text item receives its first delta
- **THEN** Warp creates a native `AgentOutput` message for that item and appends the delta

#### Scenario: Subsequent text delta
- **WHEN** the same local Codex text item receives another delta
- **THEN** Warp appends the delta to the existing message instead of creating a duplicate message

### Requirement: Local Codex reasoning can stream as reasoning
The local Codex renderer SHALL use `AgentReasoning` for reasoning deltas rather than prefixing ordinary assistant text.

#### Scenario: Reasoning delta
- **WHEN** a local Codex reasoning item receives a delta
- **THEN** Warp creates or appends a native `AgentReasoning` message

### Requirement: Transcript fallbacks are grouped and readable
The local Codex renderer SHALL group command, diff, tool, and unsupported transcript events into readable assistant output without triggering side effects.

#### Scenario: Command transcript
- **WHEN** local Codex emits command output that Warp did not execute
- **THEN** Warp displays the command transcript as assistant text and does not create a `RunShellCommand` action
