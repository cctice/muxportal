use serde::{Deserialize, Serialize};
use ssh2::{Channel, Session as SshSession};
use std::collections::HashMap;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_type: String,
    pub password: String,
    pub key_path: String,
}

pub struct SshConnection {
    pub session: SshSession,
    pub stream: TcpStream,
}

/// Establish an SSH connection
pub fn connect(config: &SshConfig) -> Result<SshConnection, String> {
    let addr = format!("{}:{}", config.host, config.port);
    let tcp =
        TcpStream::connect(&addr).map_err(|e| format!("TCP connect failed: {}", e))?;

    let mut session = SshSession::new().map_err(|e| format!("SSH init failed: {}", e))?;
    session
        .set_tcp_stream(tcp.try_clone().map_err(|e| format!("Clone stream: {}", e))?)
        .map_err(|e| format!("Set TCP stream: {}", e))?;
    session.handshake().map_err(|e| format!("SSH handshake: {}", e))?;

    match config.auth_type.as_str() {
        "key" => {
            let key_path = config
                .key_path
                .replace("~", &dirs_home())
                .replace("$HOME", &dirs_home());
            let path = Path::new(&key_path);
            session
                .pubkey_auth(&config.username, None, path, None)
                .map_err(|e| format!("Key auth failed: {}", e))?;
        }
        _ => {
            session
                .userauth_password(&config.username, &config.password)
                .map_err(|e| format!("Password auth failed: {}", e))?;
        }
    }

    Ok(SshConnection {
        session,
        stream: tcp,
    })
}

/// Execute a command over SSH and return stdout
pub fn exec_command(conn: &SshConnection, cmd: &str) -> Result<String, String> {
    let mut channel = conn
        .session
        .channel_session()
        .map_err(|e| format!("Open channel: {}", e))?;

    channel
        .exec(cmd)
        .map_err(|e| format!("Exec command: {}", e))?;

    let mut output = String::new();
    channel
        .read_to_string(&mut output)
        .map_err(|e| format!("Read output: {}", e))?;

    let _ = channel.close();
    Ok(output.trim().to_string())
}

/// Execute a command over SSH and return the channel for interactive use
pub fn exec_interactive(conn: &SshConnection, cmd: &str) -> Result<Channel, String> {
    let mut channel = conn
        .session
        .channel_session()
        .map_err(|e| format!("Open channel: {}", e))?;

    channel
        .request_pty("xterm-256color", 24, 80, None)
        .map_err(|e| format!("Request PTY: {}", e))?;

    channel
        .exec(cmd)
        .map_err(|e| format!("Exec: {}", e))?;

    Ok(channel)
}

fn dirs_home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/root".to_string())
}

/// Global store for active PTY processes
pub struct PtyStore {
    pub processes: Mutex<HashMap<u32, portable_pty::PtyProcess>>,
}

impl PtyStore {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
        }
    }
}
