pub mod notify;
pub mod screensaver;
pub mod system;
pub mod watcher;

use notify::NotifyConfig;
use screensaver::ScreensaverConfig;
use serde::{Deserialize, Serialize};
use system::SystemConfig;
use watcher::WatcherConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// App system config, like `theme`, `language`...
    pub system: SystemConfig,

    /// App notify config, like `enable_notify`...
    pub notify: NotifyConfig,

    /// App watcher Config, like `timeout`...
    pub watcher: WatcherConfig,

    /// App screensaver config, like `enable`...
    pub screensaver: ScreensaverConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            system: system::SystemConfig::default(),
            notify: notify::NotifyConfig::default(),
            watcher: watcher::WatcherConfig::default(),
            screensaver: screensaver::ScreensaverConfig::default(),
        }
    }
}

fn get_config_file_path() -> Result<String, std::io::Error> {
    use directories::ProjectDirs;
    use std::env::current_dir;
    use std::fs::create_dir_all;
    use std::path::PathBuf;

    let application = option_env!("CARGO_PKG_NAME").unwrap_or("clean-recent");
    let mut config_dir: PathBuf = current_dir()?;
    if let Some(proj_dirs) = ProjectDirs::from("app", "hellagur", application) {
        config_dir = proj_dirs.config_dir().to_path_buf();
        create_dir_all(config_dir.clone())?;
    }

    let file_path = config_dir.join(format!("{}.toml", application));
    return Ok(file_path.display().to_string());
}

pub fn load_app_config() -> Result<AppConfig, std::io::Error> {
    use std::fs::read_to_string;
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    if let Ok(path) = get_config_file_path() {
        if Path::new(&path).exists() {
            let stored_config = read_to_string(path)?;
            match toml::from_str(stored_config.as_str()) {
                Ok(config) => {
                    return Ok(config);
                }
                Err(_error) => {
                    return Err(Error::from(ErrorKind::InvalidData));
                }
            }
        }
    }

    return Err(Error::from(ErrorKind::NotFound));
}

pub fn save_app_config(config: &AppConfig) -> Result<(), std::io::Error> {
    use std::fs::File;
    use std::io::Write;

    if let Ok(path) = get_config_file_path() {
        let mut toml_file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        if let Ok(toml) = toml::to_string(config) {
            toml_file.write_all(toml.as_bytes())?;
        }
    }

    Ok(())
}
