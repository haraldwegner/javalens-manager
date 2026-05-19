# Sprint 6.3 Backlog (Settings Reframe)

## Goal

Reframe Settings so it clearly separates:

1. JavaLens runtime lifecycle (upstream vs installed on this machine)
2. machine-specific runtime controls
3. diagnostics and reset operations
4. system tray behavior
5. MCP client config location management

This sprint is Settings-focused and documentation-first.

## Scope Boundaries

In scope for Sprint 6.3:
- Settings information architecture and UX requirements
- destructive reset safety rules
- machine/runtime configuration controls
- MCP config **location and policy metadata**

Out of scope for Sprint 6.3:
- executing MCP deploy/write workflows from Settings
- client-side MCP reload/restart orchestration

## Stories

### 1) JavaLens Runtime (Release + Installed Runtime Block)

Acceptance criteria:
- Settings has a dedicated "JavaLens Runtime" section.
- The section differentiates:
  - latest upstream release info
  - installed runtime info on this machine
  - update/download actions
- Runtime actions and status are grouped in one block, not scattered.
- Service inventory is surfaced when available:
  - show an "Exposed services" list if retrievable
  - otherwise show a clear "service inventory unavailable" state

### 2) Machine Runtime Controls (Ports)

Acceptance criteria:
- Settings has a dedicated machine-level networking block.
- Port range remains configurable with explicit conflict rationale.
- UX copy explains:
  - this range is manager-owned
  - collisions with other local services are possible
  - validation is enforced before save

### 3) Diagnostics & Start-From-Scratch Reset

Acceptance criteria:
- Settings shows diagnostic paths for manager-owned config/state/cache roots.
- A destructive "Start from scratch" reset action is available in Settings.
- Reset scope is explicit (what is deleted vs preserved).
- Reset workflow includes:
  - blocking check when managed runtimes are active
  - mandatory confirmation dialog
  - success/partial-failure summary with actionable error text

### 4) System Tray Flag

Acceptance criteria:
- Settings includes a dedicated toggle for system tray usage.
- UX copy explains runtime behavior implications (background lifecycle/visibility).
- Toggle state is persisted in manager settings.

### 5) MCP Config Locations (Metadata Only)

Acceptance criteria:
- Settings includes per-client path settings for:
  - Cursor
  - Claude
  - Antigravity
  - IntelliJ
- Each client path supports:
  - auto-detected default
  - manual override path
- Settings includes write policy metadata:
  - safe merge mode
  - backup-before-overwrite mode
- Settings does **not** perform MCP deploy execution directly; it only stores defaults/overrides and policies.

### 6) Layout / Real Estate

Acceptance criteria:
- Settings layout is sectioned (cards or accordion groups) to avoid crowded controls.
- Runtime, machine, diagnostics, tray, and MCP config-location sections have clear visual separation.
- Space is reserved for future service inventory details.

## Safety and UX Rules

Acceptance criteria:
- Destructive actions are disabled while conflicting operations are running.
- "Start from scratch" cannot proceed while managed runtimes are active unless explicitly stopped first.
- Confirmation prompts describe exact delete scope.
- Partial failures are reported per path/item (not hidden behind a single generic error).

## Sprint 7 Coordination

- Sprint 6.3 owns Settings metadata and safety UX for MCP config locations/policies.
- Sprint 7 owns MCP config generation/deploy execution lifecycle and rule generation.
- Sprint 6.3 should reference Sprint 7 to avoid duplicate ownership of deploy execution behavior.
