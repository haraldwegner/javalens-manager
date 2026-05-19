# Sprint 2 Backlog

## Goal

Move from a manually wired first slice to a manager-owned JavaLens experience:

- persisted manager settings
- managed JavaLens discovery and caching
- dropdown-driven runtime selection
- a clearer manager-service boundary between release handling, runtime supervision, and UI orchestration

## Stories

### 1. Persisted manager settings

Acceptance criteria:

- the manager stores global settings in `settings.json`
- update policy is persisted as `always` or `ask`
- the selected default managed JavaLens version survives restart
- release-check metadata survives restart

### 2. Managed JavaLens acquisition

Acceptance criteria:

- the manager can check the latest upstream JavaLens release
- the manager can download and unpack the latest release into a manager-owned tools directory
- if no managed runtime exists yet, the manager can bootstrap one automatically
- a downloaded runtime is recorded with version, install location, and jar path

### 3. Dropdown-based runtime selection

Acceptance criteria:

- the project form offers a managed runtime dropdown
- the project form still supports a local JAR fallback mode
- the normal path no longer requires typing a raw JAR path when a managed runtime is available
- the selected project detail view shows the resolved runtime label and JAR path

### 4. Manager service boundary

Acceptance criteria:

- release/download logic is isolated from process supervision
- runtime launch/stop logic operates on resolved runtime references instead of raw project config
- Tauri commands call a single manager service layer rather than directly coordinating multiple low-level modules
- runtime snapshots persist enough state to surface last-known status after restart

### 5. QA for update policy and runtime resolution

Acceptance criteria:

- tests cover settings defaults and serialization
- tests cover version comparison and release-status decisions
- tests cover runtime command construction and unresolved-runtime behavior
- manual verification proves:
  1. latest release status is visible in the UI
  2. JavaLens can be downloaded into the managed tools cache
  3. a project can use a managed runtime from the dropdown
  4. a project can still use a local JAR fallback
  5. selected settings and managed runtime choices survive restart

## Team Split

- `orchestrator`: slice work across settings, release management, UI, and runtime orchestration while keeping the sprint narrow
- `requirements-analyst`: lock acceptance criteria for update policy, download behavior, and dropdown UX
- `platform-architect`: own settings schema, runtime cache layout, and the manager-service boundary
- `tauri-engineer`: implement release/download service, persisted settings, Tauri commands, and Svelte dropdown UX
- `qa-test-engineer`: add focused tests and run the manual verification flow

## Deferred

- full background daemon extraction
- full MCP `health_check` orchestration over stdio
- tray behavior
- bulk runtime orchestration
- polished log viewer and external MCP client generation
