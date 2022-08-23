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
use nvmeadm::{
    nvmf_discovery::{ConnectArgs, ConnectArgsBuilder},
    nvmf_subsystem::Subsystem,
};
use utils::NVME_TARGET_NQN_PREFIX;

/// Common error source name for all gRPC errors in HA Node agent.
const HA_AGENT_ERR_SOURCE: &str = "HA Node agent gRPC server";

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
                    HA_AGENT_ERR_SOURCE.to_string(),
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
                    HA_AGENT_ERR_SOURCE.to_string(),
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
                HA_AGENT_ERR_SOURCE.to_string(),
                "Failed to get system path for controller".to_string(),
            ))
        }
    }
}

impl NodeAgentSvc {
    fn get_nvmf_connection_args(&self, new_path: &str) -> Option<ConnectArgs> {
        let uri = new_path.parse::<Uri>().ok()?;

        let host = uri.host()?;
        let port = uri.port()?.to_string();
        let nqn = &uri.path()[1 ..];

        // Check NQN of the subsystem to make sure it belongs to the product.
        if !nqn.starts_with(NVME_TARGET_NQN_PREFIX) {
            return None;
        }

        ConnectArgsBuilder::default()
            .traddr(host)
            .trsvcid(port)
            .nqn(nqn)
            .build()
            .ok()
    }
}

#[tonic::async_trait]
impl NodeAgentOperations for NodeAgentSvc {
    async fn replace_path(&self, request: &dyn ReplacePathInfo) -> Result<(), ReplyError> {
        tracing::info!("Replacing failed NVMe path: {:?}", request);

        // Parse URI in advance to make sure it's well-formed before using it.
        let connect_args = match self.get_nvmf_connection_args(&request.new_path()) {
            Some(ca) => ca,
            None => {
                return Err(ReplyError::invalid_argument(
                    ResourceKind::Nexus,
                    "new_path",
                    HA_AGENT_ERR_SOURCE.to_string(),
                ))
            }
        };

        // Lookup NVMe controller whose path has failed.
        let ctrlr = self
            .path_cache
            .lookup_controller(request.target_nqn())
            .await
            .map_err(|_| {
                ReplyError::internal_error(
                    ResourceKind::Nexus,
                    HA_AGENT_ERR_SOURCE.to_string(),
                    "Failed to lookup controller".to_string(),
                )
            })?;

        // Step 1: populate an aditional healthy path to target NQN in addition to
        // existing failed path. Once this additional path is created, client I/O
        // automatically resumes.
        tracing::info!(uri=%request.new_path(), "Connecting to NVMe target");
        match connect_args.connect() {
            Ok(_) => {
                tracing::info!(uri=%request.new_path(), "Successfully connected to NVMe target");
            }
            Err(error) => {
                tracing::error!(
                    uri=%request.new_path(),
                    error=%error,
                    "Failed to connect to NVMe target"
                );
                return Err(ReplyError::internal_error(
                    ResourceKind::Nexus,
                    HA_AGENT_ERR_SOURCE.to_string(),
                    "Failed to connect to NVMe target".to_string(),
                ));
            }
        }

        // Step 2: disconnect broken path to leave the only new healthy path.
        // Note that errors under disconnection are not critical, since the second I/O
        // path has been successfully created, so having the first failed path in addition
        // to the second healthy one is OK: just display a warning and proceed as if
        // the call has completed successfully.
        disconnect_controller(&ctrlr).or_else(|e| {
            tracing::warn!(
                uri=%request.new_path(),
                error=%e,
                "Failed to disconnect failed path"
            );
            Ok(())
        })
    }
}
