use crate::controller::registry::Registry;
use grpc::operations::volume::server::VolumeServer;
use std::sync::Arc;

mod operations;
mod registry;
mod scheduling;
mod service;
mod specs;

/// Configure the Service and return the builder.
pub(crate) fn configure(builder: agents::Service) -> agents::Service {
    let registry = builder.shared_state::<Registry>().clone();
    let new_service = Arc::new(service::Service::new(registry));
    let volume_service = VolumeServer::new(new_service);
    builder.with_service(volume_service.into_grpc_server())
}
