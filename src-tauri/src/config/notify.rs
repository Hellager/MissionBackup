use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyConfig {
    /// Whether able to notify
    pub is_granted: bool,

    /// Whether enable notify
    pub enable: bool,

    /// Notify mask, bit 0: disable, bit 1: enable
    pub mask: u32,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        NotifyConfig {
            is_granted: false,
            enable: false,
            mask: 0,
        }
    }
}
