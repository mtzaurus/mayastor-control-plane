use nvmeadm::nvmf_subsystem::NvmeSubsystems;
use std::path::{Path, PathBuf};

const SYSFS_NVME_CTRL_PREFIX: &str = "/sys/devices/virtual/nvme-fabrics/ctl/";

pub trait NvmePathProvider {
    fn get_entries(&self) -> Vec<NvmePath>;
}

#[derive(Debug, Clone)]
pub struct NvmePath {
    name: String,
    path_buffer: PathBuf,
}

impl NvmePath {
    fn new(name: String) -> Self {
        let pb = Path::new(&format!("{}{}", SYSFS_NVME_CTRL_PREFIX, name))
            .canonicalize()
            .unwrap_or_else(|_| panic!("Can't obtain path for NVMe controller {}", name));

        Self {
            name,
            path_buffer: pb,
        }
    }

    #[inline]
    pub fn path(&self) -> &Path {
        self.path_buffer.as_path()
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct CachedNvmePathProvider {
    entries: Vec<NvmePath>,
}

impl CachedNvmePathProvider {
    async fn initialize(&mut self) -> anyhow::Result<()> {
        let subsystems = NvmeSubsystems::new().expect("Failed to intialize NVMe subsystems");

        for e in subsystems {
            match e {
                Ok(s) => {
                    info!("Caching NVMe controller: {:?}", s);
                    self.entries.push(NvmePath::new(s.name));
                }
                Err(e) => error!("Failed to read NVMe subsystem: {:?}", e),
            };
        }
        info!(
            "NVMe path cache initialized with {} entries",
            self.entries.len()
        );
        Ok(())
    }

    pub async fn new() -> anyhow::Result<Self> {
        let mut provider = Self { entries: vec![] };

        provider.initialize().await?;
        Ok(provider)
    }
}

impl NvmePathProvider for CachedNvmePathProvider {
    fn get_entries(&self) -> Vec<NvmePath> {
        self.entries.clone()
    }
}
