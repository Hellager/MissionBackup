use tauri::{plugin::TauriPlugin, Emitter, Manager, Runtime};

pub fn initialize_plugin_single_instance<R: Runtime>() -> TauriPlugin<R> {
    tauri_plugin_single_instance::init(|app, _, _| {
        if let Some(main_window) = app.get_webview_window("main") {
            let _ = app.emit("another_instance", {});
            let _ = main_window.center();
            let _ = main_window.set_focus();
        }
    })
}

pub fn initialize_plugin_autostart<R: tauri::Runtime>() -> TauriPlugin<R> {
    use tauri_plugin_autostart::MacosLauncher;

    tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None)
}

pub fn initialize_plugin_window_state<R: tauri::Runtime>() -> TauriPlugin<R> {
    use tauri_plugin_window_state::StateFlags;

    let flags: StateFlags = StateFlags::POSITION | StateFlags::VISIBLE;

    tauri_plugin_window_state::Builder::new()
        .with_state_flags(flags)
        .build()
}

#[cfg(debug_assertions)]
pub fn initialize_plugin_log<R: tauri::Runtime>() -> TauriPlugin<R> {
    use log::LevelFilter;
    use tauri_plugin_log::{fern::colors::ColoredLevelConfig, Builder, Target, TargetKind};

    const LOG_FILE_NAME: &str = env!("CARGO_PKG_NAME");

    Builder::new()
        .clear_targets()
        .target(Target::new(TargetKind::Stdout))
        .target(Target::new(TargetKind::Webview))
        .target(Target::new(TargetKind::LogDir {
            file_name: Some(LOG_FILE_NAME.to_string()),
        }))
        .level(LevelFilter::Debug)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
        .with_colors(ColoredLevelConfig::default())
        .build()
}

#[cfg(not(debug_assertions))]
pub fn initialize_plugin_log<R: tauri::Runtime>() -> TauriPlugin<R> {
    use log::LevelFilter;
    use tauri_plugin_log::{Builder, Target, TargetKind};

    const LOG_FILE_NAME: &str = env!("CARGO_PKG_NAME");

    Builder::new()
        .clear_targets()
        .target(Target::new(TargetKind::LogDir {
            file_name: Some(LOG_FILE_NAME.to_string()),
        }))
        .level(LevelFilter::Info)
        .max_file_size(50_000)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
        .format(|out, message, record| out.finish(format_args!("[{}] {}", record.level(), message)))
        .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
        .build()
}
