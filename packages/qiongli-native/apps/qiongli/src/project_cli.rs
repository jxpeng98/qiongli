use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use qiongli_project::{
    ApprovedProjectMutation, PortableProjectPreviewV1, ProjectId, ProjectKind,
    ProjectMigrationPreviewV1, ProjectMutationPreviewV1, ProjectRegistrationOptions, ProjectStage,
    ProjectStateService, ResearchLibrarySnapshotV1,
};
use serde::Serialize;

use crate::command::{CliOutput, CommandEnvironment, config_root};

pub(crate) const PROJECT_USAGE: &str = "Qiongli Research Library\n\nUsage:\n  qiongli project list\n  qiongli project show --project-id <prj_id>\n  qiongli project doctor\n  qiongli project doctor repair <preview|apply> --project-id <prj_id> [--expected-plan-digest <sha256> --approve-filesystem-write]\n  qiongli project create preview --root <absolute-path> --name <name> [--kind <article|review|dissertation-article|manuscript>] [--stage <stage>] [--project-id <prj_id>]\n  qiongli project create apply --root <absolute-path> --name <name> [--kind <kind>] [--stage <stage>] --project-id <prj_id> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project register preview --root <absolute-path> [--name <name>] [--kind <kind>] [--stage <stage>] [--project-id <prj_id>]\n  qiongli project register apply --root <absolute-path> [--name <name>] [--kind <kind>] [--stage <stage>] [--project-id <prj_id>] --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project export <preview|apply> --project-id <prj_id> --destination <absolute-path> [--expected-plan-digest <sha256> --approve-filesystem-write]\n  qiongli project import <preview|apply> --source <absolute-path> --root <absolute-path> [--expected-plan-digest <sha256> --approve-filesystem-write]\n  qiongli project migrate preview --source <legacy-absolute-path> --root <new-absolute-path> [--name <name>] [--kind <kind>] [--stage <stage>] [--project-id <prj_id>]\n  qiongli project migrate apply --source <legacy-absolute-path> --root <new-absolute-path> [--name <name>] [--kind <kind>] [--stage <stage>] --project-id <prj_id> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project <archive|restore|refresh|unregister> preview --project-id <prj_id>\n  qiongli project <archive|restore|refresh|unregister> apply --project-id <prj_id> --expected-plan-digest <sha256> --approve-filesystem-write\n  qiongli project --help\n\nPortable export format:\n  A private directory package containing qiongli-portable-project.json and project/.\n  Absolute paths, client configuration, recognizable credential files, sessions, chats, and transcripts are excluded.\n\nLegacy project migration:\n  Copies bounded academic files into a new 2.x project and leaves the source untouched.\n  Legacy .qiongli runtime state and recognizable credential/session files are not copied.\n\nStages:\n  idea | framing | literature | design | analysis | writing | review | submission\n";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProjectCliCommand {
    Help,
    List,
    Show(ProjectId),
    Doctor,
    PreviewDoctorRepair(ProjectId),
    ApplyDoctorRepair(ProjectId, String),
    Capture(crate::capture_cli::CaptureCliCommand),
    PreviewCreate(ProjectPathOptions),
    ApplyCreate(ProjectPathOptions, String),
    PreviewRegister(ProjectPathOptions),
    ApplyRegister(ProjectPathOptions, String),
    PreviewExport(ProjectExportOptions),
    ApplyExport(ProjectExportOptions, String),
    PreviewImport(ProjectImportOptions),
    ApplyImport(ProjectImportOptions, String),
    PreviewMigration(ProjectMigrationOptions),
    ApplyMigration(ProjectMigrationOptions, String),
    PreviewLifecycle(ProjectLifecycleCommand, ProjectId),
    ApplyLifecycle(ProjectLifecycleCommand, ProjectId, String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectExportOptions {
    project_id: ProjectId,
    destination: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectImportOptions {
    source: PathBuf,
    root: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectMigrationOptions {
    source: PathBuf,
    root: PathBuf,
    project_id: Option<ProjectId>,
    display_name: Option<String>,
    project_kind: Option<ProjectKind>,
    stage: Option<ProjectStage>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectPathOptions {
    root: PathBuf,
    project_id: Option<ProjectId>,
    display_name: Option<String>,
    project_kind: Option<ProjectKind>,
    stage: Option<ProjectStage>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProjectLifecycleCommand {
    Archive,
    Restore,
    Refresh,
    Unregister,
}

pub(crate) fn parse(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let Some(subcommand) = args.first().and_then(|value| value.to_str()) else {
        return Err("a project subcommand is required");
    };
    match subcommand {
        "--help" if args.len() == 1 => Ok(ProjectCliCommand::Help),
        "list" if args.len() == 1 => Ok(ProjectCliCommand::List),
        "doctor" => parse_doctor(&args[1..]),
        "show" => parse_project_id_only(&args[1..]).map(ProjectCliCommand::Show),
        "capture" => crate::capture_cli::parse(&args[1..]).map(ProjectCliCommand::Capture),
        "create" => parse_path_mutation(&args[1..], true),
        "register" => parse_path_mutation(&args[1..], false),
        "export" => parse_portable_export(&args[1..]),
        "import" => parse_portable_import(&args[1..]),
        "migrate" => parse_project_migration(&args[1..]),
        "archive" => parse_lifecycle(&args[1..], ProjectLifecycleCommand::Archive),
        "restore" => parse_lifecycle(&args[1..], ProjectLifecycleCommand::Restore),
        "refresh" => parse_lifecycle(&args[1..], ProjectLifecycleCommand::Refresh),
        "unregister" => parse_lifecycle(&args[1..], ProjectLifecycleCommand::Unregister),
        "--help" | "list" => Err("unexpected project argument"),
        _ => Err("unknown project subcommand"),
    }
}

pub(crate) fn execute(command: ProjectCliCommand, environment: &CommandEnvironment) -> CliOutput {
    if command == ProjectCliCommand::Help {
        return CliOutput::success_text(format!(
            "{PROJECT_USAGE}\n{}\n{}",
            crate::capture_cli::CAPTURE_USAGE,
            crate::repository_capture_cli::USAGE
        ));
    }
    if command == ProjectCliCommand::Capture(crate::capture_cli::CaptureCliCommand::Help) {
        return CliOutput::success_text(format!(
            "{}\n{}",
            crate::capture_cli::CAPTURE_USAGE,
            crate::repository_capture_cli::USAGE
        ));
    }
    let root = match config_root(environment) {
        Ok(root) => root,
        Err(error) => return CliOutput::operation_failure(error.reason_code()),
    };
    let service = ProjectStateService::new(root);
    let output = match command {
        ProjectCliCommand::Help => unreachable!("help returns before service creation"),
        ProjectCliCommand::List => service.snapshot().map(|library| {
            ProjectCliOutput::Library(ProjectListOutput {
                schema_version: 1,
                command: "project-list",
                library,
            })
        }),
        ProjectCliCommand::Show(project_id) => service.snapshot().and_then(|library| {
            let project = library
                .projects
                .into_iter()
                .find(|project| project.project_id == project_id)
                .ok_or(qiongli_project::ProjectError::ProjectNotRegistered)?;
            Ok(ProjectCliOutput::Project(ProjectShowOutput {
                schema_version: 1,
                command: "project-show",
                library_revision: library.revision,
                project,
            }))
        }),
        ProjectCliCommand::Doctor => service.snapshot().map(|library| {
            let blocking = library
                .projects
                .iter()
                .filter(|project| project.health != qiongli_project::ProjectHealth::Ready)
                .count();
            ProjectCliOutput::Doctor(ProjectDoctorOutput {
                schema_version: 1,
                command: "project-doctor",
                status: if blocking == 0 { "ready" } else { "attention" },
                blocking_projects: blocking,
                library,
            })
        }),
        ProjectCliCommand::PreviewDoctorRepair(project_id) => {
            service.preview_repair_manifest(&project_id).map(|plan| {
                ProjectCliOutput::Preview(ProjectPreviewOutput {
                    schema_version: 1,
                    command: "project-doctor-repair-preview",
                    preview: plan.preview().clone(),
                })
            })
        }
        ProjectCliCommand::ApplyDoctorRepair(project_id, digest) => {
            let now = match now_unix() {
                Ok(now) => now,
                Err(error) => return CliOutput::operation_failure(error),
            };
            service
                .preview_repair_manifest(&project_id)
                .and_then(|plan| {
                    service.apply(&plan, &ApprovedProjectMutation::new(digest, true), now)
                })
                .map(|commit| {
                    ProjectCliOutput::Commit(ProjectCommitOutput {
                        schema_version: 1,
                        command: "project-doctor-repair-apply",
                        commit,
                    })
                })
        }
        ProjectCliCommand::Capture(command) => {
            crate::capture_cli::execute(command, &service).map(ProjectCliOutput::Capture)
        }
        ProjectCliCommand::PreviewCreate(options) => {
            preview_path(&service, options, true).map(|preview| {
                ProjectCliOutput::Preview(ProjectPreviewOutput {
                    schema_version: 1,
                    command: "project-create-preview",
                    preview,
                })
            })
        }
        ProjectCliCommand::PreviewRegister(options) => {
            preview_path(&service, options, false).map(|preview| {
                ProjectCliOutput::Preview(ProjectPreviewOutput {
                    schema_version: 1,
                    command: "project-register-preview",
                    preview,
                })
            })
        }
        ProjectCliCommand::ApplyCreate(options, digest) => {
            apply_path(&service, options, true, digest).map(|commit| {
                ProjectCliOutput::Commit(ProjectCommitOutput {
                    schema_version: 1,
                    command: "project-create-apply",
                    commit,
                })
            })
        }
        ProjectCliCommand::ApplyRegister(options, digest) => {
            apply_path(&service, options, false, digest).map(|commit| {
                ProjectCliOutput::Commit(ProjectCommitOutput {
                    schema_version: 1,
                    command: "project-register-apply",
                    commit,
                })
            })
        }
        ProjectCliCommand::PreviewExport(options) => service
            .preview_export(&options.project_id, &options.destination)
            .map(|plan| {
                ProjectCliOutput::PortablePreview(ProjectPortablePreviewOutput {
                    schema_version: 1,
                    command: "project-export-preview",
                    preview: plan.preview().clone(),
                })
            }),
        ProjectCliCommand::ApplyExport(options, digest) => {
            let now = match now_unix() {
                Ok(now) => now,
                Err(error) => return CliOutput::operation_failure(error),
            };
            service
                .preview_export(&options.project_id, &options.destination)
                .and_then(|plan| {
                    service.apply_portable(&plan, &ApprovedProjectMutation::new(digest, true), now)
                })
                .map(|commit| {
                    ProjectCliOutput::PortableCommit(ProjectPortableCommitOutput {
                        schema_version: 1,
                        command: "project-export-apply",
                        commit,
                    })
                })
        }
        ProjectCliCommand::PreviewImport(options) => service
            .preview_import(&options.source, &options.root)
            .map(|plan| {
                ProjectCliOutput::PortablePreview(ProjectPortablePreviewOutput {
                    schema_version: 1,
                    command: "project-import-preview",
                    preview: plan.preview().clone(),
                })
            }),
        ProjectCliCommand::ApplyImport(options, digest) => {
            let now = match now_unix() {
                Ok(now) => now,
                Err(error) => return CliOutput::operation_failure(error),
            };
            service
                .preview_import(&options.source, &options.root)
                .and_then(|plan| {
                    service.apply_portable(&plan, &ApprovedProjectMutation::new(digest, true), now)
                })
                .map(|commit| {
                    ProjectCliOutput::PortableCommit(ProjectPortableCommitOutput {
                        schema_version: 1,
                        command: "project-import-apply",
                        commit,
                    })
                })
        }
        ProjectCliCommand::PreviewMigration(options) => {
            preview_migration(&service, options).map(|preview| {
                ProjectCliOutput::MigrationPreview(ProjectMigrationPreviewOutput {
                    schema_version: 1,
                    command: "project-migrate-preview",
                    preview,
                })
            })
        }
        ProjectCliCommand::ApplyMigration(options, digest) => {
            apply_migration(&service, options, digest).map(|commit| {
                ProjectCliOutput::MigrationCommit(ProjectMigrationCommitOutput {
                    schema_version: 1,
                    command: "project-migrate-apply",
                    commit,
                })
            })
        }
        ProjectCliCommand::PreviewLifecycle(operation, project_id) => {
            preview_lifecycle(&service, operation, &project_id).map(|preview| {
                ProjectCliOutput::Preview(ProjectPreviewOutput {
                    schema_version: 1,
                    command: lifecycle_preview_command(operation),
                    preview,
                })
            })
        }
        ProjectCliCommand::ApplyLifecycle(operation, project_id, digest) => {
            let now = match now_unix() {
                Ok(now) => now,
                Err(error) => return CliOutput::operation_failure(error),
            };
            preview_lifecycle_plan(&service, operation, &project_id, now)
                .and_then(|plan| {
                    service.apply(&plan, &ApprovedProjectMutation::new(digest, true), now)
                })
                .map(|commit| {
                    ProjectCliOutput::Commit(ProjectCommitOutput {
                        schema_version: 1,
                        command: lifecycle_apply_command(operation),
                        commit,
                    })
                })
        }
    };
    match output {
        Ok(output) => json_output(&output),
        Err(error) => CliOutput::operation_failure(error.reason_code()),
    }
}

fn preview_path(
    service: &ProjectStateService,
    options: ProjectPathOptions,
    create: bool,
) -> Result<ProjectMutationPreviewV1, qiongli_project::ProjectError> {
    let now = now_unix().map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    let root = options.root.clone();
    let registration = registration_options(options);
    let plan = if create {
        service.preview_create(root, registration, now)?
    } else {
        service.preview_register(root, registration, now)?
    };
    Ok(plan.preview().clone())
}

fn apply_path(
    service: &ProjectStateService,
    options: ProjectPathOptions,
    create: bool,
    digest: String,
) -> Result<qiongli_project::ProjectMutationCommitV1, qiongli_project::ProjectError> {
    let now = now_unix().map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    let root = options.root.clone();
    let registration = registration_options(options);
    let plan = if create {
        service.preview_create(root, registration, now)?
    } else {
        service.preview_register(root, registration, now)?
    };
    service.apply(&plan, &ApprovedProjectMutation::new(digest, true), now)
}

fn registration_options(options: ProjectPathOptions) -> ProjectRegistrationOptions {
    ProjectRegistrationOptions {
        project_id: options.project_id,
        display_name: options.display_name,
        project_kind: options.project_kind,
        stage: options.stage,
    }
}

fn preview_migration(
    service: &ProjectStateService,
    options: ProjectMigrationOptions,
) -> Result<ProjectMigrationPreviewV1, qiongli_project::ProjectError> {
    let now = now_unix().map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    let registration = migration_registration_options(&options);
    let plan = service.preview_migrate(&options.source, &options.root, registration, now)?;
    Ok(plan.preview().clone())
}

fn apply_migration(
    service: &ProjectStateService,
    options: ProjectMigrationOptions,
    digest: String,
) -> Result<qiongli_project::ProjectMigrationCommitV1, qiongli_project::ProjectError> {
    let now = now_unix().map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    let registration = migration_registration_options(&options);
    let plan = service.preview_migrate(&options.source, &options.root, registration, now)?;
    service.apply_migration(&plan, &ApprovedProjectMutation::new(digest, true), now)
}

fn migration_registration_options(options: &ProjectMigrationOptions) -> ProjectRegistrationOptions {
    ProjectRegistrationOptions {
        project_id: options.project_id.clone(),
        display_name: options.display_name.clone(),
        project_kind: options.project_kind,
        stage: options.stage,
    }
}

fn preview_lifecycle(
    service: &ProjectStateService,
    operation: ProjectLifecycleCommand,
    project_id: &ProjectId,
) -> Result<ProjectMutationPreviewV1, qiongli_project::ProjectError> {
    let now = now_unix().map_err(|_| qiongli_project::ProjectError::HomeUnavailable)?;
    Ok(preview_lifecycle_plan(service, operation, project_id, now)?
        .preview()
        .clone())
}

fn preview_lifecycle_plan(
    service: &ProjectStateService,
    operation: ProjectLifecycleCommand,
    project_id: &ProjectId,
    now_unix: u64,
) -> Result<qiongli_project::VerifiedProjectMutation, qiongli_project::ProjectError> {
    match operation {
        ProjectLifecycleCommand::Archive => service.preview_archive(project_id),
        ProjectLifecycleCommand::Restore => service.preview_restore(project_id),
        ProjectLifecycleCommand::Refresh => service.preview_refresh(project_id, now_unix),
        ProjectLifecycleCommand::Unregister => service.preview_unregister(project_id),
    }
}

fn parse_path_mutation(args: &[OsString], create: bool) -> Result<ProjectCliCommand, &'static str> {
    let Some(mode) = args.first().and_then(|value| value.to_str()) else {
        return Err("project mutation mode is required");
    };
    let apply = match mode {
        "preview" => false,
        "apply" => true,
        _ => return Err("project mutation mode must be preview or apply"),
    };
    let mut root = None;
    let mut project_id = None;
    let mut display_name = None;
    let mut project_kind = None;
    let mut stage = None;
    let mut expected_plan_digest = None;
    let mut approved = false;
    let mut index = 1;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("project option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or("project option value is required")?;
        match option {
            "--root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--project-id" if project_id.is_none() => {
                project_id = Some(parse_project_id(value)?);
            }
            "--name" if display_name.is_none() => {
                display_name = Some(parse_display_name(value)?);
            }
            "--kind" if project_kind.is_none() => {
                project_kind = Some(parse_project_kind(value)?);
            }
            "--stage" if stage.is_none() => stage = Some(parse_project_stage(value)?),
            "--expected-plan-digest" if apply && expected_plan_digest.is_none() => {
                expected_plan_digest = Some(parse_sha256(value)?);
            }
            "--root"
            | "--project-id"
            | "--name"
            | "--kind"
            | "--stage"
            | "--expected-plan-digest" => return Err("project option is unexpected or duplicate"),
            _ => return Err("unknown project option"),
        }
        index += 2;
    }
    if apply && (!approved || expected_plan_digest.is_none()) {
        return Err("project apply requires plan digest and filesystem approval");
    }
    if create && display_name.is_none() {
        return Err("project create requires a display name");
    }
    let options = ProjectPathOptions {
        root: root.ok_or("project root is required")?,
        project_id,
        display_name,
        project_kind,
        stage,
    };
    match (create, apply) {
        (true, false) => Ok(ProjectCliCommand::PreviewCreate(options)),
        (true, true) => Ok(ProjectCliCommand::ApplyCreate(
            options,
            expected_plan_digest.expect("validated above"),
        )),
        (false, false) => Ok(ProjectCliCommand::PreviewRegister(options)),
        (false, true) => Ok(ProjectCliCommand::ApplyRegister(
            options,
            expected_plan_digest.expect("validated above"),
        )),
    }
}

fn parse_doctor(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    if args.is_empty() {
        return Ok(ProjectCliCommand::Doctor);
    }
    if args.first() != Some(&OsString::from("repair")) {
        return Err("unknown project doctor subcommand");
    }
    let (apply, project_id, digest) = parse_identity_mutation(&args[1..])?;
    if apply {
        Ok(ProjectCliCommand::ApplyDoctorRepair(
            project_id,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewDoctorRepair(project_id))
    }
}

fn parse_portable_export(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut project_id = None;
    let mut destination = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project export option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project export option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => project_id = Some(parse_project_id(value)?),
            "--destination" if destination.is_none() => {
                destination = Some(PathBuf::from(value));
            }
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--project-id" | "--destination" | "--expected-plan-digest" => {
                return Err("project export option is unexpected or duplicate");
            }
            _ => return Err("unknown project export option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    let options = ProjectExportOptions {
        project_id: project_id.ok_or("project ID is required")?,
        destination: destination.ok_or("project export destination is required")?,
    };
    if apply {
        Ok(ProjectCliCommand::ApplyExport(
            options,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewExport(options))
    }
}

fn parse_portable_import(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut source = None;
    let mut root = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project import option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project import option value is required")?;
        match option {
            "--source" if source.is_none() => source = Some(PathBuf::from(value)),
            "--root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--source" | "--root" | "--expected-plan-digest" => {
                return Err("project import option is unexpected or duplicate");
            }
            _ => return Err("unknown project import option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    let options = ProjectImportOptions {
        source: source.ok_or("project import source is required")?,
        root: root.ok_or("project import root is required")?,
    };
    if apply {
        Ok(ProjectCliCommand::ApplyImport(
            options,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewImport(options))
    }
}

fn parse_project_migration(args: &[OsString]) -> Result<ProjectCliCommand, &'static str> {
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut source = None;
    let mut root = None;
    let mut project_id = None;
    let mut display_name = None;
    let mut project_kind = None;
    let mut stage = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project migration option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project migration option value is required")?;
        match option {
            "--source" if source.is_none() => source = Some(PathBuf::from(value)),
            "--root" if root.is_none() => root = Some(PathBuf::from(value)),
            "--project-id" if project_id.is_none() => {
                project_id = Some(parse_project_id(value)?);
            }
            "--name" if display_name.is_none() => {
                display_name = Some(parse_display_name(value)?);
            }
            "--kind" if project_kind.is_none() => {
                project_kind = Some(parse_project_kind(value)?);
            }
            "--stage" if stage.is_none() => stage = Some(parse_project_stage(value)?),
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--source"
            | "--root"
            | "--project-id"
            | "--name"
            | "--kind"
            | "--stage"
            | "--expected-plan-digest" => {
                return Err("project migration option is unexpected or duplicate");
            }
            _ => return Err("unknown project migration option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    if apply && project_id.is_none() {
        return Err("project migration apply requires the previewed project ID");
    }
    let options = ProjectMigrationOptions {
        source: source.ok_or("legacy project source is required")?,
        root: root.ok_or("migrated project root is required")?,
        project_id,
        display_name,
        project_kind,
        stage,
    };
    if apply {
        Ok(ProjectCliCommand::ApplyMigration(
            options,
            digest.expect("apply digest validated"),
        ))
    } else {
        Ok(ProjectCliCommand::PreviewMigration(options))
    }
}

fn parse_identity_mutation(
    args: &[OsString],
) -> Result<(bool, ProjectId, Option<String>), &'static str> {
    let (apply, option_args) = parse_mutation_mode(args)?;
    let mut project_id = None;
    let mut digest = None;
    let mut approved = false;
    let mut index = 0;
    while index < option_args.len() {
        let option = option_args[index]
            .to_str()
            .ok_or("project option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = option_args
            .get(index + 1)
            .ok_or("project option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => project_id = Some(parse_project_id(value)?),
            "--expected-plan-digest" if apply && digest.is_none() => {
                digest = Some(parse_sha256(value)?);
            }
            "--project-id" | "--expected-plan-digest" => {
                return Err("project option is unexpected or duplicate");
            }
            _ => return Err("unknown project option"),
        }
        index += 2;
    }
    validate_apply_approval(apply, approved, digest.as_ref())?;
    Ok((apply, project_id.ok_or("project ID is required")?, digest))
}

fn parse_mutation_mode(args: &[OsString]) -> Result<(bool, &[OsString]), &'static str> {
    let Some(mode) = args.first().and_then(|value| value.to_str()) else {
        return Err("project mutation mode is required");
    };
    match mode {
        "preview" => Ok((false, &args[1..])),
        "apply" => Ok((true, &args[1..])),
        _ => Err("project mutation mode must be preview or apply"),
    }
}

fn validate_apply_approval(
    apply: bool,
    approved: bool,
    digest: Option<&String>,
) -> Result<(), &'static str> {
    if apply && (!approved || digest.is_none()) {
        Err("project apply requires plan digest and filesystem approval")
    } else {
        Ok(())
    }
}

fn parse_lifecycle(
    args: &[OsString],
    operation: ProjectLifecycleCommand,
) -> Result<ProjectCliCommand, &'static str> {
    let Some(mode) = args.first().and_then(|value| value.to_str()) else {
        return Err("project mutation mode is required");
    };
    let apply = match mode {
        "preview" => false,
        "apply" => true,
        _ => return Err("project mutation mode must be preview or apply"),
    };
    let mut project_id = None;
    let mut expected_plan_digest = None;
    let mut approved = false;
    let mut index = 1;
    while index < args.len() {
        let option = args[index]
            .to_str()
            .ok_or("project option is not valid UTF-8")?;
        if option == "--approve-filesystem-write" {
            if !apply || approved {
                return Err("project approval is unexpected or duplicate");
            }
            approved = true;
            index += 1;
            continue;
        }
        let value = args
            .get(index + 1)
            .ok_or("project option value is required")?;
        match option {
            "--project-id" if project_id.is_none() => {
                project_id = Some(parse_project_id(value)?);
            }
            "--expected-plan-digest" if apply && expected_plan_digest.is_none() => {
                expected_plan_digest = Some(parse_sha256(value)?);
            }
            "--project-id" | "--expected-plan-digest" => {
                return Err("project option is unexpected or duplicate");
            }
            _ => return Err("unknown project option"),
        }
        index += 2;
    }
    let project_id = project_id.ok_or("project ID is required")?;
    if apply {
        if !approved {
            return Err("project apply requires filesystem approval");
        }
        Ok(ProjectCliCommand::ApplyLifecycle(
            operation,
            project_id,
            expected_plan_digest.ok_or("project apply plan digest is required")?,
        ))
    } else {
        Ok(ProjectCliCommand::PreviewLifecycle(operation, project_id))
    }
}

fn parse_project_id_only(args: &[OsString]) -> Result<ProjectId, &'static str> {
    if args.len() != 2 || args[0] != OsStr::new("--project-id") {
        return Err("exactly one project ID is required");
    }
    parse_project_id(&args[1])
}

fn parse_project_id(value: &OsStr) -> Result<ProjectId, &'static str> {
    ProjectId::parse(value.to_str().ok_or("project ID is invalid")?.to_string())
        .map_err(|_| "project ID is invalid")
}

fn parse_display_name(value: &OsStr) -> Result<String, &'static str> {
    let value = value.to_str().ok_or("project name is invalid")?;
    if value.is_empty()
        || value.len() > 160
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err("project name is invalid");
    }
    Ok(value.to_string())
}

fn parse_project_kind(value: &OsStr) -> Result<ProjectKind, &'static str> {
    match value.to_str() {
        Some("article") => Ok(ProjectKind::Article),
        Some("review") => Ok(ProjectKind::Review),
        Some("dissertation-article") => Ok(ProjectKind::DissertationArticle),
        Some("manuscript") => Ok(ProjectKind::Manuscript),
        _ => Err("project kind is invalid"),
    }
}

fn parse_project_stage(value: &OsStr) -> Result<ProjectStage, &'static str> {
    match value.to_str() {
        Some("idea") => Ok(ProjectStage::Idea),
        Some("framing") => Ok(ProjectStage::Framing),
        Some("literature") => Ok(ProjectStage::Literature),
        Some("design") => Ok(ProjectStage::Design),
        Some("analysis") => Ok(ProjectStage::Analysis),
        Some("writing") => Ok(ProjectStage::Writing),
        Some("review") => Ok(ProjectStage::Review),
        Some("submission") => Ok(ProjectStage::Submission),
        _ => Err("project stage is invalid"),
    }
}

fn parse_sha256(value: &OsStr) -> Result<String, &'static str> {
    let value = value.to_str().ok_or("project plan digest is invalid")?;
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err("project plan digest is invalid");
    }
    Ok(value.to_string())
}

fn now_unix() -> Result<u64, &'static str> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| "project-clock-unavailable")
}

fn lifecycle_preview_command(operation: ProjectLifecycleCommand) -> &'static str {
    match operation {
        ProjectLifecycleCommand::Archive => "project-archive-preview",
        ProjectLifecycleCommand::Restore => "project-restore-preview",
        ProjectLifecycleCommand::Refresh => "project-refresh-preview",
        ProjectLifecycleCommand::Unregister => "project-unregister-preview",
    }
}

fn lifecycle_apply_command(operation: ProjectLifecycleCommand) -> &'static str {
    match operation {
        ProjectLifecycleCommand::Archive => "project-archive-apply",
        ProjectLifecycleCommand::Restore => "project-restore-apply",
        ProjectLifecycleCommand::Refresh => "project-refresh-apply",
        ProjectLifecycleCommand::Unregister => "project-unregister-apply",
    }
}

fn json_output<T: Serialize>(value: &T) -> CliOutput {
    match serde_json::to_string_pretty(value) {
        Ok(rendered) => CliOutput::success_text(format!("{rendered}\n")),
        Err(_) => CliOutput::operation_failure("output-serialization-failed"),
    }
}

#[derive(Serialize)]
#[serde(untagged)]
enum ProjectCliOutput {
    Library(ProjectListOutput),
    Project(ProjectShowOutput),
    Doctor(ProjectDoctorOutput),
    Preview(ProjectPreviewOutput),
    Commit(ProjectCommitOutput),
    PortablePreview(ProjectPortablePreviewOutput),
    PortableCommit(ProjectPortableCommitOutput),
    MigrationPreview(ProjectMigrationPreviewOutput),
    MigrationCommit(ProjectMigrationCommitOutput),
    Capture(crate::capture_cli::CaptureCliOutput),
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectListOutput {
    schema_version: u32,
    command: &'static str,
    library: ResearchLibrarySnapshotV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectShowOutput {
    schema_version: u32,
    command: &'static str,
    library_revision: u64,
    project: qiongli_project::ArticleProjectSummaryV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectDoctorOutput {
    schema_version: u32,
    command: &'static str,
    status: &'static str,
    blocking_projects: usize,
    library: ResearchLibrarySnapshotV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectPreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: ProjectMutationPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectCommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: qiongli_project::ProjectMutationCommitV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectPortablePreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: PortableProjectPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectPortableCommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: qiongli_project::PortableProjectCommitV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMigrationPreviewOutput {
    schema_version: u32,
    command: &'static str,
    preview: ProjectMigrationPreviewV1,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectMigrationCommitOutput {
    schema_version: u32,
    command: &'static str,
    commit: qiongli_project::ProjectMigrationCommitV1,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parser_closes_project_mutation_shape_and_approval() {
        assert!(matches!(
            parse(&args(&[
                "create",
                "preview",
                "--root",
                "/tmp/paper",
                "--name",
                "Paper"
            ])),
            Ok(ProjectCliCommand::PreviewCreate(_))
        ));
        assert_eq!(
            parse(&args(&[
                "archive",
                "apply",
                "--project-id",
                "prj_00000000000000000000000000000000",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            ])),
            Err("project apply requires filesystem approval")
        );
        assert!(
            parse(&args(&[
                "create",
                "preview",
                "--root",
                "/tmp/paper",
                "--name",
                "Paper",
                "--name",
                "Duplicate"
            ]))
            .is_err()
        );
        assert!(matches!(
            parse(&args(&[
                "export",
                "preview",
                "--project-id",
                "prj_00000000000000000000000000000000",
                "--destination",
                "/tmp/portable-paper"
            ])),
            Ok(ProjectCliCommand::PreviewExport(_))
        ));
        assert_eq!(
            parse(&args(&[
                "import",
                "apply",
                "--source",
                "/tmp/portable-paper",
                "--root",
                "/tmp/imported-paper",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            ])),
            Err("project apply requires plan digest and filesystem approval")
        );
        assert!(matches!(
            parse(&args(&[
                "doctor",
                "repair",
                "preview",
                "--project-id",
                "prj_00000000000000000000000000000000"
            ])),
            Ok(ProjectCliCommand::PreviewDoctorRepair(_))
        ));
        assert!(matches!(
            parse(&args(&[
                "migrate",
                "preview",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--name",
                "Migrated paper"
            ])),
            Ok(ProjectCliCommand::PreviewMigration(_))
        ));
        assert_eq!(
            parse(&args(&[
                "migrate",
                "apply",
                "--source",
                "/tmp/legacy-paper",
                "--root",
                "/tmp/migrated-paper",
                "--expected-plan-digest",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "--approve-filesystem-write"
            ])),
            Err("project migration apply requires the previewed project ID")
        );
    }
}
