use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SetupStep {
    InstallWine,
    CreatePrefix,
    InstallWinetricksComponents,
    InstallGamemode,
    InstallDxvk,
    InstallVkd3d,
    DownloadVortex,
    RegisterUri,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Done,
    Needed,
    Optional,
}

#[derive(Clone, Debug, Serialize)]
pub struct PlannedStep {
    pub step: SetupStep,
    pub label: String,
    pub description: String,
    pub status: StepStatus,
    pub manual_command: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SetupPlan {
    pub steps: Vec<PlannedStep>,
}

impl SetupPlan {
    pub fn needs_setup(&self) -> bool {
        self.steps
            .iter()
            .any(|s| s.status == StepStatus::Needed)
    }
}
