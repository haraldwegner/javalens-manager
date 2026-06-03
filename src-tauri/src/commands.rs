use crate::{
    config::{AddProjectInput, ProjectRecord, UpdateSettingsInput},
    manager_service::{
        CleanupSummary, DeployToAgentsInput, DeployToAgentsResult, ManagerDashboard,
        RenameProjectInput, RenameWorkspaceInput, ServiceProbeResult, ServicesInventory,
        SetProjectWorkspaceInput, WorkspaceImportInput, WorkspaceImportResult,
        WorkspaceProjectCandidate,
    },
    runtime_manager::RuntimeStatusRecord,
    AppState,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuitPromptContext {
    pub running_services: usize,
    pub tray_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum QuitAction {
    Cancel,
    HideToTray,
    StopAndQuit,
    Quit,
}

#[tauri::command]
pub fn get_dashboard(state: State<'_, AppState>) -> Result<ManagerDashboard, String> {
    state.manager_service.load_dashboard()
}

#[tauri::command]
pub fn add_project(
    state: State<'_, AppState>,
    input: AddProjectInput,
) -> Result<ProjectRecord, String> {
    state.manager_service.add_project(input)
}

#[tauri::command]
pub fn set_project_workspace(
    state: State<'_, AppState>,
    input: SetProjectWorkspaceInput,
) -> Result<ManagerDashboard, String> {
    state.manager_service.set_project_workspace(input)
}

#[tauri::command]
pub fn rename_workspace(
    state: State<'_, AppState>,
    input: RenameWorkspaceInput,
) -> Result<ManagerDashboard, String> {
    state.manager_service.rename_workspace(input)
}

#[tauri::command]
pub fn rename_project(
    state: State<'_, AppState>,
    input: RenameProjectInput,
) -> Result<ManagerDashboard, String> {
    state.manager_service.rename_project(input)
}

#[tauri::command]
pub fn delete_workspace(
    state: State<'_, AppState>,
    workspace_name: String,
) -> Result<ManagerDashboard, String> {
    state.manager_service.delete_workspace(&workspace_name)
}

#[tauri::command]
pub fn delete_project(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<ManagerDashboard, String> {
    state.manager_service.delete_project(&project_id)
}

#[tauri::command]
pub fn start_all_runtimes(state: State<'_, AppState>) -> Result<ManagerDashboard, String> {
    state.manager_service.start_all_runtimes()
}

#[tauri::command]
pub fn stop_all_runtimes(state: State<'_, AppState>) -> Result<ManagerDashboard, String> {
    state.manager_service.stop_all_runtimes()
}

#[tauri::command]
pub fn reload_all_runtimes(state: State<'_, AppState>) -> Result<ManagerDashboard, String> {
    state.manager_service.reload_all_runtimes()
}

#[tauri::command]
pub fn delete_all_projects(state: State<'_, AppState>) -> Result<ManagerDashboard, String> {
    state.manager_service.delete_all_projects()
}

#[tauri::command]
pub fn discover_workspace_projects(
    state: State<'_, AppState>,
    workspace_file: String,
) -> Result<Vec<WorkspaceProjectCandidate>, String> {
    state
        .manager_service
        .discover_workspace_projects(&workspace_file)
}

#[tauri::command]
pub fn import_workspace_projects(
    state: State<'_, AppState>,
    input: WorkspaceImportInput,
) -> Result<WorkspaceImportResult, String> {
    state.manager_service.import_workspace_projects(input)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    input: UpdateSettingsInput,
) -> Result<ManagerDashboard, String> {
    state.manager_service.update_settings(input)
}

#[tauri::command]
pub fn redetect_mcp_client_paths(state: State<'_, AppState>) -> Result<ManagerDashboard, String> {
    state.manager_service.redetect_mcp_client_paths()
}

#[tauri::command]
pub fn download_or_update_javalens(state: State<'_, AppState>) -> Result<ManagerDashboard, String> {
    state.manager_service.download_or_update_javalens()
}

#[tauri::command]
pub fn start_runtime(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<RuntimeStatusRecord, String> {
    state.manager_service.start_runtime(&project_id)
}

#[tauri::command]
pub fn stop_runtime(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<RuntimeStatusRecord, String> {
    state.manager_service.stop_runtime(&project_id)
}

#[tauri::command]
pub fn get_runtime_status(
    state: State<'_, AppState>,
    project_id: String,
) -> Result<RuntimeStatusRecord, String> {
    state.manager_service.get_runtime_status(&project_id)
}

#[tauri::command]
pub fn get_services_inventory(state: State<'_, AppState>) -> Result<ServicesInventory, String> {
    Ok(state.manager_service.get_services_inventory())
}

#[tauri::command]
pub fn clean_logs(state: State<'_, AppState>) -> Result<CleanupSummary, String> {
    state.manager_service.clean_logs()
}

#[tauri::command]
pub fn clean_workspaces(state: State<'_, AppState>) -> Result<CleanupSummary, String> {
    state.manager_service.clean_workspaces()
}

#[tauri::command]
pub fn clean_generated_data(state: State<'_, AppState>) -> Result<CleanupSummary, String> {
    state.manager_service.clean_generated_data()
}

#[tauri::command]
pub fn probe_services(state: State<'_, AppState>) -> Result<ServiceProbeResult, String> {
    state.manager_service.probe_services()
}

#[tauri::command]
pub fn deploy_to_agents(
    state: State<'_, AppState>,
    input: DeployToAgentsInput,
) -> Result<DeployToAgentsResult, String> {
    state.manager_service.deploy_to_agents(input)
}

#[tauri::command]
pub fn get_quit_prompt_context(state: State<'_, AppState>) -> Result<QuitPromptContext, String> {
    Ok(QuitPromptContext {
        running_services: state.manager_service.running_services_count(),
        tray_enabled: state.manager_service.is_system_tray_enabled(),
    })
}

#[tauri::command]
pub fn perform_quit_action(
    app: AppHandle,
    state: State<'_, AppState>,
    action: QuitAction,
) -> Result<(), String> {
    match action {
        QuitAction::Cancel => Ok(()),
        QuitAction::HideToTray => {
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| "Main window not found".to_string())?;
            window.hide().map_err(|error| error.to_string())?;
            Ok(())
        }
        QuitAction::StopAndQuit => {
            if state.manager_service.has_running_services() {
                state.manager_service.stop_all_runtimes()?;
            }
            app.exit(0);
            Ok(())
        }
        QuitAction::Quit => {
            app.exit(0);
            Ok(())
        }
    }
}
