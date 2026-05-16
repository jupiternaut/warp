## ADDED Requirements

### Requirement: Invalid local conversation quarantine criteria are documented
V4 SHALL document the invalid persisted local Agent conversation cases that must be quarantined before the code path is marked complete.

#### Scenario: Missing root task
- **WHEN** a persisted local conversation has no root task
- **THEN** the evidence artifact records it as a quarantine criterion and links it to the known restore failure class

#### Scenario: Missing initial query
- **WHEN** a persisted local conversation has no user query or usable diff summary
- **THEN** the evidence artifact records it as a quarantine criterion and notes that future code should summarize counts instead of emitting repeated per-record warnings

### Requirement: Generative routes are classified
The product SHALL maintain an explicit classification for user-visible generative AI routes in local mode.

#### Scenario: New AI endpoint
- **WHEN** an `/ai/` endpoint or generative GraphQL operation is added
- **THEN** it must be classified as local-Codex, allowed non-generative Warp service, or explicit Warp-credit/cloud-only route

### Requirement: Windows validation evidence is explicit
Windows support SHALL be reported as source-ready until a real Windows x64 host provides validation evidence.

#### Scenario: macOS build host
- **WHEN** V4 is developed on macOS
- **THEN** OpenSpec and reports do not mark Windows installer/UI acceptance complete

#### Scenario: Windows host validation
- **WHEN** a Windows x64 host validates WarpCodexOss
- **THEN** the report records Codex CLI path, OAuth login status, installer path, process path, local model UI, fixed prompt result, and zero Warp credits evidence without recording tokens
