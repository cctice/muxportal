import type { HostConfig } from "../App";

interface Props {
  hosts: HostConfig[];
  activeHost: HostConfig | null;
  onConnect: (host: HostConfig) => void;
  onAdd: () => void;
  onEdit: (host: HostConfig) => void;
  onDelete: (id: string) => void;
}

export default function Sidebar({ hosts, activeHost, onConnect, onAdd, onEdit, onDelete }: Props) {
  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <h2>🖥 Hosts</h2>
        <button className="btn-add" onClick={onAdd} title="Add host">
          +
        </button>
      </div>
      <div className="host-list">
        {hosts.map((h) => (
          <div
            key={h.id}
            className={`host-item ${activeHost?.id === h.id ? "active" : ""}`}
            onClick={() => onConnect(h)}
          >
            <div className="host-item-label">
              <span className="name">{h.label}</span>
              <span className="addr">
                {h.username}@{h.host}:{h.port}
              </span>
            </div>
            <div className="host-item-actions" onClick={(e) => e.stopPropagation()}>
              <button onClick={() => onEdit(h)} title="Edit">✏</button>
              <button onClick={() => onDelete(h.id)} title="Delete">✕</button>
            </div>
          </div>
        ))}
      </div>
    </aside>
  );
}
