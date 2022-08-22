use crate::detector::{NvmeController, NvmePathCache};
use anyhow::anyhow;
use common_lib::transport_api::{ReplyError, ResourceKind};
use grpc::operations::ha_node::{
    server::NodeAgentServer,
    traits::{NodeAgentOperations, ReplacePathInfo},
};
use http::Uri;
use std::{net::SocketAddr, sync::Arc};
use tonic::transport::Server;
//use nvmeadm::ConnectArgsBuilder;
use crate::path_provider::get_nvme_path_buf;
use nvmeadm::nvmf_subsystem::Subsystem;

pub struct NodeAgentApiServer {
    endpoint: SocketAddr,
    path_cache: NvmePathCache,
}

impl NodeAgentApiServer {
    pub fn new(uri: Uri, path_cache: NvmePathCache) -> Self {
        let endpoint = uri.authority().unwrap().to_string().parse().unwrap();

        Self {
            endpoint,
            path_cache,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let r = NodeAgentServer::new(Arc::new(NodeAgentSvc::new(self.path_cache.clone())));
        tracing::info!("Starting gRPC server at {:?}", self.endpoint);
        Server::builder()
            .add_service(r.into_grpc_server())
            .serve(self.endpoint)
            .await
            .map_err(|err| anyhow!("Failed to start gRPC server: {err}"))
    }
}

/// gRPC server implementation for HA Node agent.
struct NodeAgentSvc {
    path_cache: NvmePathCache,
}

impl NodeAgentSvc {
    pub fn new(path_cache: NvmePathCache) -> Self {
        Self { path_cache }
    }
}

/// Disconnect cached NVMe controller.
fn disconnect_controller(ctrlr: &NvmeController) -> Result<(), ReplyError> {
    match get_nvme_path_buf(&ctrlr.path) {
        Some(pbuf) => {
            let subsystem = Subsystem::new(pbuf.as_path()).map_err(|_| {
                ReplyError::internal_error(
                    ResourceKind::Nexus,
                    "gRPC server".to_string(),
                    "Failed to get NVMe subsystem for controller".to_string(),
                )
            })?;

            tracing::info!(
                path=%ctrlr.path,
                "Disconnecting NVMe controller"
            );

            subsystem.disconnect().map_err(|e| {
                ReplyError::internal_error(
                    ResourceKind::Nexus,
                    "gRPC server".to_string(),
                    format!(
                        "Failed to disconnect NVMe controller {}: {:?}",
                        ctrlr.path, e,
                    ),
                )
            })
        }
        None => {
            tracing::error!(
                path=%ctrlr.path,
                "Failed to get system path for controller"
            );

            Err(ReplyError::internal_error(
                ResourceKind::Nexus,
                "gRPC server".to_string(),
                "Failed to get system path for controller".to_string(),
            ))
        }
    }
}

#[tonic::async_trait]
impl NodeAgentOperations for NodeAgentSvc {
    async fn replace_path(&self, request: &dyn ReplacePathInfo) -> Result<(), ReplyError> {
        tracing::info!("Replacing failed NVMe path: {:?}", request);

        let ctrlr = self
            .path_cache
            .lookup_controller(request.target_nqn())
            .await
            .map_err(|_| {
                ReplyError::internal_error(
                    ResourceKind::Nexus,
                    "gRPC server".to_string(),
                    "Failed to lookup controller".to_string(),
                )
            })?;

        // Step 1: populate an aditional healthy path to target NQN in addition to
        // existing failed path. Once this additional path is created, client I/O
        // automatically resumes.

        // Step 2: disconnect broken path.
        disconnect_controller(&ctrlr)
        // Ok(())
    }
}
