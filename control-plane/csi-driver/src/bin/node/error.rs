//! Definition of DeviceError used by the attach and detach code.
use nvmeadm::{error::NvmeError, nvmf_discovery};
use std::string::FromUtf8Error;

/// A Device Attach/Detach error.
/// todo: should this be an enum?
pub(crate) struct DeviceError {
    pub(crate) message: String,
    pub(crate) source: DeviceErrorSource,
}

/// Possible device error kind sources.
pub(crate) enum DeviceErrorSource {
    None,
    NvmeError(NvmeError),
    StdIoError(std::io::Error),
    StdIntParseError(std::num::ParseIntError),
    UuidError(uuid::Error),
    NvmfConnectArgsBuilderError(nvmf_discovery::ConnectArgsBuilderError),
    GenericError(anyhow::Error),
    SerdeError(serde_json::error::Error),
    FromUtf8Error(FromUtf8Error),
}

impl DeviceError {
    /// Return a new `Self` with the given message.
    pub(crate) fn new(message: &str) -> DeviceError {
        DeviceError {
            message: String::from(message),
            source: DeviceErrorSource::None,
        }
    }
}

impl std::fmt::Debug for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DeviceError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<std::io::Error> for DeviceError {
    fn from(error: std::io::Error) -> DeviceError {
        DeviceError {
            message: format!("{}", error),
            source: DeviceErrorSource::StdIoError(error),
        }
    }
}

impl From<std::num::ParseIntError> for DeviceError {
    fn from(error: std::num::ParseIntError) -> DeviceError {
        DeviceError {
            message: format!("{}", error),
            source: DeviceErrorSource::StdIntParseError(error),
        }
    }
}

impl From<uuid::Error> for DeviceError {
    fn from(error: uuid::Error) -> DeviceError {
        DeviceError {
            message: format!("{}", error),
            source: DeviceErrorSource::UuidError(error),
        }
    }
}

impl From<nvmf_discovery::ConnectArgsBuilderError> for DeviceError {
    fn from(error: nvmf_discovery::ConnectArgsBuilderError) -> DeviceError {
        DeviceError {
            message: format!("{}", error),
            source: DeviceErrorSource::NvmfConnectArgsBuilderError(error),
        }
    }
}

impl From<String> for DeviceError {
    fn from(message: String) -> DeviceError {
        DeviceError {
            message,
            source: DeviceErrorSource::None,
        }
    }
}

impl From<serde_json::error::Error> for DeviceError {
    fn from(error: serde_json::error::Error) -> DeviceError {
        DeviceError {
            message: format!("{}", error),
            source: DeviceErrorSource::SerdeError(error),
        }
    }
}

impl From<FromUtf8Error> for DeviceError {
    fn from(error: FromUtf8Error) -> DeviceError {
        DeviceError {
            message: format!("{}", error),
            source: DeviceErrorSource::FromUtf8Error(error),
        }
    }
}

impl From<nvmeadm::error::NvmeError> for DeviceError {
    fn from(error: nvmeadm::error::NvmeError) -> DeviceError {
        DeviceError {
            message: format!("{}", error),
            source: DeviceErrorSource::NvmeError(error),
        }
    }
}

impl From<anyhow::Error> for DeviceError {
    fn from(error: anyhow::Error) -> DeviceError {
        DeviceError {
            message: error.to_string(),
            source: DeviceErrorSource::GenericError(error),
        }
    }
}

impl From<DeviceError> for tonic::Status {
    fn from(dev_error: DeviceError) -> Self {
        match dev_error.source {
            DeviceErrorSource::NvmeError(error) => match error {
                NvmeError::IoFailed { .. } | NvmeError::ConnectFailed { .. } => {
                    tonic::Status::aborted(dev_error.message)
                }
                NvmeError::ConnectInProgress => tonic::Status::already_exists(dev_error.message),
                _ => tonic::Status::internal(dev_error.message),
            },
            _ => tonic::Status::internal(dev_error.message),
        }
    }
}
