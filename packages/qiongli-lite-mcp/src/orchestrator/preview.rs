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
    pub runtime_profile: String,
    pub run_agents_allowed: bool,
    pub task_id: String,
    pub paper_type: String,
    pub topic: String,
}

pub fn build_task_plan(input: TaskPlanInput) -> TaskPlanPreview {
    TaskPlanPreview {
        mode: "preview".to_string(),
        runtime_profile: "marketplace_lite".to_string(),
        run_agents_allowed: false,
        task_id: input.task_id,
        paper_type: input.paper_type,
        topic: input.topic,
    }
}
