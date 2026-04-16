import { useState, useCallback, useEffect } from "react";
import Sidebar from "./components/Sidebar";
import SessionTabs from "./components/SessionTabs";
import Terminal from "./components/Terminal";
import ConnectionForm from "./components/ConnectionForm";
import { invoke } from "@tauri-apps/api/core";

export interface HostConfig {
  id: string;
  label: string;
  host: string;
  port: number;
  username: string;
  auth_type: "password" | "key";
  password?: string;
  key_path?: string;
}

export interface TmuxSession {
  name: string;
  windows: number;
  created: string;
  attached: string;
}

export default function App() {
  const [hosts, setHosts] = useState<HostConfig[]>([]);
  const [activeHost, setActiveHost] = useState<HostConfig | null>(null);
  const [sessions, setSessions] = useState<TmuxSession[]>([]);
  const [activeSession, setActiveSession] = useState<string | null>(null);
  const [showConnectionForm, setShowConnectionForm] = useState(false);
  const [editingHost, setEditingHost] = useState<HostConfig | null>(null);

  const loadHosts = useCallback(() => {
    try {
      const saved = localStorage.getItem("muxportal_hosts");
      if (saved) setHosts(JSON.parse(saved));
    } catch {
      // ignore
    }
  }, []);

  useEffect(() => {
    loadHosts();
  }, [loadHosts]);

  const saveHosts = useCallback(
    (updated: HostConfig[]) => {
      setHosts(updated);
      localStorage.setItem("muxportal_hosts", JSON.stringify(updated));
    },
    []
  );

  const connectToHost = useCallback(
    async (host: HostConfig) => {
      setActiveHost(host);
      setActiveSession(null);
      try {
        const result = await invoke<TmuxSession[]>("list_tmux_sessions", {
          host: host.host,
          port: host.port,
          username: host.username,
          authType: host.auth_type,
          password: host.password || "",
          keyPath: host.key_path || "",
        });
        setSessions(result);
        // Auto-attach to first session if exists
        if (result.length > 0) setActiveSession(result[0].name);
      } catch (err) {
        console.error("Failed to list tmux sessions:", err);
        setSessions([]);
      }
    },
    []
  );

  const createSession = useCallback(
    async (name: string) => {
      if (!activeHost) return;
      try {
        await invoke("create_tmux_session", {
          host: activeHost.host,
          port: activeHost.port,
          username: activeHost.username,
          authType: activeHost.auth_type,
          password: activeHost.password || "",
          keyPath: activeHost.key_path || "",
          sessionName: name,
        });
        // Reload sessions
        await connectToHost(activeHost);
      } catch (err) {
        console.error("Failed to create tmux session:", err);
      }
    },
    [activeHost, connectToHost]
  );

  const killSession = useCallback(
    async (name: string) => {
      if (!activeHost) return;
      try {
        await invoke("kill_tmux_session", {
          host: activeHost.host,
          port: activeHost.port,
          username: activeHost.username,
          authType: activeHost.auth_type,
          password: activeHost.password || "",
          keyPath: activeHost.key_path || "",
          sessionName: name,
        });
        if (activeSession === name) setActiveSession(null);
        await connectToHost(activeHost);
      } catch (err) {
        console.error("Failed to kill tmux session:", err);
      }
    },
    [activeHost, activeSession, connectToHost]
  );

  return (
    <div className="app-container">
      <Sidebar
        hosts={hosts}
        activeHost={activeHost}
        onConnect={connectToHost}
        onAdd={() => {
          setEditingHost(null);
          setShowConnectionForm(true);
        }}
        onEdit={(h) => {
          setEditingHost(h);
          setShowConnectionForm(true);
        }}
        onDelete={(id) =>
          saveHosts(hosts.filter((h) => h.id !== id))
        }
      />

      {activeHost && (
        <div className="main-area">
          <SessionTabs
            sessions={sessions}
            activeSession={activeSession}
            onSelect={setActiveSession}
            onCreate={createSession}
            onKill={killSession}
          />

          <div className="terminal-area">
            {activeSession ? (
              <Terminal
                host={activeHost}
                session={activeSession}
              />
            ) : (
              <div className="terminal-placeholder">
                {sessions.length > 0
                  ? "Select a session to connect"
                  : "No tmux sessions. Create one to get started."}
              </div>
            )}
          </div>
        </div>
      )}

      {!activeHost && (
        <div className="welcome-screen">
          <h1>🦐 MuxPortal</h1>
          <p>Termius with tmux — SSH client with native tmux session management</p>
          <p>Select a host from the sidebar or add a new connection.</p>
        </div>
      )}

      {showConnectionForm && (
        <ConnectionForm
          host={editingHost}
          onSave={(h) => {
            if (editingHost) {
              saveHosts(hosts.map((x) => (x.id === h.id ? h : x)));
            } else {
              saveHosts([...hosts, h]);
            }
            setShowConnectionForm(false);
          }}
          onClose={() => setShowConnectionForm(false)}
        />
      )}
    </div>
  );
}
