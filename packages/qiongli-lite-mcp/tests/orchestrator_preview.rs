use qiongli_lite_mcp::orchestrator::preview::{build_task_plan, TaskPlanInput};

#[test]
fn task_plan_preview_does_not_allow_agent_execution() {
    let plan = build_task_plan(TaskPlanInput {
        task_id: "B1".to_string(),
        paper_type: "systematic-review".to_string(),
        topic: "ai-feedback".to_string(),
    });

    assert_eq!(plan.mode, "preview");
    assert_eq!(plan.runtime_profile, "marketplace_lite");
    assert!(!plan.run_agents_allowed);
}

