use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Wry,
};

fn on_tray_event(tray: &TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        let app = tray.app_handle();
        if let Some(webview_window) = app.get_webview_window("main") {
            let _ = webview_window.show();
            let _ = webview_window.set_focus();
        }
    }
}

fn on_menu_event(app: &AppHandle<Wry>, event: MenuEvent) {
    match event.id().as_ref() {
        "quit" => {
            app.exit(0);
        }
        "show" => {
            if let Some(webview_window) = app.get_webview_window("main") {
                let _ = webview_window.show();
                let _ = webview_window.set_focus();
            }
        }
        _ => (),
    }
}

pub fn create_system_tray(app: &AppHandle<Wry>) -> Result<TrayIcon, tauri::Error> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_i, &show_i])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(true)
        .on_menu_event(|app: &AppHandle, event: tauri::menu::MenuEvent| on_menu_event(app, event))
        .on_tray_icon_event(|tray, event: tauri::tray::TrayIconEvent| on_tray_event(tray, event))
        .build(app)
}
