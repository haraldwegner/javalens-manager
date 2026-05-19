# Sprint 6 Backlog

## Goal

Radical Simplification & Global Configuration. Remove per-project runtime and workspace overrides in favor of a single global JavaLens version and a unified data root.

## Architecture Change

### Before (Over-engineered)
- Projects had their own `runtime_source` (Managed vs Local JAR).
- Projects had their own `workspace_dir` override.
- Settings had a `tools_dir` for managed runtimes.

### After (Simplified)
- Global Settings has one `global_runtime_source`.
- Global Settings has one `data_root` (defaulting to `~/.cache/javalens-manager` or similar).
- `data_root` contains both `tools/` and `workspaces/`.
- Projects only store `{ id, name, project_path }` and automatically map to the global runtime and a workspace under `data_root`.

## Stories

### 1. Backend Refactor

Acceptance criteria:
- `ProjectRecord` drops `runtime_source` and `workspace_dir`.
- `ManagerSettings` drops `tools_dir` and adds `data_root` and `global_runtime_source`.
- `AppPaths` derives `tools_dir` and `workspace_root` strictly from `data_root`.
- Loading `projects.json` drops old per-project overrides.

### 2. Frontend Refactor

Acceptance criteria:
- `ProjectForm.svelte` removes the "Advanced project options" section. It only asks for the Project Folder.
- `RuntimeSettings.svelte` adds a "Global JavaLens Source" selector.
- `RuntimeSettings.svelte` changes "Install directory" to a "Manager Data Root" directory picker.

### 3. Documentation Updates

Acceptance criteria:
- `javalens-runtime-contract.md` and `javalens-management.md` reflect the single-root architecture.

## Team Split

- `platform-architect`: Update Rust structs and Tauri commands.
- `frontend-engineer`: Update Svelte components and stores.
- `technical-writer`: Update architecture documentation.