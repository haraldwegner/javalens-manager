# Sprint 1 Backlog

## Goal

Deliver the first runnable manager slice for one Java project and one JavaLens runtime.

## Stories

### 1. Manual Tauri scaffold lands in-repo

Acceptance criteria:

- frontend uses `Svelte + TypeScript + Vite`
- backend uses `Tauri + Rust`
- repo contains `src/`, `src-tauri/`, `package.json`, and Tauri config files

### 2. Project registry persists one or more managed projects

Acceptance criteria:

- the manager stores project entries in a manager-owned config file
- a project entry contains `name`, `projectPath`, `javalensJarPath`, and `workspaceDir`
- duplicate project paths are rejected

### 3. One JavaLens runtime can be started

Acceptance criteria:

- the manager launches `java -jar ... -data ...`
- `JAVA_PROJECT_PATH` is passed for auto-load
- stdout/stderr are redirected to a manager-owned log file

### 4. One JavaLens runtime can be stopped

Acceptance criteria:

- the manager can stop a previously launched process
- status returns to `stopped`
- the log path remains visible for diagnostics

### 5. UI shows current runtime state

Acceptance criteria:

- the user can register a project from the UI
- the user can start and stop the selected runtime
- the UI shows transport, workspace, PID when present, and log path
- the UI shows `starting`, `running`, `stopped`, or `failed`

## Focused QA

### Rust tests

- config slug/id/path helpers
- runtime command-spec generation for `java -jar ... -data ...`

### Manual checks

1. Start the frontend and Tauri shell locally.
2. Register one Java project and one JavaLens JAR path.
3. Confirm `projects.json` is created under the manager config directory.
4. Start the runtime and verify the UI shows `starting` then `running`.
5. Confirm a log file appears in the manager state directory.
6. Stop the runtime and verify the UI returns to `stopped`.

## Deferred To Sprint 2

- generated MCP client config
- richer health probes using upstream `health_check`
- bulk start/stop
- tray integration
- richer logs UI
