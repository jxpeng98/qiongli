use serde::Serialize;

#[derive(Debug, Clone)]
pub struct TaskPlanInput {
    pub task_id: String,
    pub paper_type: String,
    pub topic: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskPlanPreview {
    pub mode: String,
    pub preview_only: bool,
    pub runtime_profile: String,
    pub run_agents_allowed: bool,
    pub shell_execution_allowed: bool,
    pub project_writes_allowed: bool,
    pub recommended_runtime: String,
    pub upgrade: FullUpgradeRecommendation,
    pub task_id: String,
    pub paper_type: String,
    pub topic: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FullUpgradeRecommendation {
    pub required_for_execution: bool,
    pub runtime_profile: String,
    pub command: String,
}

pub fn build_task_plan(input: TaskPlanInput) -> TaskPlanPreview {
    TaskPlanPreview {
        mode: "preview".to_string(),
        preview_only: true,
        runtime_profile: "marketplace_lite".to_string(),
        run_agents_allowed: false,
        shell_execution_allowed: false,
        project_writes_allowed: false,
        recommended_runtime: "full_cli_for_execution".to_string(),
        upgrade: FullUpgradeRecommendation {
            required_for_execution: true,
            runtime_profile: "full_cli".to_string(),
            command: "qiongli mcp serve --transport stdio".to_string(),
        },
        task_id: input.task_id,
        paper_type: input.paper_type,
        topic: input.topic,
    }
}
