use serde::{Serialize, Deserialize};
use crate::utils::common::{get_sys_locale, get_sys_theme};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Theme config
    /// Supported theme:
    /// - Light -> `light`
    /// - Dark -> `dark`
    pub theme: String,

    /// Supported system theme:
    /// - Light -> `light`
    /// - Dark -> `dark`
    pub sys_theme: String,

    /// Theme option
    /// `0` - use config theme
    /// `1` - follow system theme
    pub theme_option: u8,

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
            sys_theme: get_sys_theme(),
            theme_option: 0,
            auto_start: false,
            close_option: 0,
            close_cnt: 49,
            close_limit: 50,
            language: get_sys_locale()
        }
    }
}
