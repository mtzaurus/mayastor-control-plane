use crate::ha_cluster_agent::{FailedNvmePath, ReportFailedNvmePathsRequest};
use common_lib::transport_api::ReplyError;
use tonic::transport::Uri;

use common_lib::types::v0::transport::{FailedPath, ReportFailedPaths};

/// ClusterAgentOperations trait implemented by client which supports cluster-agent operations
#[tonic::async_trait]
pub trait ClusterAgentOperations: Send + Sync {
    /// Register node with cluster-agent
    async fn register(&self, node_name: String, endpoint: Uri) -> Result<(), ReplyError>;

    /// Report failed NVMe paths.
    async fn report_failed_nvme_paths(
        &self,
        request: &dyn ReportFailedPathsInfo,
    ) -> Result<(), ReplyError>;
}

/// Trait to be implemented for ReportFailedNvmePaths operation.
pub trait ReportFailedPathsInfo: Send + Sync + std::fmt::Debug {
    /// Id of the io-engine instance
    fn node(&self) -> String;

    /// List of failed NVMe paths.
    fn failed_paths(&self) -> Vec<FailedPath>;
}

impl ReportFailedPathsInfo for ReportFailedPaths {
    fn node(&self) -> String {
        self.node.clone()
    }

    fn failed_paths(&self) -> Vec<FailedPath> {
        self.failed_paths.clone()
    }
}

impl From<&dyn ReportFailedPathsInfo> for ReportFailedNvmePathsRequest {
    fn from(info: &dyn ReportFailedPathsInfo) -> Self {
        Self {
            nodename: info.node(),
            failed_paths: info.failed_paths().into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl From<FailedPath> for FailedNvmePath {
    fn from(path: FailedPath) -> Self {
        Self {
            target_nqn: path.target_nqn,
        }
    }
}
