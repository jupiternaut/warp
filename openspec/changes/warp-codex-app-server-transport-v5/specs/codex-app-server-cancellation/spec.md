## ADDED Requirements

### Requirement: App-server cancellation interrupts the turn
The local app-server runner SHALL attempt to interrupt the active Codex turn before terminating the child process.

#### Scenario: Turn id is known
- **WHEN** the user cancels after `turn/start` returns a turn id
- **THEN** the runner sends `turn/interrupt` with the active thread id and turn id

#### Scenario: Turn id is unknown or interrupt does not complete
- **WHEN** cancellation occurs before a turn id is known or app-server does not stop promptly
- **THEN** the runner terminates the app-server child and Warp finishes the stream as cancelled
