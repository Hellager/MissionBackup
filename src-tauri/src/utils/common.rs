//! The `common` module contains various commonly used functions.

#[allow(dead_code)]
/// Get app home directory, not the exe directory
///
/// # Arguments
///
/// # Examples
///
/// ```
/// use common::get_app_home_dir;
///
/// let app_dir = get_app_home_dir();
/// match app_dir {
///     Ok(dir) => println!("app running at: {:?}", dir),
///     Err(error) => println!("get app home dir failed, errMsg: {:?}", error),
/// }
/// ```
pub fn get_app_home_dir() -> Result<std::path::PathBuf, std::io::Error> {
    #[cfg(target_os = "windows")]
    return std::env::current_dir();

    #[cfg(not(target_os = "windows"))]
    match std::env::home_dir() {
        None => Err(ErrorKind::NotFound),
        Some(path) => Ok(path.join(APP_DIR)),
    }
}

#[allow(dead_code)]
/// Gets system locale.
///
/// For tauri v2, we can use tauri-plugin-os to query os locale.
///
/// # Arguments
///
/// # Examples
///
/// ```
/// use common::get_sys_locale;
///
/// let locale = get_sys_locale();
/// println!("current locale: {:?}", locale),
/// ```
pub fn get_sys_locale() -> String {
    use sys_locale::get_locale;

    get_locale().unwrap_or_else(|| String::from("en-US"))
}

#[allow(dead_code)]
/// Get system theme.
///
/// # Arguments
///
/// # Examples
///
/// ```
/// use common::get_sys_theme;
///
/// let theme = get_sys_theme();
/// println!("current theme: {:?}", theme),
/// ```
pub fn get_sys_theme() -> String {
    let mode = dark_light::detect();

    if mode == dark_light::Mode::Dark {
        "dark".to_string()
    } else {
        "light".to_string()
    }
}

#[allow(dead_code)]
/// Get system webview versoin.
///
/// # Arguments
///
/// # Examples
///
/// ```
/// use common::get_webview_version;
///
/// let res = get_webview_version();
///
/// match res {
///     Ok(version) => println!("webview version: {:?}", version),
///     Err(error) => println!("get webview version failed, errMsg: {:?}", error),
/// }
/// ```
pub fn get_webview_version() -> Result<String, wry::Error> {
    use wry::webview_version;

    webview_version()
}

#[allow(dead_code)]
/// Generate a random i32 number
///
/// # Arguments
///
/// # Examples
///
/// ```
/// use common::rand_number;
///
/// let number = rand_number();
/// println!("the random number is {}", number);
/// ```
pub fn rand_number() -> i32 {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    rng.gen::<i32>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_app_home_dir() {
        use std::env::current_dir;

        let std_dir = current_dir().expect("");
        let home_dir = get_app_home_dir().expect("");

        assert_ne!(std_dir.display().to_string(), "".to_string());
        assert_ne!(home_dir.display().to_string(), "".to_string());
        assert_eq!(std_dir, home_dir);
    }

    #[test]
    fn test_get_sys_locale() {
        let sys_locale = sys_locale::get_locale().expect("en-US");
        let locale = get_sys_locale();

        assert_ne!(locale, "".to_string());
        assert_eq!(locale, sys_locale);
    }

    #[test]
    fn test_get_sys_theme() {
        let sys_theme = dark_light::detect();
        let theme = get_sys_theme();

        assert_ne!(theme, "".to_string());
        match sys_theme {
            dark_light::Mode::Dark => {
                assert_eq!(theme, "dark");
            }
            dark_light::Mode::Light => {
                assert_eq!(theme, "light");
            }
            dark_light::Mode::Default => {
                assert_eq!(theme, "light");
            }
        }
    }

    #[test]
    fn test_get_webview_version() {
        let sys_webview_version = wry::webview_version();
        let version = get_webview_version().expect("");

        if let Ok(webview_version) = sys_webview_version {
            assert_eq!(webview_version, version);
        } else {
            assert_eq!(version, "");
        }
    }
}
