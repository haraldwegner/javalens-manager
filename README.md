# javalens-manager

Desktop manager for running and orchestrating JavaLens MCP servers across multiple Java projects.

`javalens-manager` provides a clean desktop experience for managing `javalens-mcp` instances, project workspaces, runtime state, and MCP client setup. By default it pulls the runtime from a maintained fork ([hw1964/javalens-mcp](https://github.com/hw1964/javalens-mcp)) that ships fixes upstream has not picked up yet — see *Release source* in the Settings UI to switch sources. The manager itself remains a pure orchestrator.

## Status

**Beta (v0.13.1)**: `javalens-manager` is a fully functional desktop application on Linux (x86_64 and aarch64). It supports named workspaces of multiple Java projects (each running as one shared JavaLens MCP service), live `workspace.json`-driven reconciliation, automatic fork-runtime download/update, and one-click deploy of MCP entries into Cursor / Claude Desktop / Antigravity / IntelliJ-style configs. The system-tray menu drives per-workspace lifecycle without opening the window. macOS and Windows builds are not yet automated; broader QA and cross-platform testing continue before a stable 1.0.

### Version timeline

- **v0.9.x** (Sprint 7) — initial Tauri shell, project registry, per-project runtime spawn.
- **v0.10.0–v0.10.6** (Sprint 9 + Sprint 10) — configurable release source (fork by default), source-resolution fix (Maven `<sourceDirectory>` / Eclipse `.classpath`), named workspaces, multi-select bulk move + drag-drop, workspace-first dashboard, `workspace.json` file-watcher for live updates.
- **v0.11.0** — Sprint 11 cutover; Help.md / README updates for fork v1.5.0's tool consolidation (66 → 55 tools) and v1.5.1's five JDT-LTK structural-refactoring tools (60 tools per service).
- **v0.11.1** — Sprint 11 closeout; refreshed help screenshots; help/README cross-links for the new "System tray on Linux" caveat.
- **v0.12.0** — Sprint 12: tray menu lifecycle controls — per-workspace toggle entries with status icons, Start all / Stop all peers, 5-second background refresh so external state changes (process death) propagate. Paired with [fork v1.6.0](https://github.com/hw1964/javalens-mcp/releases/tag/v1.6.0) which adds `compile_workspace` and `run_tests` (62 tools per service).
- **v0.13.0** — Sprint 13: tray menu refined for the GNOME / AppIndicator reality. Per-menu-item icons get stripped on GNOME, so the colored disks shipped in v0.12.0 never reached the user — replaced with monochrome unicode bullets (`●` running, `◐` starting, `○` stopped, `✗` failed) that render reliably in the menu's own font. Cleaner menu shape: `Open dashboard` (raises the main window) → workspaces with bullets (click toggles) → Start all / Stop all → Quit. Plus a real fix: tray menu now reflects workspace renames within ~1 s instead of waiting for a runtime restart (workspace_status_summary reads names from the live config_store, not the cached runtime snapshots). 1-second poll with cache-keyed change detection so the menu doesn't flicker on steady state. Paired with [fork v1.7.0](https://github.com/hw1964/javalens-mcp/releases/tag/v1.7.0) which ships 11 new MCP tools across Ring 2 (code generation), Ring 3 (Maven dependency management), Ring 4 (formatter / workflow polish) — **73 tools per service**.

- **v0.13.1** — packaging patch: GitHub release workflow now also builds for Linux **aarch64** on a free `ubuntu-22.04-arm` runner, so every tag publishes both `_amd64` and `_arm64` artifacts. `install.sh` now detects `uname -m` and pulls the matching AppImage automatically — the same `curl … | bash` works on x86_64 and ARM laptops/servers (verified on NVIDIA DGX Spark / GB10).

See [`docs/release-notes/`](docs/release-notes/) for per-release detail.

## Docs

- [`docs/plan.md`](docs/plan.md) - project scope, boundaries, and core decisions
- [`docs/roadmap.md`](docs/roadmap.md) - sprint-level roadmap for this repository
- [`docs/architecture.md`](docs/architecture.md) - architecture direction and module boundaries
- [`docs/adr/README.md`](docs/adr/README.md) - accepted architecture decision records
- [`docs/javalens-management.md`](docs/javalens-management.md) - how upstream `javalens-mcp` is imported, versioned, and managed
- [`docs/javalens-runtime-contract.md`](docs/javalens-runtime-contract.md) - exact Sprint 1 launch, transport, health, and config contract
- [`docs/sprints/sprint-1-backlog.md`](docs/sprints/sprint-1-backlog.md) - first runnable-slice backlog and acceptance criteria
- [`docs/sprints/sprint-2-backlog.md`](docs/sprints/sprint-2-backlog.md) - managed-runtime upgrade backlog, team split, and acceptance criteria
- [`docs/sprints/sprint-5-backlog.md`](docs/sprints/sprint-5-backlog.md) - paths, workspace override, and Dashboard vs Settings UX documentation
- [`docs/sprints/sprint-6-backlog.md`](docs/sprints/sprint-6-backlog.md) - radical simplification and global configuration
- [`docs/tauri-bootstrap.md`](docs/tauri-bootstrap.md) - Sprint 0 and Sprint 1 bootstrap path for the Tauri app

## Installation

You can install or update `javalens-manager` on Linux using the provided installation script. This script will download the latest `.AppImage` and set up a desktop entry for you.

Run the following command in your terminal:

```bash
curl -sSL https://raw.githubusercontent.com/hw1964/javalens-manager/main/install.sh | bash
```

Alternatively, you can download the `.deb` or `.AppImage` files manually from the [GitHub Releases page](https://github.com/hw1964/javalens-manager/releases).

If you launch the `.AppImage` manually, ensure it has executable permission first:

```bash
chmod +x javalens-manager_0.13.1_amd64.AppImage   # or _aarch64 on ARM
./javalens-manager_0.13.1_amd64.AppImage
```

### System tray on Linux

`javalens-manager` exposes a system-tray icon for show / start / stop / quit, with per-workspace status icons from v0.12.0 onward. The tray relies on a **StatusNotifierItem / AppIndicator** host being available in your desktop environment:

- **Pop!_OS, Ubuntu (22.04+), KDE Plasma, XFCE, Cinnamon, MATE** — works out of the box, nothing to install.
- **Vanilla GNOME (Fedora Workstation, Debian GNOME)** — install `gnome-shell-extension-appindicator` once. On Fedora: `sudo dnf install gnome-shell-extension-appindicator && gnome-extensions enable appindicatorsupport@rgcjonas.gmail.com`. On Debian: `sudo apt install gnome-shell-extension-appindicator`. Log out and back in after enabling.

If the extension isn't installed, the manager itself still runs — you just won't see the tray icon. Disable the tray entirely from Settings → *Use system tray* if you'd rather it not try.

## What It Is

`javalens-manager` is a desktop application for:

- registering Java projects
- starting and stopping `javalens-mcp` instances
- managing per-project runtime state and workspaces
- showing health and status for running servers
- helping MCP clients connect to the right `javalens` instance
- reducing friction when using JavaLens across more than one project

## What It Is Not

`javalens-manager` is not:

- a fork of `javalens-mcp`
- a replacement for `javalens-mcp`
- an Eclipse plugin
- a Java code analysis engine by itself

Java semantic analysis and refactoring remain the responsibility of upstream `javalens-mcp`.

## Why This Exists

`javalens-mcp` is a strong semantic Java MCP server built on Eclipse JDT, but it is centered on a single running server/session and project-loading workflow.

This project exists to provide a higher-level desktop experience for people who want to use JavaLens across multiple Java projects without manually managing processes, workspaces, configuration, and client setup.

## Goals

- Provide a modern desktop UI for JavaLens operations
- Keep upstream `javalens-mcp` completely unchanged
- Support multiple Java projects through managed JavaLens instances
- Make project registration and runtime management simple
- Improve day-to-day usability for MCP-based Java workflows
- Work well with Cursor, VS Code, Claude Code, and other MCP-capable clients

## Non-Goals

- Reimplement Java semantic analysis
- Add custom Java refactoring logic outside JavaLens
- Replace IDE-native Java tooling
- Bundle proprietary project-specific integrations into this repository

## Shipped today

- Named workspaces of multiple Java projects (one shared MCP service per workspace).
- Live workspace updates via `workspace.json` file-watcher (no MCP-client restart needed).
- Workspace-first dashboard with multi-select bulk move + drag-drop between workspaces.
- Per-workspace and global start / stop / restart, with health and status display.
- Auto-download and auto-update of the JavaLens runtime from a configurable release source (fork by default).
- One-click deploy of MCP entries into Cursor / Claude Desktop / Antigravity / IntelliJ-style configs, with safe-merge or replace-managed-section semantics, optional pre-write backups, and dry-run mode.
- System-tray icon with per-workspace toggle entries (monochrome status bullets since v0.13.0 — `●` running, `◐` starting, `○` stopped, `✗` failed) and Start all / Stop all peers.
- Diagnostics, log cleanup, JDT-workspace cleanup, "start from scratch" reset.

## Planned

- macOS and Windows packaging in CI.
- Broader QA, cross-platform testing, and edge-case hardening.
- Auto-update UX after the .AppImage download (currently the user replaces the binary by hand).

## Architecture Direction

`javalens-manager` is planned as a Tauri desktop application with:

- a Rust backend for process/runtime orchestration
- a desktop UI for project and server management
- integration with upstream `javalens-mcp` as an external dependency
- clear separation between UI, configuration, process management, and MCP client setup

## Relationship to JavaLens

This project is built around upstream [`javalens-mcp`](https://github.com/pzalutski-pixel/javalens-mcp).

Important design rule:

- `javalens-manager` manages `javalens-mcp`
- `javalens-manager` does not modify `javalens-mcp`

If you need semantic Java analysis, navigation, refactoring, or diagnostics, those capabilities come from JavaLens itself.

## Roadmap

### Current focus
- Cross-platform packaging (macOS, Windows) in CI.
- v1.6.1 fork release: Tycho-test fixture-build pipeline so the disabled `run_tests` happy-path tests run; cross-bundle `compile_workspace` integration test.

### Completed (manager-side)
- **Sprint 13 (v0.13.0):** Tray menu refined for the GNOME / AppIndicator reality (monochrome unicode bullets, `Open dashboard` opens the main window, single-second polling with cache-keyed change detection — no flicker, name-after-rename bug fixed). Paired with fork v1.7.0 (73 tools per service: Ring 2 codegen, Ring 3 Maven dep management, Ring 4 formatter / workflow polish).
- **Sprint 12 (v0.12.0):** Tray menu lifecycle controls — per-workspace toggle entries, Start all / Stop all, 5-second background refresh.
- **Sprint 11 (v0.11.0–v0.11.1):** Cutover for fork v1.5.0–v1.5.2 (Tycho-aware Maven, workspace bundle pool for `Require-Bundle`, Gradle Tooling API, parametric tool consolidation, JDT-LTK structural refactorings).
- **Sprint 10 (v0.10.4–v0.10.6):** Named workspaces, multi-select bulk move + drag-drop, workspace-first dashboard, `workspace.json` file-watcher.
- **Sprint 9 (v0.10.0–v0.10.3):** Configurable release source, fork-default runtime, source-resolution fix shipped via fork v1.2.1.
- **Sprint 8:** Packaging and distribution (automated GitHub Releases, Linux `.deb` and `.AppImage` installers).
- **Sprint 7 (v0.9.x):** Tauri shell, system-tray scaffolding, initial deploy-to-agents flow.
- **Sprint 6:** Radical simplification and global configuration.
- **Sprint 5:** Paths, workspace overrides, and Dashboard vs Settings UX.
- **Sprint 2:** Managed-runtime upgrades and team split.
- **Sprint 1:** First runnable slice — project registry, start/stop controls, health display.

## Tech Stack

Planned stack:

- Tauri
- Rust
- desktop frontend UI
- upstream `javalens-mcp`

## Development

Project setup instructions now start in [`docs/tauri-bootstrap.md`](docs/tauri-bootstrap.md).

Expected local prerequisites will include:

- Rust toolchain
- Tauri prerequisites
- Java 21+
- access to `javalens-mcp`

## License

MIT

This project is intended to stay permissively licensed and compatible with upstream `javalens-mcp`.
