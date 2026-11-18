use crate::AppId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    Stopped,
    Running,
    Stopping,
    RestartPending,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExitInfo {
    pub code: Option<i32>,
    pub signal: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppStatus {
    pub id: AppId,
    pub name: String,
    pub state: AppState,
    pub pid: Option<u32>,
    pub restarts: u32,
    pub last_exit: Option<ExitInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum EventKind {
    Started { pid: u32 },
    StopRequested,
    ForceKilled,
    Exited(ExitInfo),
    RestartScheduled { attempt: u32 },
    Stopped,
    Failed { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppEvent {
    pub app_id: AppId,
    pub kind: EventKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub sequence: u64,
    pub event: AppEvent,
}
