## ADDED Requirements

### Requirement: Display-only Codex cards must not execute in Warp
WarpCodexOss SHALL distinguish native cards that represent Codex-executed work from executable Warp-owned tool calls.

#### Scenario: Display-only command card
- **WHEN** Codex app-server reports a command item that includes the command text
- **THEN** Warp renders a native command card with a display-only local Codex tool call id

#### Scenario: Queue exclusion
- **WHEN** a local Codex display-only card is present in a completed response stream
- **THEN** Warp does not queue that card for execution

### Requirement: Codex output remains inspectable
WarpCodexOss SHALL keep Codex command output, diff output, tool progress, and unsupported events visible even when a display-only native card is present.

#### Scenario: Command output after display card
- **WHEN** Codex app-server emits command output deltas for a display-only command card
- **THEN** Warp renders the output as transcript text without executing the command

#### Scenario: Missing command metadata
- **WHEN** Codex app-server emits command output without a prior command item containing the command text
- **THEN** Warp falls back to transcript text and does not create a native executable card
