//! Interactive presentation over the existing managed content write owner.
use std::io::{self, BufRead, IsTerminal, Read, Write};

use qiongli_content::EmbeddedContent;

use crate::managed_operation::ManagedOperationCliCommand;
use crate::{CliOutput, CommandEnvironment};

pub(crate) fn prepare_plan(
    command: &ManagedOperationCliCommand,
    environment: &CommandEnvironment,
    content: &EmbeddedContent,
) -> Result<String, &'static str> {
    let result = crate::managed_operation::execute(command, environment, content);
    if let (
        ManagedOperationCliCommand::PlanPluginSource {
            target,
            destination,
            ..
        },
        Err("plugin-source-already-exists"),
    ) = (command, &result)
    {
        return crate::managed_operation::execute(
            &ManagedOperationCliCommand::PlanPluginSource {
                action: crate::plugin_source::PluginSourceAction::Update,
                target: *target,
                destination: destination.clone(),
            },
            environment,
            content,
        );
    }
    result
}

pub struct BundledContentReview {
    pub(crate) plan_json: String,
}

impl BundledContentReview {
    pub fn run(self, environment: &CommandEnvironment, content: &EmbeddedContent) -> CliOutput {
        if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
            return CliOutput::usage_text(
                "installation requires a terminal; use --dry-run, then qiongli app apply for scripts",
            );
        }
        match self.review(
            environment,
            content,
            &mut io::stdin().lock(),
            &mut io::stdout().lock(),
        ) {
            Ok(()) => CliOutput::success_text(""),
            Err(code) => {
                let hint = match code {
                    "local-host-other-qiongli-enabled" => {
                        "Disable the other Qiongli Plugin in the Host, then retry this command."
                    }
                    "local-host-marketplace-conflict" => {
                        "Review the Host's qiongli-cli-local marketplace path; choose the matching export destination."
                    }
                    "host-plugin-executable-unavailable" | "local-host-version-unsupported" => {
                        "Install or update the selected Codex/Claude CLI, then retry this command."
                    }
                    "local-host-plugin-scope-conflict" => {
                        "Review duplicate or project-scoped Qiongli Plugins in the Host; this command uses user scope."
                    }
                    _ => {
                        "Review the reported step and retained files before retrying. Use --help for the installation workflow."
                    }
                };
                CliOutput::operation_failure(code).with_stderr(format!("error: {code}\n{hint}\n"))
            }
        }
    }

    fn review(
        &self,
        environment: &CommandEnvironment,
        content: &EmbeddedContent,
        reader: &mut impl BufRead,
        writer: &mut impl Write,
    ) -> Result<(), &'static str> {
        let value: serde_json::Value =
            serde_json::from_str(&self.plan_json).map_err(|_| "managed-operation-plan-invalid")?;
        if let Some(preset) = value["operation"].get("preset") {
            let preset = serde_json::from_value(preset.clone())
                .map_err(|_| "managed-operation-plan-invalid")?;
            let destination = crate::managed_operation::skills_preset_path(environment, preset)?;
            show_json(
                writer,
                &serde_json::json!({"destination": destination}).to_string(),
            )?;
        }
        show_json(writer, &self.plan_json)?;
        if !confirm(
            reader,
            writer,
            "Apply these bundled-content file changes? [y/N] ",
        )? {
            return line(writer, "Cancelled; no changes made.\n");
        }
        let result =
            crate::managed_operation::apply_reviewed_plan(environment, content, &self.plan_json)?;
        show_json(writer, &result)?;
        if value["operation"]["kind"] == "plugin-source" {
            let source: crate::plugin_source::PluginSourcePlan =
                serde_json::from_value(value["operation"]["source"].clone())
                    .map_err(|_| "managed-operation-plan-invalid")?;
            if let Err(code) =
                crate::plugin_host::register(environment, content, &source, reader, writer)
            {
                line(
                    writer,
                    "Host registration did not finish. Exported files remain available; resolve the reported conflict or Host error, then rerun upgrade plugin with the same target/destination. Other Plugins are not removed automatically.\n",
                )?;
                return Err(code);
            }
        } else {
            line(
                writer,
                "Skills are in .qiongli-skills under the selected home/project. Use install plugin to load the workflow and Full MCP in a Host.\n",
            )?;
        }
        Ok(())
    }
}

pub(crate) fn show_json(writer: &mut impl Write, json: &str) -> Result<(), &'static str> {
    let value = serde_json::from_str(json).map_err(|_| "installation-preview-invalid")?;
    let mut text = String::new();
    crate::cli_presentation::fields(&mut text, &value, 0);
    line(writer, &text)
}

pub(crate) fn line(writer: &mut impl Write, text: &str) -> Result<(), &'static str> {
    writer
        .write_all(text.as_bytes())
        .map_err(|_| "installation-output-failed")
}

pub(crate) fn confirm(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    prompt: &str,
) -> Result<bool, &'static str> {
    line(writer, prompt)?;
    writer.flush().map_err(|_| "installation-output-failed")?;
    let mut response = String::new();
    reader
        .take(128)
        .read_line(&mut response)
        .map_err(|_| "installation-input-failed")?;
    Ok(response.ends_with('\n') && matches!(response.trim(), "y" | "Y" | "yes" | "YES"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managed_operation::ManagedSkillsPresetV1;
    use qiongli_content::ProfileId;

    #[test]
    fn content_review_requires_confirmation_and_revalidates_the_original_plan() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/cli-content-review")
            .canonicalize()
            .unwrap_or_else(|_| {
                let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../target/cli-content-review");
                std::fs::create_dir_all(&path).unwrap();
                path.canonicalize().unwrap()
            })
            .join(std::process::id().to_string());
        std::fs::create_dir(&root).unwrap();
        let environment = CommandEnvironment::with_paths(None, Some(root.clone()), None);
        let content = crate::embedded_content().unwrap();
        let command = ManagedOperationCliCommand::PlanSkillsReconcile {
            preset: ManagedSkillsPresetV1::QiongliManaged,
            profile: ProfileId::Full,
        };
        let review = BundledContentReview {
            plan_json: prepare_plan(&command, &environment, &content).unwrap(),
        };
        let mut output = Vec::new();
        for answer in ["\n", "n\n", "yes", "", "not-approval\n"] {
            review
                .review(&environment, &content, &mut answer.as_bytes(), &mut output)
                .unwrap();
            assert!(!root.join(".qiongli-skills").exists());
        }
        review
            .review(&environment, &content, &mut "yes\n".as_bytes(), &mut output)
            .unwrap();
        assert!(root.join(".qiongli-skills").is_dir());
        assert!(
            review
                .review(&environment, &content, &mut "yes\n".as_bytes(), &mut output)
                .is_err()
        );
        let refreshed = BundledContentReview {
            plan_json: prepare_plan(&command, &environment, &content).unwrap(),
        };
        refreshed
            .review(&environment, &content, &mut "y\n".as_bytes(), &mut output)
            .unwrap();
        let unexpected = root.join(".qiongli-skills/user-notes.txt");
        std::fs::write(&unexpected, "retain user content").unwrap();
        assert!(prepare_plan(&command, &environment, &content).is_err());
        assert_eq!(
            std::fs::read_to_string(&unexpected).unwrap(),
            "retain user content"
        );
        let displayed = String::from_utf8(output).unwrap();
        assert!(displayed.contains("Plan digest sha256"));
        assert!(displayed.contains(".qiongli-skills"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
