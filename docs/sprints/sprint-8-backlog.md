# Sprint 8 Backlog

## Goal

Ship and distribute the complete javalens-manager application. This sprint is split into two phases:
1. **Phase 1: Helper functionality and Help/License+About**
2. **Phase 2: Package / Build / Installer / Update + Github Releases**

## Problem Statement

- The app is runnable in development, but deployment workflow is not yet productized.
- Users need an installable package and a repeatable upgrade path.
- Release publishing should be automated and reproducible, not manual-only.
- The app lacks user-facing documentation (Help) and proper attribution (About/Licenses).

---

## Phase 1: Helper functionality and Help/License+About

### 1. Help View & Navigation
*Discussion on Format (MD vs PDF):*
While a PDF is a fixed, printable format, it requires an external PDF viewer or a heavy PDF rendering library to display inside the app. Since Tauri uses a webview (Svelte/HTML), **Markdown (MD) rendered to HTML** is the recommended approach. It is lightweight, native to the web stack, easy to version control, and can be styled to perfectly match the app's dark theme. We will bundle the Help as a Markdown file and render it directly in a native Svelte view.

Acceptance criteria:
- Add a "Help" button in the main navigation sidebar, on the same level as "Dashboard" and "Settings".
- Clicking "Help" opens a new primary view in the app.
- The Help view renders a bundled Markdown documentation file detailing how to use the manager, add projects, and deploy to agents.
- At the bottom of the Help view, add a distinct button (e.g., "About javalens-manager" or "View Licenses & Credits") to open the About Box.

### 2. About Box & Licensing
Acceptance criteria:
- Add an "About" dialog or modal accessible from the button at the bottom of the Help view.
- Display the application version and build information.
- Display the primary author: **Harald**.
- Display the primary license: **MIT License**.
- Display explicit credits to **P. Zalutski** for the upstream `javalens-mcp` project.
- Include an acknowledgment/list of other open-source licenses used in the project (Rust crates, Svelte/NPM dependencies, Tauri).

### 3. Deployment UX in App
Acceptance criteria:
- Add an in-app "How to Install/Update" entry (link/panel) pointing to release assets and instructions.
- Show update availability status tied to release metadata (integrating with the existing release manager).

---

## Phase 2: Package / Build / Installer / Update + Github Releases

### 4. Packaging Targets and Build Artifacts
Acceptance criteria:
- Define supported deployment targets and artifact types (minimum: Linux desktop package + standalone executable).
- Produce consistent versioned artifacts per release via Tauri's bundler (`.deb`, `.AppImage`, etc.).
- Include checksum files for all release artifacts.
- Validate artifact startup on clean test environments.

### 5. Installer and Update Script
Acceptance criteria:
- Provide install script(s) that:
  - download release assets
  - verify checksum/signature policy
  - install binaries/assets into expected paths
  - create desktop entry/launcher where applicable
- Provide update script with safe in-place upgrade and rollback guidance.
- Provide uninstall script or documented uninstall procedure.

### 6. GitHub Release Automation
Acceptance criteria:
- Add a GitHub Actions release workflow that builds artifacts from tags/releases and uploads them to GitHub Releases.
- Publish release notes template with:
  - version
  - supported platforms
  - install/update instructions
  - known limitations
- Fail release on missing artifacts/checksums/validation failures.

### 7. Runtime/Config Migration Safety
Acceptance criteria:
- Validate compatibility/migration behavior for existing config/state directories.
- Preserve user settings and projects across upgrades.
- Add backup or migration guard for incompatible schema changes.

### 8. Documentation
Acceptance criteria:
- Add deployment runbook for maintainers:
  - preparing a release
  - validating artifacts
  - publishing and rollback
- Add user-facing install/update quickstart docs.
- Document platform-specific caveats and troubleshooting.

---

## Team Split

- `frontend-engineer`: Phase 1 Help view, Markdown rendering, About box, and License attribution.
- `release-engineer`: Phase 2 CI/CD release workflow, artifact validation, GitHub publishing.
- `tauri-engineer`: Phase 2 packaging config, runtime migration safety, installer hooks.
- `qa-test-engineer`: Install/update/uninstall matrix and rollback verification.
- `docs-engineer`: Help markdown content, release runbook, and end-user install/update docs.

## Deferred

- Delta/binary patch updates.
- Auto-update service integration beyond scripted installer/update flow.
- Enterprise deployment channels (MSI repos, apt/yum repos, managed MDM integration).