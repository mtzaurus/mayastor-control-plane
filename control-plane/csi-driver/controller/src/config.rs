use anyhow::Context;
use clap::ArgMatches;
use once_cell::sync::OnceCell;
use std::time::Duration;

static CONFIG: OnceCell<CsiControllerConfig> = OnceCell::new();

// Global CSI Controller config.
pub struct CsiControllerConfig {
    /// REST API endpoint URL.
    rest_endpoint: String,
    /// I/O timeout for REST API operations.
    io_timeout: Duration,
}
impl CsiControllerConfig {
    /// Initialize global instance of the CSI config. Must be called prior to using the config.
    pub fn initialize(args: &ArgMatches) -> anyhow::Result<()> {
        assert!(
            CONFIG.get().is_none(),
            "CSI Controller config already initialized"
        );

        let rest_endpoint = args
            .value_of("endpoint")
            .context("rest endpoint must be specified")?;

        let io_timeout = args
            .value_of("timeout")
            .context("I/O timeout must be specified")?
            .parse::<humantime::Duration>()?;

        CONFIG.get_or_init(|| Self {
            rest_endpoint: rest_endpoint.into(),
            io_timeout: io_timeout.into(),
        });
        Ok(())
    }

    /// Default IO-Engine DaemonSet selector label.
    pub(crate) fn default_io_selector() -> String {
        format!(
            "{}:{}",
            utils::IO_ENGINE_SELECTOR_KEY,
            utils::IO_ENGINE_SELECTOR_VALUE
        )
    }

    /// Get global instance of CSI controller config.
    pub fn get_config() -> &'static CsiControllerConfig {
        CONFIG
            .get()
            .expect("CSI Controller config is not initialized")
    }

    /// Get REST API endpoint.
    pub fn rest_endpoint(&self) -> &str {
        &self.rest_endpoint
    }

    /// Get I/O timeout for REST API operations.
    pub fn io_timeout(&self) -> Duration {
        self.io_timeout
    }
}
