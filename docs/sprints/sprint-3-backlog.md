# Sprint 3 Backlog

## Goal

Clean up the main UI by extracting global settings and diagnostics into a dedicated view, separating project orchestration from manager configuration.

## Stories

### 1. Tabbed Navigation

Acceptance criteria:
- The main app shell provides a tabbed navigation structure.
- "Dashboard" view focuses on project registration and runtime status.
- "Settings" view focuses on global manager preferences and diagnostics.

### 2. Settings Extraction

Acceptance criteria:
- Move the `RuntimeSettings` component from the main dashboard into the new Settings tab.
- Move the System Information / Bootstrap diagnostic grid into the Settings tab.
- Ensure the Dashboard remains focused purely on the project list, project form, and selected runtime details.

## Team Split

- `frontend-engineer`: Implement tabbed navigation and move components.
- `qa-test-engineer`: Verify state persists when switching tabs and forms still work.