use eframe::egui::{
    self, Color32, ComboBox, Frame, Grid, Id, Modal, RichText, ScrollArea, TextEdit, Ui,
};
use zeroize::Zeroizing;

use std::time::Duration;

use crate::{
    CapabilityView, DesktopEvent, DesktopIntent, DesktopSection, DesktopService, DesktopSnapshotV1,
    GlobalSettingsPatch, IntegrationSelection, IntegrationTarget, McpSelfTestState,
    McpSelfTestView, OperationKind, OperationPreview, PrivateDisplayText, PrivateText, ProfileKind,
    ProviderKind, ProviderSecretChange, ProviderSettingsPatch, PublicSettingChange,
    SkillsDestinationPreset, StatusCode, UpdatePhaseView, UpdateRemediation, UpdateStreamView,
    UpdateView,
};

const TWO_COLUMN_MINIMUM_WIDTH: f32 = 760.0;
const NAVIGATION_WIDTH: f32 = 184.0;
const CONTENT_MAXIMUM_WIDTH: f32 = 960.0;

#[derive(Clone, Copy)]
struct Feedback {
    status: StatusCode,
    message: &'static str,
    code: &'static str,
}

struct GlobalSettingsEditor {
    revision: u64,
    original_default_profile: ProfileKind,
    default_profile: ProfileKind,
}

struct ProviderSettingsEditor {
    revision: u64,
    providers_enabled: [bool; 5],
    openalex_setting_present: bool,
    crossref_setting_present: bool,
    openalex_email: Zeroizing<String>,
    crossref_email: Zeroizing<String>,
    clear_openalex_email: bool,
    clear_crossref_email: bool,
}

impl GlobalSettingsEditor {
    fn from_snapshot(snapshot: &DesktopSnapshotV1) -> Option<Self> {
        Some(Self {
            revision: snapshot.config.revision?,
            original_default_profile: snapshot.config.default_profile?,
            default_profile: snapshot.config.default_profile?,
        })
    }

    fn patch(&self) -> GlobalSettingsPatch {
        GlobalSettingsPatch {
            expected_revision: self.revision,
            default_profile: self.default_profile,
        }
    }
}

impl ProviderSettingsEditor {
    fn from_snapshot(snapshot: &DesktopSnapshotV1) -> Option<Self> {
        Some(Self {
            revision: snapshot.config.revision?,
            providers_enabled: snapshot.config.providers.map(|provider| provider.enabled),
            openalex_setting_present: snapshot.config.providers[0].public_setting_present,
            crossref_setting_present: snapshot.config.providers[2].public_setting_present,
            openalex_email: Zeroizing::new(String::new()),
            crossref_email: Zeroizing::new(String::new()),
            clear_openalex_email: false,
            clear_crossref_email: false,
        })
    }

    fn patch(&self) -> ProviderSettingsPatch {
        ProviderSettingsPatch {
            expected_revision: self.revision,
            providers_enabled: self.providers_enabled,
            openalex_email: public_setting_change(&self.openalex_email, self.clear_openalex_email),
            crossref_email: public_setting_change(&self.crossref_email, self.clear_crossref_email),
        }
    }
}

fn public_setting_change(value: &str, clear: bool) -> PublicSettingChange {
    if clear {
        PublicSettingChange::Clear
    } else if value.trim().is_empty() {
        PublicSettingChange::Keep
    } else {
        PublicSettingChange::Replace(PrivateText::new(value.to_owned()))
    }
}

pub struct QiongliDesktopApp {
    service: Box<dyn DesktopService>,
    snapshot: DesktopSnapshotV1,
    section: DesktopSection,
    provider: ProviderKind,
    global_settings: Option<GlobalSettingsEditor>,
    provider_settings: Option<ProviderSettingsEditor>,
    provider_secret: Zeroizing<String>,
    skills_profile: ProfileKind,
    skills_preset: SkillsDestinationPreset,
    skills_destination: Option<PrivateDisplayText>,
    integration_selection: IntegrationSelection,
    active_integration: IntegrationTarget,
    mcp_self_test: Option<McpSelfTestView>,
    feedback: Option<Feedback>,
    preview: Option<OperationPreview>,
    show_exact_paths: bool,
    close_requested: bool,
}

impl QiongliDesktopApp {
    #[must_use]
    pub fn new(mut service: Box<dyn DesktopService>) -> Self {
        let mut snapshot = service.snapshot();
        let feedback = snapshot.validate().err().map(|error| {
            quarantine_invalid_snapshot(&mut snapshot);
            Feedback {
                status: StatusCode::Invalid,
                message: "The desktop snapshot was rejected. No operation is available.",
                code: error.code(),
            }
        });
        Self {
            service,
            snapshot,
            section: DesktopSection::Overview,
            provider: ProviderKind::OpenAlex,
            global_settings: None,
            provider_settings: None,
            provider_secret: Zeroizing::new(String::new()),
            skills_profile: ProfileKind::SkillOnly,
            skills_preset: SkillsDestinationPreset::QiongliManaged,
            skills_destination: None,
            integration_selection: IntegrationSelection::ALL,
            active_integration: IntegrationTarget::Codex,
            mcp_self_test: None,
            feedback,
            preview: None,
            show_exact_paths: false,
            close_requested: false,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if self
            .mcp_self_test
            .as_ref()
            .is_some_and(|test| test.state == McpSelfTestState::Running)
        {
            let event = self.service.execute(DesktopIntent::PollLiteMcpSelfTest);
            self.handle_event(event);
            if self
                .mcp_self_test
                .as_ref()
                .is_some_and(|test| test.state == McpSelfTestState::Running)
            {
                ui.ctx().request_repaint_after(Duration::from_millis(50));
            }
        }
        if self.snapshot.update.phase.is_busy() {
            let event = self.service.execute(DesktopIntent::PollUpdate);
            self.handle_event(event);
            if self.snapshot.update.phase.is_busy() {
                ui.ctx().request_repaint_after(Duration::from_millis(100));
            }
        }
        configure_visuals(ui);
        let mut intent = None;
        Frame::central_panel(ui.style())
            .inner_margin(20)
            .show(ui, |ui| {
                render_header(ui, &self.snapshot);
                ui.add_space(10.0);
                ui.separator();
                if let Some(feedback) = self.feedback {
                    ui.add_space(8.0);
                    render_feedback(ui, feedback);
                }
                ui.add_space(16.0);
                if ui.available_width() >= TWO_COLUMN_MINIMUM_WIDTH {
                    ui.horizontal_top(|ui| {
                        navigation_frame(ui).show(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.set_width(NAVIGATION_WIDTH);
                                render_side_navigation(ui, &mut self.section);
                            });
                        });
                        ui.add_space(16.0);
                        ui.vertical(|ui| {
                            ui.set_width(ui.available_width().min(CONTENT_MAXIMUM_WIDTH));
                            intent = self.render_current_view(ui);
                        });
                    });
                } else {
                    render_compact_navigation(ui, &mut self.section);
                    ui.separator();
                    intent = self.render_current_view(ui);
                }

                if intent.is_some() {
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Submitting a typed operation…");
                    });
                }
            });

        if let Some(dialog_intent) = self.render_preview(ui.ctx()) {
            intent = Some(dialog_intent);
        }
        if let Some(intent) = intent {
            self.dispatch(intent);
            ui.ctx().request_repaint();
        }
        if self.close_requested {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn render_current_view(&mut self, ui: &mut Ui) -> Option<DesktopIntent> {
        ScrollArea::vertical()
            .id_salt("desktop-content")
            .auto_shrink([false, false])
            .show(ui, |ui| match self.section {
                DesktopSection::Overview => self.render_overview(ui),
                DesktopSection::Skills => self.render_skills(ui),
                DesktopSection::Mcp => render_mcp(ui, &self.snapshot, self.mcp_self_test.as_ref()),
                DesktopSection::Providers => self.render_providers(ui),
                DesktopSection::Integrations => render_integrations(
                    ui,
                    &self.snapshot,
                    &mut self.integration_selection,
                    &mut self.active_integration,
                ),
                DesktopSection::Settings => self.render_settings(ui),
                DesktopSection::About => self.render_about(ui),
                DesktopSection::Diagnostics => {
                    let (intent, section) =
                        render_diagnostics(ui, &self.snapshot, &mut self.show_exact_paths);
                    if let Some(section) = section {
                        self.section = section;
                    }
                    intent
                }
            })
            .inner
    }

    fn render_overview(&mut self, ui: &mut Ui) -> Option<DesktopIntent> {
        section_heading(
            ui,
            "Overview",
            "Read-only product health and the single recommended next action.",
        );
        show_content_card(ui, |ui| {
            ui.strong("Product health");
            Grid::new("overview-status-grid")
                .num_columns(2)
                .spacing([32.0, 10.0])
                .show(ui, |ui| {
                    ui.label("Embedded content");
                    status_label(ui, self.snapshot.content.status);
                    ui.end_row();
                    ui.label("Global configuration");
                    status_label(ui, self.snapshot.config.status);
                    ui.end_row();
                    ui.label("Lite MCP");
                    status_label(ui, self.snapshot.mcp.status);
                    ui.end_row();
                    ui.label("Apply operations");
                    status_label(
                        ui,
                        if self.snapshot.capabilities.apply {
                            StatusCode::Ready
                        } else {
                            StatusCode::Unavailable
                        },
                    );
                    ui.end_row();
                    for integration in self.snapshot.integrations {
                        ui.label(integration.target.label());
                        status_label(ui, integration.overall);
                        ui.end_row();
                    }
                });
        });
        ui.add_space(12.0);
        show_content_card(ui, |ui| {
            ui.strong("Recommended next action");
            if !matches!(
                self.snapshot.config.status,
                StatusCode::Ready | StatusCode::Missing
            ) {
                ui.label("Open Global Settings and resolve the configuration state.");
                if ui.button("Open Global Settings").clicked() {
                    self.section = DesktopSection::Settings;
                }
            } else if self
                .snapshot
                .integrations
                .iter()
                .any(|integration| integration.overall != StatusCode::Ready)
            {
                ui.label("Open Integrations and install or repair the recommended clients.");
                if ui.button("Open Integrations").clicked() {
                    self.section = DesktopSection::Integrations;
                }
            } else {
                ui.label("No blocking product action is required.");
            }
        });
        ui.add_space(12.0);
        let intent = ui
            .horizontal(|ui| {
                if ui
                    .add_enabled(
                        self.snapshot.capabilities.refresh,
                        egui::Button::new("Refresh overview"),
                    )
                    .clicked()
                {
                    return Some(DesktopIntent::Refresh);
                }
                None
            })
            .inner;
        ui.add_space(12.0);
        egui::CollapsingHeader::new("Release boundary")
            .default_open(false)
            .show(ui, |ui| {
                show_content_card(ui, |ui| {
                    ui.label(if self.snapshot.capabilities.apply {
                        "This trusted release session can activate verified local integrations only after exact preview confirmation. Credentials remain owned by Literature Providers."
                    } else {
                        "This source-build window can edit supported settings and materialize embedded Skills. Product-bound plugin installation still requires a verified packaged session."
                    });
                });
            });
        intent
    }

    fn render_settings(&mut self, ui: &mut Ui) -> Option<DesktopIntent> {
        section_heading(
            ui,
            "Global Settings",
            "Owns product-wide defaults only; literature provider settings live in Literature Providers.",
        );
        show_content_card(ui, |ui| {
            ui.strong("Current global settings");
            Grid::new("current-global-settings-grid")
                .num_columns(2)
                .spacing([24.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Status");
                    status_label(ui, self.snapshot.config.status);
                    ui.end_row();
                    ui.label("Configuration revision");
                    ui.label(
                        self.snapshot.config.revision.map_or_else(
                            || "Unavailable".to_owned(),
                            |revision| revision.to_string(),
                        ),
                    );
                    ui.end_row();
                    ui.label("Active default profile");
                    ui.monospace(
                        self.snapshot
                            .config
                            .default_profile
                            .map_or("Unavailable", ProfileKind::id),
                    );
                    ui.end_row();
                    ui.label("Provider settings");
                    ui.label("Managed separately in Literature Providers");
                    ui.end_row();
                });
        });
        ui.add_space(12.0);
        if self.global_settings.is_none()
            && ui
                .add_enabled(
                    self.snapshot.capabilities.config_edit,
                    egui::Button::new("Edit global settings"),
                )
                .clicked()
        {
            self.global_settings = GlobalSettingsEditor::from_snapshot(&self.snapshot);
            self.feedback = None;
        }
        self.render_global_settings_editor(ui)
    }

    fn render_global_settings_editor(&mut self, ui: &mut Ui) -> Option<DesktopIntent> {
        let Some(editor) = self.global_settings.as_mut() else {
            if !self.snapshot.capabilities.config_edit {
                ui.label("Global settings cannot be edited in the current configuration state.");
            }
            return None;
        };
        ui.add_space(16.0);
        let mut cancel = false;
        let mut preview = false;
        let changed = editor.default_profile != editor.original_default_profile;
        show_content_card(ui, |ui| {
            ui.heading("Global settings");
            ui.label(format!("Editing revision {}", editor.revision));
            ui.label("Literature providers and credentials are intentionally not editable here.");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let label = ui.label("Default profile");
                ComboBox::from_id_salt("global-default-profile")
                    .selected_text(editor.default_profile.id())
                    .show_ui(ui, |ui| {
                        for profile in ProfileKind::ALL {
                            ui.selectable_value(&mut editor.default_profile, profile, profile.id());
                        }
                    })
                    .response
                    .labelled_by(label.id);
            });

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel settings edit").clicked() {
                    cancel = true;
                }
                if ui.button("Preview settings changes").clicked() {
                    preview = true;
                }
            });
            if !changed {
                ui.label("No pending change. Preview will be read-only until the profile changes.");
            }
        });
        if cancel {
            self.global_settings = None;
            self.feedback = Some(Feedback {
                status: StatusCode::Missing,
                message: "The settings edit was cancelled. No changes were made.",
                code: "global-settings-edit-cancelled",
            });
            None
        } else if preview {
            Some(DesktopIntent::PreviewGlobalSettingsPatch(
                self.global_settings
                    .as_ref()
                    .expect("settings editor remains available")
                    .patch(),
            ))
        } else {
            None
        }
    }

    fn render_skills(&mut self, ui: &mut Ui) -> Option<DesktopIntent> {
        section_heading(
            ui,
            "Skills",
            "Advanced management for standalone or custom academic workflow content.",
        );
        show_content_card(ui, |ui| {
            ui.strong("Recommended client installation: Qiongli plugin");
            ui.label(
                "Use Integrations → Install recommended for Codex or Claude Code. The plugin is the installation unit and includes Qiongli Skills plus the dependency-free Lite MCP adapter.",
            );
            if ui.button("Manage client plugins").clicked() {
                self.section = DesktopSection::Integrations;
            }
        });
        ui.add_space(12.0);
        egui::CollapsingHeader::new("Pack and profile details")
            .default_open(false)
            .show(ui, |ui| {
                show_content_card(ui, |ui| {
                    ui.label(format!("Pack: {}", self.snapshot.content.pack_id));
                    ui.label(format!(
                        "Content version: {}",
                        self.snapshot.content.content_version
                    ));
                    ui.label(format!("Entries: {}", self.snapshot.content.entry_count));
                    ui.add_space(8.0);
                    Grid::new("profile-grid")
                        .num_columns(3)
                        .striped(true)
                        .spacing([24.0, 8.0])
                        .show(ui, |ui| {
                            ui.strong("Profile");
                            ui.strong("Resource kinds");
                            ui.strong("Purpose");
                            ui.end_row();
                            for profile in self.snapshot.content.profiles {
                                ui.label(profile.profile.id());
                                ui.label(profile.included_resource_kinds.to_string());
                                ui.label(profile.profile.description());
                                ui.end_row();
                            }
                        });
                });
            });

        ui.add_space(12.0);
        show_content_card(ui, |ui| {
                ui.strong("Standalone Skills installation");
                ui.label(
                    "Use this advanced path only when a client plugin is not the right installation unit.",
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let label = ui.label("Profile to materialize");
                    ComboBox::from_id_salt("skills-profile")
                        .selected_text(self.skills_profile.id())
                        .show_ui(ui, |ui| {
                            for profile in ProfileKind::ALL {
                                ui.selectable_value(
                                    &mut self.skills_profile,
                                    profile,
                                    profile.id(),
                                );
                            }
                        })
                        .response
                        .labelled_by(label.id);
                });
                ui.horizontal(|ui| {
                    let label = ui.label("Destination preset");
                    ComboBox::from_id_salt("skills-destination-preset")
                        .selected_text(self.skills_preset.label())
                        .show_ui(ui, |ui| {
                            for preset in SkillsDestinationPreset::ALL {
                                ui.selectable_value(
                                    &mut self.skills_preset,
                                    preset,
                                    preset.label(),
                                );
                            }
                        })
                        .response
                        .labelled_by(label.id);
                });
                ui.label(format!(
                    "Install method: {}",
                    self.skills_preset.install_method().label()
                ));
                ui.label("Destination");
                if self.skills_preset == SkillsDestinationPreset::CustomFolder {
                    if ui
                        .add_enabled(
                            self.snapshot.capabilities.skills_materialize,
                            egui::Button::new("Choose custom Skills folder"),
                        )
                        .clicked()
                    {
                        self.feedback = None;
                        return Some(DesktopIntent::SelectSkillsDestination);
                    }
                    if let Some(destination) = &self.skills_destination {
                        ui.monospace(destination.expose());
                    } else {
                        ui.label("Choose an empty or Qiongli-managed folder.");
                    }
                } else {
                    ui.monospace(self.skills_preset.symbolic_path());
                }
                ui.add_space(8.0);
                let destination_ready = self.snapshot.capabilities.skills_materialize
                    && (self.skills_preset != SkillsDestinationPreset::CustomFolder
                        || self.skills_destination.is_some());
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(
                            destination_ready,
                            egui::Button::new("Install or update Skills"),
                        )
                        .clicked()
                    {
                        return Some(DesktopIntent::PreviewSkillsPresetMaterialization {
                            profile: self.skills_profile,
                            preset: self.skills_preset,
                        });
                    }
                    if ui
                        .add_enabled(destination_ready, egui::Button::new("Verify Skills"))
                        .clicked()
                    {
                        return Some(DesktopIntent::VerifySkillsPreset {
                            preset: self.skills_preset,
                        });
                    }
                    if ui
                        .add_enabled(
                            destination_ready,
                            egui::Button::new("Remove managed Skills"),
                        )
                        .clicked()
                    {
                        return Some(DesktopIntent::PreviewSkillsPresetRemoval {
                            preset: self.skills_preset,
                        });
                    }
                    None
                })
                .inner
        })
        .inner
    }

    fn render_providers(&mut self, ui: &mut Ui) -> Option<DesktopIntent> {
        section_heading(
            ui,
            "Literature Providers",
            "Owns provider enablement, public settings, credentials, and readiness tests.",
        );
        show_content_card(ui, |ui| {
            ui.strong("Provider status");
            Grid::new("provider-status-grid")
                .num_columns(4)
                .striped(true)
                .spacing([24.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("Provider");
                    ui.strong("Enabled");
                    ui.strong("Readiness");
                    ui.strong("Secret reference");
                    ui.end_row();
                    for provider in self.snapshot.config.providers {
                        ui.label(provider.provider.label());
                        ui.label(if provider.enabled { "Yes" } else { "No" });
                        ui.label(provider.readiness.label());
                        ui.label(if provider.secret_reference_present {
                            "Present (redacted)"
                        } else {
                            "Not present"
                        });
                        ui.end_row();
                    }
                });
        });

        ui.add_space(16.0);
        if self.provider_settings.is_none()
            && ui
                .add_enabled(
                    self.snapshot.capabilities.config_edit,
                    egui::Button::new("Edit literature providers"),
                )
                .clicked()
        {
            self.provider_settings = ProviderSettingsEditor::from_snapshot(&self.snapshot);
        }
        let mut cancel_settings = false;
        let mut preview_settings = false;
        if let Some(editor) = self.provider_settings.as_mut() {
            show_content_card(ui, |ui| {
                ui.heading("Provider settings");
                ui.label(format!("Editing revision {}", editor.revision));
                for (index, provider) in ProviderKind::ALL.into_iter().enumerate() {
                    ui.checkbox(
                        &mut editor.providers_enabled[index],
                        format!("Enable {}", provider.label()),
                    );
                }
                let openalex_label = ui.label("OpenAlex public contact email");
                ui.add_enabled(
                    !editor.clear_openalex_email,
                    TextEdit::singleline(&mut *editor.openalex_email)
                        .id_salt("provider-openalex-email")
                        .hint_text(if editor.openalex_setting_present {
                            "Stored value retained when blank"
                        } else {
                            "researcher@example.org"
                        })
                        .char_limit(320)
                        .desired_width(360.0),
                )
                .labelled_by(openalex_label.id);
                ui.add_enabled(
                    editor.openalex_setting_present,
                    egui::Checkbox::new(
                        &mut editor.clear_openalex_email,
                        "Clear stored OpenAlex public email",
                    ),
                );
                let crossref_label = ui.label("Crossref public contact email");
                ui.add_enabled(
                    !editor.clear_crossref_email,
                    TextEdit::singleline(&mut *editor.crossref_email)
                        .id_salt("provider-crossref-email")
                        .hint_text(if editor.crossref_setting_present {
                            "Stored value retained when blank"
                        } else {
                            "researcher@example.org"
                        })
                        .char_limit(320)
                        .desired_width(360.0),
                )
                .labelled_by(crossref_label.id);
                ui.add_enabled(
                    editor.crossref_setting_present,
                    egui::Checkbox::new(
                        &mut editor.clear_crossref_email,
                        "Clear stored Crossref public email",
                    ),
                );
                ui.horizontal(|ui| {
                    if ui.button("Cancel provider settings").clicked() {
                        cancel_settings = true;
                    }
                    if ui.button("Preview provider settings").clicked() {
                        preview_settings = true;
                    }
                });
            });
        }
        if cancel_settings {
            self.provider_settings = None;
        } else if preview_settings {
            return Some(DesktopIntent::PreviewProviderSettingsPatch(
                self.provider_settings
                    .as_ref()
                    .expect("provider settings editor remains available")
                    .patch(),
            ));
        }

        ui.add_space(20.0);
        let mut intent = None;
        show_content_card(ui, |ui| {
            ui.heading("API credentials");
            ui.label("Credentials are masked and stored outside the configuration document.");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                let label = ui.label("Credential provider");
                ComboBox::from_id_salt("provider-secret-setting")
                    .selected_text(self.provider.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.provider, ProviderKind::OpenAlex, "OpenAlex");
                        ui.selectable_value(
                            &mut self.provider,
                            ProviderKind::SemanticScholar,
                            "Semantic Scholar",
                        );
                    })
                    .response
                    .labelled_by(label.id);
            });
            if !matches!(
                self.provider,
                ProviderKind::OpenAlex | ProviderKind::SemanticScholar
            ) {
                self.provider = ProviderKind::OpenAlex;
            }
            let label = ui.label(format!("{} API key", self.provider.label()));
            ui.add(
                TextEdit::singleline(&mut *self.provider_secret)
                    .id_salt("provider-api-key")
                    .password(true)
                    .hint_text("Enter a new key to save or replace")
                    .char_limit(16 * 1024)
                    .desired_width(360.0),
            )
            .labelled_by(label.id);
            let provider_index = if self.provider == ProviderKind::OpenAlex {
                0
            } else {
                1
            };
            let secret_present =
                self.snapshot.config.providers[provider_index].secret_reference_present;
            let secret_store_ready = self.snapshot.config.secret_store == StatusCode::Ready;
            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(
                        secret_store_ready && !self.provider_secret.is_empty(),
                        egui::Button::new("Save or replace API key"),
                    )
                    .clicked()
                {
                    let value = PrivateText::new(std::mem::take(&mut *self.provider_secret));
                    intent = Some(DesktopIntent::PreviewProviderSecretChange {
                        provider: self.provider,
                        change: ProviderSecretChange::Replace(value),
                    });
                }
                if ui
                    .add_enabled(
                        secret_store_ready && secret_present,
                        egui::Button::new("Remove API key"),
                    )
                    .clicked()
                {
                    intent = Some(DesktopIntent::PreviewProviderSecretChange {
                        provider: self.provider,
                        change: ProviderSecretChange::Remove,
                    });
                }
                if ui.button("Test provider readiness").clicked() {
                    intent = Some(DesktopIntent::TestLiteratureProvider {
                        provider: self.provider,
                    });
                }
            });
        });
        intent
    }

    fn render_about(&mut self, ui: &mut Ui) -> Option<DesktopIntent> {
        section_heading(
            ui,
            "About",
            "Product identity, build target, trust boundary, and unified software update.",
        );
        show_content_card(ui, |ui| {
            ui.strong("Qiongli application");
            Grid::new("about-product-grid")
                .num_columns(2)
                .spacing([32.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Product");
                    ui.label("Qiongli 2");
                    ui.end_row();
                    ui.label("Version");
                    ui.label(&self.snapshot.product.version);
                    ui.end_row();
                    ui.label("Build");
                    ui.monospace(&self.snapshot.product.build);
                    ui.end_row();
                    ui.label("Target");
                    ui.label(format!(
                        "{} · {}",
                        self.snapshot.product.operating_system.label(),
                        self.snapshot.product.architecture.label()
                    ));
                    ui.end_row();
                    ui.label("Trust");
                    ui.label(self.snapshot.product.trust.label());
                    ui.end_row();
                });
            ui.add_space(8.0);
            ui.hyperlink_to("View Qiongli", env!("CARGO_PKG_REPOSITORY"));
        });
        ui.add_space(16.0);
        render_update_card(ui, &self.snapshot.update)
    }

    fn render_preview(&mut self, context: &egui::Context) -> Option<DesktopIntent> {
        let preview = self.preview.as_ref()?;
        let mut intent = None;
        let response = Modal::new(Id::new("operation-preview")).show(context, |ui| {
            ui.set_max_width(460.0);
            ui.heading(preview.title);
            ui.label(preview.summary);
            if let Some(target) = &preview.display_target {
                ui.label("Selected destination");
                ui.monospace(target.expose());
            }
            if let Some(digest) = &preview.plan_digest_sha256 {
                ui.label("Exact plan digest");
                ui.monospace(digest);
            }
            if !preview.approvals_required.is_empty() {
                ui.label("Confirmation approves:");
                for approval in &preview.approvals_required {
                    ui.label(format!("• {}", approval.label()));
                }
            }
            if let Some(reason) = preview.blocked_reason {
                ui.label(format!("Blocked: {reason}"));
            }
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel preview").clicked() {
                    intent = Some(DesktopIntent::CancelOperation {
                        token: preview.token,
                    });
                }
                if ui
                    .add_enabled(preview.can_confirm, egui::Button::new("Confirm operation"))
                    .clicked()
                {
                    intent = Some(DesktopIntent::ConfirmOperation {
                        token: preview.token,
                    });
                }
            });
            if !preview.can_confirm {
                ui.label(blocked_preview_guidance(preview.blocked_reason));
            }
        });
        if intent.is_none() && response.should_close() {
            intent = Some(DesktopIntent::CancelOperation {
                token: preview.token,
            });
        }
        intent
    }

    fn dispatch(&mut self, intent: DesktopIntent) {
        let event = self.service.execute(intent);
        self.handle_event(event);
    }

    fn handle_event(&mut self, event: DesktopEvent) {
        match event {
            DesktopEvent::SnapshotReplaced(snapshot) => match snapshot.validate() {
                Ok(()) => {
                    self.snapshot = *snapshot;
                    self.feedback = Some(Feedback {
                        status: StatusCode::Ready,
                        message: "The read-only desktop snapshot was refreshed.",
                        code: "desktop-snapshot-refreshed",
                    });
                }
                Err(error) => {
                    self.feedback = Some(Feedback {
                        status: StatusCode::Invalid,
                        message: "The refreshed snapshot was rejected. Existing data is unchanged.",
                        code: error.code(),
                    });
                }
            },
            DesktopEvent::McpSelfTestUpdated(self_test) => {
                if !self_test.validate() {
                    self.mcp_self_test = None;
                    self.feedback = Some(Feedback {
                        status: StatusCode::Invalid,
                        message: "The MCP self-test result was rejected.",
                        code: "mcp-self-test-result-invalid",
                    });
                    return;
                }
                let state = self_test.state;
                self.mcp_self_test = Some(self_test);
                self.feedback = match state {
                    McpSelfTestState::Running => None,
                    McpSelfTestState::Passed => Some(Feedback {
                        status: StatusCode::Ready,
                        message: "The bounded Lite MCP self-test passed.",
                        code: "mcp-self-test-passed",
                    }),
                    McpSelfTestState::Failed => Some(Feedback {
                        status: StatusCode::Blocked,
                        message: "The Lite MCP self-test found a blocking failure.",
                        code: "mcp-self-test-failed",
                    }),
                    McpSelfTestState::Cancelled => Some(Feedback {
                        status: StatusCode::Missing,
                        message: "The Lite MCP self-test was cancelled.",
                        code: "mcp-self-test-cancelled",
                    }),
                    McpSelfTestState::TimedOut => Some(Feedback {
                        status: StatusCode::Blocked,
                        message: "The Lite MCP self-test reached its fixed timeout.",
                        code: "mcp-self-test-timed-out",
                    }),
                };
            }
            DesktopEvent::UpdateChanged {
                update,
                close_requested,
            } => {
                if !update.validate() {
                    self.feedback = Some(Feedback {
                        status: StatusCode::Invalid,
                        message: "The update state was rejected. Existing data is unchanged.",
                        code: "desktop-update-state-invalid",
                    });
                    return;
                }
                let phase = update.phase;
                let reason_code = update.reason_code;
                self.preview = None;
                self.snapshot.update = update;
                self.close_requested |= close_requested;
                self.feedback = match phase {
                    UpdatePhaseView::Checking
                    | UpdatePhaseView::Downloading
                    | UpdatePhaseView::Verifying
                    | UpdatePhaseView::Staging
                    | UpdatePhaseView::Installing
                    | UpdatePhaseView::AwaitingRestart
                    | UpdatePhaseView::Cancelling => None,
                    UpdatePhaseView::Current => Some(Feedback {
                        status: StatusCode::Ready,
                        message: "Qiongli is up to date on the selected channel.",
                        code: reason_code,
                    }),
                    UpdatePhaseView::Available => Some(Feedback {
                        status: StatusCode::Attention,
                        message: "A verified Qiongli 2 update is available.",
                        code: reason_code,
                    }),
                    UpdatePhaseView::ReadyToInstall => Some(Feedback {
                        status: StatusCode::Ready,
                        message: "The verified update is ready to install and restart.",
                        code: reason_code,
                    }),
                    UpdatePhaseView::Cancelled => Some(Feedback {
                        status: StatusCode::Missing,
                        message: "The update transaction was cancelled and staged bytes were removed.",
                        code: reason_code,
                    }),
                    UpdatePhaseView::RecoveryRequired => Some(Feedback {
                        status: StatusCode::RecoveryRequired,
                        message: "The update requires recovery before another update can begin.",
                        code: reason_code,
                    }),
                    UpdatePhaseView::Failed => Some(Feedback {
                        status: StatusCode::Blocked,
                        message: "The update did not complete. The installed application is unchanged.",
                        code: reason_code,
                    }),
                    UpdatePhaseView::Unavailable | UpdatePhaseView::Idle => None,
                };
            }
            DesktopEvent::SkillsDestinationSelected { display_path } => {
                self.skills_destination = Some(display_path);
                self.feedback = Some(Feedback {
                    status: StatusCode::Ready,
                    message: "The Skills destination was selected and validated.",
                    code: "skills-destination-selected",
                });
            }
            DesktopEvent::ValidationFailed { code } => {
                self.feedback = Some(Feedback {
                    status: StatusCode::Attention,
                    message: "The request is invalid. No value was stored.",
                    code,
                });
            }
            DesktopEvent::PreviewReady(preview) => {
                if preview.validate() {
                    self.preview = Some(preview);
                    self.feedback = None;
                } else {
                    self.preview = None;
                    self.feedback = Some(Feedback {
                        status: StatusCode::Invalid,
                        message: "The operation preview was rejected. No changes were made.",
                        code: "operation-preview-invalid",
                    });
                }
            }
            DesktopEvent::AgentRunCompleted(_) => {
                self.preview = None;
                self.feedback = Some(Feedback {
                    status: StatusCode::Ready,
                    message: "The bounded project query completed in the native App surface.",
                    code: "agent-run-completed",
                });
            }
            DesktopEvent::Completed { code } => {
                let completed_kind = self.preview.as_ref().map(|preview| preview.kind);
                self.preview = None;
                let snapshot = self.service.snapshot();
                if snapshot.validate().is_ok() {
                    self.snapshot = snapshot;
                    if completed_kind == Some(OperationKind::GlobalSettings) {
                        self.global_settings = None;
                    }
                    if completed_kind == Some(OperationKind::ProviderSettings) {
                        self.provider_settings = None;
                    }
                    if completed_kind == Some(OperationKind::ProviderSecret) {
                        self.provider_secret.clear();
                    }
                    if completed_kind == Some(OperationKind::SkillsRemoval) {
                        self.skills_destination = None;
                    }
                    self.feedback = Some(Feedback {
                        status: StatusCode::Ready,
                        message: "The approved operation completed.",
                        code,
                    });
                } else {
                    self.feedback = Some(Feedback {
                        status: StatusCode::Invalid,
                        message: "The operation completed, but its refreshed snapshot was rejected.",
                        code: "operation-snapshot-invalid",
                    });
                }
            }
            DesktopEvent::Cancelled { code } => {
                self.preview = None;
                self.feedback = Some(Feedback {
                    status: StatusCode::Missing,
                    message: "The preview was cancelled. No changes were made.",
                    code,
                });
            }
            DesktopEvent::Failed { code } => {
                self.preview = None;
                self.feedback = Some(Feedback {
                    status: StatusCode::Blocked,
                    message: "The operation is unavailable. No changes were made.",
                    code,
                });
            }
        }
    }
}

fn blocked_preview_guidance(reason: Option<&str>) -> &'static str {
    match reason {
        Some("source-build-read-only") => {
            "Confirmation is unavailable because this source build has no packaged-product authority."
        }
        Some("packaged-product-replace-required") => {
            "Qiongli preserved the unmanaged installation. Inspect its marketplace path in Diagnostics, then remove or rename the conflicting qiongli-next entry before refreshing discovery."
        }
        Some("packaged-product-recovery-required") => {
            "Complete the pending Qiongli recovery shown in Diagnostics, then refresh discovery before retrying."
        }
        _ => "Resolve the reported blocking condition before retrying this operation.",
    }
}

fn quarantine_invalid_snapshot(snapshot: &mut DesktopSnapshotV1) {
    snapshot.product.version = "unavailable".to_owned();
    snapshot.product.build = "unavailable".to_owned();
    snapshot.content.pack_id = "unavailable".to_owned();
    snapshot.content.content_version = "unavailable".to_owned();
    snapshot.update = UpdateView {
        status: StatusCode::Unavailable,
        selected_stream: UpdateStreamView::Stable,
        phase: UpdatePhaseView::Unavailable,
        available_version: None,
        archive_size_bytes: None,
        progress: None,
        reason_code: "desktop-update-state-invalid",
        remediation: UpdateRemediation::ReinstallApplication,
        can_select_stream: false,
        can_check: false,
        can_prepare: false,
        can_install: false,
        can_cancel: false,
    };
    snapshot.capabilities = CapabilityView {
        refresh: false,
        config_edit: false,
        skills_materialize: false,
        provider_preview: false,
        mcp_self_test: false,
        integration_discovery: false,
        integration_preview: false,
        apply: false,
    };
}

impl eframe::App for QiongliDesktopApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DesktopApplicationMetadata {
    product_name: &'static str,
    window_title: &'static str,
    version: &'static str,
    application_identifier: &'static str,
    license: &'static str,
    startup_error_code: &'static str,
}

impl DesktopApplicationMetadata {
    #[must_use]
    pub const fn new(
        product_name: &'static str,
        window_title: &'static str,
        version: &'static str,
        application_identifier: &'static str,
        license: &'static str,
        startup_error_code: &'static str,
    ) -> Self {
        Self {
            product_name,
            window_title,
            version,
            application_identifier,
            license,
            startup_error_code,
        }
    }

    #[must_use]
    pub const fn product_name(self) -> &'static str {
        self.product_name
    }

    #[must_use]
    pub const fn window_title(self) -> &'static str {
        self.window_title
    }

    #[must_use]
    pub const fn version(self) -> &'static str {
        self.version
    }

    #[must_use]
    pub const fn application_identifier(self) -> &'static str {
        self.application_identifier
    }

    #[must_use]
    pub const fn license(self) -> &'static str {
        self.license
    }

    #[must_use]
    pub const fn startup_error_code(self) -> &'static str {
        self.startup_error_code
    }
}

pub fn run_native_application(
    metadata: DesktopApplicationMetadata,
    service: Box<dyn DesktopService>,
) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(metadata.window_title())
            .with_app_id(metadata.application_identifier())
            .with_icon(native_application_icon())
            .with_inner_size([1_080.0, 720.0])
            .with_min_inner_size([680.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        metadata.product_name(),
        options,
        Box::new(move |_creation_context| Ok(Box::new(QiongliDesktopApp::new(service)))),
    )
}

#[must_use]
pub fn native_application_icon() -> egui::IconData {
    const SIZE: u32 = 64;
    const BACKGROUND: [u8; 4] = [232, 237, 243, 255];
    const ACCENT: [u8; 4] = [11, 102, 94, 255];
    const TRANSPARENT: [u8; 4] = [0, 0, 0, 0];

    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let x = i32::try_from(x).expect("icon coordinate must fit i32");
            let y = i32::try_from(y).expect("icon coordinate must fit i32");
            let corner_x = if x < 12 { 12 } else { 51 };
            let corner_y = if y < 12 { 12 } else { 51 };
            let corner_dx = x - corner_x;
            let corner_dy = y - corner_y;
            let inside_rounded_square = (12..=51).contains(&x)
                || (12..=51).contains(&y)
                || corner_dx * corner_dx + corner_dy * corner_dy <= 64;

            let dx = x - 30;
            let dy = y - 29;
            let distance_squared = dx * dx + dy * dy;
            let q_ring = (196..=400).contains(&distance_squared);
            let q_tail = (39..=53).contains(&x) && (39..=53).contains(&y) && (x - y).abs() <= 3;

            let pixel = if !inside_rounded_square {
                TRANSPARENT
            } else if q_ring || q_tail {
                ACCENT
            } else {
                BACKGROUND
            };
            rgba.extend_from_slice(&pixel);
        }
    }

    egui::IconData {
        rgba,
        width: SIZE,
        height: SIZE,
    }
}

#[derive(Clone, Copy)]
struct ThemePalette {
    canvas: Color32,
    surface: Color32,
    surface_muted: Color32,
    surface_input: Color32,
    text_primary: Color32,
    text_secondary: Color32,
    accent: Color32,
    accent_strong: Color32,
    accent_soft: Color32,
    on_accent: Color32,
    border: Color32,
    warning: Color32,
    warning_soft: Color32,
    danger: Color32,
    neutral_soft: Color32,
    code: Color32,
}

impl ThemePalette {
    fn for_dark_mode(dark_mode: bool) -> Self {
        if dark_mode {
            Self {
                canvas: Color32::from_rgb(17, 24, 32),
                surface: Color32::from_rgb(24, 33, 43),
                surface_muted: Color32::from_rgb(34, 46, 58),
                surface_input: Color32::from_rgb(15, 22, 29),
                text_primary: Color32::from_rgb(245, 247, 250),
                text_secondary: Color32::from_rgb(194, 203, 214),
                accent: Color32::from_rgb(121, 213, 200),
                accent_strong: Color32::from_rgb(11, 102, 94),
                accent_soft: Color32::from_rgb(34, 63, 60),
                on_accent: Color32::WHITE,
                border: Color32::from_rgb(102, 119, 134),
                warning: Color32::from_rgb(253, 186, 116),
                warning_soft: Color32::from_rgb(74, 43, 28),
                danger: Color32::from_rgb(252, 165, 165),
                neutral_soft: Color32::from_rgb(39, 49, 59),
                code: Color32::from_rgb(13, 20, 27),
            }
        } else {
            Self {
                canvas: Color32::from_rgb(243, 245, 247),
                surface: Color32::WHITE,
                surface_muted: Color32::from_rgb(232, 237, 243),
                surface_input: Color32::from_rgb(248, 250, 252),
                text_primary: Color32::from_rgb(17, 24, 39),
                text_secondary: Color32::from_rgb(52, 64, 84),
                accent: Color32::from_rgb(11, 102, 94),
                accent_strong: Color32::from_rgb(11, 102, 94),
                accent_soft: Color32::from_rgb(220, 238, 233),
                on_accent: Color32::WHITE,
                border: Color32::from_rgb(137, 148, 164),
                warning: Color32::from_rgb(181, 71, 8),
                warning_soft: Color32::from_rgb(254, 240, 199),
                danger: Color32::from_rgb(180, 35, 24),
                neutral_soft: Color32::from_rgb(234, 236, 240),
                code: Color32::from_rgb(232, 237, 243),
            }
        }
    }
}

fn theme_palette(ui: &Ui) -> ThemePalette {
    ThemePalette::for_dark_mode(ui.visuals().dark_mode)
}

fn configure_visuals(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
    ui.spacing_mut().button_padding = egui::vec2(14.0, 9.0);
    ui.spacing_mut().interact_size.y = 38.0;
    let palette = theme_palette(ui);
    let visuals = ui.visuals_mut();

    visuals.override_text_color = None;
    visuals.panel_fill = palette.canvas;
    visuals.window_fill = palette.surface;
    visuals.faint_bg_color = palette.surface_muted;
    visuals.extreme_bg_color = palette.surface_input;
    visuals.code_bg_color = palette.code;
    visuals.hyperlink_color = palette.accent;
    visuals.warn_fg_color = palette.warning;
    visuals.error_fg_color = palette.danger;
    visuals.window_stroke = egui::Stroke::new(1.0, palette.border);
    visuals.selection.bg_fill = palette.accent_strong;
    visuals.selection.stroke = egui::Stroke::new(1.0, palette.on_accent);

    visuals.widgets.noninteractive.bg_fill = palette.surface;
    visuals.widgets.noninteractive.weak_bg_fill = palette.surface;
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, palette.border);
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, palette.text_primary);

    visuals.widgets.inactive.bg_fill = palette.surface;
    visuals.widgets.inactive.weak_bg_fill = palette.surface_muted;
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, palette.border);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, palette.text_primary);

    visuals.widgets.hovered.bg_fill = palette.accent_soft;
    visuals.widgets.hovered.weak_bg_fill = palette.accent_soft;
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, palette.accent);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, palette.text_primary);

    visuals.widgets.active.bg_fill = palette.accent_strong;
    visuals.widgets.active.weak_bg_fill = palette.accent_strong;
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, palette.accent_strong);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, palette.on_accent);

    visuals.widgets.open.bg_fill = palette.accent_soft;
    visuals.widgets.open.weak_bg_fill = palette.accent_soft;
    visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, palette.accent);
    visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, palette.text_primary);
}

fn render_header(ui: &mut Ui, snapshot: &DesktopSnapshotV1) {
    let palette = theme_palette(ui);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Qiongli 2")
                .size(24.0)
                .strong()
                .color(palette.text_primary),
        );
        ui.label(
            RichText::new(format!("Alpha · {}", snapshot.product.version))
                .small()
                .strong()
                .color(palette.accent)
                .background_color(palette.accent_soft),
        );
    });
    ui.label(
        RichText::new(format!(
            "Native academic research manager · {} · {}",
            snapshot.product.operating_system.label(),
            snapshot.product.architecture.label()
        ))
        .color(palette.text_secondary),
    );
}

fn render_side_navigation(ui: &mut Ui, section: &mut DesktopSection) {
    let palette = theme_palette(ui);
    ui.label(
        RichText::new("WORKSPACE")
            .small()
            .strong()
            .color(palette.text_secondary),
    );
    ui.add_space(4.0);
    for destination in [
        DesktopSection::Overview,
        DesktopSection::Skills,
        DesktopSection::Mcp,
        DesktopSection::Providers,
        DesktopSection::Integrations,
    ] {
        ui.selectable_value(section, destination, destination.label());
    }
    ui.add_space(16.0);
    ui.label(
        RichText::new("SYSTEM")
            .small()
            .strong()
            .color(palette.text_secondary),
    );
    ui.add_space(4.0);
    for destination in [
        DesktopSection::Settings,
        DesktopSection::Diagnostics,
        DesktopSection::About,
    ] {
        ui.selectable_value(section, destination, destination.label());
    }
}

fn render_compact_navigation(ui: &mut Ui, section: &mut DesktopSection) {
    ui.horizontal(|ui| {
        let label = ui.label("View");
        ComboBox::from_id_salt("desktop-section")
            .selected_text(section.label())
            .show_ui(ui, |ui| {
                for destination in DesktopSection::ALL {
                    ui.selectable_value(section, destination, destination.label());
                }
            })
            .response
            .labelled_by(label.id);
    });
}

fn render_update_card(ui: &mut Ui, update: &UpdateView) -> Option<DesktopIntent> {
    let mut intent = None;
    show_content_card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Software update");
            status_label(ui, update.status);
        });
        ui.label(
            "Update the Qiongli 2 application, embedded Skills, and supported local plugin content as one verified product.",
        );
        ui.add_space(8.0);

        Grid::new("overview-update-grid")
            .num_columns(2)
            .spacing([32.0, 8.0])
            .show(ui, |ui| {
                let channel_label = ui.label("Update channel");
                let mut selected_stream = update.selected_stream;
                let response = ui.add_enabled_ui(update.can_select_stream, |ui| {
                    ComboBox::from_id_salt("overview-update-channel")
                        .selected_text(selected_stream.label())
                        .show_ui(ui, |ui| {
                            for stream in UpdateStreamView::ALL {
                                ui.selectable_value(&mut selected_stream, stream, stream.label());
                            }
                        })
                        .response
                        .labelled_by(channel_label.id)
                });
                if response.inner.changed() {
                    intent = Some(DesktopIntent::SelectUpdateStream {
                        stream: selected_stream,
                    });
                }
                ui.end_row();

                ui.label("Status");
                ui.strong(update.phase.label());
                ui.end_row();

                ui.label("Status code");
                ui.monospace(update.reason_code);
                ui.end_row();

                ui.label("Available version");
                ui.label(update.available_version.as_deref().unwrap_or("Not checked"));
                ui.end_row();

                ui.label("Download size");
                ui.label(
                    update
                        .archive_size_bytes
                        .map(format_update_size)
                        .unwrap_or_else(|| "Not available".to_owned()),
                );
                ui.end_row();
            });

        if let Some(progress) = update.progress {
            ui.add_space(8.0);
            let fraction = if progress.indeterminate {
                0.0
            } else {
                f32::from(progress.completed_steps) / f32::from(progress.total_steps)
            };
            ui.add(
                egui::ProgressBar::new(fraction)
                    .animate(progress.indeterminate)
                    .text(format!(
                        "Step {} of {} · {}",
                        progress.completed_steps, progress.total_steps, progress.label
                    )),
            );
        }

        if update.remediation != UpdateRemediation::None {
            ui.add_space(8.0);
            ui.strong("Next step");
            ui.label(update.remediation.guidance());
            ui.monospace(update.remediation.code());
        }

        ui.add_space(12.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(update.can_check, egui::Button::new("Check for updates"))
                .clicked()
            {
                intent = Some(DesktopIntent::CheckForUpdates);
            }
            let prepare_label = if matches!(
                update.remediation,
                UpdateRemediation::RetryPreparation | UpdateRemediation::CancelAndRetry
            ) || update.phase == UpdatePhaseView::Cancelled
            {
                "Retry update preparation"
            } else {
                "Download and prepare"
            };
            if ui
                .add_enabled(update.can_prepare, egui::Button::new(prepare_label))
                .clicked()
            {
                intent = Some(DesktopIntent::PrepareUpdate);
            }
            if ui
                .add_enabled(update.can_install, egui::Button::new("Install and restart"))
                .clicked()
            {
                intent = Some(DesktopIntent::PreviewUpdateInstall);
            }
            if ui
                .add_enabled(update.can_cancel, egui::Button::new("Cancel update"))
                .clicked()
            {
                intent = Some(DesktopIntent::CancelUpdate);
            }
        });
        ui.label(
            "Stable excludes prereleases. Beta receives eligible Qiongli 2 alpha and beta builds. Qiongli 1.x is not modified.",
        );
    });
    intent
}

fn format_update_size(bytes: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    if bytes >= MIB {
        format!("{:.1} MiB", bytes as f64 / MIB as f64)
    } else {
        format!("{bytes} bytes")
    }
}

fn render_mcp(
    ui: &mut Ui,
    snapshot: &DesktopSnapshotV1,
    self_test: Option<&McpSelfTestView>,
) -> Option<DesktopIntent> {
    section_heading(
        ui,
        "MCP",
        "Dependency-free Lite MCP contract served by the canonical native binary.",
    );
    let running = self_test.is_some_and(|test| test.state == McpSelfTestState::Running);
    let mut intent = None;
    show_content_card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.strong("Lite MCP service");
            status_label(ui, snapshot.mcp.status);
        });
        Grid::new("mcp-grid")
            .num_columns(2)
            .spacing([32.0, 10.0])
            .show(ui, |ui| {
                ui.label("Profile");
                ui.label(snapshot.mcp.profile.id());
                ui.end_row();
                ui.label("Transport");
                ui.label("stdio");
                ui.end_row();
                ui.label("Public tools");
                ui.label(snapshot.mcp.public_tool_count.to_string());
                ui.end_row();
            });
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    snapshot.capabilities.mcp_self_test && !running,
                    egui::Button::new("Run Lite MCP self-test"),
                )
                .clicked()
            {
                intent = Some(DesktopIntent::RunLiteMcpSelfTest);
            }
            if ui
                .add_enabled(running, egui::Button::new("Cancel MCP self-test"))
                .clicked()
            {
                intent = Some(DesktopIntent::CancelLiteMcpSelfTest);
            }
            if running {
                ui.spinner();
                ui.label("Running bounded offline checks…");
            }
        });
        ui.label("The default self-test performs no network request and no mutation.");
        ui.label(
            "Lite MCP protocol health is independent from client registration and activation.",
        );
    });
    ui.add_space(12.0);
    egui::CollapsingHeader::new("Connection details")
        .default_open(false)
        .show(ui, |ui| {
            show_content_card(ui, |ui| {
                ui.monospace("qiongli mcp serve --profile marketplace-lite --transport stdio");
                ui.label(
                    "No Python, Node.js, Rust toolchain, or separate MCP runtime is required after packaging.",
                );
            });
        });

    if let Some(self_test) = self_test {
        ui.add_space(12.0);
        show_content_card(ui, |ui| {
            ui.heading(format!("Self-test: {}", self_test.state.label()));
            Grid::new("mcp-self-test-grid")
                .num_columns(4)
                .striped(true)
                .spacing([24.0, 8.0])
                .show(ui, |ui| {
                    ui.strong("Check");
                    ui.strong("Status");
                    ui.strong("Code");
                    ui.strong("Remediation");
                    ui.end_row();
                    for check in &self_test.checks[..5] {
                        ui.label(check.check.label());
                        status_label(ui, check.status);
                        ui.monospace(check.code);
                        ui.monospace(check.remediation);
                        ui.end_row();
                    }
                });
            let attachment = self_test.checks[5];
            ui.add_space(8.0);
            ui.strong("Client attachment advisory");
            ui.horizontal_wrapped(|ui| {
                ui.label(attachment.check.label());
                status_label(ui, attachment.status);
                ui.monospace(attachment.code);
                ui.monospace(attachment.remediation);
            });
            ui.label(format!(
                "Tools: {} · Providers ready: {}/{} · Clients registered: {}/{} discovered",
                self_test.public_tool_count,
                self_test.ready_provider_count,
                self_test.enabled_provider_count,
                self_test.registered_client_count,
                self_test.discovered_client_count,
            ));
        });
    }
    intent
}

fn render_integrations(
    ui: &mut Ui,
    snapshot: &DesktopSnapshotV1,
    selection: &mut IntegrationSelection,
    active_integration: &mut IntegrationTarget,
) -> Option<DesktopIntent> {
    section_heading(
        ui,
        "Integrations",
        "Local Codex and Claude Code discovery through accepted read-only adapters.",
    );
    let mut intent = None;
    show_content_card(ui, |ui| {
        ui.strong("Recommended setup");
        ui.label(
            "Install the complete Qiongli plugin for supported clients, including Skills and the Lite MCP adapter.",
        );
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    snapshot.capabilities.integration_preview,
                    egui::Button::new("Install recommended"),
                )
                .clicked()
            {
                intent = Some(DesktopIntent::PreviewInstallRecommended);
            }
            if ui
                .add_enabled(
                    snapshot.capabilities.integration_discovery,
                    egui::Button::new("Refresh integration discovery"),
                )
                .clicked()
            {
                intent = Some(DesktopIntent::RefreshIntegrationDiscovery);
            }
        });
        ui.label("Discovery is read-only and does not require a signed release candidate.");
    });
    ui.add_space(16.0);
    ui.strong("Client");
    ui.horizontal_wrapped(|ui| {
        for integration in snapshot.integrations {
            ui.selectable_value(
                active_integration,
                integration.target,
                format!(
                    "{} · {}",
                    integration.target.label(),
                    integration.overall.label()
                ),
            );
        }
    });
    ui.add_space(8.0);

    let integration = snapshot
        .integrations
        .iter()
        .find(|integration| integration.target == *active_integration)
        .expect("validated desktop snapshot must contain every integration target");
    show_content_card(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(integration.target.label())
                    .size(20.0)
                    .strong(),
            );
            status_label(ui, integration.overall);
        });
        ui.label(
            RichText::new(format!(
                "Symbolic location: {}",
                integration.symbolic_location.label()
            ))
            .color(theme_palette(ui).text_secondary),
        );
        ui.add_space(8.0);
        Grid::new(("integration-summary-grid", integration.target.label()))
            .num_columns(2)
            .spacing([24.0, 6.0])
            .show(ui, |ui| {
                ui.label("Discovery");
                ui.label(integration.discovery.label());
                ui.end_row();
                ui.label("Client version");
                ui.label(
                    integration
                        .client_version
                        .map_or_else(|| "Unavailable".to_owned(), |version| version.label()),
                );
                ui.end_row();
                ui.label("Ownership");
                ui.label(integration.ownership.label());
                ui.end_row();
                ui.label("Next safe action");
                ui.label(integration.next_action.label());
                ui.end_row();
            });
        ui.monospace(format!("Evidence: {}", integration.evidence_code));
        if integration.candidate_required {
            ui.label("Candidate required for install");
        } else if integration.discovery == crate::IntegrationDiscoveryState::DiscoveredUnmanaged {
            ui.label("Installation authority is available for this session.");
        }
        ui.add_space(8.0);
        ui.strong("Component health");
        Grid::new(("integration-grid", integration.target.label()))
            .num_columns(2)
            .spacing([24.0, 6.0])
            .show(ui, |ui| {
                ui.label("Client");
                status_label(ui, integration.client);
                ui.end_row();
                ui.label("Plugin source");
                status_label(ui, integration.source);
                ui.end_row();
                ui.label("Skills");
                status_label(ui, integration.skills);
                ui.end_row();
                ui.label("Marketplace");
                status_label(ui, integration.marketplace);
                ui.end_row();
                if let Some(direct_package) = integration.direct_package {
                    ui.label("Direct skills package");
                    status_label(ui, direct_package);
                    ui.end_row();
                }
                ui.label("Registration");
                status_label(ui, integration.registration);
                ui.end_row();
                ui.label("Activation");
                ui.horizontal(|ui| {
                    status_label(ui, integration.activation_status);
                    ui.label(integration.activation.label());
                });
                ui.end_row();
                ui.label("MCP attachment");
                status_label(ui, integration.mcp_attachment);
                ui.end_row();
                ui.label("Overall");
                status_label(ui, integration.overall);
                ui.end_row();
            });
        if integration.path_count > 0 {
            ui.add_space(8.0);
            egui::CollapsingHeader::new("Supported locations")
                .default_open(false)
                .show(ui, |ui| {
                    Grid::new(("integration-paths", integration.target.label()))
                        .num_columns(4)
                        .spacing([16.0, 5.0])
                        .show(ui, |ui| {
                            ui.strong("Surface / scope");
                            ui.strong("Symbolic path");
                            ui.strong("Evidence");
                            ui.strong("State");
                            ui.end_row();
                            for path in integration.paths.into_iter().flatten() {
                                ui.label(format!(
                                    "{} / {}",
                                    path.surface.label(),
                                    path.scope.label()
                                ));
                                ui.monospace(path.symbolic_path);
                                ui.label(format!(
                                    "{} · {}{}",
                                    path.source.label(),
                                    path.management.label(),
                                    if path.selected { " · selected" } else { "" }
                                ));
                                status_label(ui, path.state);
                                ui.end_row();
                            }
                        });
                });
        }
        ui.add_space(12.0);
        let button_label = match integration.next_action {
            crate::IntegrationActionView::InstallReady => "Install this client",
            crate::IntegrationActionView::RepairReady => "Repair this client",
            crate::IntegrationActionView::ResolveConflict => "Inspect conflict",
            crate::IntegrationActionView::Current => "Verify this client",
            crate::IntegrationActionView::InspectOnly => "Inspect this client",
            crate::IntegrationActionView::Unavailable => "Action unavailable",
        };
        if ui
            .add_enabled(
                snapshot.capabilities.integration_preview
                    && integration.next_action != crate::IntegrationActionView::Unavailable,
                egui::Button::new(button_label),
            )
            .clicked()
        {
            intent = Some(DesktopIntent::PreviewIntegration {
                target: integration.target,
            });
        }
    });

    ui.add_space(12.0);
    egui::CollapsingHeader::new("Multi-client maintenance")
        .default_open(false)
        .show(ui, |ui| {
            show_content_card(ui, |ui| {
                ui.label("Select one or both clients for lifecycle maintenance.");
                ui.horizontal_wrapped(|ui| {
                    ui.checkbox(&mut selection.codex, "Codex selected");
                    ui.checkbox(&mut selection.claude_code, "Claude Code selected");
                });
                let selection_ready =
                    !selection.is_empty() && snapshot.capabilities.integration_preview;
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(selection_ready, egui::Button::new("Install selected"))
                        .clicked()
                    {
                        intent = Some(DesktopIntent::PreviewInstallSelected {
                            selection: *selection,
                        });
                    }
                    if ui
                        .add_enabled(selection_ready, egui::Button::new("Verify selected"))
                        .clicked()
                    {
                        intent = Some(DesktopIntent::VerifyIntegrations {
                            selection: *selection,
                        });
                    }
                    if ui
                        .add_enabled(
                            snapshot.capabilities.integration_preview,
                            egui::Button::new("Repair all"),
                        )
                        .clicked()
                    {
                        intent = Some(DesktopIntent::PreviewRepairAll);
                    }
                    if ui
                        .add_enabled(selection_ready, egui::Button::new("Update selected"))
                        .clicked()
                    {
                        intent = Some(DesktopIntent::PreviewUpdateIntegrations {
                            selection: *selection,
                        });
                    }
                    if ui
                        .add_enabled(selection_ready, egui::Button::new("Remove selected"))
                        .clicked()
                    {
                        intent = Some(DesktopIntent::PreviewRemoveIntegrations {
                            selection: *selection,
                        });
                    }
                });
            });
        });
    ui.add_space(12.0);
    ui.label(
        "Claude Desktop, Codex Desktop marketplace bypass, cloud surfaces, and public marketplace publication are not supported by this alpha.",
    );
    intent
}

fn render_diagnostics(
    ui: &mut Ui,
    snapshot: &DesktopSnapshotV1,
    show_exact_paths: &mut bool,
) -> (Option<DesktopIntent>, Option<DesktopSection>) {
    section_heading(
        ui,
        "Diagnostics",
        "Native Product Doctor checks with explicit, source-attributed path inspection.",
    );
    let mut destination = None;
    show_content_card(ui, |ui| {
        ui.strong("Product checks");
        Grid::new("diagnostic-grid")
            .num_columns(6)
            .striped(true)
            .spacing([18.0, 8.0])
            .show(ui, |ui| {
                ui.strong("Check");
                ui.strong("Status");
                ui.strong("Blocking");
                ui.strong("Code");
                ui.strong("Remediation");
                ui.strong("Location");
                ui.end_row();
                for diagnostic in snapshot.diagnostics {
                    ui.label(diagnostic.check.label());
                    status_label(ui, diagnostic.status);
                    ui.label(if diagnostic.blocking { "Yes" } else { "No" });
                    ui.monospace(diagnostic.check.code());
                    ui.monospace(diagnostic.remediation.code());
                    let section = diagnostic.check.section();
                    if section == DesktopSection::Diagnostics {
                        ui.label(section.label());
                    } else if ui.link(section.label()).clicked() {
                        destination = Some(section);
                    }
                    ui.end_row();
                }
            });
    });
    ui.add_space(12.0);
    show_content_card(ui, |ui| {
        ui.strong("Resolved product paths");
        ui.label(format!(
            "{} read-only locations are available. Exact paths are hidden until explicitly requested.",
            snapshot.diagnostic_paths.len()
        ));
        if ui
            .button(if *show_exact_paths {
                "Hide exact paths"
            } else {
                "Show exact paths"
            })
            .clicked()
        {
            *show_exact_paths = !*show_exact_paths;
        }
        if *show_exact_paths {
            ui.label(
                "Exact paths may identify your account or project. Copy and reveal actions occur only from this explicit view.",
            );
        }
    });
    if *show_exact_paths {
        ui.add_space(12.0);
        for path in &snapshot.diagnostic_paths {
            show_content_card(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.strong(&path.label);
                    ui.monospace(format!("[{}]", path.id));
                    status_label(ui, path.status);
                    if path.selected {
                        ui.label("Selected");
                    }
                });
                ui.monospace(path.exact_path.expose());
                ui.label(format!("Symbolic: {}", path.symbolic_path));
                ui.label(&path.details);
                if let Some(target) = path.resolved_target.as_ref() {
                    ui.monospace(format!("Resolved target: {}", target.expose()));
                }
                ui.horizontal(|ui| {
                    if ui.button("Copy exact path").clicked() {
                        ui.ctx().copy_text(path.exact_path.expose().to_owned());
                    }
                    if ui.button("Reveal in file manager").clicked()
                        && let Some(url) = file_manager_url(path.reveal_path.expose())
                    {
                        ui.ctx().open_url(egui::OpenUrl {
                            url,
                            new_tab: false,
                        });
                    }
                });
            });
            ui.add_space(8.0);
        }
    }
    let mut intent = None;
    if ui
        .add_enabled(
            snapshot.capabilities.refresh,
            egui::Button::new("Refresh diagnostics"),
        )
        .clicked()
    {
        intent = Some(DesktopIntent::Refresh);
    }
    (intent, destination)
}

fn file_manager_url(path: &str) -> Option<String> {
    if path.is_empty() || path.chars().any(char::is_control) {
        return None;
    }
    let normalized = path.replace('\\', "/");
    let windows_drive = normalized.as_bytes().get(1) == Some(&b':')
        && normalized
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic);
    if !normalized.starts_with('/') && !windows_drive {
        return None;
    }
    let mut encoded = String::with_capacity(normalized.len());
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for byte in normalized.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    Some(if encoded.starts_with("//") {
        format!("file:{encoded}")
    } else if encoded.starts_with('/') {
        format!("file://{encoded}")
    } else {
        format!("file:///{encoded}")
    })
}

fn section_heading(ui: &mut Ui, title: &str, description: &str) {
    let palette = theme_palette(ui);
    ui.label(
        RichText::new(title)
            .size(24.0)
            .strong()
            .color(palette.text_primary),
    );
    ui.label(
        RichText::new(description)
            .size(14.0)
            .color(palette.text_secondary),
    );
    ui.add_space(14.0);
}

fn navigation_frame(ui: &Ui) -> Frame {
    let palette = theme_palette(ui);
    Frame::group(ui.style())
        .fill(palette.surface_muted)
        .stroke(egui::Stroke::new(1.0, palette.border))
        .corner_radius(10)
        .inner_margin(14)
}

fn content_card(ui: &Ui) -> Frame {
    let palette = theme_palette(ui);
    Frame::group(ui.style())
        .fill(palette.surface)
        .stroke(egui::Stroke::new(1.0, palette.border))
        .corner_radius(10)
        .inner_margin(18)
}

fn show_content_card<R>(
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> egui::InnerResponse<R> {
    content_card(ui).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        add_contents(ui)
    })
}

fn status_label(ui: &mut Ui, status: StatusCode) {
    let palette = theme_palette(ui);
    let (color, background) = if status.requires_attention() {
        (palette.warning, palette.warning_soft)
    } else if status == StatusCode::Ready {
        (palette.accent, palette.accent_soft)
    } else {
        (palette.text_secondary, palette.neutral_soft)
    };
    ui.label(
        RichText::new(status.label())
            .color(color)
            .background_color(background)
            .strong(),
    );
}

fn render_feedback(ui: &mut Ui, feedback: Feedback) {
    ui.horizontal_wrapped(|ui| {
        status_label(ui, feedback.status);
        ui.label(feedback.message);
        ui.monospace(feedback.code);
    });
}

#[cfg(test)]
mod tests {
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    use super::*;
    use crate::model::sample_snapshot;
    use crate::{DesktopEvent, DesktopIntent, OperationToken};

    struct FakeService {
        snapshot: DesktopSnapshotV1,
    }

    #[test]
    fn native_application_icon_is_complete_and_product_coloured() {
        let icon = native_application_icon();
        assert_eq!(icon.width, 64);
        assert_eq!(icon.height, 64);
        assert_eq!(icon.rgba.len(), 64 * 64 * 4);
        assert!(icon.rgba.chunks_exact(4).any(|pixel| pixel == [0, 0, 0, 0]));
        assert!(
            icon.rgba
                .chunks_exact(4)
                .any(|pixel| pixel == [11, 102, 94, 255])
        );
    }

    #[test]
    fn light_theme_combines_polar_neutrals_with_the_jade_brand_accent() {
        let palette = ThemePalette::for_dark_mode(false);

        assert_eq!(palette.canvas, Color32::from_rgb(243, 245, 247));
        assert_eq!(palette.surface, Color32::WHITE);
        assert_eq!(palette.surface_muted, Color32::from_rgb(232, 237, 243));
        assert_eq!(palette.text_primary, Color32::from_rgb(17, 24, 39));
        assert_eq!(palette.text_secondary, Color32::from_rgb(52, 64, 84));
        assert_eq!(palette.border, Color32::from_rgb(137, 148, 164));
        assert_eq!(palette.accent, Color32::from_rgb(11, 102, 94));
        assert_eq!(palette.accent_strong, palette.accent);
    }

    #[test]
    fn visual_tokens_keep_selection_and_attention_text_accessible_in_both_themes() {
        struct ThemeProbe {
            dark_mode: bool,
            panel: Color32,
            card: Color32,
            primary_text: Color32,
            secondary_text: Color32,
            border: Color32,
            accent_text: Color32,
            accent_background: Color32,
            selection_fill: Color32,
            selection_text: Color32,
            attention_text: Color32,
            attention_background: Color32,
        }

        for dark_mode in [false, true] {
            let mut harness = Harness::builder().build_ui_state(
                |ui, probe: &mut ThemeProbe| {
                    ui.style_mut().visuals = if probe.dark_mode {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    };
                    configure_visuals(ui);
                    let palette = theme_palette(ui);
                    probe.panel = ui.visuals().panel_fill;
                    probe.card = palette.surface;
                    probe.primary_text = palette.text_primary;
                    probe.secondary_text = palette.text_secondary;
                    probe.border = palette.border;
                    probe.accent_text = palette.accent;
                    probe.accent_background = palette.accent_soft;
                    probe.selection_fill = ui.visuals().selection.bg_fill;
                    probe.selection_text = ui.visuals().selection.stroke.color;
                    probe.attention_text = ui.visuals().warn_fg_color;
                    probe.attention_background = palette.warning_soft;
                },
                ThemeProbe {
                    dark_mode,
                    panel: Color32::TRANSPARENT,
                    card: Color32::TRANSPARENT,
                    primary_text: Color32::TRANSPARENT,
                    secondary_text: Color32::TRANSPARENT,
                    border: Color32::TRANSPARENT,
                    accent_text: Color32::TRANSPARENT,
                    accent_background: Color32::TRANSPARENT,
                    selection_fill: Color32::TRANSPARENT,
                    selection_text: Color32::TRANSPARENT,
                    attention_text: Color32::TRANSPARENT,
                    attention_background: Color32::TRANSPARENT,
                },
            );
            harness.step();
            let probe = harness.state();
            assert!(
                contrast_ratio(probe.panel, probe.primary_text) >= 7.0,
                "primary text must remain exceptionally clear against the page surface"
            );
            assert!(
                contrast_ratio(probe.panel, probe.secondary_text) >= 4.5,
                "secondary text must remain readable against the page surface"
            );
            assert!(
                contrast_ratio(probe.card, probe.primary_text) >= 7.0,
                "primary text must remain exceptionally clear inside cards"
            );
            assert!(
                contrast_ratio(probe.card, probe.secondary_text) >= 4.5,
                "secondary text must remain readable inside cards"
            );
            assert!(
                contrast_ratio(probe.card, probe.border) >= 3.0,
                "card boundaries must remain visible in both themes"
            );
            assert!(
                contrast_ratio(probe.accent_background, probe.accent_text) >= 4.5,
                "accent badges must retain readable text in both themes"
            );
            assert!(
                contrast_ratio(probe.selection_fill, probe.selection_text) >= 4.5,
                "selected navigation must retain readable text in both themes"
            );
            assert!(
                contrast_ratio(probe.panel, probe.attention_text) >= 4.5,
                "attention text must remain readable against the page surface"
            );
            assert!(
                contrast_ratio(probe.attention_background, probe.attention_text) >= 4.5,
                "attention badges must retain readable text in both themes"
            );
        }
    }

    fn contrast_ratio(first: Color32, second: Color32) -> f32 {
        fn luminance(color: Color32) -> f32 {
            fn channel(value: u8) -> f32 {
                let value = f32::from(value) / 255.0;
                if value <= 0.040_45 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            }
            0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
        }

        let first = luminance(first);
        let second = luminance(second);
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    fn fake_mcp_self_test(state: McpSelfTestState) -> McpSelfTestView {
        use crate::{McpSelfTestCheckId, McpSelfTestCheckView};

        McpSelfTestView {
            state,
            checks: McpSelfTestCheckId::ALL.map(|check| McpSelfTestCheckView {
                check,
                status: match state {
                    McpSelfTestState::Running | McpSelfTestState::Cancelled => StatusCode::Missing,
                    McpSelfTestState::Passed => StatusCode::Ready,
                    McpSelfTestState::Failed => StatusCode::Invalid,
                    McpSelfTestState::TimedOut => StatusCode::Blocked,
                },
                code: match state {
                    McpSelfTestState::Running => "check-pending",
                    McpSelfTestState::Passed => "check-passed",
                    McpSelfTestState::Failed => "check-failed",
                    McpSelfTestState::Cancelled => "check-cancelled",
                    McpSelfTestState::TimedOut => "check-timed-out",
                },
                remediation: "none",
            }),
            public_tool_count: 12,
            enabled_provider_count: 0,
            ready_provider_count: 0,
            discovered_client_count: 0,
            registered_client_count: 0,
        }
    }

    impl DesktopService for FakeService {
        fn snapshot(&mut self) -> DesktopSnapshotV1 {
            self.snapshot.clone()
        }

        fn execute(&mut self, intent: DesktopIntent) -> DesktopEvent {
            match intent {
                DesktopIntent::Refresh
                | DesktopIntent::RefreshIntegrationDiscovery
                | DesktopIntent::RefreshZoteroIntegration => {
                    DesktopEvent::SnapshotReplaced(Box::new(self.snapshot.clone()))
                }
                DesktopIntent::PreviewZoteroCompanionStage => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::ZoteroCompanionStage,
                        title: "Prepare Zotero Companion installation",
                        summary: "A bounded fake Zotero Companion staging preview.",
                        display_target: Some(PrivateDisplayText::new(
                            "<qiongli-state>/zotero/companion".to_owned(),
                        )),
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![crate::OperationApproval::FilesystemWrite],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::RevealZoteroCompanion
                | DesktopIntent::OpenZotero
                | DesktopIntent::VerifyZoteroIntegration => DesktopEvent::Completed {
                    code: "zotero-companion-action-completed",
                },
                DesktopIntent::PrepareLegacyMigration { .. } => DesktopEvent::Completed {
                    code: "legacy-migration-preview-ready",
                },
                DesktopIntent::PreviewLegacyMigrationNext => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::LegacyMigrationStage,
                        title: "Install Qiongli 2.x before migration",
                        summary: "A bounded fake legacy migration preview.",
                        display_target: None,
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![
                            crate::OperationApproval::FilesystemWrite,
                            crate::OperationApproval::ClientConfigChange,
                        ],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::SelectUpdateStream { stream } => {
                    self.snapshot.update.selected_stream = stream;
                    DesktopEvent::UpdateChanged {
                        update: self.snapshot.update.clone(),
                        close_requested: false,
                    }
                }
                DesktopIntent::CheckForUpdates => {
                    self.snapshot.update = available_update();
                    DesktopEvent::UpdateChanged {
                        update: self.snapshot.update.clone(),
                        close_requested: false,
                    }
                }
                DesktopIntent::PrepareUpdate => {
                    self.snapshot.update = ready_update();
                    DesktopEvent::UpdateChanged {
                        update: self.snapshot.update.clone(),
                        close_requested: false,
                    }
                }
                DesktopIntent::PollUpdate => DesktopEvent::UpdateChanged {
                    update: self.snapshot.update.clone(),
                    close_requested: false,
                },
                DesktopIntent::CancelUpdate => {
                    self.snapshot.update = cancelled_update();
                    DesktopEvent::UpdateChanged {
                        update: self.snapshot.update.clone(),
                        close_requested: false,
                    }
                }
                DesktopIntent::PreviewUpdateInstall => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::UpdateInstall,
                        title: "Install Qiongli update",
                        summary: "Quit Qiongli and activate the verified application update.",
                        display_target: None,
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![crate::OperationApproval::FilesystemWrite],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::PreviewCliInstall => DesktopEvent::PreviewReady(OperationPreview {
                    token: OperationToken::new(8),
                    kind: OperationKind::CliInstall,
                    title: "Install Qiongli CLI",
                    summary: "Install the native CLI bundled with this App.",
                    display_target: Some(PrivateDisplayText::new(
                        "<user-home>/.local/bin/qiongli".to_owned(),
                    )),
                    plan_digest_sha256: Some("8".repeat(64)),
                    approvals_required: vec![crate::OperationApproval::FilesystemWrite],
                    can_confirm: true,
                    blocked_reason: None,
                }),
                DesktopIntent::RunLiteMcpSelfTest | DesktopIntent::PollLiteMcpSelfTest => {
                    DesktopEvent::McpSelfTestUpdated(fake_mcp_self_test(McpSelfTestState::Passed))
                }
                DesktopIntent::CancelLiteMcpSelfTest => DesktopEvent::McpSelfTestUpdated(
                    fake_mcp_self_test(McpSelfTestState::Cancelled),
                ),
                DesktopIntent::PreviewGlobalSettingsPatch(_) => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::GlobalSettings,
                        title: "Global settings preview",
                        summary: "A bounded fake settings preview.",
                        display_target: None,
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![crate::OperationApproval::ClientConfigChange],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::PreviewProviderSettingsPatch(_) => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::ProviderSettings,
                        title: "Literature provider settings preview",
                        summary: "A bounded fake provider settings preview.",
                        display_target: None,
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![crate::OperationApproval::ClientConfigChange],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::PreviewProviderSecretChange { .. } => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::ProviderSecret,
                        title: "Provider credential preview",
                        summary: "A bounded fake credential preview.",
                        display_target: None,
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![
                            crate::OperationApproval::SecretStoreWrite,
                            crate::OperationApproval::ClientConfigChange,
                        ],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::PreviewAgentBackendSettingsPatch(_) => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::AgentBackendSettings,
                        title: "Agent backend settings preview",
                        summary: "A bounded fake agent backend settings preview.",
                        display_target: None,
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![crate::OperationApproval::ClientConfigChange],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::PreviewAgentBackendSecretChange { .. } => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::AgentBackendSecret,
                        title: "Agent backend credential preview",
                        summary: "A bounded fake agent backend credential preview.",
                        display_target: None,
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![
                            crate::OperationApproval::SecretStoreWrite,
                            crate::OperationApproval::ClientConfigChange,
                        ],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::PreviewAgentRun(_) => DesktopEvent::PreviewReady(OperationPreview {
                    token: OperationToken::new(7),
                    kind: OperationKind::AgentRun,
                    title: "Agent run preview",
                    summary: "A bounded fake project query preview.",
                    display_target: None,
                    plan_digest_sha256: Some("7".repeat(64)),
                    approvals_required: vec![crate::OperationApproval::NetworkRequest],
                    can_confirm: true,
                    blocked_reason: None,
                }),
                DesktopIntent::TestOpenAiBackend => DesktopEvent::Completed {
                    code: "openai-backend-connection-passed",
                },
                DesktopIntent::TestLiteratureProvider { .. } => DesktopEvent::Completed {
                    code: "literature-provider-ready",
                },
                DesktopIntent::SelectSkillsDestination => DesktopEvent::SkillsDestinationSelected {
                    display_path: PrivateDisplayText::new(
                        "/private/fake-skills-destination".to_owned(),
                    ),
                },
                DesktopIntent::PreviewSkillsMaterialization { .. }
                | DesktopIntent::PreviewSkillsPresetMaterialization { .. } => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::SkillsMaterialization,
                        title: "Skills materialization preview",
                        summary: "A bounded fake Skills preview.",
                        display_target: Some(PrivateDisplayText::new(
                            "/private/fake-skills-destination".to_owned(),
                        )),
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![crate::OperationApproval::FilesystemWrite],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::VerifySkillsMaterialization
                | DesktopIntent::VerifySkillsPreset { .. } => DesktopEvent::Completed {
                    code: "skills-materialization-verified",
                },
                DesktopIntent::PreviewSkillsRemoval
                | DesktopIntent::PreviewSkillsPresetRemoval { .. } => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::SkillsRemoval,
                        title: "Skills removal preview",
                        summary: "A bounded fake Skills removal preview.",
                        display_target: Some(PrivateDisplayText::new(
                            "/private/fake-skills-destination".to_owned(),
                        )),
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: vec![crate::OperationApproval::FilesystemWrite],
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::PreviewProviderPublicSetting { .. } => {
                    DesktopEvent::ValidationFailed {
                        code: "provider-public-setting-invalid",
                    }
                }
                DesktopIntent::PreviewIntegration { .. }
                | DesktopIntent::PreviewInstallRecommended
                | DesktopIntent::PreviewInstallSelected { .. }
                | DesktopIntent::PreviewRepairAll
                | DesktopIntent::PreviewUpdateIntegrations { .. }
                | DesktopIntent::PreviewRemoveIntegrations { .. } => {
                    DesktopEvent::PreviewReady(OperationPreview {
                        token: OperationToken::new(7),
                        kind: OperationKind::Activation,
                        title: "Test operation preview",
                        summary: "A bounded fake service preview.",
                        display_target: None,
                        plan_digest_sha256: Some("7".repeat(64)),
                        approvals_required: crate::OperationApproval::ACTIVATION.to_vec(),
                        can_confirm: true,
                        blocked_reason: None,
                    })
                }
                DesktopIntent::VerifyIntegrations { .. } => DesktopEvent::Completed {
                    code: "packaged-product-install-verified",
                },
                DesktopIntent::ConfirmOperation { token } => {
                    assert_eq!(token, OperationToken::new(7));
                    DesktopEvent::Completed {
                        code: "test-operation-completed",
                    }
                }
                DesktopIntent::CancelOperation { token } => {
                    assert_eq!(token, OperationToken::new(7));
                    DesktopEvent::Cancelled {
                        code: "test-operation-cancelled",
                    }
                }
            }
        }
    }

    fn available_update() -> UpdateView {
        UpdateView {
            status: StatusCode::Attention,
            selected_stream: UpdateStreamView::Beta,
            phase: UpdatePhaseView::Available,
            available_version: Some("2.0.0-alpha.2".to_owned()),
            archive_size_bytes: Some(24 * 1024 * 1024),
            progress: None,
            reason_code: "update-available",
            remediation: UpdateRemediation::None,
            can_select_stream: true,
            can_check: true,
            can_prepare: true,
            can_install: false,
            can_cancel: false,
        }
    }

    fn ready_update() -> UpdateView {
        UpdateView {
            status: StatusCode::Attention,
            selected_stream: UpdateStreamView::Beta,
            phase: UpdatePhaseView::ReadyToInstall,
            available_version: Some("2.0.0-alpha.2".to_owned()),
            archive_size_bytes: Some(24 * 1024 * 1024),
            progress: Some(crate::UpdateProgressView {
                completed_steps: 3,
                total_steps: 4,
                label: "Verified and prepared",
                indeterminate: false,
            }),
            reason_code: "update-ready-to-install",
            remediation: UpdateRemediation::None,
            can_select_stream: false,
            can_check: false,
            can_prepare: false,
            can_install: true,
            can_cancel: true,
        }
    }

    fn cancelled_update() -> UpdateView {
        UpdateView {
            status: StatusCode::Missing,
            selected_stream: UpdateStreamView::Beta,
            phase: UpdatePhaseView::Cancelled,
            available_version: None,
            archive_size_bytes: None,
            progress: None,
            reason_code: "update-cancelled",
            remediation: UpdateRemediation::RetryCheck,
            can_select_stream: true,
            can_check: true,
            can_prepare: false,
            can_install: false,
            can_cancel: false,
        }
    }

    fn failed_update(
        reason_code: &'static str,
        remediation: UpdateRemediation,
        can_cancel: bool,
        can_prepare: bool,
    ) -> UpdateView {
        UpdateView {
            status: StatusCode::Blocked,
            selected_stream: UpdateStreamView::Beta,
            phase: UpdatePhaseView::Failed,
            available_version: Some("2.0.0-alpha.2".to_owned()),
            archive_size_bytes: None,
            progress: None,
            reason_code,
            remediation,
            can_select_stream: !can_cancel && !can_prepare,
            can_check: !can_cancel && !can_prepare,
            can_prepare,
            can_install: false,
            can_cancel,
        }
    }

    #[test]
    fn accesskit_navigation_reaches_all_eight_views() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 720.0], 1.0);
        for (destination, marker) in [
            (
                "Skills",
                "Advanced management for standalone or custom academic workflow content.",
            ),
            (
                "MCP",
                "Dependency-free Lite MCP contract served by the canonical native binary.",
            ),
            (
                "Literature Providers",
                "Owns provider enablement, public settings, credentials, and readiness tests.",
            ),
            (
                "Integrations",
                "Local Codex and Claude Code discovery through accepted read-only adapters.",
            ),
            (
                "Global Settings",
                "Owns product-wide defaults only; literature provider settings live in Literature Providers.",
            ),
            (
                "About",
                "Product identity, build target, trust boundary, and unified software update.",
            ),
            (
                "Diagnostics",
                "Native Product Doctor checks with explicit, source-attributed path inspection.",
            ),
            (
                "Overview",
                "Read-only product health and the single recommended next action.",
            ),
        ] {
            harness.get_by_label(destination).click_accesskit();
            let _ = harness.run();
            assert!(harness.query_all_by_value(marker).next().is_some());
        }
    }

    #[test]
    fn diagnostics_requires_explicit_action_before_rendering_exact_paths() {
        let mut harness = desktop_harness(sample_snapshot(), [1_240.0, 900.0], 1.0);
        harness.get_by_label("Diagnostics").click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("1 read-only locations are available. Exact paths are hidden until explicitly requested.")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_all_by_value("/Users/example/.config/qiongli/v2")
                .next()
                .is_none()
        );
        harness.get_by_label("Show exact paths").click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("/Users/example/.config/qiongli/v2")
                .next()
                .is_some()
        );
        assert!(harness.query_by_label("Copy exact path").is_some());
        assert!(harness.query_by_label("Reveal in file manager").is_some());
    }

    #[test]
    fn diagnostic_file_urls_are_absolute_and_percent_encoded() {
        assert_eq!(
            file_manager_url("/Users/example/My Project"),
            Some("file:///Users/example/My%20Project".to_owned())
        );
        assert_eq!(
            file_manager_url(r"C:\Users\example\My Project"),
            Some("file:///C:/Users/example/My%20Project".to_owned())
        );
        assert_eq!(file_manager_url("relative/path"), None);
        assert_eq!(file_manager_url("/unsafe\npath"), None);
    }

    #[test]
    fn overview_update_card_exposes_channel_progress_and_typed_install_confirmation() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 900.0], 1.0);
        harness.get_by_label("About").click_accesskit();
        let _ = harness.run();
        for value in [
            "Software update",
            "Update channel",
            "Ready to check",
            "Stable excludes prereleases. Beta receives eligible Qiongli 2 alpha and beta builds. Qiongli 1.x is not modified.",
        ] {
            assert!(
                harness.query_all_by_value(value).next().is_some(),
                "missing update card marker: {value}"
            );
        }
        assert!(harness.query_by_label("View Qiongli").is_some());
        assert!(harness.query_by_label("Check for updates").is_some());

        harness.get_by_label("Check for updates").click_accesskit();
        let _ = harness.run();
        assert!(harness.query_all_by_value("2.0.0-alpha.2").next().is_some());
        harness
            .get_by_label("Download and prepare")
            .click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("Ready to install")
                .next()
                .is_some()
        );
        harness
            .get_by_label("Install and restart")
            .click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("Install Qiongli update")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_all_by_value("• Filesystem write")
                .next()
                .is_some()
        );
    }

    #[test]
    fn update_state_matrix_exposes_fixed_status_progress_and_recovery() {
        let cases = [
            (
                UpdateView {
                    status: StatusCode::Ready,
                    selected_stream: UpdateStreamView::Beta,
                    phase: UpdatePhaseView::Current,
                    available_version: Some("2.0.0-alpha.2".to_owned()),
                    archive_size_bytes: None,
                    progress: None,
                    reason_code: "update-current",
                    remediation: UpdateRemediation::None,
                    can_select_stream: true,
                    can_check: true,
                    can_prepare: false,
                    can_install: false,
                    can_cancel: false,
                },
                ["Up to date", "update-current", "2.0.0-alpha.2"],
            ),
            (
                available_update(),
                ["Update available", "update-available", "2.0.0-alpha.2"],
            ),
            (
                failed_update(
                    "native-update-manifest-timeout",
                    UpdateRemediation::RetryCheck,
                    false,
                    false,
                ),
                [
                    "Update failed",
                    "native-update-manifest-timeout",
                    "retry-update-check",
                ],
            ),
            (
                failed_update(
                    "native-update-archive-digest-mismatch",
                    UpdateRemediation::CancelAndRetry,
                    true,
                    false,
                ),
                [
                    "Update failed",
                    "native-update-archive-digest-mismatch",
                    "cancel-update-and-retry",
                ],
            ),
            (
                failed_update(
                    "native-update-manifest-expired",
                    UpdateRemediation::RetryCheck,
                    false,
                    false,
                ),
                [
                    "Update failed",
                    "native-update-manifest-expired",
                    "retry-update-check",
                ],
            ),
            (
                failed_update(
                    "native-update-installation-location-not-writable",
                    UpdateRemediation::MoveToApplications,
                    true,
                    true,
                ),
                [
                    "Update failed",
                    "native-update-installation-location-not-writable",
                    "move-qiongli-to-applications",
                ],
            ),
            (
                UpdateView {
                    status: StatusCode::Busy,
                    selected_stream: UpdateStreamView::Beta,
                    phase: UpdatePhaseView::Cancelling,
                    available_version: Some("2.0.0-alpha.2".to_owned()),
                    archive_size_bytes: Some(24 * 1024 * 1024),
                    progress: Some(crate::UpdateProgressView {
                        completed_steps: 1,
                        total_steps: 4,
                        label: "Removing staged update bytes",
                        indeterminate: true,
                    }),
                    reason_code: "update-cancelling",
                    remediation: UpdateRemediation::None,
                    can_select_stream: false,
                    can_check: false,
                    can_prepare: false,
                    can_install: false,
                    can_cancel: false,
                },
                [
                    "Cancelling",
                    "update-cancelling",
                    "Step 1 of 4 · Removing staged update bytes",
                ],
            ),
            (
                failed_update(
                    "native-update-health-check-failed",
                    UpdateRemediation::ReinstallApplication,
                    false,
                    false,
                ),
                [
                    "Update failed",
                    "native-update-health-check-failed",
                    "reinstall-qiongli",
                ],
            ),
            (
                UpdateView {
                    status: StatusCode::RecoveryRequired,
                    selected_stream: UpdateStreamView::Beta,
                    phase: UpdatePhaseView::RecoveryRequired,
                    available_version: Some("2.0.0-alpha.2".to_owned()),
                    archive_size_bytes: None,
                    progress: None,
                    reason_code: "native-update-recovery-required",
                    remediation: UpdateRemediation::RestartApplication,
                    can_select_stream: false,
                    can_check: false,
                    can_prepare: false,
                    can_install: false,
                    can_cancel: false,
                },
                [
                    "Recovery required",
                    "native-update-recovery-required",
                    "restart-qiongli",
                ],
            ),
            (
                UpdateView {
                    status: StatusCode::Busy,
                    selected_stream: UpdateStreamView::Beta,
                    phase: UpdatePhaseView::AwaitingRestart,
                    available_version: Some("2.0.0-alpha.2".to_owned()),
                    archive_size_bytes: None,
                    progress: Some(crate::UpdateProgressView {
                        completed_steps: 4,
                        total_steps: 4,
                        label: "Completing application replacement",
                        indeterminate: true,
                    }),
                    reason_code: "update-restart-in-progress",
                    remediation: UpdateRemediation::RestartApplication,
                    can_select_stream: false,
                    can_check: false,
                    can_prepare: false,
                    can_install: false,
                    can_cancel: false,
                },
                [
                    "Restarting",
                    "update-restart-in-progress",
                    "Step 4 of 4 · Completing application replacement",
                ],
            ),
        ];

        for (update, markers) in cases {
            assert!(update.validate());
            let mut snapshot = sample_snapshot();
            snapshot.update = update;
            let mut harness = desktop_harness(snapshot, [1_080.0, 920.0], 1.0);
            harness.get_by_label("About").click_accesskit();
            harness.step();
            for marker in markers {
                assert!(
                    harness.query_all_by_value(marker).next().is_some()
                        || harness.query_by_label(marker).is_some(),
                    "missing update state marker: {marker}"
                );
            }
        }
    }

    #[test]
    fn mcp_self_test_control_reports_bounded_checks() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 820.0], 1.0);
        harness.get_by_label("MCP").click_accesskit();
        let _ = harness.run();
        harness
            .get_by_label("Run Lite MCP self-test")
            .click_accesskit();
        let _ = harness.run();

        for value in [
            "Self-test: Passed",
            "Exact tools registry",
            "Offline dispatch",
            "Client attachment advisory",
            "Lite MCP protocol health is independent from client registration and activation.",
            "Tools: 12 · Providers ready: 0/0 · Clients registered: 0/0 discovered",
            "The default self-test performs no network request and no mutation.",
        ] {
            assert!(
                harness.query_all_by_value(value).next().is_some(),
                "missing MCP self-test marker: {value}"
            );
        }
    }

    #[test]
    fn integrations_distinguish_discovery_management_and_authority() {
        let mut snapshot = sample_snapshot();
        snapshot.integrations[0].discovery = crate::IntegrationDiscoveryState::DiscoveredUnmanaged;
        snapshot.integrations[0].candidate_required = true;
        snapshot.integrations[0].client_version = Some(crate::ClientVersionView {
            major: 0,
            minor: 144,
            patch: 4,
        });
        let mut harness = desktop_harness(snapshot, [1_080.0, 820.0], 1.0);
        harness.get_by_label("Integrations").click_accesskit();
        let _ = harness.run();

        let _ = harness.get_by_label("Refresh integration discovery");

        for value in [
            "Discovered but unmanaged",
            "0.144.4",
            "Candidate required for install",
            "Discovery is read-only and does not require a signed release candidate.",
        ] {
            assert!(
                harness.query_all_by_value(value).next().is_some(),
                "missing integration discovery marker: {value}"
            );
        }
    }

    #[test]
    fn integration_lifecycle_actions_and_components_have_accessible_labels() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 900.0], 1.0);
        harness.get_by_label("Integrations").click_accesskit();
        let _ = harness.run();

        for tab in ["Codex · Missing", "Claude Code · Missing"] {
            assert!(harness.query_by_label(tab).is_some(), "missing tab: {tab}");
        }
        harness
            .get_by_label("Multi-client maintenance")
            .click_accesskit();
        let _ = harness.run();

        for label in [
            "Codex selected",
            "Claude Code selected",
            "Install recommended",
            "Install selected",
            "Verify selected",
            "Repair all",
            "Update selected",
            "Remove selected",
        ] {
            assert!(
                harness.query_by_label(label).is_some(),
                "missing action: {label}"
            );
        }
        for value in [
            "Client",
            "Plugin source",
            "Skills",
            "Registration",
            "Activation",
            "MCP attachment",
            "Overall",
        ] {
            assert!(
                harness.query_all_by_value(value).next().is_some(),
                "missing component: {value}"
            );
        }
    }

    #[test]
    fn integrations_use_tabs_and_render_only_the_active_client() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 900.0], 1.0);
        harness.get_by_label("Integrations").click_accesskit();
        let _ = harness.run();

        assert!(
            harness
                .query_all_by_value("Symbolic location: Codex personal marketplace")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_all_by_value("Symbolic location: Claude Code marketplace")
                .next()
                .is_none()
        );

        harness
            .get_by_label("Claude Code · Missing")
            .click_accesskit();
        let _ = harness.run();

        assert!(
            harness
                .query_all_by_value("Symbolic location: Codex personal marketplace")
                .next()
                .is_none()
        );
        assert!(
            harness
                .query_all_by_value("Symbolic location: Claude Code marketplace")
                .next()
                .is_some()
        );
    }

    #[test]
    fn unavailable_integration_action_is_disabled() {
        let mut snapshot = sample_snapshot();
        snapshot.integrations[1].next_action = crate::IntegrationActionView::Unavailable;
        let mut harness = desktop_harness(snapshot, [1_080.0, 900.0], 1.0);
        harness.get_by_label("Integrations").click_accesskit();
        let _ = harness.run();
        harness
            .get_by_label("Claude Code · Missing")
            .click_accesskit();
        let _ = harness.run();

        let action =
            harness.get_by_role_and_label(egui::accesskit::Role::Button, "Action unavailable");
        assert!(action.accesskit_node().is_disabled());
    }

    #[test]
    fn keyboard_activation_reaches_a_navigation_destination() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 720.0], 1.0);
        harness.get_by_label("Skills").focus();
        let _ = harness.run();
        harness.key_press(egui::Key::Enter);
        let _ = harness.run();

        assert!(
            harness
                .query_all_by_value(
                    "Advanced management for standalone or custom academic workflow content.",
                )
                .next()
                .is_some()
        );
    }

    #[test]
    fn compact_navigation_combobox_has_an_accessible_name() {
        let harness = desktop_harness(sample_snapshot(), [680.0, 720.0], 1.0);

        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::ComboBox, "View")
                .is_some()
        );
    }

    #[test]
    fn global_settings_editor_is_labelled_and_uses_typed_confirmation() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 900.0], 1.0);
        harness.get_by_label("Global Settings").click_accesskit();
        let _ = harness.run();
        harness
            .get_by_label("Edit global settings")
            .click_accesskit();
        let _ = harness.run();

        assert!(
            harness
                .query_all_by_value("Current global settings")
                .next()
                .is_some()
        );
        assert!(harness.query_by_label("Active default profile").is_some());
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Default profile")
                .is_some()
        );
        assert!(harness.query_by_label("Enable Crossref").is_none());
        assert!(
            harness
                .query_by_label("OpenAlex public contact email")
                .is_none()
        );
        harness
            .get_by_label("Preview settings changes")
            .click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("Global settings preview")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_all_by_value("• Client configuration change")
                .next()
                .is_some()
        );
        harness.get_by_label("Confirm operation").click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("test-operation-completed")
                .next()
                .is_some()
        );
        assert!(harness.query_by_label("Cancel settings edit").is_none());
    }

    #[test]
    fn literature_providers_owns_enablement_public_settings_and_credentials() {
        let mut snapshot = sample_snapshot();
        snapshot.config.secret_store = StatusCode::Ready;
        let mut harness = desktop_harness(snapshot, [1_080.0, 940.0], 1.0);
        harness
            .get_by_label("Literature Providers")
            .click_accesskit();
        let _ = harness.run();
        harness
            .get_by_label("Edit literature providers")
            .click_accesskit();
        let _ = harness.run();

        assert!(harness.query_by_label("Enable Crossref").is_some());
        assert!(
            harness
                .query_by_label("OpenAlex public contact email")
                .is_some()
        );
        assert!(
            harness
                .query_by_label("Crossref public contact email")
                .is_some()
        );
        assert!(harness.query_by_label("OpenAlex API key").is_some());
        assert!(
            harness
                .query_by_role_and_label(egui::accesskit::Role::ComboBox, "Credential provider")
                .is_some()
        );
        assert!(harness.query_by_label("Default profile").is_none());
        harness.get_by_label("Enable Crossref").click_accesskit();
        let _ = harness.run();
        harness
            .get_by_label("Preview provider settings")
            .click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("Literature provider settings preview")
                .next()
                .is_some()
        );
    }

    #[test]
    fn skills_destination_can_preview_and_verify() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 900.0], 1.0);
        harness.get_by_label("Skills").click_accesskit();
        let _ = harness.run();
        for label in ["Profile to materialize", "Destination preset"] {
            assert!(
                harness
                    .query_by_role_and_label(egui::accesskit::Role::ComboBox, label)
                    .is_some(),
                "missing named Skills combo box: {label}"
            );
        }
        assert!(
            harness
                .query_all_by_value("Recommended client installation: Qiongli plugin")
                .next()
                .is_some()
        );
        assert!(harness.query_by_label("Manage client plugins").is_some());
        assert!(
            harness
                .query_all_by_value("Qiongli Managed")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_all_by_value("Install method: Receipt-owned copy")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_all_by_value("<user-home>/.qiongli-skills")
                .next()
                .is_some()
        );
        harness
            .get_by_label("Install or update Skills")
            .click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("Skills materialization preview")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_all_by_value("• Filesystem write")
                .next()
                .is_some()
        );
        harness.get_by_label("Cancel preview").click_accesskit();
        let _ = harness.run();
        harness.get_by_label("Verify Skills").click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("skills-materialization-verified")
                .next()
                .is_some()
        );
        harness
            .get_by_label("Remove managed Skills")
            .click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("Skills removal preview")
                .next()
                .is_some()
        );
        harness.get_by_label("Confirm operation").click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("<user-home>/.qiongli-skills")
                .next()
                .is_some()
        );
    }

    #[test]
    fn provider_feedback_does_not_echo_transient_input() {
        let mut snapshot = sample_snapshot();
        snapshot.config.secret_store = StatusCode::Ready;
        let mut harness = desktop_harness(snapshot, [1_080.0, 820.0], 1.0);
        harness
            .get_by_label("Literature Providers")
            .click_accesskit();
        let _ = harness.run();
        harness.get_by_label("OpenAlex API key").focus();
        let _ = harness.run();
        harness
            .get_by_label("OpenAlex API key")
            .type_text("private-api-key-canary");
        let _ = harness.run();
        harness
            .get_by_label("Save or replace API key")
            .click_accesskit();
        let _ = harness.run();

        assert!(
            harness
                .query_all_by_value("Provider credential preview")
                .next()
                .is_some()
        );
        let tree = format!("{harness:?}");
        assert!(!tree.contains("private-api-key-canary"));
    }

    #[test]
    fn invalid_initial_snapshot_is_quarantined_without_echoing_dynamic_text() {
        let mut snapshot = sample_snapshot();
        snapshot.product.version = "/private/initial-snapshot-canary".to_owned();

        let harness = desktop_harness(snapshot, [1_080.0, 720.0], 1.0);

        assert_eq!(harness.state().snapshot.product.version, "unavailable");
        assert_eq!(
            harness.state().snapshot.capabilities,
            CapabilityView {
                refresh: false,
                config_edit: false,
                skills_materialize: false,
                provider_preview: false,
                mcp_self_test: false,
                integration_discovery: false,
                integration_preview: false,
                apply: false,
            }
        );
        assert!(
            harness
                .query_all_by_value("The desktop snapshot was rejected. No operation is available.")
                .next()
                .is_some()
        );
        assert!(!format!("{harness:?}").contains("initial-snapshot-canary"));
    }

    #[test]
    fn blocked_packaged_conflict_explains_safe_recovery_without_source_build_claim() {
        let mut app = QiongliDesktopApp::new(Box::new(FakeService {
            snapshot: sample_snapshot(),
        }));
        app.preview = Some(OperationPreview {
            token: OperationToken::new(7),
            kind: OperationKind::Activation,
            title: "Codex packaged installation preview",
            summary: "An unmanaged qiongli-next installation was preserved.",
            display_target: None,
            plan_digest_sha256: None,
            approvals_required: Vec::new(),
            can_confirm: false,
            blocked_reason: Some("packaged-product-replace-required"),
        });
        let harness = Harness::builder()
            .with_size([1_080.0, 720.0])
            .build_ui_state(|ui, app| app.show(ui), app);

        assert!(
            harness
                .query_all_by_value("Qiongli preserved the unmanaged installation. Inspect its marketplace path in Diagnostics, then remove or rename the conflicting qiongli-next entry before refreshing discovery.")
                .next()
                .is_some()
        );
        assert!(
            harness
                .query_by_label("Confirm operation")
                .expect("blocked preview must keep the confirm control visible")
                .accesskit_node()
                .is_disabled()
        );
        assert!(!format!("{harness:?}").contains("source-build alpha"));
    }

    #[test]
    fn typed_preview_can_confirm_and_cancel() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 720.0], 1.0);
        harness.get_by_label("Integrations").click_accesskit();
        let _ = harness.run();
        harness
            .get_by_label("Multi-client maintenance")
            .click_accesskit();
        let _ = harness.run();
        harness.get_by_label("Install selected").click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("Test operation preview")
                .next()
                .is_some()
        );
        assert!(harness.query_all_by_value(&"7".repeat(64)).next().is_some());
        for approval in [
            "• Filesystem write",
            "• Client configuration change",
            "• Host trust",
        ] {
            assert!(harness.query_all_by_value(approval).next().is_some());
        }
        harness.get_by_label("Confirm operation").click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("test-operation-completed")
                .next()
                .is_some()
        );

        harness
            .get_by_label("Install recommended")
            .click_accesskit();
        let _ = harness.run();
        harness.get_by_label("Cancel preview").click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("test-operation-cancelled")
                .next()
                .is_some()
        );
    }

    #[test]
    fn progress_and_recovery_states_are_accessible() {
        let mut snapshot = sample_snapshot();
        snapshot.config.status = StatusCode::RecoveryRequired;
        snapshot.diagnostics[1].status = StatusCode::RecoveryRequired;
        let mut harness = desktop_harness(snapshot, [1_080.0, 720.0], 1.0);
        assert!(
            harness
                .query_all_by_value("Recovery required")
                .next()
                .is_some()
        );

        harness.get_by_label("Refresh overview").click_accesskit();
        harness.step();
        assert!(
            harness
                .query_all_by_value("Submitting a typed operation…")
                .next()
                .is_some()
        );
        harness.step();
        assert!(
            harness
                .query_all_by_value("desktop-snapshot-refreshed")
                .next()
                .is_some()
        );
    }

    #[test]
    fn critical_controls_survive_supported_widths_and_scales() {
        for scale in [1.0, 1.5, 2.0] {
            let normal = desktop_harness(sample_snapshot(), [1_080.0, 720.0], scale);
            assert!(normal.query_by_label("Diagnostics").is_some());
            assert!(normal.query_by_label("Refresh overview").is_some());

            let narrow = desktop_harness(sample_snapshot(), [680.0, 520.0], scale);
            assert!(narrow.query_by_label("View").is_some());
            assert!(narrow.query_by_label("Refresh overview").is_some());
            assert!(narrow.query_by_label("Release boundary").is_some());
        }
    }

    fn desktop_harness(
        snapshot: DesktopSnapshotV1,
        size: [f32; 2],
        pixels_per_point: f32,
    ) -> Harness<'static, QiongliDesktopApp> {
        let app = QiongliDesktopApp::new(Box::new(FakeService { snapshot }));
        Harness::builder()
            .with_size(size)
            .with_pixels_per_point(pixels_per_point)
            .build_ui_state(|ui, app| app.show(ui), app)
    }
}
