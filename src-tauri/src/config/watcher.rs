use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatcherConfig {
    /// Watcher timeout in secs
    pub timeout: u64,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        WatcherConfig {
            timeout: 3
        }
    }
}
