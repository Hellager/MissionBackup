//! # System
//! 
//! `system` module contains all configuration about app's system related.

use serde::{Serialize, Deserialize};
use crate::utils::common::{get_sys_locale, get_sys_theme};

/// Configuration for system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Theme config
    /// Supported theme:
    /// - Light -> `light`
    /// - Dark -> `dark`
    pub theme: String,

    /// Whether auto start for app
    pub auto_start: bool,

    /// Close option for app
    /// `0` -> exit app
    /// `1` -> hide to tray
    pub close_option: u8,

    /// Close count for app
    pub close_cnt: u8,

    /// Close count limit for app
    /// Default is `50`
    pub close_limit: u8,

    /// Language config
    /// Supported language:
    /// - English -> `en-US`
    /// - Chinese -> `zh-CN`
    pub language: String,
}

impl Default for SystemConfig {
    fn default() -> Self {
        SystemConfig {
            theme: get_sys_theme(),
            auto_start: false,
            close_option: 0,
            close_cnt: 0,
            close_limit: 50,
            language: get_sys_locale()
        }
    }
}
