# javalens-manager — Bug Tracker

Living log of issues found during real-world usage. Mirrors the fork's [`docs/bugs.md`](https://github.com/hw1964/javalens-mcp/blob/master/docs/bugs.md) format so the two repos read the same.

Append new bugs at the **top**. Status values: `OPEN`, `IN_PROGRESS`, `FIXED in vX.Y.Z`, `WONTFIX`, `DUPLICATE`.

For each entry include: ID, date observed, severity, reproducer, expected vs actual, environment, and (when known) suspected root cause.

---

## #3 — Launching the app while already running spawns a second instance (and a second tray icon)

- **Status:** FIXED in v0.14.0 (Sprint 14, commit `62cc728`) — `tauri-plugin-single-instance` v2.4.2 chained FIRST in the Tauri Builder; callback does `unminimize → show → set_focus` on the running process's `main` window. Duplicate process exits before any expensive setup.
- **Date observed:** 2026-05-11
- **Reporter:** Harald
- **Server version:** all versions through manager v0.13.1.
- **Severity:** MEDIUM — UX confusion + risk of two manager processes racing on the same `~/.config/javalens-manager/projects.json` and the same workspace JVMs. Two tray icons appearing for one app is the visible symptom; the underlying problem is the absence of single-instance enforcement.

### Reproducer

1. Launch `javalens-manager` — it appears in the system tray.
2. Close the window (or send it to the tray) so only the tray icon remains.
3. Click the `javalens-manager` entry in the GNOME app menu (or run `~/.local/bin/javalens-manager` from a shell) a second time.
4. Observe: a **new** manager window opens and a **second** tray icon appears alongside the original.

### Expected

The single-instance pattern most desktop apps follow: a second launch either (a) raises the existing window if it's hidden / minimised, OR (b) is a no-op when the existing window is already visible. The tray icon must remain a single instance regardless. Same pattern as Slack / Spotify / VS Code / virtually every Tauri-bundled app with system-tray integration.

### Actual

Each invocation creates a separate JVM-monitoring process with its own webview window, its own tray icon, its own poller threads watching the same workspace JVMs. The two manager processes are not aware of each other.

### Suspected root cause

`src-tauri/src/lib.rs` does not register `tauri-plugin-single-instance`. Without it, Tauri's `tauri::Builder` happily spawns a fresh app per process. The plugin is the standard Tauri-ecosystem solution: a platform-specific IPC mechanism (Unix domain socket on Linux/macOS, named pipe on Windows) detects when a second invocation occurs and either exits the new instance after firing a hook on the original, or hands the new instance's CLI args off to the original.

### Suggested fix

1. Add `tauri-plugin-single-instance` to `src-tauri/Cargo.toml` and register it as the **first** plugin in `tauri::Builder::default().plugin(...)` chain — earlier the better so the second-instance exit happens before any expensive setup.
2. In the single-instance hook (passed to the plugin's setup): get the `main` webview window via `app.get_webview_window("main")`, call `.unminimize()`, `.show()`, `.set_focus()`. If the window is already visible, the calls are no-ops.
3. Verify on both X11 and Wayland: GNOME may behave differently for `set_focus()` — the standard pattern is to also call `.request_user_attention(Some(UserAttentionType::Informational))` as a fallback so the taskbar entry pulses if focus-stealing is blocked.
4. Side effects to confirm:
   - Tray icon stays singular (it's registered only once on the original process).
   - The release-poller and the periodic `refresh_tray_menu` thread don't double up.
   - `commands::quit_app` from either window terminates the *one* process cleanly (the second-instance path no longer exists by construction).

### Why this matters beyond the visible duplicate-icon symptom

Two processes both writing to `~/.config/javalens-manager/projects.json` is an undefined-behaviour race. Likely outcomes: lost writes, corrupted JSON, the lower-numbered process's snapshot winning on a Last-Writer-Wins basis. So far apparently not observed in the wild — but the lack of single-instance enforcement is the door for that class of bug to walk through whenever a user double-launches and then both windows mutate state independently. Fixing this closes the door before someone reports a real data-loss incident.

### Cross-reference

- Tauri's official guidance on system-tray apps: enable `tauri-plugin-single-instance` from the project's first day with a tray. Sprint 7 introduced the tray, Sprint 12/13 refined it; the plugin was never added. Pre-existing oversight, not a regression.

---

## #2 — Process-death flips workspace to `Stopped` instead of `Failed` (tray glyph stays gray, not red)

- **Status:** FIXED in v0.14.0 (Sprint 14, commit `ef8b4c8`) — `RuntimeStatusRecord` gained an additive `exit_code: Option<i32>` field; `try_join_running_workspace*` now persists `RuntimePhase::Failed` on detected external death instead of `Stopped`. `RuntimePhase` stays a 4-variant unit enum (preserves frontend `status.phase === "failed"` wire format). All stop paths hold the handles mutex while they `handles.remove() + kill() + wait()` synchronously, so reaching the `try_wait → Some` branch IS the external-death case.
- **Date observed:** Sprint 12 (~2026-04-23); re-flagged 2026-05-11.
- **Reporter:** Harald, via the agent-feedback / Sprint-13 cut-line trail.
- **Server version:** manager v0.13.1 (all v0.12.x and v0.13.x affected).
- **Severity:** LOW-MEDIUM — no data loss, no functional regression, but users can't tell at a glance whether a workspace was *intentionally stopped* or *externally killed*. Defeats half the tray-status promise from Sprint 12.

### Reproducer

1. Manager running with at least one workspace at status `running` (tray glyph `●`).
2. From a shell, `kill -9 <PID>` of the javalens.jar process for that workspace.
3. Wait 5 seconds for the 1s-poll cache-keyed change-detection to notice.
4. Observe the tray glyph and the dashboard runtime status.

### Expected

Per the Sprint 12 design and the v0.13.0 popover-plan Verification, the workspace transitions to `RuntimePhase::Failed`. Tray glyph flips to `✗` (red on the popover, glyph `✗` on the native menu). Dashboard shows the workspace card as "Failed" so the user can decide to restart or investigate.

### Actual

The polling sees the PID has exited and transitions the workspace to `RuntimePhase::Stopped`. Tray glyph flips to `○` (gray). Indistinguishable from a user-initiated *Stop*.

### Suspected root cause

The supervisor in `runtime_manager.rs` (or wherever the per-workspace poll lives) doesn't track *how* a workspace transitioned out of `running`. The state machine has `Starting → Running → Stopping → Stopped`, with no branch for unexpected exits. When polling notices the process is gone, it falls through to `Stopped` because that's the only terminal state defined.

### Suggested fix

1. Track an *expected-stopping* flag set by `commands::stop_runtime` and `manager_service::toggle_workspace` when the user (or peer-stop-all) initiates a clean shutdown.
2. On poll-detected exit:
   - If the expected-stopping flag was set within the last N seconds → transition to `Stopped` (current behaviour).
   - Otherwise → transition to `Failed` with the exit code (or "no exit code captured" if the child wait() lost the race with the kill).
3. Phase machine: extend with `Failed { exit_code: Option<i32>, killed_at: SystemTime }`. The 1s-poll cache-keyed comparator already treats phase changes as the trigger, so the tray glyph flips correctly once `Failed` is plumbed in.
4. UI side: `phase_glyph` in `src-tauri/src/lib.rs` already maps `Failed → "✗"`. The popover CSS already has `.dot--failed { background: #EF4444 }`. No frontend work needed.

### Why this got cut from Sprint 13

Sprint 13 was scoped to the tray-menu refinement against AppIndicator/GNOME constraints. Process-death detection is a state-machine concern, not a tray-rendering concern. The plan's Verification §5 said the cut was acceptable because users can still see the workspace name went non-green; calling it `Failed` vs `Stopped` is information-rich UX, not blocking-functional.

### Cross-reference

- Sprint 12 backlog originally promised this; deferred to v0.12.x patch and never landed.
- Sprint 13 popover plan acknowledged it as a separate concern.
- Fork's `docs/upgrade-checklist.md` Sprint 14 (v1.8.x) backlog section now references it (commit `91cbf5c` in the fork).

---

## #1 — Tauri webview renders blank on aarch64 / NVIDIA Grace (and some x86_64 GPU stacks)

- **Status:** FIXED in v0.14.0 (Sprint 14, commit `ddd20d4`) — CI post-bundle step bakes `WEBKIT_DISABLE_DMABUF_RENDERER=1` into the AppImage's `AppRun` AND a `/usr/bin/javalens-manager` wrapper inside the `.deb` (binary moved to `/usr/lib/javalens-manager/`). Every install path is covered (curl one-liner, `dpkg -i`, direct AppImage double-click). install.sh simplified accordingly — wrapper-write block removed; AppImage installed directly to `$BIN_DIR/$APP_NAME`.
- **Date observed:** 2026-05-11
- **Reporter:** Harald, on Gigabyte AI Top Atom (NVIDIA DGX Spark / GB10).
- **Server version:** manager v0.13.1 (`.deb` and `.AppImage` aarch64 builds from the v0.13.1 ARM CI matrix).
- **Severity:** HIGH on affected hosts — app is unusable (no UI). Affects 100% of aarch64 + NVIDIA Grace launches and an unknown fraction of x86_64 + NVIDIA-proprietary setups. Unaffected hosts (most Intel/AMD desktops) see no symptom.

### Environment

- Host: Gigabyte AI Top Atom / NVIDIA Project DIGITS / DGX Spark ("GB10").
- CPU: NVIDIA Grace (ARM Neoverse V2, aarch64).
- OS: NVIDIA's Ubuntu 24.04 fork for Spark DGX.
- Stack: Tauri 2.10.x + WebKitGTK 4.1 (≥ 2.42).

### Reproducer

1. `curl -sSL https://raw.githubusercontent.com/hw1964/javalens-manager/main/install.sh | bash` (or `sudo dpkg -i javalens-manager_0.13.1_arm64.deb`).
2. Launch `javalens-manager` from the app menu (or `~/.local/bin/javalens-manager`).

### Expected

Manager window opens showing Workspaces / Dashboard / Settings tabs and the registered projects.

### Actual

GTK window opens with the title bar and window controls (minimize / maximize / close). **The webview content area is a solid flat colour (GTK theme background).** Svelte UI never paints. No error on stderr beyond generic WebKit init log lines. The app doesn't crash — the JS just never gets composited.

Screenshot: blank dark-grey rectangle with only the `javalens-manager` title bar visible.

### Suspected root cause

WebKitGTK ≥ 2.42 added a DMABUF-based GPU compositor path. On NVIDIA Grace + Blackwell (and some x86_64 NVIDIA-proprietary stacks) the DMABUF init silently fails, the WebKit-managed surface is never composited, and we get a blank webview. Window chrome is fine because that's GTK's job. Same root cause as the upstream Tauri issue cluster (e.g. tauri-apps/tauri#9304 family).

### Fix

Set `WEBKIT_DISABLE_DMABUF_RENDERER=1` in the environment before launching the binary. Disables the DMABUF compositor and falls back to the previous WebKitGTK rendering path. Harmless on systems that don't need it.

**Shipped in install.sh as of commit `7f1f7b7`** — the script now writes a wrapper at `~/.local/bin/javalens-manager` that exports the env-var and `exec`s the real AppImage at `~/.local/bin/javalens-manager.AppImage`. Both shell launches and GNOME app-menu launches go through the wrapper. Existing users re-running `curl ... | bash` pick up the wrapper automatically.

### What's still missing (v0.13.2 follow-up)

`.deb`-installed users (without `install.sh`) still see the blank webview. Two ways to fix:

1. **`.deb` postinst hook** that writes the same wrapper into `/usr/bin/` (or wherever the .deb installs the launcher), pointing at the AppImage location. Or modifies the bundled `.desktop` to `Exec=env WEBKIT_DISABLE_DMABUF_RENDERER=1 …`.
2. **AppImage entry-point** — patch the AppImage's `AppRun` to `export WEBKIT_DISABLE_DMABUF_RENDERER=1` before invoking the bundled binary. That fixes both `.deb` and AppImage paths uniformly.

Option 2 is preferable (single fix, no per-package divergence). Tauri-bundler doesn't directly expose AppRun customisation, so the v0.13.2 implementation probably has to post-process the bundled AppImage during CI to inject the env-var.

### Cross-reference

- Memory: [`webkit_dmabuf_blank_webview.md`](/home/harald/.claude/projects/-home-harald-CursorProjects-javalens-manager/memory/webkit_dmabuf_blank_webview.md) for the diagnostic-ladder pattern reusable across any Tauri-on-Linux app.
- Same env-var has been recommended by Tauri's maintainers for several years; we're late to bake it in.
