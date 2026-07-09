use qiongli_lite_mcp::orchestrator::preview::{build_task_plan, TaskPlanInput};

#[test]
fn task_plan_preview_does_not_allow_agent_execution() {
    let plan = build_task_plan(TaskPlanInput {
        task_id: "B1".to_string(),
        paper_type: "systematic-review".to_string(),
        topic: "ai-feedback".to_string(),
    });

    assert_eq!(plan.mode, "preview");
    assert!(plan.preview_only);
    assert_eq!(plan.runtime_profile, "marketplace_lite");
    assert!(!plan.run_agents_allowed);
    assert!(!plan.shell_execution_allowed);
    assert!(!plan.project_writes_allowed);
    assert_eq!(plan.recommended_runtime, "full_cli_for_execution");
    assert!(plan.upgrade.required_for_execution);
    assert_eq!(plan.upgrade.runtime_profile, "full_cli");
    assert_eq!(plan.upgrade.command, "qiongli mcp serve --transport stdio");
}
