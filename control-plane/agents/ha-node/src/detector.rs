use crate::{
    path_provider::{CachedNvmePathProvider, NvmePathProvider},
    Cli,
};
use nvmeadm::nvmf_subsystem::Subsystem;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

/// Possible states of every path record.
#[derive(Debug, Clone)]
enum PathState {
    Good,
    Suspected,
    Failed,
}

/// Object that represents a broken/suspected NVMe path.
#[derive(Debug, Clone)]
struct PathRecord {
    /// Used to allow detection of outdated records.
    epoch: u64,
    nqn: String,
    state: PathState,
}

impl PathRecord {
    fn new(nqn: String, epoch: u64) -> Self {
        Self {
            nqn,
            epoch,
            state: PathState::Good,
        }
    }

    fn get_nqn(&self) -> &str {
        &self.nqn
    }

    #[inline]
    fn get_epoch(&self) -> u64 {
        self.epoch
    }

    #[inline]
    fn set_epoch(&mut self, epoch: u64) {
        self.epoch = epoch;
    }

    fn report_connecting(&mut self) {
        match self.state {
            PathState::Good => {
                self.state = PathState::Suspected;
                info!("Target {} transitioned into SUSPECTED state", self.nqn);
            }
            PathState::Suspected => {
                self.state = PathState::Failed;
                error!("Target {} transitioned into FAILED state", self.nqn);
            }
            PathState::Failed => {} // Multiple failures don't cause any state transitions.
        }
    }

    fn report_live(&mut self) {
        self.state = PathState::Good;
    }
}

#[derive(Debug)]
pub struct PathFailureDetector {
    epoch: u64,
    _detection_period: Duration,
    path_provider: CachedNvmePathProvider,
    suspected_paths: HashMap<String, PathRecord>,
}

impl PathFailureDetector {
    pub async fn new(args: &Cli) -> anyhow::Result<Self> {
        let path_provider = CachedNvmePathProvider::new().await?;

        Ok(Self {
            epoch: 0,
            _detection_period: Duration::from_secs(args.detection_period),
            path_provider,
            suspected_paths: HashMap::new(),
        })
    }

    fn rescan_paths(&mut self) {
        // Update epoch before scanning controllers.
        self.epoch += 1;

        // Scan all reported NVMe paths on system and check for connectivity.
        for ctrlr in self.path_provider.get_entries() {
            match Subsystem::new(ctrlr.path()) {
                Ok(subsystem) => {
                    let existing_record = match subsystem.state.as_str() {
                        "connecting" => {
                            // Add a new record in case no record exists for target NQN.
                            if !self.suspected_paths.contains_key(&subsystem.nqn) {
                                self.suspected_paths.insert(
                                    subsystem.nqn.clone(),
                                    PathRecord::new(subsystem.nqn.clone(), self.epoch),
                                );
                            }

                            let rec = self.suspected_paths.get_mut(&subsystem.nqn).unwrap();
                            rec.report_connecting();
                            Some(rec)
                        }
                        "live" => self.suspected_paths.get_mut(&subsystem.nqn).map(|rec| {
                            rec.report_live();
                            rec
                        }),
                        _ => None,
                    };

                    // Update epoch for the existing record.
                    if let Some(rec) = existing_record {
                        rec.set_epoch(self.epoch);
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to get status for NVMe subsystem {}: {}",
                        ctrlr.name(),
                        e
                    );
                }
            }
        }

        // Remove all existing records that don't have underlaying NVMe controllers:
        // can happen in case controller was removed after it had been identified as
        // broken/suspected. Stalled/outdated records have a different (old) epoch number
        // since they have not been touched during the current iteration.
        let mut to_remove = vec![];

        for v in self.suspected_paths.values() {
            if v.get_epoch() != self.epoch {
                to_remove.push(v.get_nqn().to_owned());
            }
        }

        if !to_remove.is_empty() {
            for k in to_remove {
                self.suspected_paths.remove(&k);
                debug!("Removing stalled path record for NQN {}", k);
            }
        }
    }

    pub async fn start(mut self) {
        self.rescan_paths();

        loop {
            info!("Sleeping ...");
            sleep(Duration::from_secs(3)).await;
            self.rescan_paths();
        }
    }
}
