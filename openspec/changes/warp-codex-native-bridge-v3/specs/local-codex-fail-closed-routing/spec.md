## ADDED Requirements

### Requirement: Local mode prevents silent Warp AI fallback
When local Codex mode is enabled, user-visible generative AI entry points SHALL either use local Codex or return a local Codex error; they MUST NOT silently call Warp `/ai/*` credit-consuming endpoints.

#### Scenario: Codex is unavailable
- **WHEN** local Codex mode is enabled and the Codex CLI is missing or logged out
- **THEN** the entry point returns an actionable local Codex error instead of calling Warp AI

#### Scenario: Warp AI fallback is explicit
- **WHEN** the user disables local mode with `WARP_LOCAL_CODEX_AI=0`
- **THEN** Warp may use upstream AI and the UI or logs identify that Warp AI credits are being used

### Requirement: Local cost labeling is visible
Local Codex Agent responses SHALL display a local-cost label instead of a positive Warp credit charge.

#### Scenario: Local Codex response
- **WHEN** a local Codex Agent response finishes
- **THEN** the UI displays `Local Codex / 0 Warp credits` or an equivalent zero-Warp-credit label

### Requirement: Request path is logged
Every local Codex generative entry point SHALL log enough request-path information to distinguish local Codex from Warp AI.

#### Scenario: Agent request path
- **WHEN** a local Codex Agent request runs
- **THEN** logs include a local Codex routing message and do not imply a Warp `/ai/multi-agent` call
