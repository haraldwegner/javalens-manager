# Tauri Bootstrap Guide

## Goal

Bootstrap `javalens-manager` as a Tauri desktop application with:

- Rust for runtime/process orchestration
- Svelte for the desktop UI
- one first vertical slice: project registry + one managed runtime + status view

This guide is for Sprint 0 and Sprint 1. It is intentionally narrow so the repo starts from a stable baseline instead of over-scaffolding.

For the exact JavaLens runtime contract and the first task breakdown, also use:

- [`docs/javalens-runtime-contract.md`](javalens-runtime-contract.md)
- [`docs/sprints/sprint-1-backlog.md`](sprints/sprint-1-backlog.md)
- [`docs/sprints/sprint-2-backlog.md`](sprints/sprint-2-backlog.md)

## Prerequisites

Minimum local prerequisites:

- Rust toolchain via `rustup`
- Node.js LTS and an npm-compatible package manager
- Java 21+
- Tauri system prerequisites for your platform

Linux note: install the Tauri system packages from the official Tauri prerequisites documentation before scaffolding. Do that first so the generated app can build immediately.

Recommended checks:

```bash
rustc --version
cargo --version
node --version
npm --version
java --version
```

## Bootstrap Decision

The initial bootstrap should use:

- Tauri
- Svelte
- TypeScript

Keep the scaffold modest. The goal is not to finish the UI framework on day one. The goal is to make the first manager slice runnable.

## Initial Repository Shape

After scaffolding, the repository should contain the normal Tauri runtime boundary plus the existing docs:

```text
javalens-manager/
  docs/
  src-tauri/
    src/
      main.rs
      commands.rs
      config.rs
      runtime_manager.rs
  src/
    App.svelte
    lib/
      api/
      stores/
      components/
  package.json
  tauri.conf.json
  README.md
```

This keeps the early structure close to standard Tauri conventions while leaving room to split modules later.

## Recommended Scaffold Flow

From the repository root, scaffold the app in place so the docs stay where they are:

```bash
npm create tauri-app@latest .
```

Recommended answers:

- template: `Svelte`
- language: `TypeScript`
- package manager: your local default
- app name: `javalens-manager`
- window title: `javalens-manager`

If the generator refuses to scaffold into a non-empty directory, create the app in a temporary folder, then move the generated `src-tauri/`, frontend files, and package metadata into this repository in a single clean pass.

## Sprint 0 Output

Sprint 0 should finish with:

- ADRs accepted
- JavaLens management contract documented
- Tauri prerequisites confirmed locally
- frontend choice locked
- first scaffold target agreed

Do not add deep product features in Sprint 0. The only goal is to remove setup ambiguity.

## Sprint 1 Thin Vertical Slice

The first working slice should prove the full manager loop for one project.

### Backend

Implement only these Rust concerns first:

- config model for one registered project
- project registry persistence
- runtime launch/stop for one `javalens-mcp` process
- runtime status reporting
- basic log capture path wiring

### Frontend

Implement only these Svelte screens or panels first:

- project list with one registered entry
- start / stop buttons
- current runtime status
- path/port summary for the selected project

### End-to-end outcome

The slice is done when a user can:

1. register one Java project
2. configure the `javalens-mcp` artifact path
3. start the runtime
4. see whether it is running
5. stop it again cleanly

## Suggested Tauri Command Boundary

Keep the frontend thin. Start with Tauri commands for:

- `list_projects`
- `add_project`
- `start_runtime`
- `stop_runtime`
- `get_runtime_status`

The UI should call commands and render state. Process orchestration logic should stay in Rust modules, not in the Svelte layer.

## Early Non-Goals

Do not add these in the first scaffold or first slice:

- system tray behavior
- bulk start/stop for many projects
- automatic artifact download
- complex preferences pages
- generated client config for many editor types
- polished logs UI beyond simple status plumbing

## Definition of Ready for Broader Work

Move beyond the first slice only after:

- one-project lifecycle works reliably
- runtime state is persisted clearly
- logs and failures are visible enough to debug
- the module boundaries still match `docs/architecture.md`
