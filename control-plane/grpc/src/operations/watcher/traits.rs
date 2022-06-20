use crate::{
    context::Context,
    misc::traits::{StringValue, ValidateRequestTypes},
    watcher,
    watcher::watch_resource_id,
};
use common_lib::{
    mbus_api::{v0::Watches, ReplyError, ResourceKind},
    types::v0::message_bus::{
        CreateWatch, DeleteWatch, GetWatchers, NexusId, ReplicaId, VolumeId, Watch, WatchCallback,
        WatchResourceId, WatchType,
    },
};
use std::convert::TryFrom;

/// All watcher crud operations to be a part of the WatcherOperations trait
#[tonic::async_trait]
pub trait WatcherOperations: Send + Sync {
    /// Create a watcher
    async fn create(&self, req: &dyn WatcherInfo, ctx: Option<Context>) -> Result<(), ReplyError>;
    /// Get watchers
    async fn get(
        &self,
        req: &dyn GetWatcherInfo,
        ctx: Option<Context>,
    ) -> Result<Watches, ReplyError>;
    /// Destroy a watcher
    async fn destroy(&self, req: &dyn WatcherInfo, ctx: Option<Context>) -> Result<(), ReplyError>;
}

/// WatcherInfo trait for the watch creation to be implemented by entities which want to avail
/// this operation
pub trait WatcherInfo: Send + Sync + std::fmt::Debug {
    /// id of the resource to watch on
    fn id(&self) -> WatchResourceId;
    /// callback used to notify the watcher of a change
    fn callback(&self) -> WatchCallback;
    /// type of watch
    fn watch_type(&self) -> WatchType;
}

/// GetWatcherInfo trait for the get watch operation to be implemented by entities which want to
/// avail this operation
pub trait GetWatcherInfo: Send + Sync + std::fmt::Debug {
    /// id of the resource to get
    fn resource_id(&self) -> WatchResourceId;
}

impl From<WatchResourceId> for watcher::WatchResourceId {
    fn from(resource_id: WatchResourceId) -> Self {
        match resource_id {
            WatchResourceId::Node(node_id) => Self {
                resource_id: Some(watch_resource_id::ResourceId::NodeId(node_id.to_string())),
            },
            WatchResourceId::Pool(pool_id) => Self {
                resource_id: Some(watch_resource_id::ResourceId::PoolId(pool_id.to_string())),
            },
            WatchResourceId::Replica(replica_id) => Self {
                resource_id: Some(watch_resource_id::ResourceId::ReplicaId(
                    replica_id.to_string(),
                )),
            },
            WatchResourceId::ReplicaState(replica_id) => Self {
                resource_id: Some(watch_resource_id::ResourceId::ReplicaStateId(
                    replica_id.to_string(),
                )),
            },
            WatchResourceId::ReplicaSpec(replica_id) => Self {
                resource_id: Some(watch_resource_id::ResourceId::ReplicaSpecId(
                    replica_id.to_string(),
                )),
            },
            WatchResourceId::Nexus(nexus_id) => Self {
                resource_id: Some(watch_resource_id::ResourceId::NexusId(nexus_id.to_string())),
            },
            WatchResourceId::Volume(volume_id) => Self {
                resource_id: Some(watch_resource_id::ResourceId::VolumeId(
                    volume_id.to_string(),
                )),
            },
        }
    }
}

impl TryFrom<watcher::WatchResourceId> for WatchResourceId {
    type Error = ReplyError;

    fn try_from(value: watcher::WatchResourceId) -> Result<Self, Self::Error> {
        match value.resource_id {
            Some(resource_id) => Ok(match resource_id {
                watch_resource_id::ResourceId::NodeId(id) => WatchResourceId::Node(id.into()),
                watch_resource_id::ResourceId::PoolId(id) => WatchResourceId::Pool(id.into()),
                watch_resource_id::ResourceId::ReplicaId(id) => {
                    WatchResourceId::Replica(ReplicaId::try_from(StringValue(Some(id)))?)
                }
                watch_resource_id::ResourceId::ReplicaStateId(id) => {
                    WatchResourceId::ReplicaSpec(ReplicaId::try_from(StringValue(Some(id)))?)
                }
                watch_resource_id::ResourceId::ReplicaSpecId(id) => {
                    WatchResourceId::ReplicaState(ReplicaId::try_from(StringValue(Some(id)))?)
                }
                watch_resource_id::ResourceId::NexusId(id) => {
                    WatchResourceId::Nexus(NexusId::try_from(StringValue(Some(id)))?)
                }
                watch_resource_id::ResourceId::VolumeId(id) => {
                    WatchResourceId::Volume(VolumeId::try_from(StringValue(Some(id)))?)
                }
            }),
            None => Err(ReplyError::invalid_argument(
                ResourceKind::Watch,
                "watch_resource_id",
                "".to_string(),
            )),
        }
    }
}

impl From<WatchCallback> for watcher::WatchCallback {
    fn from(value: WatchCallback) -> Self {
        match value {
            WatchCallback::Uri(uri) => Self {
                callback: Some(watcher::watch_callback::Callback::Uri(watcher::Uri {
                    content: uri,
                })),
            },
        }
    }
}

impl TryFrom<watcher::WatchCallback> for WatchCallback {
    type Error = ReplyError;

    fn try_from(value: watcher::WatchCallback) -> Result<Self, Self::Error> {
        match value.callback {
            Some(watch_callback) => match watch_callback {
                watcher::watch_callback::Callback::Uri(uri) => Ok(Self::Uri(uri.content)),
            },
            None => Err(ReplyError::invalid_argument(
                ResourceKind::Watch,
                "watch_callback",
                "".to_string(),
            )),
        }
    }
}

impl From<WatchType> for watcher::WatchType {
    fn from(value: WatchType) -> Self {
        match value {
            WatchType::Desired => Self::Desired,
            WatchType::Actual => Self::Actual,
            WatchType::All => Self::All,
        }
    }
}

impl From<watcher::WatchType> for WatchType {
    fn from(value: watcher::WatchType) -> Self {
        match value {
            watcher::WatchType::Desired => Self::Desired,
            watcher::WatchType::Actual => Self::Actual,
            watcher::WatchType::All => Self::All,
        }
    }
}

impl From<Watch> for watcher::Watch {
    fn from(value: Watch) -> Self {
        let watch_type: watcher::WatchType = value.watch_type.into();
        Self {
            id: Some(value.id.into()),
            callback: Some(value.callback.into()),
            watch_type: watch_type as i32,
        }
    }
}

impl TryFrom<watcher::Watch> for Watch {
    type Error = ReplyError;

    fn try_from(value: watcher::Watch) -> Result<Self, Self::Error> {
        Ok(Self {
            id: match value.id {
                Some(id) => WatchResourceId::try_from(id)?,
                None => {
                    return Err(ReplyError::invalid_argument(
                        ResourceKind::Watch,
                        "watch_resource_id",
                        "".to_string(),
                    ))
                }
            },
            callback: match value.callback {
                Some(callback) => WatchCallback::try_from(callback)?,
                None => {
                    return Err(ReplyError::invalid_argument(
                        ResourceKind::Watch,
                        "watch_callback",
                        "".to_string(),
                    ))
                }
            },
            watch_type: watcher::WatchType::from_i32(value.watch_type)
                .ok_or_else(|| {
                    ReplyError::invalid_argument(ResourceKind::Watch, "watch_type", "".to_string())
                })?
                .into(),
        })
    }
}

impl TryFrom<watcher::Watches> for Watches {
    type Error = ReplyError;
    fn try_from(grpc_watches_type: watcher::Watches) -> Result<Self, Self::Error> {
        let mut watches: Vec<Watch> = vec![];
        for watch in grpc_watches_type.watches {
            watches.push(Watch::try_from(watch.clone())?)
        }
        Ok(Watches(watches))
    }
}

impl From<Watches> for watcher::Watches {
    fn from(watches: Watches) -> Self {
        watcher::Watches {
            watches: watches
                .into_inner()
                .iter()
                .map(|watchers| watchers.clone().into())
                .collect(),
        }
    }
}

impl WatcherInfo for CreateWatch {
    fn id(&self) -> WatchResourceId {
        self.id.clone()
    }

    fn callback(&self) -> WatchCallback {
        self.callback.clone()
    }

    fn watch_type(&self) -> WatchType {
        self.watch_type.clone()
    }
}

impl WatcherInfo for DeleteWatch {
    fn id(&self) -> WatchResourceId {
        self.id.clone()
    }

    fn callback(&self) -> WatchCallback {
        self.callback.clone()
    }

    fn watch_type(&self) -> WatchType {
        self.watch_type.clone()
    }
}

/// Intermediate structure that validates the conversion to grpc Watch type
#[derive(Debug)]
pub struct ValidatedWatchRequest {
    id: WatchResourceId,
    callback: WatchCallback,
    watch_type: WatchType,
}

impl WatcherInfo for ValidatedWatchRequest {
    fn id(&self) -> WatchResourceId {
        self.id.clone()
    }

    fn callback(&self) -> WatchCallback {
        self.callback.clone()
    }

    fn watch_type(&self) -> WatchType {
        self.watch_type.clone()
    }
}

impl ValidateRequestTypes for watcher::Watch {
    type Validated = ValidatedWatchRequest;
    fn validated(self) -> Result<Self::Validated, ReplyError> {
        Ok(ValidatedWatchRequest {
            id: WatchResourceId::try_from(match self.id {
                Some(id) => id,
                None => {
                    return Err(ReplyError::invalid_argument(
                        ResourceKind::Watch,
                        "watch_resource_id",
                        "".to_string(),
                    ))
                }
            })?,
            callback: WatchCallback::try_from(match self.callback {
                Some(callback) => callback,
                None => {
                    return Err(ReplyError::invalid_argument(
                        ResourceKind::Watch,
                        "watch_callback",
                        "".to_string(),
                    ))
                }
            })?,
            watch_type: match watcher::WatchType::from_i32(self.watch_type) {
                Some(watch_type) => watch_type.into(),
                None => {
                    return Err(ReplyError::invalid_argument(
                        ResourceKind::Watch,
                        "watch_type",
                        "".to_string(),
                    ))
                }
            },
        })
    }
}

impl From<&dyn WatcherInfo> for Watch {
    fn from(data: &dyn WatcherInfo) -> Self {
        Self {
            id: data.id(),
            callback: data.callback(),
            watch_type: data.watch_type(),
        }
    }
}

impl From<&dyn WatcherInfo> for DeleteWatch {
    fn from(data: &dyn WatcherInfo) -> Self {
        Self {
            id: data.id(),
            callback: data.callback(),
            watch_type: data.watch_type(),
        }
    }
}

impl From<&dyn WatcherInfo> for watcher::Watch {
    fn from(data: &dyn WatcherInfo) -> Self {
        let watch_type: watcher::WatchType = data.watch_type().into();
        Self {
            id: Some(data.id().into()),
            callback: Some(data.callback().into()),
            watch_type: watch_type as i32,
        }
    }
}

impl GetWatcherInfo for GetWatchers {
    fn resource_id(&self) -> WatchResourceId {
        self.resource.clone()
    }
}

/// Intermediate structure that validates the conversion to GetWatcherRequest type
#[derive(Debug)]
pub struct ValidatedGetWatchesRequest {
    resource: WatchResourceId,
}

impl GetWatcherInfo for ValidatedGetWatchesRequest {
    fn resource_id(&self) -> WatchResourceId {
        self.resource.clone()
    }
}

impl ValidateRequestTypes for watcher::GetWatchesRequest {
    type Validated = ValidatedGetWatchesRequest;
    fn validated(self) -> Result<Self::Validated, ReplyError> {
        Ok(ValidatedGetWatchesRequest {
            resource: WatchResourceId::try_from(match self.resource {
                Some(id) => id,
                None => {
                    return Err(ReplyError::invalid_argument(
                        ResourceKind::Watch,
                        "watch_resource_id",
                        "".to_string(),
                    ))
                }
            })?,
        })
    }
}

impl From<&dyn GetWatcherInfo> for GetWatchers {
    fn from(data: &dyn GetWatcherInfo) -> Self {
        Self {
            resource: data.resource_id(),
        }
    }
}

impl From<&dyn GetWatcherInfo> for watcher::GetWatchesRequest {
    fn from(data: &dyn GetWatcherInfo) -> Self {
        Self {
            resource: Some(data.resource_id().into()),
        }
    }
}
