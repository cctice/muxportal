use crate::ssh::{connect, SshConfig};
use crate::tmux::{self, TmuxSessionInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{AppHandle, Emitter};
use std::io::{Read, Write};
use std::time::Duration;

/// Store active terminal channels
static TERM_STORE: LazyLock<Mutex<HashMap<u32, Arc<Mutex<ssh2::Channel>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static NEXT_PID: LazyLock<Mutex<u32>> = LazyLock::new(|| Mutex::new(1));

#[derive(Debug, Serialize, Deserialize)]
pub struct TmuxSession {
    pub name: String,
    pub windows: u32,
    pub created: String,
    pub attached: String,
}

fn make_ssh_config(
    host: String, port: u16, username: String, auth_type: String,
    password: String, key_path: String,
) -> SshConfig {
    SshConfig { host, port, username, auth_type, password, key_path }
}

fn next_pid() -> u32 {
    let mut n = NEXT_PID.lock().unwrap();
    let id = *n;
    *n += 1;
    id
}

#[tauri::command]
pub fn list_tmux_sessions(
    host: String, port: u16, username: String, auth_type: String,
    password: String, key_path: String,
) -> Result<Vec<TmuxSession>, String> {
    let config = make_ssh_config(host, port, username, auth_type, password, key_path);
    let conn = connect(&config)?;
    let sessions = tmux::list_sessions(&conn)?;
    Ok(sessions.into_iter().map(|s| TmuxSession {
        name: s.name, windows: s.windows, created: s.created, attached: s.attached,
    }).collect())
}

#[tauri::command]
pub fn create_tmux_session(
    host: String, port: u16, username: String, auth_type: String,
    password: String, key_path: String, session_name: String,
) -> Result<(), String> {
    let config = make_ssh_config(host, port, username, auth_type, password, key_path);
    let conn = connect(&config)?;
    tmux::create_session(&conn, &session_name)?;
    Ok(())
}

#[tauri::command]
pub fn kill_tmux_session(
    host: String, port: u16, username: String, auth_type: String,
    password: String, key_path: String, session_name: String,
) -> Result<(), String> {
    let config = make_ssh_config(host, port, username, auth_type, password, key_path);
    let conn = connect(&config)?;
    tmux::kill_session(&conn, &session_name)?;
    Ok(())
}

#[tauri::command]
pub fn attach_tmux_session(
    app: AppHandle,
    host: String, port: u16, username: String, auth_type: String,
    password: String, key_path: String, session_name: String,
    rows: u16, cols: u16,
) -> Result<u32, String> {
    let config = make_ssh_config(host, port, username, auth_type, password, key_path);
    let conn = connect(&config)?;

    let channel = tmux::attach_session(&conn, &session_name, rows, cols)?;

    let pid = next_pid();
    let channel = Arc::new(Mutex::new(channel));

    {
        let mut store = TERM_STORE.lock().unwrap();
        store.insert(pid, Arc::clone(&channel));
    }

    // Spawn reader thread
    let app_handle = app.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            let result = {
                let mut ch = channel.lock().unwrap();
                ch.read(&mut buf)
            };
            match result {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_handle.emit("terminal-output", data);
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        std::thread::sleep(Duration::from_millis(16));
                        continue;
                    }
                    break;
                }
            }
        }
        let _ = app_handle.emit(
            "terminal-output",
            "\r\n\x1b[90m[Session disconnected]\x1b[0m".to_string(),
        );
    });

    Ok(pid)
}

#[tauri::command]
pub fn write_to_pty(pid: u32, data: String) -> Result<(), String> {
    let store = TERM_STORE.lock().unwrap();
    let entry = store.get(&pid).ok_or(format!("No terminal for pid {}", pid))?;
    let mut ch = entry.lock().unwrap();
    ch.write_all(data.as_bytes()).map_err(|e| format!("Write error: {}", e))
}

#[tauri::command]
pub fn resize_pty(pid: u32, rows: u16, cols: u16) -> Result<(), String> {
    let store = TERM_STORE.lock().unwrap();
    let entry = store.get(&pid).ok_or(format!("No terminal for pid {}", pid))?;
    let mut ch = entry.lock().unwrap();
    ch.request_pty_size(cols as u32, rows as u32, Some(0), Some(0)).map_err(|e| format!("Resize error: {}", e))
}

#[tauri::command]
pub fn kill_pty(pid: u32) -> Result<(), String> {
    let mut store = TERM_STORE.lock().unwrap();
    store.remove(&pid).map(|_| ()).ok_or(format!("No terminal for pid {}", pid))
}
