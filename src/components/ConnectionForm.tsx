import { useState } from "react";
import type { HostConfig } from "../App";

interface Props {
  host: HostConfig | null;
  onSave: (host: HostConfig) => void;
  onClose: () => void;
}

export default function ConnectionForm({ host, onSave, onClose }: Props) {
  const isEdit = !!host;
  const [label, setLabel] = useState(host?.label || "");
  const [hostname, setHostname] = useState(host?.host || "");
  const [port, setPort] = useState(host?.port || 22);
  const [username, setUsername] = useState(host?.username || "");
  const [authType, setAuthType] = useState<"password" | "key">(
    host?.auth_type || "password"
  );
  const [password, setPassword] = useState(host?.password || "");
  const [keyPath, setKeyPath] = useState(host?.key_path || "");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!label || !hostname || !username) return;

    onSave({
      id: host?.id || crypto.randomUUID(),
      label,
      host: hostname,
      port,
      username,
      auth_type: authType,
      password: authType === "password" ? password : undefined,
      key_path: authType === "key" ? keyPath : undefined,
    });
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <h3>{isEdit ? "Edit Connection" : "New Connection"}</h3>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Label</label>
            <input
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              placeholder="My Server"
              autoFocus
            />
          </div>
          <div className="form-row">
            <div className="form-group">
              <label>Host</label>
              <input
                value={hostname}
                onChange={(e) => setHostname(e.target.value)}
                placeholder="192.168.1.100"
              />
            </div>
            <div className="form-group">
              <label>Port</label>
              <input
                type="number"
                value={port}
                onChange={(e) => setPort(parseInt(e.target.value) || 22)}
              />
            </div>
          </div>
          <div className="form-group">
            <label>Username</label>
            <input
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="root"
            />
          </div>
          <div className="form-group">
            <label>Auth Type</label>
            <select
              value={authType}
              onChange={(e) => setAuthType(e.target.value as "password" | "key")}
            >
              <option value="password">Password</option>
              <option value="key">Private Key</option>
            </select>
          </div>
          {authType === "password" ? (
            <div className="form-group">
              <label>Password</label>
              <input
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="••••••••"
              />
            </div>
          ) : (
            <div className="form-group">
              <label>Key Path</label>
              <input
                value={keyPath}
                onChange={(e) => setKeyPath(e.target.value)}
                placeholder="~/.ssh/id_rsa"
              />
            </div>
          )}
          <div className="modal-actions">
            <button type="button" className="btn-secondary" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" className="btn-primary">
              {isEdit ? "Save" : "Connect"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
