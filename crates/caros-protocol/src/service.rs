use crate::{AppId, AppState};
use serde::{Deserialize, Serialize};

/// A Core-owned provider. Running means spawned, not hardware/session readiness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub name: String,
    pub state: AppState,
    pub pid: Option<u32>,
    pub owner: Option<AppId>,
    pub resources: Vec<String>,
}
