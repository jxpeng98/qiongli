use std::sync::Mutex;

use crate::desktop_api::{
    AppEvent, AppIntent, AppOrchestrationControlAction, AppSnapshotV1, app_event,
};
use crate::orchestration_control::{
    FullOrchestrationService, OrchestrationControlAction, OrchestrationRunReference,
};

use super::*;

struct DesktopAppState {
    service: Mutex<NativeDesktopService>,
    projects: Mutex<ProjectDesktopState>,
    orchestration: Mutex<Option<FullOrchestrationService>>,
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
    intent.validate()?;
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
            match projects.select_register_root(root) {
                Ok((token, root_label)) => {
                    Ok(AppEvent::ProjectDirectorySelected { token, root_label })
                }
                Err(code) => Ok(AppEvent::ValidationFailed { code }),
            }
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
        AppIntent::SelectProjectMigrationLocations { suggested_name } => {
            validate_project_dialog_name(&suggested_name)?;
            let mut service = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?;
            let Some(source) = service.folder_picker.pick_project_migration_source() else {
                return Ok(AppEvent::Cancelled {
                    code: "project-migration-source-selection-cancelled",
                });
            };
            let Some(destination) = service
                .folder_picker
                .pick_project_migration_destination(&suggested_name)
            else {
                return Ok(AppEvent::Cancelled {
                    code: "project-migration-destination-selection-cancelled",
                });
            };
            drop(service);
            let (token, root_label) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .select_migration_locations(source, destination)?;
            Ok(AppEvent::ProjectDirectorySelected { token, root_label })
        }
        AppIntent::PreviewProjectMigration {
            directory_token,
            display_name,
            project_kind,
            stage,
        } => {
            let preview = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_migration(&directory_token, display_name, project_kind, stage)?;
            Ok(AppEvent::Preview { preview })
        }
        AppIntent::SelectProjectMigrationRecoveryLocations => {
            let mut service = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?;
            let Some(source) = service
                .folder_picker
                .pick_project_migration_recovery_source()
            else {
                return Ok(AppEvent::Cancelled {
                    code: "project-migration-recovery-source-selection-cancelled",
                });
            };
            let Some(destination) = service
                .folder_picker
                .pick_project_migration_recovery_destination()
            else {
                return Ok(AppEvent::Cancelled {
                    code: "project-migration-recovery-destination-selection-cancelled",
                });
            };
            drop(service);
            let (token, root_label) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .select_migration_recovery_locations(source, destination)?;
            Ok(AppEvent::ProjectDirectorySelected { token, root_label })
        }
        AppIntent::PreviewProjectMigrationRecovery { directory_token } => {
            let preview = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_migration_recovery(&directory_token)?;
            Ok(AppEvent::Preview { preview })
        }
        AppIntent::SelectProjectMigrationRollbackLocations => {
            let mut service = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?;
            let Some(source) = service
                .folder_picker
                .pick_project_migration_rollback_source()
            else {
                return Ok(AppEvent::Cancelled {
                    code: "project-migration-rollback-source-selection-cancelled",
                });
            };
            let Some(destination) = service
                .folder_picker
                .pick_project_migration_rollback_destination()
            else {
                return Ok(AppEvent::Cancelled {
                    code: "project-migration-rollback-destination-selection-cancelled",
                });
            };
            drop(service);
            let (token, root_label) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .select_migration_rollback_locations(source, destination)?;
            Ok(AppEvent::ProjectDirectorySelected { token, root_label })
        }
        AppIntent::PreviewProjectMigrationRollback { directory_token } => {
            let preview = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_migration_rollback(&directory_token)?;
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
        AppIntent::LoadCaptureInbox { project_id } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let inbox = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .capture_inbox(&project_id)?;
            Ok(AppEvent::CaptureInbox { inbox })
        }
        AppIntent::LoadCaptureCoverage { project_id } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let coverage = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .capture_coverage(&project_id)?;
            Ok(AppEvent::CaptureCoverage { coverage })
        }
        AppIntent::LoadArtifactChanges { project_id } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let changes = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .artifact_changes(&project_id)?;
            Ok(AppEvent::ArtifactChanges { changes })
        }
        AppIntent::LoadAcademicGraph { project_id } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let (graph, comparison) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .academic_graph(&project_id)?;
            Ok(AppEvent::AcademicGraph { graph, comparison })
        }
        AppIntent::LoadAcademicGraphPortfolio => {
            let portfolio = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .academic_graph_portfolio()?;
            Ok(AppEvent::AcademicGraphPortfolio { portfolio })
        }
        AppIntent::QueryAcademicGraph { project_id, query } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let result = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .query_academic_graph(&project_id, &query)?;
            Ok(AppEvent::AcademicGraphQuery { result })
        }
        AppIntent::QueryAcademicGraphPath { project_id, query } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let result = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .query_academic_graph_path(&project_id, &query)?;
            Ok(AppEvent::AcademicGraphPath { result })
        }
        AppIntent::OpenAcademicGraphArtifact {
            project_id,
            expected_project_revision,
            expected_projection_id,
            entity,
        } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let opened_entity = entity.clone();
            let (entity_kind, entity_id) = entity.into_parts();
            let target = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .resolve_academic_graph_artifact(
                    &project_id,
                    expected_project_revision,
                    &expected_projection_id,
                    entity_kind,
                    &entity_id,
                )?;
            tauri_plugin_opener::open_path(target.path(), None::<&str>)
                .map_err(|_| "academic-graph-artifact-open-unavailable")?;
            Ok(AppEvent::AcademicGraphArtifactOpened {
                project_id: target.project_id,
                project_revision: target.project_revision,
                projection_id: target.projection_id,
                entity: opened_entity,
            })
        }
        AppIntent::ReadCapture {
            project_id,
            capture_id,
        } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let capture_id = CaptureId::parse(capture_id).map_err(|error| error.reason_code())?;
            let capture = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .read_capture(&project_id, &capture_id)?;
            Ok(AppEvent::CaptureRead { capture })
        }
        AppIntent::SelectCaptureFile { project_id } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let selected = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?
                .folder_picker
                .pick_capture_file();
            let Some(source) = selected else {
                return Ok(AppEvent::Cancelled {
                    code: "capture-file-selection-cancelled",
                });
            };
            let (token, file_label) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .select_capture_file(project_id, source)?;
            Ok(AppEvent::CaptureFileSelected { token, file_label })
        }
        AppIntent::PreviewCaptureIntake { file_token } => {
            let (intake, preview) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_capture_intake(&file_token)?;
            Ok(AppEvent::CaptureIntakePreview { intake, preview })
        }
        AppIntent::PreviewCaptureConsolidation {
            project_id,
            capture_id,
        } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let capture_id = CaptureId::parse(capture_id).map_err(|error| error.reason_code())?;
            let (consolidation, preview) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_capture_consolidation(&project_id, &capture_id)?;
            Ok(AppEvent::CaptureConsolidationPreview {
                consolidation,
                preview,
            })
        }
        AppIntent::LoadCaptureDeliveries { request } => {
            let page = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .capture_deliveries(request)?;
            Ok(AppEvent::CaptureDeliveries { page })
        }
        AppIntent::InspectCaptureDelivery { envelope_id } => {
            let delivery = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .inspect_capture_delivery(&envelope_id)?;
            Ok(AppEvent::CaptureDeliveryInspected { delivery })
        }
        AppIntent::RetryCaptureDelivery {
            envelope_id,
            expected_generation,
            expected_record_sha256,
            retried_at_unix,
            cause,
        } => {
            let delivery = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .retry_capture_delivery(
                    &envelope_id,
                    expected_generation,
                    &expected_record_sha256,
                    retried_at_unix,
                    cause,
                )?;
            Ok(AppEvent::CaptureDeliveryUpdated { delivery })
        }
        AppIntent::CancelCaptureDelivery {
            envelope_id,
            expected_generation,
            expected_record_sha256,
            cancelled_at_unix,
        } => {
            let delivery = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .cancel_capture_delivery(
                    &envelope_id,
                    expected_generation,
                    &expected_record_sha256,
                    cancelled_at_unix,
                )?;
            Ok(AppEvent::CaptureDeliveryUpdated { delivery })
        }
        AppIntent::PreviewCaptureDeliveryAcknowledgement {
            envelope_id,
            destination_project_id,
            accepted_capture_id,
            expected_project_revision,
            resulting_project_revision,
            acknowledged_at_unix,
            expected_generation,
            expected_record_sha256,
        } => {
            let request = qiongli_project::CaptureDeliveryAcknowledgementRequestV1 {
                envelope_id,
                destination_project_id,
                accepted_capture_id,
                expected_project_revision,
                resulting_project_revision,
                acknowledged_at_unix,
            };
            let (acknowledgement, preview) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_capture_delivery_acknowledgement(
                    request,
                    expected_generation,
                    &expected_record_sha256,
                )?;
            Ok(AppEvent::CaptureDeliveryAcknowledgementPreview {
                acknowledgement,
                preview,
            })
        }
        AppIntent::LoadCaptureAssignments { request } => {
            let page = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .capture_assignments(request)?;
            Ok(AppEvent::CaptureAssignments { page })
        }
        AppIntent::InspectCaptureAssignment { intent_id } => {
            let assignment = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .inspect_capture_assignment(&intent_id)?;
            Ok(AppEvent::CaptureAssignmentInspected { assignment })
        }
        AppIntent::PreviewCaptureAssignment {
            source_envelope_id,
            target_project_id,
            decision,
            decided_at_unix,
        } => {
            let (assignment, preview) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_capture_assignment(
                    &source_envelope_id,
                    &target_project_id,
                    decision,
                    decided_at_unix,
                )?;
            Ok(AppEvent::CaptureAssignmentPreview {
                assignment,
                preview,
            })
        }
        AppIntent::LoadCaptureResolutions { request } => {
            let page = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .capture_resolutions(request)?;
            Ok(AppEvent::CaptureResolutions { page })
        }
        AppIntent::InspectCaptureResolution {
            project_id,
            receipt_id,
        } => {
            let resolution = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .inspect_capture_resolution(&project_id, &receipt_id)?;
            Ok(AppEvent::CaptureResolutionInspected { resolution })
        }
        AppIntent::PreviewCaptureResolution {
            assignment_receipt_id,
            reviewed_at_unix,
            selections,
        } => {
            let mut projects = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?;
            if let Some(selections) = selections {
                let (resolution, selections, preview) = projects.preview_capture_resolution(
                    &assignment_receipt_id,
                    reviewed_at_unix,
                    selections,
                )?;
                Ok(AppEvent::CaptureResolutionPreview {
                    resolution,
                    selections,
                    preview,
                })
            } else {
                let resolution =
                    projects.capture_resolution_plan(&assignment_receipt_id, reviewed_at_unix)?;
                Ok(AppEvent::CaptureResolutionPlan { resolution })
            }
        }
        AppIntent::LoadPortfolioStatus => {
            let portfolio = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .portfolio_status()?;
            Ok(AppEvent::PortfolioStatus { portfolio })
        }
        AppIntent::QueryPortfolio { request } => {
            let result = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .query_portfolio(request)?;
            Ok(AppEvent::PortfolioQuery { result })
        }
        AppIntent::LoadSemanticTimeline { request } => {
            let result = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .semantic_timeline(request)?;
            Ok(AppEvent::SemanticTimeline { result })
        }
        AppIntent::LoadPortfolioDoctor => {
            let doctor = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .portfolio_doctor()?;
            Ok(AppEvent::PortfolioDoctor { doctor })
        }
        AppIntent::PreviewPortfolioMaintenance { operation } => {
            let (maintenance, preview) = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .preview_portfolio_maintenance(operation)?;
            Ok(AppEvent::PortfolioMaintenancePreview {
                maintenance,
                preview,
            })
        }
        AppIntent::PollContinuityOperation { operation_id } => {
            let poll = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .poll_continuity_operation(&operation_id)?;
            Ok(continuity_poll_event(poll))
        }
        AppIntent::CancelContinuityOperation { operation_id } => {
            let poll = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .cancel_continuity_operation(&operation_id)?;
            Ok(continuity_poll_event(poll))
        }
        AppIntent::LoadOrchestration {
            project_id,
            expected_project_revision,
        } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let orchestration = state
                .orchestration
                .lock()
                .map_err(|_| "orchestration-service-lock-failed")?;
            let runs = orchestration
                .as_ref()
                .ok_or("orchestration-service-unavailable")?
                .list_runs(&project_id, expected_project_revision)
                .map_err(|error| error.reason_code())?;
            Ok(AppEvent::OrchestrationLoaded { runs })
        }
        AppIntent::ControlOrchestration {
            project_id,
            expected_project_revision,
            run_id,
            expected_generation,
            expected_document_sha256,
            action_name,
        } => {
            let project_id = ProjectId::parse(project_id).map_err(|error| error.reason_code())?;
            let run_id =
                qiongli_execution::RunId::parse(run_id).map_err(|error| error.reason_code())?;
            let reference = OrchestrationRunReference {
                project_id: project_id.clone(),
                expected_project_revision,
                run_id,
                expected_generation,
                expected_document_sha256,
            };
            let action = match action_name {
                AppOrchestrationControlAction::Pause => OrchestrationControlAction::Pause,
                AppOrchestrationControlAction::Recover => OrchestrationControlAction::Recover,
                AppOrchestrationControlAction::Resume => OrchestrationControlAction::Resume,
                AppOrchestrationControlAction::Cancel => OrchestrationControlAction::Cancel,
            };
            let orchestration = state
                .orchestration
                .lock()
                .map_err(|_| "orchestration-service-lock-failed")?;
            let orchestration = orchestration
                .as_ref()
                .ok_or("orchestration-service-unavailable")?;
            let run = orchestration
                .control(&reference, action)
                .map_err(|error| error.reason_code())?;
            let runs = orchestration
                .list_runs(&project_id, expected_project_revision)
                .map_err(|error| error.reason_code())?;
            Ok(AppEvent::OrchestrationRunUpdated { run, runs })
        }
        AppIntent::PreviewOrchestrationTest { .. }
        | AppIntent::PreviewOrchestrationContinue { .. } => Err("host-handoff-not-ready"),
        AppIntent::RevealZoteroCompanion => {
            let path = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?
                .staged_zotero_companion_path()?;
            reveal_in_file_manager(&path)?;
            Ok(AppEvent::Completed {
                code: "zotero-companion-revealed",
                snapshot: app_snapshot_from_state(&state)?,
            })
        }
        AppIntent::OpenZotero => {
            let path = state
                .service
                .lock()
                .map_err(|_| "desktop-service-lock-failed")?
                .zotero_application_path()?;
            tauri_plugin_opener::open_path(path, None::<&str>)
                .map_err(|_| "zotero-application-open-unavailable")?;
            Ok(AppEvent::Completed {
                code: "zotero-application-opened",
                snapshot: app_snapshot_from_state(&state)?,
            })
        }
        AppIntent::ConfirmOperation { token } => {
            let project_result = state
                .projects
                .lock()
                .map_err(|_| "project-service-lock-failed")?
                .confirm(&token);
            if let Some(result) = project_result {
                let ConfirmedProjectOperation {
                    code,
                    capture_project_id,
                    continuity,
                    continuity_operation,
                    migration_qualification,
                } = result?;
                if let Some(progress) = continuity_operation {
                    return Ok(AppEvent::ContinuityOperationProgress { progress });
                }
                if let Some(project_id) = capture_project_id {
                    let projects = state
                        .projects
                        .lock()
                        .map_err(|_| "project-service-lock-failed")?;
                    let inbox = projects.capture_inbox(&project_id)?;
                    let coverage = projects.capture_coverage(&project_id)?;
                    let changes = projects.artifact_changes(&project_id)?;
                    drop(projects);
                    let (delivery, assignment, resolution) = match continuity {
                        Some(ConfirmedCaptureContinuity::Delivery(delivery)) => {
                            (Some(delivery), None, None)
                        }
                        Some(ConfirmedCaptureContinuity::Assignment(assignment)) => {
                            (None, Some(assignment), None)
                        }
                        Some(ConfirmedCaptureContinuity::Resolution(resolution)) => {
                            (None, None, Some(resolution))
                        }
                        None => (None, None, None),
                    };
                    return Ok(AppEvent::CaptureOperationCompleted {
                        code,
                        snapshot: Box::new(app_snapshot_from_state(&state)?),
                        inbox,
                        coverage,
                        changes,
                        delivery: Box::new(delivery),
                        assignment: Box::new(assignment),
                        resolution: Box::new(resolution),
                    });
                }
                if let Some(qualification) = migration_qualification {
                    return Ok(AppEvent::ProjectMigrationCompleted {
                        code,
                        snapshot: app_snapshot_from_state(&state)?,
                        qualification,
                    });
                }
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

#[cfg(target_os = "macos")]
fn reveal_in_file_manager(path: &Path) -> Result<(), &'static str> {
    let status = std::process::Command::new("/usr/bin/open")
        .arg("-R")
        .arg(path)
        .status()
        .map_err(|_| "zotero-companion-reveal-unavailable")?;
    status
        .success()
        .then_some(())
        .ok_or("zotero-companion-reveal-unavailable")
}

#[cfg(target_os = "windows")]
fn reveal_in_file_manager(path: &Path) -> Result<(), &'static str> {
    let status = std::process::Command::new("explorer.exe")
        .arg("/select,")
        .arg(path)
        .status()
        .map_err(|_| "zotero-companion-reveal-unavailable")?;
    status
        .success()
        .then_some(())
        .ok_or("zotero-companion-reveal-unavailable")
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn reveal_in_file_manager(path: &Path) -> Result<(), &'static str> {
    let parent = path.parent().ok_or("zotero-companion-reveal-unavailable")?;
    tauri_plugin_opener::open_path(parent, None::<&str>)
        .map_err(|_| "zotero-companion-reveal-unavailable")
}

fn continuity_poll_event(poll: DesktopContinuityPoll) -> AppEvent {
    match poll {
        DesktopContinuityPoll::Progress(progress) => {
            AppEvent::ContinuityOperationProgress { progress }
        }
        DesktopContinuityPoll::Completed(result) => {
            AppEvent::PortfolioMaintenanceCompleted { result }
        }
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
    let orchestration = project_service
        .as_ref()
        .map(|projects| {
            let registry = FullProjectToolRegistry::from_embedded_content(&service.content)
                .map_err(|_| DesktopLaunchError)?;
            FullOrchestrationService::from_embedded_content(
                projects.clone(),
                registry,
                &service.content,
            )
            .map_err(|_| DesktopLaunchError)
        })
        .transpose()?;
    tauri::Builder::default()
        .manage(DesktopAppState {
            service: Mutex::new(service),
            projects: Mutex::new(ProjectDesktopState::new(project_service)),
            orchestration: Mutex::new(orchestration),
        })
        .invoke_handler(tauri::generate_handler![qiongli_snapshot, qiongli_execute])
        .run(tauri::generate_context!())
        .map_err(|_| DesktopLaunchError)
}
