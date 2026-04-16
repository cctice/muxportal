mod commands;
mod ssh;
mod tmux;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_tmux_sessions,
            commands::create_tmux_session,
            commands::kill_tmux_session,
            commands::attach_tmux_session,
            commands::write_to_pty,
            commands::resize_pty,
            commands::kill_pty,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
