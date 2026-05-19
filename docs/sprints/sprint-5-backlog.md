# Sprint 5 Backlog

## Goal

Clarify **default filesystem paths**, the meaning of **workspace dir override**, and **where controls live** in the UI (Dashboard vs Settings) so users and contributors are not surprised by naming or placement.

## Q&A (frozen for this sprint)

### What is workspace dir override?

It is an **optional per-project path** for the Eclipse/JDT **`-data`** directory passed to `javalens-mcp` (`java -jar … -data <workspace>`). If left empty at registration time, the manager assigns a **manager-owned** directory under the cache workspace root (typically `~/.cache/javalens-manager/workspaces/<project-id>` on Linux). If the user sets an override, that path is stored on the project record and used instead.

See [`docs/javalens-runtime-contract.md`](javalens-runtime-contract.md) for the persisted field (`workspaceDir`) and launch contract.

### Where does the install directory put managed JavaLens?

**Default (Linux, typical XDG):** managed releases unpack under:

`~/.cache/javalens-manager/tools/javalens/` (versioned subfolders such as `javalens-<version>`).

The value is persisted as `toolsDir` in `settings.json` and can be changed in **Settings**. After a custom path is saved, that directory is authoritative—not the default above.

### Why is “Advanced project options” on the Dashboard and not in Settings?

**Dashboard** holds **per-project** registration: JavaLens source (managed vs local JAR), local JAR path, and workspace override. Those options apply to one project entry.

**Settings** holds **global** preferences: install directory, update policy, default managed version, download actions, plus a **diagnostics** panel for config paths.

Per-project options belong with project registration; global paths and policies belong in Settings. Clear labeling reduces confusion between the two areas.

## Stories

### 1. Document default paths and workspace override

Acceptance criteria:

- [`docs/javalens-runtime-contract.md`](javalens-runtime-contract.md) includes a concise **Default paths** section (tools vs workspaces vs logs, with Linux examples).
- [`docs/javalens-management.md`](docs/javalens-management.md) aligns the **Linux default shape** with the exact `tools/javalens` segment used in code.

### 2. UI copy: distinguish project vs diagnostics

Acceptance criteria:

- Dashboard project form uses **“Advanced project options”** (not a generic “Advanced Options”) for the per-project `<details>` block.
- Settings uses **“Diagnostics & paths”** (or equivalent) for the collapsed system paths panel.
- Workspace override field includes a short hint that the default is under the manager workspace root (see bootstrap / contract).

### 3. Discoverability

Acceptance criteria:

- [`README.md`](../README.md) and [`docs/roadmap.md`](roadmap.md) link to this sprint backlog.

## Team Split

- `requirements-analyst`: keep Q&A and acceptance criteria aligned with the runtime contract
- `technical-writer`: paths table and cross-doc consistency
- `tauri-engineer`: Svelte copy and hints only (no schema changes)

## QA

- Spot-check: fresh user on Linux sees documented defaults match `AppPaths::detect` behavior in `src-tauri/src/config.rs`.
- Manual: Settings shows diagnostics paths; Dashboard project form advanced section still saves projects and optional workspace override.

## Deferred

- Global default workspace root override in Settings (would be a new setting and UX)
- Moving per-project editing out of the registration form into a dedicated project editor (larger UX sprint)
