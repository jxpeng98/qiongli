use eframe::egui::{
    self, Color32, ComboBox, Frame, Grid, Id, Modal, RichText, ScrollArea, TextEdit, Ui,
};
use zeroize::Zeroizing;

use std::time::Duration;

use crate::{
    CapabilityView, DesktopEvent, DesktopIntent, DesktopSection, DesktopService, DesktopSnapshotV1,
    GlobalSettingsPatch, IntegrationTarget, McpSelfTestState, McpSelfTestView, OperationKind,
    OperationPreview, PrivateDisplayText, PrivateText, ProfileKind, ProviderKind,
    PublicSettingChange, StatusCode,
};

const TWO_COLUMN_MINIMUM_WIDTH: f32 = 760.0;
const NAVIGATION_WIDTH: f32 = 168.0;

#[derive(Clone, Copy)]
struct Feedback {
    status: StatusCode,
    message: &'static str,
    code: &'static str,
}

struct GlobalSettingsEditor {
    revision: u64,
    default_profile: ProfileKind,
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
            default_profile: snapshot.config.default_profile?,
            providers_enabled: snapshot.config.providers.map(|provider| provider.enabled),
            openalex_setting_present: snapshot.config.providers[0].public_setting_present,
            crossref_setting_present: snapshot.config.providers[2].public_setting_present,
            openalex_email: Zeroizing::new(String::new()),
            crossref_email: Zeroizing::new(String::new()),
            clear_openalex_email: false,
            clear_crossref_email: false,
        })
    }

    fn patch(&self) -> GlobalSettingsPatch {
        GlobalSettingsPatch {
            expected_revision: self.revision,
            default_profile: self.default_profile,
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
    public_email: String,
    global_settings: Option<GlobalSettingsEditor>,
    skills_profile: ProfileKind,
    skills_destination: Option<PrivateDisplayText>,
    mcp_self_test: Option<McpSelfTestView>,
    feedback: Option<Feedback>,
    preview: Option<OperationPreview>,
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
            provider: ProviderKind::Crossref,
            public_email: String::new(),
            global_settings: None,
            skills_profile: ProfileKind::SkillOnly,
            skills_destination: None,
            mcp_self_test: None,
            feedback,
            preview: None,
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
        configure_visuals(ui);
        let mut intent = None;
        Frame::central_panel(ui.style()).show(ui, |ui| {
            render_header(ui, &self.snapshot);
            if let Some(feedback) = self.feedback {
                ui.separator();
                render_feedback(ui, feedback);
            }
            ui.add_space(8.0);
            if ui.available_width() >= TWO_COLUMN_MINIMUM_WIDTH {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(NAVIGATION_WIDTH);
                        render_side_navigation(ui, &mut self.section);
                    });
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.set_min_width(ui.available_width());
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
                    ui.label("Working. No background process has been started.");
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
                DesktopSection::Integrations => render_integrations(ui, &self.snapshot),
                DesktopSection::Diagnostics => render_diagnostics(ui, &self.snapshot),
            })
            .inner
    }

    fn render_overview(&mut self, ui: &mut Ui) -> Option<DesktopIntent> {
        section_heading(
            ui,
            "Overview",
            "Inspect the native research platform and manage supported global settings.",
        );
        Frame::group(ui.style()).inner_margin(12).show(ui, |ui| {
            ui.strong("Alpha boundary");
            ui.label(if self.snapshot.capabilities.apply {
                "This trusted release session can activate a verified local plugin only after exact preview confirmation. Secrets and MCP processes remain unavailable."
            } else {
                "This source-build window can edit supported public settings and materialize embedded Skills. Plugin installation still requires a trusted release session."
            });
        });
        ui.add_space(16.0);
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
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.snapshot.capabilities.refresh,
                    egui::Button::new("Refresh overview"),
                )
                .clicked()
            {
                return Some(DesktopIntent::Refresh);
            }
            if ui
                .add_enabled(
                    self.snapshot.capabilities.config_edit,
                    egui::Button::new("Edit global settings"),
                )
                .clicked()
            {
                self.global_settings = GlobalSettingsEditor::from_snapshot(&self.snapshot);
                self.feedback = None;
            }
            None
        })
        .inner
        .or_else(|| self.render_global_settings_editor(ui))
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
        Frame::group(ui.style()).inner_margin(12).show(ui, |ui| {
            ui.heading("Global settings");
            ui.label(format!("Editing revision {}", editor.revision));
            ui.label("Stored secret values and references remain read-only in Alpha.1.");
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Default profile");
                ComboBox::from_id_salt("global-default-profile")
                    .selected_text(editor.default_profile.id())
                    .show_ui(ui, |ui| {
                        for profile in ProfileKind::ALL {
                            ui.selectable_value(&mut editor.default_profile, profile, profile.id());
                        }
                    });
            });

            ui.add_space(8.0);
            ui.strong("Providers");
            for (index, provider) in ProviderKind::ALL.into_iter().enumerate() {
                ui.checkbox(
                    &mut editor.providers_enabled[index],
                    format!("Enable {}", provider.label()),
                );
            }

            ui.add_space(8.0);
            let openalex_label = ui.label("OpenAlex public contact email");
            ui.add_enabled(
                !editor.clear_openalex_email,
                TextEdit::singleline(&mut *editor.openalex_email)
                    .id_salt("global-openalex-email")
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
                    .id_salt("global-crossref-email")
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
            ui.label("Blank replacement fields preserve existing public values.");

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Cancel settings edit").clicked() {
                    cancel = true;
                }
                if ui.button("Preview settings changes").clicked() {
                    preview = true;
                }
            });
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
            "Select, materialize, and verify embedded academic workflow content.",
        );
        ui.label(format!("Pack: {}", self.snapshot.content.pack_id));
        ui.label(format!(
            "Content version: {}",
            self.snapshot.content.content_version
        ));
        ui.label(format!("Entries: {}", self.snapshot.content.entry_count));
        ui.add_space(12.0);
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

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            ui.label("Profile to materialize");
            ComboBox::from_id_salt("skills-profile")
                .selected_text(self.skills_profile.id())
                .show_ui(ui, |ui| {
                    for profile in ProfileKind::ALL {
                        ui.selectable_value(&mut self.skills_profile, profile, profile.id());
                    }
                });
        });
        if ui
            .add_enabled(
                self.snapshot.capabilities.skills_materialize,
                egui::Button::new("Choose Skills destination"),
            )
            .clicked()
        {
            self.feedback = None;
            return Some(DesktopIntent::SelectSkillsDestination);
        }
        if let Some(destination) = &self.skills_destination {
            ui.label("Selected destination");
            ui.monospace(destination.expose());
        } else {
            ui.label("No destination selected. Choose an empty or Qiongli-managed folder.");
        }
        ui.add_space(8.0);
        let destination_ready =
            self.skills_destination.is_some() && self.snapshot.capabilities.skills_materialize;
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    destination_ready,
                    egui::Button::new("Preview Skills materialization"),
                )
                .clicked()
            {
                return Some(DesktopIntent::PreviewSkillsMaterialization {
                    profile: self.skills_profile,
                });
            }
            if ui
                .add_enabled(
                    destination_ready,
                    egui::Button::new("Verify Skills materialization"),
                )
                .clicked()
            {
                return Some(DesktopIntent::VerifySkillsMaterialization);
            }
            if ui
                .add_enabled(
                    destination_ready,
                    egui::Button::new("Remove Skills materialization"),
                )
                .clicked()
            {
                return Some(DesktopIntent::PreviewSkillsRemoval);
            }
            None
        })
        .inner
    }

    fn render_providers(&mut self, ui: &mut Ui) -> Option<DesktopIntent> {
        section_heading(
            ui,
            "Providers",
            "Read-only readiness from the redacted global configuration.",
        );
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

        ui.add_space(24.0);
        ui.heading("Public setting preview");
        ui.label(
            "Validate a public contact email through the typed service boundary. This alpha never stores it.",
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Provider");
            ComboBox::from_id_salt("provider-public-setting")
                .selected_text(self.provider.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.provider, ProviderKind::Crossref, "Crossref");
                    ui.selectable_value(&mut self.provider, ProviderKind::OpenAlex, "OpenAlex");
                });
        });
        let label = ui.label("Public contact email");
        ui.add(
            TextEdit::singleline(&mut self.public_email)
                .id_salt("provider-public-email")
                .hint_text("researcher@example.org")
                .char_limit(320)
                .desired_width(360.0),
        )
        .labelled_by(label.id);
        ui.label("The value is cleared after preview and is never included in feedback or logs.");
        ui.add_space(8.0);
        let enabled = self.snapshot.capabilities.provider_preview;
        if ui
            .add_enabled(enabled, egui::Button::new("Preview provider setting"))
            .clicked()
        {
            self.feedback = None;
            let public_email = PrivateText::new(std::mem::take(&mut self.public_email));
            return Some(DesktopIntent::PreviewProviderPublicSetting {
                provider: self.provider,
                public_email,
            });
        }
        if !enabled {
            ui.label("Provider preview is unavailable in this build.");
        }
        None
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
                ui.label("Confirmation is unavailable in this source-build alpha.");
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
                    self.snapshot = snapshot;
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
            DesktopEvent::Completed { code } => {
                let completed_kind = self.preview.as_ref().map(|preview| preview.kind);
                self.preview = None;
                let snapshot = self.service.snapshot();
                if snapshot.validate().is_ok() {
                    self.snapshot = snapshot;
                    if completed_kind == Some(OperationKind::GlobalSettings) {
                        self.global_settings = None;
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

fn quarantine_invalid_snapshot(snapshot: &mut DesktopSnapshotV1) {
    snapshot.product.version = "unavailable".to_owned();
    snapshot.content.pack_id = "unavailable".to_owned();
    snapshot.content.content_version = "unavailable".to_owned();
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

pub fn run_native(service: Box<dyn DesktopService>) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Qiongli 2")
            .with_inner_size([1_080.0, 720.0])
            .with_min_inner_size([680.0, 520.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Qiongli 2",
        options,
        Box::new(move |_creation_context| Ok(Box::new(QiongliDesktopApp::new(service)))),
    )
}

fn configure_visuals(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
    ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);
    let dark_mode = ui.visuals().dark_mode;
    let accent = if dark_mode {
        Color32::from_rgb(196, 161, 92)
    } else {
        Color32::from_rgb(121, 91, 31)
    };
    ui.visuals_mut().selection.bg_fill = accent;
    ui.visuals_mut().hyperlink_color = accent;
}

fn render_header(ui: &mut Ui, snapshot: &DesktopSnapshotV1) {
    ui.horizontal(|ui| {
        ui.heading("Qiongli 2");
        ui.label(format!("Alpha · {}", snapshot.product.version));
    });
    ui.label(format!(
        "Native academic research manager · {} · {}",
        snapshot.product.operating_system.label(),
        snapshot.product.architecture.label()
    ));
}

fn render_side_navigation(ui: &mut Ui, section: &mut DesktopSection) {
    ui.strong("Workspace");
    ui.add_space(4.0);
    for destination in DesktopSection::ALL {
        ui.selectable_value(section, destination, destination.label());
    }
}

fn render_compact_navigation(ui: &mut Ui, section: &mut DesktopSection) {
    ui.horizontal(|ui| {
        ui.label("View");
        ComboBox::from_id_salt("desktop-section")
            .selected_text(section.label())
            .show_ui(ui, |ui| {
                for destination in DesktopSection::ALL {
                    ui.selectable_value(section, destination, destination.label());
                }
            });
    });
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
    Grid::new("mcp-grid")
        .num_columns(2)
        .spacing([32.0, 10.0])
        .show(ui, |ui| {
            ui.label("Status");
            status_label(ui, snapshot.mcp.status);
            ui.end_row();
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
    ui.add_space(12.0);
    ui.monospace("qiongli mcp serve --profile marketplace-lite --transport stdio");
    ui.label(
        "No Python, Node.js, Rust toolchain, or separate MCP runtime is required after packaging.",
    );
    ui.add_space(12.0);
    let running = self_test.is_some_and(|test| test.state == McpSelfTestState::Running);
    let mut intent = None;
    ui.horizontal(|ui| {
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

    if let Some(self_test) = self_test {
        ui.add_space(12.0);
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
                for check in self_test.checks {
                    ui.label(check.check.label());
                    status_label(ui, check.status);
                    ui.monospace(check.code);
                    ui.monospace(check.remediation);
                    ui.end_row();
                }
            });
        ui.label(format!(
            "Tools: {} · Providers ready: {}/{} · Clients registered: {}/{} discovered",
            self_test.public_tool_count,
            self_test.ready_provider_count,
            self_test.enabled_provider_count,
            self_test.registered_client_count,
            self_test.discovered_client_count,
        ));
    }
    intent
}

fn render_integrations(ui: &mut Ui, snapshot: &DesktopSnapshotV1) -> Option<DesktopIntent> {
    section_heading(
        ui,
        "Integrations",
        "Local Codex and Claude Code discovery through accepted read-only adapters.",
    );
    let mut intent = None;
    if ui
        .add_enabled(
            snapshot.capabilities.integration_discovery,
            egui::Button::new("Refresh integration discovery"),
        )
        .clicked()
    {
        return Some(DesktopIntent::RefreshIntegrationDiscovery);
    }
    ui.label("Discovery is read-only and does not require a signed release candidate.");
    ui.add_space(12.0);
    for integration in snapshot.integrations {
        Frame::group(ui.style()).inner_margin(12).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(integration.target.label());
                status_label(ui, integration.overall);
            });
            ui.label(format!(
                "Symbolic location: {}",
                integration.symbolic_location.label()
            ));
            ui.strong(integration.discovery.label());
            if integration.candidate_required {
                ui.label("Candidate required for install");
            } else if integration.discovery == crate::IntegrationDiscoveryState::DiscoveredUnmanaged
            {
                ui.label("Installation authority is available for this session.");
            }
            Grid::new(("integration-grid", integration.target.label()))
                .num_columns(2)
                .spacing([24.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Plugin source");
                    status_label(ui, integration.source);
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
                    ui.label(integration.activation.label());
                    ui.end_row();
                });
            let button_label = match integration.target {
                IntegrationTarget::Codex => "Preview Codex installation",
                IntegrationTarget::ClaudeCode => "Preview Claude Code installation",
            };
            if ui
                .add_enabled(
                    snapshot.capabilities.integration_preview,
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
    }
    ui.label(
        "Claude Desktop, Codex Desktop marketplace bypass, cloud surfaces, and public marketplace publication are not supported by this alpha.",
    );
    intent
}

fn render_diagnostics(ui: &mut Ui, snapshot: &DesktopSnapshotV1) -> Option<DesktopIntent> {
    section_heading(
        ui,
        "Diagnostics",
        "Fixed, path-free health checks and remediation codes.",
    );
    Grid::new("diagnostic-grid")
        .num_columns(4)
        .striped(true)
        .spacing([24.0, 8.0])
        .show(ui, |ui| {
            ui.strong("Check");
            ui.strong("Status");
            ui.strong("Blocking");
            ui.strong("Remediation");
            ui.end_row();
            for diagnostic in snapshot.diagnostics {
                ui.label(diagnostic.check.label());
                status_label(ui, diagnostic.status);
                ui.label(if diagnostic.blocking { "Yes" } else { "No" });
                ui.monospace(diagnostic.remediation.code());
                ui.end_row();
            }
        });
    ui.add_space(12.0);
    if ui
        .add_enabled(
            snapshot.capabilities.refresh,
            egui::Button::new("Refresh diagnostics"),
        )
        .clicked()
    {
        return Some(DesktopIntent::Refresh);
    }
    None
}

fn section_heading(ui: &mut Ui, title: &str, description: &str) {
    ui.heading(title);
    ui.label(description);
    ui.add_space(12.0);
}

fn status_label(ui: &mut Ui, status: StatusCode) {
    let color = if status.requires_attention() {
        ui.visuals().warn_fg_color
    } else if status == StatusCode::Ready {
        ui.visuals().hyperlink_color
    } else {
        ui.visuals().weak_text_color()
    };
    ui.label(RichText::new(status.label()).color(color).strong());
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
    use egui_kittest::{Harness, kittest::Queryable};

    use super::*;
    use crate::model::sample_snapshot;
    use crate::{DesktopEvent, DesktopIntent, OperationToken};

    struct FakeService {
        snapshot: DesktopSnapshotV1,
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
                DesktopIntent::Refresh | DesktopIntent::RefreshIntegrationDiscovery => {
                    DesktopEvent::SnapshotReplaced(self.snapshot.clone())
                }
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
                DesktopIntent::SelectSkillsDestination => DesktopEvent::SkillsDestinationSelected {
                    display_path: PrivateDisplayText::new(
                        "/private/fake-skills-destination".to_owned(),
                    ),
                },
                DesktopIntent::PreviewSkillsMaterialization { .. } => {
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
                DesktopIntent::VerifySkillsMaterialization => DesktopEvent::Completed {
                    code: "skills-materialization-verified",
                },
                DesktopIntent::PreviewSkillsRemoval => {
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
                DesktopIntent::PreviewIntegration { .. } => {
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

    #[test]
    fn accesskit_navigation_reaches_all_six_views() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 720.0], 1.0);
        for (destination, marker) in [
            (
                "Skills",
                "Select, materialize, and verify embedded academic workflow content.",
            ),
            (
                "MCP",
                "Dependency-free Lite MCP contract served by the canonical native binary.",
            ),
            (
                "Providers",
                "Read-only readiness from the redacted global configuration.",
            ),
            (
                "Integrations",
                "Local Codex and Claude Code discovery through accepted read-only adapters.",
            ),
            (
                "Diagnostics",
                "Fixed, path-free health checks and remediation codes.",
            ),
            (
                "Overview",
                "Inspect the native research platform and manage supported global settings.",
            ),
        ] {
            harness.get_by_label(destination).click_accesskit();
            let _ = harness.run();
            assert!(harness.query_all_by_value(marker).next().is_some());
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
        let mut harness = desktop_harness(snapshot, [1_080.0, 820.0], 1.0);
        harness.get_by_label("Integrations").click_accesskit();
        let _ = harness.run();

        let _ = harness.get_by_label("Refresh integration discovery");

        for value in [
            "Discovered but unmanaged",
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
    fn keyboard_activation_reaches_a_navigation_destination() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 720.0], 1.0);
        harness.get_by_label("Skills").focus();
        let _ = harness.run();
        harness.key_press(egui::Key::Enter);
        let _ = harness.run();

        assert!(
            harness
                .query_all_by_value(
                    "Select, materialize, and verify embedded academic workflow content.",
                )
                .next()
                .is_some()
        );
    }

    #[test]
    fn global_settings_editor_is_labelled_and_uses_typed_confirmation() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 900.0], 1.0);
        harness
            .get_by_label("Edit global settings")
            .click_accesskit();
        let _ = harness.run();

        assert!(harness.query_by_label("Default profile").is_some());
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

        harness.get_by_label("Enable Crossref").click_accesskit();
        let _ = harness.run();
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
    fn skills_destination_can_preview_and_verify() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 900.0], 1.0);
        harness.get_by_label("Skills").click_accesskit();
        let _ = harness.run();
        harness
            .get_by_label("Choose Skills destination")
            .click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("/private/fake-skills-destination")
                .next()
                .is_some()
        );
        harness
            .get_by_label("Preview Skills materialization")
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
        harness
            .get_by_label("Verify Skills materialization")
            .click_accesskit();
        let _ = harness.run();
        assert!(
            harness
                .query_all_by_value("skills-materialization-verified")
                .next()
                .is_some()
        );
        harness
            .get_by_label("Remove Skills materialization")
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
                .query_all_by_value(
                    "No destination selected. Choose an empty or Qiongli-managed folder.",
                )
                .next()
                .is_some()
        );
    }

    #[test]
    fn provider_feedback_does_not_echo_transient_input() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 720.0], 1.0);
        harness.get_by_label("Providers").click_accesskit();
        let _ = harness.run();
        harness.get_by_label("Public contact email").focus();
        let _ = harness.run();
        harness
            .get_by_label("Public contact email")
            .type_text("private-canary@example.org");
        let _ = harness.run();
        harness
            .get_by_label("Preview provider setting")
            .click_accesskit();
        let _ = harness.run();

        let feedback = harness.state().feedback.expect("feedback must be set");
        assert_eq!(feedback.code, "provider-public-setting-invalid");
        assert!(
            harness
                .query_all_by_value("The request is invalid. No value was stored.")
                .next()
                .is_some()
        );
        let tree = format!("{harness:?}");
        assert!(!tree.contains("private-canary@example.org"));
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
    fn typed_preview_can_confirm_and_cancel() {
        let mut harness = desktop_harness(sample_snapshot(), [1_080.0, 720.0], 1.0);
        harness.get_by_label("Integrations").click_accesskit();
        let _ = harness.run();
        harness
            .get_by_label("Preview Codex installation")
            .click_accesskit();
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
            .get_by_label("Preview Claude Code installation")
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
                .query_all_by_value("Working. No background process has been started.")
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
            assert!(narrow.query_by_label("Alpha boundary").is_some());
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
