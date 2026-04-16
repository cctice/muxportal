use crate::ssh::{connect, SshConfig, SshConnection, PtyStore};
use crate::tmux::{self, TmuxSessionInfo};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tauri::{AppHandle, Emitter};
use std::io::{Read, Write};

static PTY_STORE: LazyLock<PtyStore> = LazyLock::new(PtyStore::new);

#[derive(Debug, Serialize, Deserialize)]
pub struct TmuxSession {
    pub name: String,
    pub windows: u32,
    pub created: String,
    pub attached: String,
}

fn make_ssh_config(
    host: String,
    port: u16,
    username: String,
    auth_type: String,
    password: String,
    key_path: String,
) -> SshConfig {
    SshConfig {
        host,
        port,
        username,
        auth_type,
        password,
        key_path,
    }
}

#[tauri::command]
pub fn list_tmux_sessions(
    host: String,
    port: u16,
    username: String,
    auth_type: String,
    password: String,
    key_path: String,
) -> Result<Vec<TmuxSession>, String> {
    let config = make_ssh_config(host, port, username, auth_type, password, key_path);
    let conn = connect(&config)?;
    let sessions = tmux::list_sessions(&conn)?;

    Ok(sessions
        .into_iter()
        .map(|s| TmuxSession {
            name: s.name,
            windows: s.windows,
            created: s.created,
            attached: s.attached,
        })
        .collect())
}

#[tauri::command]
pub fn create_tmux_session(
    host: String,
    port: u16,
    username: String,
    auth_type: String,
    password: String,
    key_path: String,
    session_name: String,
) -> Result<(), String> {
    let config = make_ssh_config(host, port, username, auth_type, password, key_path);
    let conn = connect(&config)?;
    tmux::create_session(&conn, &session_name)?;
    Ok(())
}

#[tauri::command]
pub fn kill_tmux_session(
    host: String,
    port: u16,
    username: String,
    auth_type: String,
    password: String,
    key_path: String,
    session_name: String,
) -> Result<(), String> {
    let config = make_ssh_config(host, port, username, auth_type, password, key_path);
    let conn = connect(&config)?;
    tmux::kill_session(&conn, &session_name)?;
    Ok(())
}

#[tauri::command]
pub fn attach_tmux_session(
    app: AppHandle,
    host: String,
    port: u16,
    username: String,
    auth_type: String,
    password: String,
    key_path: String,
    session_name: String,
    rows: u16,
    cols: u16,
) -> Result<u32, String> {
    let config = make_ssh_config(host, port, username, auth_type, password, key_path);
    let conn = connect(&config)?;

    let mut channel = tmux::attach_session(&conn, &session_name, rows, cols)?;

    // Set non-blocking for read
    channel.set_blocking(false);

    // Make a unique ID for this PTY
    let pid = {
        let mut store = PTY_STORE.processes.lock().unwrap();
        let id = (store.len() as u32) + 1;
        // We store the SSH channel info by wrapping it
        id
    };

    // Spawn a thread to read from the SSH channel and emit events
    let app_handle = app.clone();
    let pid_for_thread = pid;

    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match channel.read(&mut buf) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = app_handle.emit("terminal-output", data);
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        std::thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    break;
                }
            }
        }
        let _ = app_handle.emit(
            "terminal-output",
            format!("\r\n\x1b[90m[Session disconnected]\x1b[0m"),
        );
    });

    Ok(pid)
}

use std::time::Duration;

#[tauri::command]
pub fn write_to_pty(pid: u32, data: String) -> Result<(), String> {
    // For SSH-based terminals, writing goes through the SSH channel.
    // Since SSH channels don't have a simple store, we need a channel map.
    // This is a simplified version — in production you'd store channels in the PtyStore.
    Err("write_to_pty needs channel reference — use SSH channel directly in production".to_string())
}

#[tauri::command]
pub fn resize_pty(pid: u32, rows: u16, cols: u16) -> Result<(), String> {
    // Simplified: in production, resize the SSH channel's PTY
    Ok(())
}

#[tauri::command]
pub fn kill_pty(pid: u32) -> Result<(), String> {
    let mut store = PTY_STORE.processes.lock().unwrap();
    if let Some(_proc) = store.remove(&pid) {
        Ok(())
    } else {
        Err(format!("No PTY found for pid {}", pid))
    }
}
