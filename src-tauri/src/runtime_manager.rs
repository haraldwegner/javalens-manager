use crate::config::{display_path, AppPaths};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    path::Path,
    process::{Child, Command, Stdio},
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimePhase {
    Stopped,
    Starting,
    Running,
    Failed,
}

/// Status record for a project's runtime. Sprint 10 v0.10.4: multiple
/// projects sharing a `workspace_name` reflect the same underlying javalens
/// process — same PID, same workspace_dir, same log file. They differ only
/// in `project_id`. The frontend continues to read these per-project for
/// rendering, but the underlying process is shared.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatusRecord {
    pub project_id: String,
    pub phase: RuntimePhase,
    /// Sprint 10 v0.10.4: the logical workspace this project belongs to.
    /// All projects sharing this name run as one MCP service.
    pub workspace_name: String,
    pub transport: String,
    pub pid: Option<u32>,
    pub workspace_dir: String,
    pub log_path: String,
    pub runtime_label: String,
    pub resolved_jar_path: String,
    pub service_mode: String,
    pub detail: String,
    /// Sprint 14 (v0.14.0, bugs.md #2): captured from the dead-process
    /// branch of `try_join_running_workspace*` when the child exited
    /// without the manager initiating the stop. None for healthy or
    /// cleanly-stopped runtimes. `#[serde(default)]` lets older
    /// runtime-state.json files (pre-v0.14.0) deserialize cleanly.
    #[serde(default)]
    pub exit_code: Option<i32>,
}

/// Sprint 12 (v0.12.0): one entry per workspace_name, with a phase
/// aggregated from the workspace's member projects. Used by the system-tray
/// menu to render per-workspace status icons and toggle entries.
///
/// Aggregation rules (applied in order):
///   any project Failed   → Failed
///   else any Starting    → Starting
///   else all Running     → Running
///   else (Stopped/empty) → Stopped
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatusSummary {
    pub workspace_name: String,
    pub phase: RuntimePhase,
    pub project_count: usize,
}

impl RuntimeStatusRecord {
    pub fn unresolved(
        project_id: String,
        workspace_name: String,
        workspace_dir: String,
        runtime_label: String,
        detail: String,
    ) -> Self {
        Self {
            phase: RuntimePhase::Failed,
            workspace_name,
            transport: "stdio".into(),
            pid: None,
            log_path: String::new(),
            resolved_jar_path: String::new(),
            service_mode: "manager-process".into(),
            project_id,
            workspace_dir,
            runtime_label,
            detail,
            exit_code: None,
        }
    }
}

/// Reference to a project's runtime — identifies which workspace process
/// the project belongs to. Built by manager_service from a `ProjectRecord`
/// + the resolved javalens runtime location.
#[derive(Debug, Clone)]
pub struct RuntimeReference {
    pub project_id: String,
    pub workspace_name: String,
    /// Eclipse `-data` directory for the workspace's javalens process.
    /// Lives at `<data_root>/workspaces/<workspace_name>/`. Manager_service
    /// writes `workspace.json` into here before spawn.
    pub workspace_dir: String,
    pub runtime_label: String,
    pub resolved_jar_path: String,
}

/// Launch request for one javalens spawn. Manager_service has already
/// written `<workspace_dir>/workspace.json` with the full project list of
/// the workspace before calling `start_runtime`.
#[derive(Debug, Clone)]
pub struct RuntimeLaunchRequest {
    pub project_path: String,
    pub reference: RuntimeReference,
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    pub log_path: String,
}

/// One javalens process owned by the manager. Sprint 10 v0.10.4: shared
/// across every project whose `workspace_name` matches this entry's key.
struct ManagedRuntime {
    child: Child,
    started_at: Instant,
    log_path: String,
    /// Project IDs whose `workspace_name` made them members of this
    /// process. The process is killed when the last member leaves.
    members: HashSet<String>,
    /// Snapshot of the reference used to start the process — re-applied
    /// to per-project status records when other members query status.
    workspace_dir: String,
    runtime_label: String,
    resolved_jar_path: String,
}

pub struct RuntimeManager {
    paths: AppPaths,
    /// Sprint 10 v0.10.4: keyed by `workspace_name`, not `project_id`.
    handles: Mutex<HashMap<String, ManagedRuntime>>,
    /// Per-project snapshot cache. Multiple snapshots may point at the
    /// same `workspace_name` and reflect the same workspace process.
    snapshots: Mutex<HashMap<String, RuntimeStatusRecord>>,
}

impl RuntimeManager {
    pub fn new(paths: AppPaths) -> Self {
        let snapshots = read_runtime_state(&paths.runtime_state_file).unwrap_or_default();
        Self {
            paths,
            handles: Mutex::new(HashMap::new()),
            snapshots: Mutex::new(snapshots),
        }
    }

    /// Start (or join) the workspace's runtime for `launch_request.reference`.
    /// If the workspace's process is already running, this just adds the
    /// project as a member and returns the workspace's status. Otherwise
    /// spawns javalens. Caller (manager_service) must have written
    /// `workspace.json` into `workspace_dir` before calling.
    pub fn start_runtime(
        &self,
        launch_request: &RuntimeLaunchRequest,
    ) -> Result<RuntimeStatusRecord, String> {
        let spec = self.command_spec_for(launch_request);
        self.start_runtime_with_spec(&launch_request.reference, spec)
    }

    /// Internal entry point that takes the already-built `CommandSpec`.
    /// Public-in-crate so unit tests can spawn a tiny stand-in command
    /// (e.g. `sleep`) instead of `java -jar javalens.jar` to verify the
    /// workspace-grouped membership lifecycle without depending on a real
    /// javalens runtime.
    pub(crate) fn start_runtime_with_spec(
        &self,
        reference: &RuntimeReference,
        spec: CommandSpec,
    ) -> Result<RuntimeStatusRecord, String> {
        // Fast path: workspace already running. Add membership, return
        // workspace's PID as this project's status.
        if let Some(status) = self.try_join_running_workspace(reference)? {
            return Ok(status);
        }

        self.paths.ensure_dirs()?;
        fs::create_dir_all(&reference.workspace_dir).map_err(|error| {
            format!(
                "failed to create workspace dir {}: {error}",
                reference.workspace_dir
            )
        })?;

        let log_path = spec.log_path.clone();
        let log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|error| format!("failed to open {log_path}: {error}"))?;
        let stderr_file = log_file
            .try_clone()
            .map_err(|error| format!("failed to clone log file handle: {error}"))?;

        let mut command = Command::new(&spec.command);
        command.args(&spec.args);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::from(log_file));
        command.stderr(Stdio::from(stderr_file));

        for (key, value) in &spec.env {
            command.env(key, value);
        }

        let child = command.spawn().map_err(|error| {
            format!(
                "failed to launch JavaLens. Confirm Java and the resolved runtime path are valid: {error}"
            )
        })?;

        let pid = child.id();
        let status = RuntimeStatusRecord {
            project_id: reference.project_id.clone(),
            phase: RuntimePhase::Starting,
            workspace_name: reference.workspace_name.clone(),
            transport: "stdio".into(),
            pid: Some(pid),
            workspace_dir: reference.workspace_dir.clone(),
            log_path: log_path.clone(),
            runtime_label: reference.runtime_label.clone(),
            resolved_jar_path: reference.resolved_jar_path.clone(),
            service_mode: "manager-process".into(),
            detail: "Process launched. workspace.json drives in-process project loading.".into(),
            exit_code: None,
        };

        let mut members = HashSet::new();
        members.insert(reference.project_id.clone());
        self.handles.lock().expect("runtime mutex poisoned").insert(
            reference.workspace_name.clone(),
            ManagedRuntime {
                child,
                started_at: Instant::now(),
                log_path,
                members,
                workspace_dir: reference.workspace_dir.clone(),
                runtime_label: reference.runtime_label.clone(),
                resolved_jar_path: reference.resolved_jar_path.clone(),
            },
        );
        self.persist_snapshot(status.clone())?;

        Ok(status)
    }

    /// Test helper: returns the membership snapshot for a workspace, or
    /// None if no process is registered for that workspace. Used by unit
    /// tests to assert join/leave semantics without poking at private
    /// state. `pub(crate)` to keep it out of the public API.
    #[cfg(test)]
    pub(crate) fn workspace_members(&self, workspace_name: &str) -> Option<Vec<String>> {
        let handles = self.handles.lock().expect("runtime mutex poisoned");
        handles
            .get(workspace_name)
            .map(|h| h.members.iter().cloned().collect())
    }

    /// Test helper: returns the PID of the workspace's process, or None.
    #[cfg(test)]
    pub(crate) fn workspace_pid(&self, workspace_name: &str) -> Option<u32> {
        let handles = self.handles.lock().expect("runtime mutex poisoned");
        handles.get(workspace_name).map(|h| h.child.id())
    }

    /// Project leaves the workspace. If the workspace's process has no
    /// remaining members, it is killed. The caller (manager_service) is
    /// responsible for rewriting `workspace.json` so the still-running
    /// javalens (when there are remaining members) drops the leaving
    /// project from its in-memory state via the file watcher.
    pub fn stop_runtime(
        &self,
        reference: &RuntimeReference,
    ) -> Result<RuntimeStatusRecord, String> {
        let mut handles = self.handles.lock().expect("runtime mutex poisoned");

        let mut killed = false;
        if let Some(handle) = handles.get_mut(&reference.workspace_name) {
            handle.members.remove(&reference.project_id);
            if handle.members.is_empty() {
                if let Some(mut handle) = handles.remove(&reference.workspace_name) {
                    handle
                        .child
                        .kill()
                        .map_err(|error| format!("failed to stop JavaLens process: {error}"))?;
                    let _ = handle.child.wait();
                    killed = true;
                }
            }
        }
        drop(handles);

        let detail = if killed {
            "Workspace runtime stopped (last project left).".into()
        } else {
            "Project left the workspace; runtime continues for remaining members.".into()
        };
        let status = RuntimeStatusRecord {
            project_id: reference.project_id.clone(),
            phase: RuntimePhase::Stopped,
            workspace_name: reference.workspace_name.clone(),
            transport: "stdio".into(),
            pid: None,
            workspace_dir: reference.workspace_dir.clone(),
            log_path: self.default_log_path(&reference.workspace_name),
            runtime_label: reference.runtime_label.clone(),
            resolved_jar_path: reference.resolved_jar_path.clone(),
            service_mode: "manager-process".into(),
            detail,
            exit_code: None,
        };

        self.persist_snapshot(status.clone())?;
        Ok(status)
    }

    /// Sprint 10 v0.10.4: stop the entire workspace process unconditionally.
    /// All members' snapshots become Stopped. Used by the "Stop workspace"
    /// button in the grouped Dashboard view.
    pub fn stop_workspace_runtime(&self, workspace_name: &str) -> Result<(), String> {
        let removed = {
            let mut handles = self.handles.lock().expect("runtime mutex poisoned");
            handles.remove(workspace_name)
        };
        if let Some(mut handle) = removed {
            let members = handle.members.clone();
            handle
                .child
                .kill()
                .map_err(|error| format!("failed to stop JavaLens process: {error}"))?;
            let _ = handle.child.wait();

            // Mark every member's snapshot as Stopped.
            for project_id in members {
                let snapshot = {
                    let snapshots = self
                        .snapshots
                        .lock()
                        .expect("runtime snapshot mutex poisoned");
                    snapshots.get(&project_id).cloned()
                };
                if let Some(mut s) = snapshot {
                    s.phase = RuntimePhase::Stopped;
                    s.pid = None;
                    s.detail = "Workspace runtime stopped.".into();
                    self.persist_snapshot(s)?;
                }
            }
        }
        Ok(())
    }

    pub fn get_runtime_status(
        &self,
        reference: &RuntimeReference,
    ) -> Result<RuntimeStatusRecord, String> {
        if let Some(status) = self.try_join_running_workspace_readonly(reference)? {
            return Ok(status);
        }

        Ok(self
            .snapshots
            .lock()
            .expect("runtime snapshot mutex poisoned")
            .get(&reference.project_id)
            .cloned()
            .unwrap_or_else(|| RuntimeStatusRecord {
                project_id: reference.project_id.clone(),
                phase: RuntimePhase::Stopped,
                workspace_name: reference.workspace_name.clone(),
                transport: "stdio".into(),
                pid: None,
                workspace_dir: reference.workspace_dir.clone(),
                log_path: self.default_log_path(&reference.workspace_name),
                runtime_label: reference.runtime_label.clone(),
                resolved_jar_path: reference.resolved_jar_path.clone(),
                service_mode: "manager-process".into(),
                detail: "Runtime has not been started yet.".into(),
                exit_code: None,
            }))
    }

    /// Forcefully forget a project's runtime association. Removes from
    /// snapshots and from any workspace's member set. If the project was
    /// the last member, the workspace process is killed.
    pub fn remove_project_runtime(&self, project_id: &str) -> Result<(), String> {
        // Find which workspace (if any) hosts this project, and leave it.
        let host_workspace = {
            let handles = self.handles.lock().expect("runtime mutex poisoned");
            handles
                .iter()
                .find_map(|(ws, h)| {
                    if h.members.contains(project_id) {
                        Some(ws.clone())
                    } else {
                        None
                    }
                })
        };

        if let Some(ws) = host_workspace {
            let mut handles = self.handles.lock().expect("runtime mutex poisoned");
            if let Some(handle) = handles.get_mut(&ws) {
                handle.members.remove(project_id);
                if handle.members.is_empty() {
                    if let Some(mut handle) = handles.remove(&ws) {
                        let _ = handle.child.kill();
                        let _ = handle.child.wait();
                    }
                }
            }
        }

        let snapshots = {
            let mut snapshots = self
                .snapshots
                .lock()
                .expect("runtime snapshot mutex poisoned");
            snapshots.remove(project_id);
            snapshots.clone()
        };
        write_runtime_state(&self.paths.runtime_state_file, &snapshots)
    }

    pub fn command_spec_for(&self, launch_request: &RuntimeLaunchRequest) -> CommandSpec {
        let log_path = self.default_log_path(&launch_request.reference.workspace_name);

        // Sprint 10 v0.10.4: javalens reads its project list from
        // <workspace_dir>/workspace.json (written by manager_service).
        // No JAVA_PROJECT_PATH env var; that legacy single-project flow is
        // preserved in javalens-mcp v1.4.0 for direct manual launches but
        // the manager always uses the workspace.json contract.
        CommandSpec {
            command: "java".into(),
            args: vec![
                "-jar".into(),
                launch_request.reference.resolved_jar_path.clone(),
                "-data".into(),
                launch_request.reference.workspace_dir.clone(),
            ],
            env: vec![],
            log_path,
        }
    }

    /// If the workspace's process is already running, register the
    /// project as a member and return the workspace's PID as this
    /// project's status. Returns None if no process exists for the
    /// workspace yet (caller should spawn).
    fn try_join_running_workspace(
        &self,
        reference: &RuntimeReference,
    ) -> Result<Option<RuntimeStatusRecord>, String> {
        let mut handles = self.handles.lock().expect("runtime mutex poisoned");
        let Some(handle) = handles.get_mut(&reference.workspace_name) else {
            return Ok(None);
        };

        // Check if the process is still alive.
        if let Some(exit_status) = handle
            .child
            .try_wait()
            .map_err(|error| format!("failed to inspect JavaLens process state: {error}"))?
        {
            // Sprint 14 (v0.14.0, bugs.md #2): the process died without the
            // manager initiating the stop. All stop paths (stop_runtime /
            // stop_workspace_runtime / remove_project_runtime) hold the
            // handles mutex while they `handles.remove()` + `child.kill()`
            // + `child.wait()`, so by the time anyone re-locks the map a
            // manager-initiated stop is visible as handle-not-present.
            // Reaching this branch with the handle still in the map
            // therefore IS the external-death case (kill -9, crash, OOM)
            // — persist Failed with the captured exit code. Pre-v0.14.0
            // wrote Stopped here; the tray glyph stayed gray on external
            // kill instead of going red ✗.
            let exit_code = exit_status.code();
            let detail = if exit_status.success() {
                "Previous workspace runtime exited unexpectedly (status 0); respawning.".into()
            } else {
                format!(
                    "Previous workspace runtime exited unexpectedly with status {exit_status}; respawning."
                )
            };
            handles.remove(&reference.workspace_name);
            drop(handles);

            let failed = RuntimeStatusRecord {
                project_id: reference.project_id.clone(),
                phase: RuntimePhase::Failed,
                workspace_name: reference.workspace_name.clone(),
                transport: "stdio".into(),
                pid: None,
                workspace_dir: reference.workspace_dir.clone(),
                log_path: self.default_log_path(&reference.workspace_name),
                runtime_label: reference.runtime_label.clone(),
                resolved_jar_path: reference.resolved_jar_path.clone(),
                service_mode: "manager-process".into(),
                detail,
                exit_code,
            };
            self.persist_snapshot(failed)?;
            return Ok(None);
        }

        handle.members.insert(reference.project_id.clone());
        let phase = if handle.started_at.elapsed() < Duration::from_secs(2) {
            RuntimePhase::Starting
        } else {
            RuntimePhase::Running
        };
        let status = RuntimeStatusRecord {
            project_id: reference.project_id.clone(),
            phase,
            workspace_name: reference.workspace_name.clone(),
            transport: "stdio".into(),
            pid: Some(handle.child.id()),
            workspace_dir: handle.workspace_dir.clone(),
            log_path: handle.log_path.clone(),
            runtime_label: handle.runtime_label.clone(),
            resolved_jar_path: handle.resolved_jar_path.clone(),
            service_mode: "manager-process".into(),
            detail: "Joined live workspace runtime; tools/list reflects current workspace.json."
                .into(),
            exit_code: None,
        };
        drop(handles);
        self.persist_snapshot(status.clone())?;
        Ok(Some(status))
    }

    /// Same as try_join_running_workspace but does not add the project to
    /// the member set. Used by get_runtime_status, which mustn't have
    /// side effects on membership.
    fn try_join_running_workspace_readonly(
        &self,
        reference: &RuntimeReference,
    ) -> Result<Option<RuntimeStatusRecord>, String> {
        let mut handles = self.handles.lock().expect("runtime mutex poisoned");
        let Some(handle) = handles.get_mut(&reference.workspace_name) else {
            return Ok(None);
        };

        if let Some(exit_status) = handle
            .child
            .try_wait()
            .map_err(|error| format!("failed to inspect JavaLens process state: {error}"))?
        {
            // Sprint 14 (v0.14.0, bugs.md #2): same external-death case as
            // `try_join_running_workspace`. The readonly variant previously
            // just removed the handle and returned None, leaving a stale
            // Running snapshot visible to the next get_runtime_status. Now
            // persist a Failed snapshot here too so the dashboard / tray
            // reflect the death without waiting for an explicit start.
            let exit_code = exit_status.code();
            let detail = if exit_status.success() {
                "Workspace runtime exited unexpectedly (status 0).".into()
            } else {
                format!("Workspace runtime exited unexpectedly with status {exit_status}.")
            };
            let failed = RuntimeStatusRecord {
                project_id: reference.project_id.clone(),
                phase: RuntimePhase::Failed,
                workspace_name: reference.workspace_name.clone(),
                transport: "stdio".into(),
                pid: None,
                workspace_dir: reference.workspace_dir.clone(),
                log_path: self.default_log_path(&reference.workspace_name),
                runtime_label: reference.runtime_label.clone(),
                resolved_jar_path: reference.resolved_jar_path.clone(),
                service_mode: "manager-process".into(),
                detail,
                exit_code,
            };
            handles.remove(&reference.workspace_name);
            drop(handles);
            self.persist_snapshot(failed)?;
            return Ok(None);
        }

        let phase = if handle.started_at.elapsed() < Duration::from_secs(2) {
            RuntimePhase::Starting
        } else {
            RuntimePhase::Running
        };
        let status = RuntimeStatusRecord {
            project_id: reference.project_id.clone(),
            phase,
            workspace_name: reference.workspace_name.clone(),
            transport: "stdio".into(),
            pid: Some(handle.child.id()),
            workspace_dir: handle.workspace_dir.clone(),
            log_path: handle.log_path.clone(),
            runtime_label: handle.runtime_label.clone(),
            resolved_jar_path: handle.resolved_jar_path.clone(),
            service_mode: "manager-process".into(),
            detail: "Live workspace runtime.".into(),
            exit_code: None,
        };
        Ok(Some(status))
    }

    fn persist_snapshot(&self, status: RuntimeStatusRecord) -> Result<(), String> {
        let snapshots = {
            let mut snapshots = self
                .snapshots
                .lock()
                .expect("runtime snapshot mutex poisoned");
            snapshots.insert(status.project_id.clone(), status);
            snapshots.clone()
        };

        write_runtime_state(&self.paths.runtime_state_file, &snapshots)
    }

    fn default_log_path(&self, workspace_name: &str) -> String {
        // Sprint 10 v0.10.4: log path keyed by workspace_name (one log per
        // workspace process), not per project_id.
        display_path(&self.paths.log_dir.join(format!("{workspace_name}.log")))
    }
}

fn read_runtime_state(path: &Path) -> Result<HashMap<String, RuntimeStatusRecord>, String> {
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read runtime state {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("failed to parse runtime state {}: {error}", path.display()))
}

fn write_runtime_state(
    path: &Path,
    snapshots: &HashMap<String, RuntimeStatusRecord>,
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(snapshots).map_err(|error| {
        format!(
            "failed to serialize runtime state {}: {error}",
            path.display()
        )
    })?;
    fs::write(path, format!("{json}\n"))
        .map_err(|error| format!("failed to write runtime state {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppPaths;
    use std::path::PathBuf;

    fn fake_paths() -> AppPaths {
        AppPaths {
            config_dir: PathBuf::from("/tmp/javalens-manager/config"),
            state_dir: PathBuf::from("/tmp/javalens-manager/state"),
            cache_dir: PathBuf::from("/tmp/javalens-manager/cache"),
            projects_file: PathBuf::from("/tmp/javalens-manager/config/projects.json"),
            settings_file: PathBuf::from("/tmp/javalens-manager/config/settings.json"),
            runtime_state_file: PathBuf::from("/tmp/javalens-manager/state/runtime-state.json"),
            default_data_root: PathBuf::from("/tmp/javalens-manager/cache"),
            log_dir: PathBuf::from("/tmp/javalens-manager/state/logs"),
        }
    }

    fn fake_launch_request() -> RuntimeLaunchRequest {
        RuntimeLaunchRequest {
            project_path: "/projects/example-service".into(),
            reference: RuntimeReference {
                project_id: "example-service-1".into(),
                workspace_name: "test-ws".into(),
                workspace_dir: "/cache/javalens/test-ws".into(),
                runtime_label: "Managed JavaLens 1.4.0".into(),
                resolved_jar_path: "/tools/javalens/javalens.jar".into(),
            },
        }
    }

    #[test]
    fn command_spec_uses_workspace_dir_and_no_env_var() {
        let manager = RuntimeManager::new(fake_paths());
        let launch_request = fake_launch_request();

        let spec = manager.command_spec_for(&launch_request);

        assert_eq!(spec.command, "java");
        assert_eq!(
            spec.args,
            vec![
                "-jar",
                "/tools/javalens/javalens.jar",
                "-data",
                "/cache/javalens/test-ws"
            ]
        );
        // Sprint 10 v0.10.4: no JAVA_PROJECT_PATH — workspace.json drives
        // project loading inside javalens.
        assert!(spec.env.is_empty());
        assert!(spec.log_path.ends_with("test-ws.log"));
    }

    #[test]
    fn unresolved_runtime_status_carries_runtime_label() {
        let status = RuntimeStatusRecord::unresolved(
            "project-1".into(),
            "test-ws".into(),
            "/tmp/workspace".into(),
            "Managed JavaLens 1.4.0".into(),
            "Missing runtime".into(),
        );

        assert!(matches!(status.phase, RuntimePhase::Failed));
        assert_eq!(status.workspace_name, "test-ws");
        assert_eq!(status.runtime_label, "Managed JavaLens 1.4.0");
        assert_eq!(status.detail, "Missing runtime");
    }

    // ============================================================
    // Sprint 10 v0.10.4: workspace-grouped spawn lifecycle tests.
    //
    // These spawn real processes (a tiny `sleep` command stand-in for
    // javalens) so the membership / kill / join lifecycle is exercised
    // end-to-end without depending on a real javalens runtime.
    // Skipped on non-Unix platforms because `sleep` isn't on Windows.
    // ============================================================

    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_tempdir(label: &str) -> PathBuf {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "javalens-manager-rmtest-{label}-{}-{}-{}",
            std::process::id(),
            nanos,
            n
        ));
        std::fs::create_dir_all(&dir).expect("failed to create test tempdir");
        std::fs::create_dir_all(dir.join("logs")).unwrap();
        std::fs::create_dir_all(dir.join("ws")).unwrap();
        dir
    }

    fn paths_in(dir: &std::path::Path) -> AppPaths {
        AppPaths {
            config_dir: dir.to_path_buf(),
            state_dir: dir.to_path_buf(),
            cache_dir: dir.to_path_buf(),
            projects_file: dir.join("projects.json"),
            settings_file: dir.join("settings.json"),
            runtime_state_file: dir.join("runtime-state.json"),
            default_data_root: dir.to_path_buf(),
            log_dir: dir.join("logs"),
        }
    }

    /// Build a CommandSpec that runs `sleep 60` — long enough to outlast
    /// the test's lifecycle assertions but quick to clean up via kill.
    fn sleep_spec(workspace_dir: &str, log_path: String) -> CommandSpec {
        CommandSpec {
            command: "sleep".into(),
            args: vec!["60".into()],
            env: vec![],
            log_path,
        }
    }

    fn make_reference(project_id: &str, workspace_name: &str, workspace_dir: &str) -> RuntimeReference {
        RuntimeReference {
            project_id: project_id.into(),
            workspace_name: workspace_name.into(),
            workspace_dir: workspace_dir.into(),
            runtime_label: "test-runtime".into(),
            resolved_jar_path: "/dev/null".into(),
        }
    }

    #[cfg(unix)]
    #[test]
    fn workspace_grouped_spawn_two_projects_share_one_process() {
        // Two projects sharing a workspace_name → start_runtime spawns
        // ONCE for the first project and the second project JOINS the
        // same process (no second spawn). Both members are tracked.
        let dir = unique_tempdir("two-share");
        let paths = paths_in(&dir);
        let manager = RuntimeManager::new(paths);

        let ws_dir = dir.join("ws").join("test").to_string_lossy().to_string();
        let ref_a = make_reference("p-a", "test", &ws_dir);
        let ref_b = make_reference("p-b", "test", &ws_dir);

        let log_a = dir.join("logs").join("test.log").to_string_lossy().to_string();
        let status_a = manager
            .start_runtime_with_spec(&ref_a, sleep_spec(&ws_dir, log_a.clone()))
            .expect("first spawn must succeed");
        let pid_a = status_a.pid.expect("first spawn produces a PID");

        let status_b = manager
            .start_runtime_with_spec(&ref_b, sleep_spec(&ws_dir, log_a.clone()))
            .expect("second start must JOIN the running workspace");
        let pid_b = status_b.pid.expect("joining returns a PID too");

        // Same process for both projects — no second spawn.
        assert_eq!(pid_a, pid_b, "joining must reuse the running PID");

        let members = manager.workspace_members("test").unwrap();
        assert_eq!(members.len(), 2, "both projects in members set");
        assert!(members.contains(&"p-a".to_string()));
        assert!(members.contains(&"p-b".to_string()));

        // Cleanup.
        manager.stop_workspace_runtime("test").unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn workspace_stop_runtime_keeps_process_alive_for_remaining_members() {
        // Two members → stop one → workspace process keeps running for
        // the other; only the leaving project's snapshot is "stopped".
        let dir = unique_tempdir("keeps-alive");
        let paths = paths_in(&dir);
        let manager = RuntimeManager::new(paths);

        let ws_dir = dir.join("ws").join("test").to_string_lossy().to_string();
        let ref_a = make_reference("p-a", "test", &ws_dir);
        let ref_b = make_reference("p-b", "test", &ws_dir);
        let log = dir.join("logs").join("test.log").to_string_lossy().to_string();

        manager.start_runtime_with_spec(&ref_a, sleep_spec(&ws_dir, log.clone())).unwrap();
        manager.start_runtime_with_spec(&ref_b, sleep_spec(&ws_dir, log.clone())).unwrap();
        let pid_before = manager.workspace_pid("test").unwrap();

        // p-a leaves. Workspace must still be alive for p-b.
        manager.stop_runtime(&ref_a).unwrap();
        let members_after = manager.workspace_members("test").unwrap();
        assert_eq!(members_after, vec!["p-b".to_string()]);
        let pid_after = manager.workspace_pid("test").unwrap();
        assert_eq!(pid_before, pid_after, "process must NOT have been killed");

        // Cleanup.
        manager.stop_workspace_runtime("test").unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn workspace_stop_runtime_kills_process_when_last_member_leaves() {
        // Single member → stop_runtime → kills the process and removes
        // the handle (workspace_pid returns None).
        let dir = unique_tempdir("kill-last");
        let paths = paths_in(&dir);
        let manager = RuntimeManager::new(paths);

        let ws_dir = dir.join("ws").join("test").to_string_lossy().to_string();
        let ref_only = make_reference("p-only", "test", &ws_dir);
        let log = dir.join("logs").join("test.log").to_string_lossy().to_string();

        manager.start_runtime_with_spec(&ref_only, sleep_spec(&ws_dir, log)).unwrap();
        assert!(manager.workspace_pid("test").is_some());

        // Last member leaves → process killed.
        manager.stop_runtime(&ref_only).unwrap();
        assert!(
            manager.workspace_pid("test").is_none(),
            "workspace handle removed when last member leaves"
        );
        assert!(
            manager.workspace_members("test").is_none(),
            "no members map for a dead workspace"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn workspace_stop_workspace_runtime_kills_unconditionally() {
        // stop_workspace_runtime is the "Stop workspace" button — it
        // kills regardless of how many members are still attached.
        let dir = unique_tempdir("force-stop-ws");
        let paths = paths_in(&dir);
        let manager = RuntimeManager::new(paths);

        let ws_dir = dir.join("ws").join("test").to_string_lossy().to_string();
        let ref_a = make_reference("p-a", "test", &ws_dir);
        let ref_b = make_reference("p-b", "test", &ws_dir);
        let log = dir.join("logs").join("test.log").to_string_lossy().to_string();

        manager.start_runtime_with_spec(&ref_a, sleep_spec(&ws_dir, log.clone())).unwrap();
        manager.start_runtime_with_spec(&ref_b, sleep_spec(&ws_dir, log)).unwrap();
        assert_eq!(manager.workspace_members("test").unwrap().len(), 2);

        manager.stop_workspace_runtime("test").unwrap();
        assert!(
            manager.workspace_pid("test").is_none(),
            "workspace handle removed by force-stop"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn workspace_remove_project_runtime_decrements_membership() {
        // remove_project_runtime is called by manager_service.delete_project.
        // It scans handles for the project and removes its membership
        // (killing the process iff this was the last member).
        let dir = unique_tempdir("remove-project");
        let paths = paths_in(&dir);
        let manager = RuntimeManager::new(paths);

        let ws_dir = dir.join("ws").join("test").to_string_lossy().to_string();
        let ref_a = make_reference("p-a", "test", &ws_dir);
        let ref_b = make_reference("p-b", "test", &ws_dir);
        let log = dir.join("logs").join("test.log").to_string_lossy().to_string();

        manager.start_runtime_with_spec(&ref_a, sleep_spec(&ws_dir, log.clone())).unwrap();
        manager.start_runtime_with_spec(&ref_b, sleep_spec(&ws_dir, log)).unwrap();

        manager.remove_project_runtime("p-a").unwrap();
        // p-a gone, p-b still a member, process still alive.
        let members = manager.workspace_members("test").unwrap();
        assert_eq!(members, vec!["p-b".to_string()]);
        assert!(manager.workspace_pid("test").is_some());

        // Now remove the last member → process dies.
        manager.remove_project_runtime("p-b").unwrap();
        assert!(manager.workspace_pid("test").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn workspace_two_distinct_workspaces_have_independent_processes() {
        // Two workspace_names → two processes. Stopping one does NOT
        // affect the other.
        let dir = unique_tempdir("two-ws-independent");
        let paths = paths_in(&dir);
        let manager = RuntimeManager::new(paths);

        let ws_a_dir = dir.join("ws").join("a").to_string_lossy().to_string();
        let ws_b_dir = dir.join("ws").join("b").to_string_lossy().to_string();
        let ref_a = make_reference("p-a", "ws-a", &ws_a_dir);
        let ref_b = make_reference("p-b", "ws-b", &ws_b_dir);
        let log_a = dir.join("logs").join("a.log").to_string_lossy().to_string();
        let log_b = dir.join("logs").join("b.log").to_string_lossy().to_string();

        manager.start_runtime_with_spec(&ref_a, sleep_spec(&ws_a_dir, log_a)).unwrap();
        manager.start_runtime_with_spec(&ref_b, sleep_spec(&ws_b_dir, log_b)).unwrap();

        let pid_a = manager.workspace_pid("ws-a").unwrap();
        let pid_b = manager.workspace_pid("ws-b").unwrap();
        assert_ne!(pid_a, pid_b, "distinct workspaces → distinct processes");

        // Stop ws-a only.
        manager.stop_workspace_runtime("ws-a").unwrap();
        assert!(manager.workspace_pid("ws-a").is_none());
        assert_eq!(manager.workspace_pid("ws-b"), Some(pid_b), "ws-b unaffected");

        // Cleanup.
        manager.stop_workspace_runtime("ws-b").unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ============================================================
    // Sprint 14 (v0.14.0, bugs.md #2): process-death → Failed (was Stopped)
    // ============================================================

    /// Spawn a command that exits immediately with code 1. The next
    /// status read must observe Failed with exit_code Some(1) — not
    /// Stopped (the pre-v0.14.0 behavior that this bug fix corrects).
    #[cfg(unix)]
    #[test]
    fn unexpected_exit_transitions_to_failed_with_code() {
        let dir = unique_tempdir("unexpected-exit");
        let paths = paths_in(&dir);
        let manager = RuntimeManager::new(paths);

        let ws_dir = dir.join("ws").join("test").to_string_lossy().to_string();
        let reference = make_reference("p-1", "test", &ws_dir);
        let log = dir
            .join("logs")
            .join("test.log")
            .to_string_lossy()
            .to_string();

        let spec = CommandSpec {
            command: "sh".into(),
            args: vec!["-c".into(), "exit 1".into()],
            env: vec![],
            log_path: log,
        };
        manager
            .start_runtime_with_spec(&reference, spec)
            .expect("start must succeed before the child exits");

        // Give the child time to exit and be reaped by try_wait on the
        // next status read.
        std::thread::sleep(Duration::from_millis(200));

        let status = manager
            .get_runtime_status(&reference)
            .expect("status read must succeed after death");
        assert!(
            matches!(status.phase, RuntimePhase::Failed),
            "phase should be Failed after unexpected exit, was {:?}",
            status.phase
        );
        assert_eq!(
            status.exit_code,
            Some(1),
            "exit_code should reflect the child's exit status"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// User-initiated stop (single-member workspace) → handle removed +
    /// kill + wait, all synchronous under the handles mutex. The
    /// resulting status must be Stopped with no exit_code attribution.
    #[cfg(unix)]
    #[test]
    fn user_stop_transitions_to_stopped() {
        let dir = unique_tempdir("user-stop");
        let paths = paths_in(&dir);
        let manager = RuntimeManager::new(paths);

        let ws_dir = dir.join("ws").join("test").to_string_lossy().to_string();
        let reference = make_reference("p-1", "test", &ws_dir);
        let log = dir
            .join("logs")
            .join("test.log")
            .to_string_lossy()
            .to_string();

        manager
            .start_runtime_with_spec(&reference, sleep_spec(&ws_dir, log))
            .unwrap();
        let status = manager.stop_runtime(&reference).unwrap();

        assert!(
            matches!(status.phase, RuntimePhase::Stopped),
            "user stop should produce Stopped, was {:?}",
            status.phase
        );
        assert_eq!(status.exit_code, None);
        assert!(manager.workspace_pid("test").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
