# Sprint 7 QA Matrix

## Axes

- Clients: Cursor, Claude, Antigravity, IntelliJ
- Modes: `deploy`, `dryRun`, `preview`, `regenerate`
- Merge mode: `safeMerge`, `replaceManagedSection`
- Backup flag: `true`, `false`
- Runtime resolution: resolved / unresolved
- Settings deploy flag: enabled / disabled
- Run-scoped target override: default / custom subset

## Core Matrix

| Case | Client | Mode | Merge | Backup | Runtime | Expected |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | all | deploy | safeMerge | true | resolved | success, managed sections written, backup present |
| 2 | all | deploy | replaceManagedSection | false | resolved | success, managed section rewritten |
| 3 | all | dryRun | safeMerge | true | resolved | success, no file writes |
| 4 | all | preview | safeMerge | true | resolved | success, preview content returned |
| 5 | all | regenerate | safeMerge | true | resolved | success, managed section rewritten even if unchanged |
| 6 | all | deploy | safeMerge | true | unresolved | failed or partial failed with validation errors |
| 7 | cursor+claude only (flags) | deploy | safeMerge | true | resolved | cursor+claude processed, others skipped with disabled reason |
| 8 | custom override (single client) | dryRun | safeMerge | true | resolved | only selected client processed, others skipped with not-selected reason |

## Validation Checks

- Parent path validation errors are surfaced per client.
- Rule artifacts reference generated MCP server ids.
- Partial invalid writes are blocked for failed clients.

## Tray Behavior Checks

- `useSystemTray=true` + running service + close => hide to tray.
- `useSystemTray=false` + close => explicit shutdown confirmation prompt.
- `useSystemTray=true` + no running service + close => explicit shutdown confirmation prompt.
- Tray actions:
  - `Show` restores/focuses window.
  - `Stop all services` transitions runtimes to stopped.
  - `Quit` with running services opens UI and prompts stop-and-quit vs cancel/hide.
  - `Quit` without running services prompts confirmation before exit.
