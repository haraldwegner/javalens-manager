# Sprint 4 Backlog

## Goal

Polish the Settings UI and make the JavaLens installation directory user-configurable.

## Stories

### 1. Configurable Install Directory

Acceptance criteria:
- The manager settings schema includes a `tools_dir` field.
- The `ReleaseManager` uses the configured `tools_dir` instead of a hardcoded cache path for downloading and listing JavaLens releases.
- The UI provides a directory picker (using Tauri's dialog plugin) to select the install directory.

### 2. Settings UI Polish

Acceptance criteria:
- Hide the verbose "System Information" diagnostic grid behind an "Advanced Options" toggle.
- Clean up the layout and spacing of the settings cards.
- Ensure the UI feels denser and less wasteful of screen real estate.

## Team Split

- `platform-architect`: Update settings schema and `ReleaseManager` paths.
- `tauri-engineer`: Integrate `tauri-plugin-dialog` for the directory picker.
- `frontend-engineer`: Implement the directory picker UI and advanced toggle.