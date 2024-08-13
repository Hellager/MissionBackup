use tauri::{ Wry, Manager, AppHandle,
    SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, SystemTrayEvent
};

pub fn create_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit Program");
    let hide = CustomMenuItem::new("hide".to_string(), "Close to tray");
    let tray_menu = SystemTrayMenu::new()
      .add_item(hide)
      .add_native_item(SystemTrayMenuItem::Separator)
      .add_item(quit);
    SystemTray::new().with_menu(tray_menu)
}

pub fn on_system_tray_event(app: &AppHandle<Wry>, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::DoubleClick { position: _ , size: _, .. } => {
            let window = app.get_window("main").unwrap();
            window.unminimize().unwrap();
            window.show().unwrap(); 
            window.set_focus().unwrap();
        },
        SystemTrayEvent::MenuItemClick { id, ..} => {
            match id.as_str() {
              "quit" => {
                app.get_window("main").unwrap().hide().unwrap();
                
                std::process::exit(0);
              }
              "hide" => {
                app.get_window("main").unwrap().hide().unwrap();
              }
              _ => {}
            }
        }
        _ => {}
    }
}
