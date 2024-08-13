use tauri::{ Wry, GlobalWindowEvent, WindowEvent, FileDropEvent, Theme, Window, Manager };

pub fn on_window_event(event: GlobalWindowEvent<Wry>) {
    use super::cmd::Response;
    use log::{info, error};

    match event.event() {
        WindowEvent::CloseRequested { api, .. } => {
            let window = event.window();
            window.hide().unwrap();
            api.prevent_close();
            println!("window {} try to close", window.label());

            std::process::exit(0);
        },
        WindowEvent::FileDrop(drop_event) => {
            match drop_event {
                FileDropEvent::Hovered(files) => {
                    println!("files: {:?} hovered", files);
                },
                FileDropEvent::Dropped(files) => {
                    println!("files: {:?} dropped", files);
                },
                FileDropEvent::Cancelled => {

                },
                _ => {}
            }
        },
        WindowEvent::ThemeChanged(theme) => {
            match theme {
                Theme::Light => {
                    let window = event.window();
                    let payload = Response::success("light");
                    match window.emit_all("sys_theme", payload) {
                        Ok(()) => {
                            info!("Detect system theme update to light");
                        },
                        Err(error) => {
                            error!("Failed to send sys_theme update event, errMsg: {:?}", error);
                        }
                    }
                },
                Theme::Dark => {
                    let window = event.window();
                    let payload = Response::success("dark");
                    match window.emit_all("sys_theme", payload) {
                        Ok(()) => {
                            info!("Detect system theme update to dark");
                        },
                        Err(error) => {
                            error!("Failed to send sys_theme update event, errMsg: {:?}", error);
                        }
                    }
                },
                _ => {}
            }
        }
        _ => {}
    }
}

pub fn init_window_shadow(window: &Window, is_enable: bool) {
    use window_shadows::set_shadow;

    if let Err(e) = set_shadow(window, is_enable) {
      println!("Failed to add native window shadow, errMsg: {:?}", e);
    }
}
