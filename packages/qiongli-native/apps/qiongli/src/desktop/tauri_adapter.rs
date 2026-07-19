use std::sync::Mutex;

use crate::desktop_api::{AppEvent, AppIntent, AppSnapshotV1, app_event};

use super::*;

struct DesktopAppState {
    service: Mutex<NativeDesktopService>,
    projects: Mutex<ProjectDesktopState>,
}

#[tauri::command]
fn qiongli_snapshot(
    state: tauri::State<'_, DesktopAppState>,
) -> Result<AppSnapshotV1, &'static str> {
    app_snapshot_from_state(&state)
}

#[tauri::command]
fn qiongli_execute(
    intent: AppIntent,
    state: tauri::State<'_, DesktopAppState>,
) -> Result<AppEvent, &'static str> {
    match intent {
        AppIntent::RefreshResearchLibrary => Ok(AppEvent::Snapshot {
            snapshot: app_snapshot_from_state(&state)?,
        }),
        AppIntent::SelectProjectDirectory => {
            let selected = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?
                .folder_picker
                .pick_project_folder();
            let Some(root) = selected else {
                return Ok(AppEvent::Cancelled {
                    code: "project-directory-selection-cancelled",
                });
            };
            let mut projects = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?;
            let (token, root_label) = projects.select_register_root(root)?;
            Ok(AppEvent::ProjectDirectorySelected { token, root_label })
        }
        AppIntent::SelectProjectCreateDestination { suggested_name } => {
            validate_project_dialog_name(&suggested_name)?;
            let selected = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?
                .folder_picker
                .pick_project_create_destination(&suggested_name);
            let Some(root) = selected else {
                return Ok(AppEvent::Cancelled {
                    code: "project-create-destination-selection-cancelled",
                });
            };
            let (token, root_label) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .select_create_root(root)?;
            Ok(AppEvent::ProjectDirectorySelected { token, root_label })
        }
        AppIntent::PreviewProjectCreate {
            directory_token,
            display_name,
            project_kind,
            stage,
        } => {
            let preview = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_create(&directory_token, display_name, project_kind, stage)?;
            Ok(AppEvent::Preview { preview })
        }
        AppIntent::PreviewProjectRegister { directory_token } => {
            let preview = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_register(&directory_token)?;
            Ok(AppEvent::Preview { preview })
        }
        AppIntent::OpenProject { project_id } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let root = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .resolve_root(&project_id)?;
            tauri_plugin_opener::open_path(root.path(), None::<&str>)
                .map_err(|_| "project-open-unavailable")?;
            Ok(AppEvent::Completed {
                code: "project-opened",
                snapshot: app_snapshot_from_state(&state)?,
            })
        }
        AppIntent::SelectProjectExportDestination { project_id } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let selected = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?
                .folder_picker
                .pick_project_export_destination("qiongli-portable-project");
            let Some(destination) = selected else {
                return Ok(AppEvent::Cancelled {
                    code: "project-export-destination-selection-cancelled",
                });
            };
            let (token, root_label) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .select_export_destination(project_id, destination)?;
            Ok(AppEvent::ProjectDirectorySelected { token, root_label })
        }
        AppIntent::PreviewProjectExport { directory_token } => {
            let preview = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_export(&directory_token)?;
            Ok(AppEvent::Preview { preview })
        }
        AppIntent::SelectProjectImportLocations { suggested_name } => {
            validate_project_dialog_name(&suggested_name)?;
            let mut service = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?;
            let Some(source) = service.folder_picker.pick_project_import_source() else {
                return Ok(AppEvent::Cancelled {
                    code: "project-import-source-selection-cancelled",
                });
            };
            let Some(destination) = service
                .folder_picker
                .pick_project_import_destination(&suggested_name)
            else {
                return Ok(AppEvent::Cancelled {
                    code: "project-import-destination-selection-cancelled",
                });
            };
            drop(service);
            let (token, root_label) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .select_import_locations(source, destination)?;
            Ok(AppEvent::ProjectDirectorySelected { token, root_label })
        }
        AppIntent::PreviewProjectImport { directory_token } => {
            let preview = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_import(&directory_token)?;
            Ok(AppEvent::Preview { preview })
        }
        AppIntent::PreviewProjectRepairManifest { project_id } => {
            preview_project_lifecycle(&state, project_id, ProjectMutationKind::RepairManifest)
        }
        AppIntent::PreviewProjectArchive { project_id } => {
            preview_project_lifecycle(&state, project_id, ProjectMutationKind::Archive)
        }
        AppIntent::PreviewProjectRestore { project_id } => {
            preview_project_lifecycle(&state, project_id, ProjectMutationKind::Restore)
        }
        AppIntent::PreviewProjectRefresh { project_id } => {
            preview_project_lifecycle(&state, project_id, ProjectMutationKind::Refresh)
        }
        AppIntent::PreviewProjectUnregister { project_id } => {
            preview_project_lifecycle(&state, project_id, ProjectMutationKind::Unregister)
        }
        AppIntent::ConfirmOperation { token } => {
            let project_result = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .confirm(&token);
            if let Some(result) = project_result {
                let code = result?;
                return Ok(AppEvent::Completed {
                    code,
                    snapshot: app_snapshot_from_state(&state)?,
                });
            }
            execute_desktop_intent(AppIntent::ConfirmOperation { token }, &state)
        }
        AppIntent::CancelOperation { token } => {
            let cancelled = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .cancel(&token);
            if cancelled {
                return Ok(AppEvent::Cancelled {
                    code: "project-operation-cancelled",
                });
            }
            execute_desktop_intent(AppIntent::CancelOperation { token }, &state)
        }
        other => execute_desktop_intent(other, &state),
    }
}

fn execute_desktop_intent(
    intent: AppIntent,
    state: &tauri::State<'_, DesktopAppState>,
) -> Result<AppEvent, &'static str> {
    let desktop_intent = intent.into_desktop()?;
    let mut service = state
        .service
        .lock()
        .map_err(|_| "desktop-service-lock-failed")?;
    let event = service.execute(desktop_intent);
    app_event(
        event,
        &mut *service,
        state
            .projects
            .lock()
            .map_err(|_| "project-service-lock-failed")?
            .snapshot(),
    )
}

fn preview_project_lifecycle(
    state: &tauri::State<'_, DesktopAppState>,
    project_id: String,
    operation: ProjectMutationKind,
) -> Result<AppEvent, &'static str> {
    let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
    let preview = state
        .projects
        .lock()
        .map_err(|_| "project-service-lock-failed")?
        .preview_lifecycle(&project_id, operation)?;
    Ok(AppEvent::Preview { preview })
}

fn app_snapshot_from_state(
    state: &tauri::State<'_, DesktopAppState>,
) -> Result<AppSnapshotV1, &'static str> {
    let desktop_snapshot = state
        .service
        .lock()
        .map_err(|_| "desktop-service-lock-failed")?
        .snapshot();
    let project_snapshot = state
        .projects
        .lock()
        .map_err(|_| "project-service-lock-failed")?
        .snapshot();
    AppSnapshotV1::from_desktop(desktop_snapshot, project_snapshot)
}

pub(super) fn run_tauri_application(
    service: NativeDesktopService,
    project_service: Option<ProjectStateService>,
) -> Result<(), DesktopLaunchError> {
    tauri::Builder::default()
        .manage(DesktopAppState {
            service: Mutex::new(service),
            projects: Mutex::new(ProjectDesktopState::new(project_service)),
        })
        .invoke_handler(tauri::generate_handler![qiongli_snapshot, qiongli_execute])
        .run(tauri::generate_context!())
        .map_err(|_| DesktopLaunchError)
}
