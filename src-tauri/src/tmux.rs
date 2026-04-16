use crate::ssh::{exec_command, exec_interactive, SshConfig, SshConnection};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxSessionInfo {
    pub name: String,
    pub windows: u32,
    pub created: String,
    pub attached: String,
}

/// Check if tmux is available on the remote host
pub fn has_tmux(conn: &SshConnection) -> Result<bool, String> {
    let output = exec_command(conn, "which tmux 2>/dev/null && echo yes || echo no")?;
    Ok(output.trim() == "yes")
}

/// List tmux sessions
pub fn list_sessions(conn: &SshConnection) -> Result<Vec<TmuxSessionInfo>, String> {
    let output = exec_command(
        conn,
        r#"tmux list-sessions -F '#{session_name} #{session_windows} #{session_created} #{session_attached}' 2>/dev/null || echo ""#,
    )?;

    if output.is_empty() {
        return Ok(Vec::new());
    }

    let sessions: Vec<TmuxSessionInfo> = output
        .lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let parts: Vec<&str> = line.splitn(4, ' ').collect();
            if parts.len() >= 4 {
                let created_epoch: i64 = parts[2].parse().unwrap_or(0);
                let created = format_timestamp(created_epoch);
                Some(TmuxSessionInfo {
                    name: parts[0].to_string(),
                    windows: parts[1].parse().unwrap_or(1),
                    created,
                    attached: parts[3].to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    Ok(sessions)
}

/// Create a new tmux session
pub fn create_session(conn: &SshConnection, name: &str) -> Result<(), String> {
    exec_command(conn, &format!("tmux new-session -d -s {}", shell_escape(name)))?;
    Ok(())
}

/// Kill a tmux session
pub fn kill_session(conn: &SshConnection, name: &str) -> Result<(), String> {
    exec_command(conn, &format!("tmux kill-session -t {}", shell_escape(name)))?;
    Ok(())
}

/// Attach to a tmux session interactively
pub fn attach_session(
    conn: &SshConnection,
    name: &str,
    rows: u16,
    cols: u16,
) -> Result<ssh2::Channel, String> {
    let cmd = format!(
        "tmux attach-session -t {}",
        shell_escape(name)
    );

    let mut channel = exec_interactive(conn, &cmd)?;

    // Resize the remote PTY to match local dimensions
    channel
        .request_pty_size(cols as u32, rows as u32, Some(0), Some(0))
        .map_err(|e| format!("Resize PTY: {}", e))?;

    Ok(channel)
}

fn shell_escape(s: &str) -> String {
    s.replace("'", "'\\''")
}

fn format_timestamp(epoch: i64) -> String {
    if epoch == 0 {
        return "unknown".to_string();
    }
    let secs = if epoch > 1_000_000_000_000 {
        epoch / 1000
    } else {
        epoch
    };
    let dt = chrono::DateTime::from_timestamp(secs, 0);
    match dt {
        Some(d) => d.format("%Y-%m-%d %H:%M").to_string(),
        None => "unknown".to_string(),
    }
}
