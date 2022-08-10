use super::*;

use serde::{Deserialize, Serialize};

/// Failed NVMe path.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FailedPath {
    pub target_nqn: String,
}

/// Report failed NVMe paths.
#[derive(Serialize, Deserialize, Default, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReportFailedPaths {
    pub node: String,
    pub failed_paths: Vec<FailedPath>,
}
