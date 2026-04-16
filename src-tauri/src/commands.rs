use crate::ssh::{connect, SshConfig, SshConnection};
use crate::tmux::{self, TmuxSessionInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use tauri::{AppHandle, Emitter};
use std::io::{Read, Write};
use std::time::Duration;

/// Store active SSH channels by ID for write/resize/kill
struct ChannelEntry {
    channel: ssh2::Channel,
    session: ssh2::Session,
    stream: std::net::TcpStream,
}

static CHANNEL_STORE: LazyLock<Mutex<HashMap<u32, ChannelEntry>>> =
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

fn next_pid() -> u32 {
    let mut n = NEXT_PID.lock().unwrap();
    let id = *n;
    *n += 1;
    id
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
    channel.set_blocking(false);

    let pid = next_pid();

    // Store the SSH connection for later write/resize/kill
    {
        let mut store = CHANNEL_STORE.lock().unwrap();
        store.insert(
            pid,
            ChannelEntry {
                channel: conn.session.channel_session().map_err(|e| e.to_string())?,
                session: conn.session,
                stream: conn.stream,
            },
        );
    }

    // Spawn reader thread
    let app_handle = app.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match channel.read(&mut buf) {
                Ok(0) => break,
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
            "\r\n\x1b[90m[Session disconnected]\x1b[0m".to_string(),
        );
    });

    Ok(pid)
}

#[tauri::command]
pub fn write_to_pty(pid: u32, data: String) -> Result<(), String> {
    let store = CHANNEL_STORE.lock().unwrap();
    let entry = store.get(&pid).ok_or_else(|| format!("No channel for pid {}", pid))?;
    entry
        .channel
        .write_all(data.as_bytes())
        .map_err(|e| format!("Write error: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn resize_pty(pid: u32, rows: u16, cols: u16) -> Result<(), String> {
    let store = CHANNEL_STORE.lock().unwrap();
    let entry = store.get(&pid).ok_or_else(|| format!("No channel for pid {}", pid))?;
    entry
        .channel
        .request_pty_size(rows, cols)
        .map_err(|e| format!("Resize error: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn kill_pty(pid: u32) -> Result<(), String> {
    let mut store = CHANNEL_STORE.lock().unwrap();
    if store.remove(&pid).is_some() {
        Ok(())
    } else {
        Err(format!("No channel found for pid {}", pid))
    }
}
