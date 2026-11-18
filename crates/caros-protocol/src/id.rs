use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

/// Stable opaque identity. Generate once when installing/registering an app,
/// persist it in its manifest, and keep it when its display name changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AppId(Uuid);

impl AppId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
impl Default for AppId {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for AppId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl FromStr for AppId {
    type Err = uuid::Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_round_trip_and_names_are_not_ids() {
        let id = AppId::new();
        assert_eq!(id.to_string().parse::<AppId>().unwrap(), id);
        assert_eq!(
            serde_json::from_str::<AppId>(&serde_json::to_string(&id).unwrap()).unwrap(),
            id
        );
        assert!("carplay".parse::<AppId>().is_err());
        assert!(serde_json::from_str::<AppId>("\"carplay\"").is_err());
        assert_ne!(id, AppId::new());
    }
}
