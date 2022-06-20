use crate::{
    misc::traits::ValidateRequestTypes,
    operations::watcher::traits::WatcherOperations,
    watcher,
    watcher::{
        get_watches_reply,
        watcher_grpc_server::{WatcherGrpc, WatcherGrpcServer},
        GetWatchesReply, GetWatchesRequest, WatchReply,
    },
};
use std::sync::Arc;
use tonic::Response;

/// RPC Watcher Server
#[derive(Clone)]
pub struct WatcherServer {
    /// Service which executes the operations.
    service: Arc<dyn WatcherOperations>,
}

impl WatcherServer {
    /// returns a new WatcherServer with the service implementing Watcher operations
    pub fn new(service: Arc<dyn WatcherOperations>) -> Self {
        Self { service }
    }
    /// coverts the replicaserver to its corresponding grpc server type
    pub fn into_grpc_server(self) -> WatcherGrpcServer<WatcherServer> {
        WatcherGrpcServer::new(self)
    }
}

/// Implementation of the RPC methods.
#[tonic::async_trait]
impl WatcherGrpc for WatcherServer {
    async fn get_watches(
        &self,
        request: tonic::Request<GetWatchesRequest>,
    ) -> Result<tonic::Response<GetWatchesReply>, tonic::Status> {
        let req = request.into_inner().validated()?;
        match self.service.get(&req, None).await {
            Ok(watches) => Ok(Response::new(GetWatchesReply {
                reply: Some(get_watches_reply::Reply::Watches(watches.into())),
            })),
            Err(err) => Ok(Response::new(GetWatchesReply {
                reply: Some(get_watches_reply::Reply::Error(err.into())),
            })),
        }
    }
    async fn delete_watch(
        &self,
        request: tonic::Request<watcher::Watch>,
    ) -> Result<tonic::Response<WatchReply>, tonic::Status> {
        let req = request.into_inner().validated()?;
        match self.service.destroy(&req, None).await {
            Ok(()) => Ok(Response::new(WatchReply { error: None })),
            Err(e) => Ok(Response::new(WatchReply {
                error: Some(e.into()),
            })),
        }
    }
    async fn create_watch(
        &self,
        request: tonic::Request<watcher::Watch>,
    ) -> Result<tonic::Response<WatchReply>, tonic::Status> {
        let req = request.into_inner().validated()?;
        match self.service.create(&req, None).await {
            Ok(()) => Ok(Response::new(WatchReply { error: None })),
            Err(e) => Ok(Response::new(WatchReply {
                error: Some(e.into()),
            })),
        }
    }
}
